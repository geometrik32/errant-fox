use std::path::Path;
use std::process::Stdio;

use serde::Deserialize;
use tokio::process::Command;

use crate::{db::DbPool, errors::AppError, seafile::SeafileClient};

const N_FRAMES: u32 = 1;

/// User-Agent sent by ffmpeg/ffprobe to Seafile's seafhttp server.
/// The default Lavf user-agent gets HTTP 403; a browser UA may pass.
const FAKE_USER_AGENT: &str = "Mozilla/5.0";

/// Sentinel stored in `videos.preview_count` when generation fails
/// permanently so we never retry.
const PREVIEW_FAILED: i32 = -1;

// ── ffprobe ────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct FfprobeOutput {
    format: FfprobeFormat,
}

#[derive(Deserialize)]
struct FfprobeFormat {
    duration: String,
}

/// Get video duration in seconds via ffprobe.
async fn get_duration(download_url: &str) -> Result<f64, String> {
    let output = Command::new("ffprobe")
        .arg("-user_agent")
        .arg(FAKE_USER_AGENT)
        .arg("-v")
        .arg("error")
        .arg("-show_entries")
        .arg("format=duration")
        .arg("-of")
        .arg("json")
        .arg("-i")
        .arg(download_url)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("ffprobe spawn: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let code = output.status.code().unwrap_or(-1);
        return Err(format!("ffprobe exit={code}: {stderr}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: FfprobeOutput =
        serde_json::from_str(&stdout).map_err(|e| format!("ffprobe json: {e} (stdout: {stdout:.200})"))?;
    let secs: f64 = parsed.format.duration.parse().unwrap_or(60.0);
    Ok(secs)
}

// ── ffmpeg ─────────────────────────────────────────────────────────────────────

/// Extract one JPEG frame at `seek_secs`.
async fn extract_frame(
    download_url: &str,
    seek_secs: f64,
    output_pattern: &str,
) -> Result<(), String> {
    let output = Command::new("ffmpeg")
        .arg("-user_agent")
        .arg(FAKE_USER_AGENT)
        .arg("-y")
        .arg("-ss")
        .arg(format!("{:.3}", seek_secs))
        .arg("-i")
        .arg(download_url)
        .arg("-vf")
        .arg("scale=480:-1")
        .arg("-vframes")
        .arg("1")
        .arg("-start_number")
        .arg("0")
        .arg(output_pattern)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("ffmpeg spawn: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let code = output.status.code().unwrap_or(-1);
        return Err(format!("ffmpeg exit={code}: {stderr}"));
    }
    Ok(())
}

// ── helpers ────────────────────────────────────────────────────────────────────

fn is_http_error(stderr: &str) -> bool {
    stderr.contains("HTTP error 403")
        || stderr.contains("HTTP error 404")
        || stderr.contains("HTTP error 410")
}

async fn set_preview_count(db: &DbPool, video_id: &str, value: i32) -> Result<(), AppError> {
    let vid = video_id.to_string();
    let db = db.clone();
    tokio::task::spawn_blocking(move || {
        use crate::db::schema::videos;
        use diesel::prelude::*;
        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;
        diesel::update(videos::table.filter(videos::id.eq(&vid)))
            .set(videos::preview_count.eq(value))
            .execute(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok::<(), AppError>(())
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;
    Ok(())
}

async fn set_duration(db: &DbPool, video_id: &str, duration_ms: i32) -> Result<(), AppError> {
    let vid = video_id.to_string();
    let db = db.clone();
    tokio::task::spawn_blocking(move || {
        use crate::db::schema::videos;
        use diesel::prelude::*;
        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;
        diesel::update(videos::table.filter(videos::id.eq(&vid)))
            .set(videos::duration_ms.eq(duration_ms))
            .execute(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok::<(), AppError>(())
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;
    Ok(())
}

/// Run one full attempt: ffprobe → midpoint → ffmpeg.
/// Returns the extracted duration and any stderr from a failed step.
async fn attempt(
    seafile: &SeafileClient,
    seafile_path: &str,
    output_pattern: &str,
) -> Result<f64, String> {
    // ── step 1: get duration ──
    let probe_url = seafile
        .get_download_url(seafile_path)
        .await
        .map_err(|e| format!("seafile url (probe): {e}"))?;

    let duration_secs = get_duration(&probe_url).await?;
    let midpoint = duration_secs / 2.0;

    // ── step 2: extract mid-frame ──
    let frame_url = seafile
        .get_download_url(seafile_path)
        .await
        .map_err(|e| format!("seafile url (frame): {e}"))?;

    extract_frame(&frame_url, midpoint, output_pattern).await?;

    Ok(duration_secs)
}

// ── public API ─────────────────────────────────────────────────────────────────

pub async fn generate_previews(
    video_id: &str,
    seafile: &SeafileClient,
    seafile_path: &str,
    previews_dir: &Path,
    db: &DbPool,
) -> Result<(), AppError> {
    let output_dir = previews_dir.join(video_id);
    tokio::fs::create_dir_all(&output_dir)
        .await
        .map_err(|e| AppError::Internal(format!("create_dir_all: {e}")))?;

    let output_pattern = output_dir
        .join("%d.jpg")
        .to_string_lossy()
        .into_owned();

    match attempt(seafile, seafile_path, &output_pattern).await {
        Ok(duration_secs) => {
            let duration_ms = (duration_secs * 1000.0) as i32;
            set_duration(db, video_id, duration_ms).await?;
            set_preview_count(db, video_id, N_FRAMES as i32).await?;
            tracing::info!("previews generated for {video_id} (duration {duration_secs:.1}s)");
            Ok(())
        }
        Err(first_err) => {
            if !is_http_error(&first_err) {
                tracing::error!("ffmpeg/ffprobe failed for {video_id}:\n{first_err}");
                return Err(AppError::Internal(
                    "ffmpeg exited with non-zero status".to_string(),
                ));
            }

            // Retry once with fresh download URLs.
            tracing::warn!(
                "HTTP error for {video_id}, retrying \
                 ({:.200})",
                first_err.lines().find(|l| l.contains("HTTP error")).unwrap_or("")
            );

            match attempt(seafile, seafile_path, &output_pattern).await {
                Ok(duration_secs) => {
                    let duration_ms = (duration_secs * 1000.0) as i32;
                    set_duration(db, video_id, duration_ms).await?;
                    set_preview_count(db, video_id, N_FRAMES as i32).await?;
                    tracing::info!("previews generated for {video_id} (after retry)");
                    Ok(())
                }
                Err(retry_err) => {
                    if is_http_error(&retry_err) {
                        tracing::error!(
                            "double HTTP failure for {video_id}, \
                             marking permanently failed:\n{retry_err}"
                        );
                        set_preview_count(db, video_id, PREVIEW_FAILED).await?;
                        Ok(())
                    } else {
                        tracing::error!(
                            "retry also failed for {video_id}:\n{retry_err}"
                        );
                        Err(AppError::Internal(
                            "ffmpeg exited with non-zero status".to_string(),
                        ))
                    }
                }
            }
        }
    }
}
