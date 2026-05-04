use std::path::Path;
use std::process::Stdio;

use serde::Deserialize;
use tokio::process::Command;

use crate::{db::DbPool, errors::AppError, seafile::SeafileClient};

const N_FRAMES: u32 = 1;
const FAKE_USER_AGENT: &str = "Mozilla/5.0";
const PREVIEW_FAILED: i32 = -1;

// ── ffprobe / ffmpeg helpers ────────────────────────────────────────────────────

#[derive(Deserialize)]
struct FfprobeOutput {
    format: FfprobeFormat,
}

#[derive(Deserialize)]
struct FfprobeFormat {
    duration: String,
}

/// Run ffprobe on `input` (URL or file path), return duration in seconds.
async fn get_duration(input: &str) -> Result<f64, String> {
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
        .arg(input)
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
    let parsed: FfprobeOutput = serde_json::from_str(&stdout)
        .map_err(|e| format!("ffprobe json: {e} (stdout: {stdout:.200})"))?;
    let secs: f64 = parsed.format.duration.parse().unwrap_or(60.0);
    Ok(secs)
}

/// Run ffmpeg on `input` (URL or file path), extract one frame at `seek_secs`.
async fn extract_frame(input: &str, seek_secs: f64, output_pattern: &str) -> Result<(), String> {
    let output = Command::new("ffmpeg")
        .arg("-user_agent")
        .arg(FAKE_USER_AGENT)
        .arg("-y")
        .arg("-ss")
        .arg(format!("{:.3}", seek_secs))
        .arg("-i")
        .arg(input)
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

fn is_moov_error(stderr: &str) -> bool {
    stderr.contains("moov atom not found")
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

// ── strategies ─────────────────────────────────────────────────────────────────

/// Fast path: ffprobe + ffmpeg directly against the Seafile download URL.
/// Works for videos where the moov atom is reachable over HTTP.
async fn attempt_remote(
    seafile: &SeafileClient,
    seafile_path: &str,
    output_pattern: &str,
) -> Result<f64, String> {
    let probe_url = seafile
        .get_download_url(seafile_path)
        .await
        .map_err(|e| format!("seafile url (probe): {e}"))?;
    let duration_secs = get_duration(&probe_url).await?;
    let midpoint = duration_secs / 2.0;

    let frame_url = seafile
        .get_download_url(seafile_path)
        .await
        .map_err(|e| format!("seafile url (frame): {e}"))?;
    extract_frame(&frame_url, midpoint, output_pattern).await?;

    Ok(duration_secs)
}

/// Fallback: download the entire video via reqwest to a temp file, then
/// extract the frame locally.  Used when `attempt_remote` hits "moov atom
/// not found" (common for GoPro chaptered files and joined MP4s).
async fn attempt_local(
    seafile: &SeafileClient,
    seafile_path: &str,
    output_pattern: &str,
    tmp_path: &Path,
) -> Result<f64, String> {
    let download_url = seafile
        .get_download_url(seafile_path)
        .await
        .map_err(|e| format!("seafile url (download): {e}"))?;

    // Download to temp file.
    let response = reqwest::get(&download_url)
        .await
        .map_err(|e| format!("reqwest GET: {e}"))?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("reqwest returned HTTP {}", status.as_u16()));
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("reqwest download: {e}"))?;
    tokio::fs::write(tmp_path, &bytes)
        .await
        .map_err(|e| format!("tmp write: {e}"))?;

    let tmp_str = tmp_path.to_string_lossy();

    // Process locally.
    let duration_secs = get_duration(&tmp_str).await?;
    let midpoint = duration_secs / 2.0;
    extract_frame(&tmp_str, midpoint, output_pattern).await?;

    // Clean up.
    let _ = tokio::fs::remove_file(tmp_path).await;

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
    let tmp_path = output_dir.join("video.tmp");

    // ── 1. ffmpeg: try remote (fast path) ─────────────────────────────────
    match attempt_remote(seafile, seafile_path, &output_pattern).await {
        Ok(dur) => {
            set_duration(db, video_id, (dur * 1000.0) as i32).await?;
            set_preview_count(db, video_id, N_FRAMES as i32).await?;
            tracing::info!("previews generated for {video_id} (duration {dur:.1}s)");
            return Ok(());
        }
        Err(err) if is_http_error(&err) => {
            tracing::warn!("HTTP error for {video_id}, retrying ({:.200})",
                err.lines().find(|l| l.contains("HTTP error")).unwrap_or(""));
            match attempt_remote(seafile, seafile_path, &output_pattern).await {
                Ok(dur) => {
                    set_duration(db, video_id, (dur * 1000.0) as i32).await?;
                    set_preview_count(db, video_id, N_FRAMES as i32).await?;
                    tracing::info!("previews generated for {video_id} (after retry)");
                    return Ok(());
                }
                Err(retry_err) => {
                    tracing::error!("double HTTP failure for {video_id}, marking permanently failed:\n{retry_err}");
                    set_preview_count(db, video_id, PREVIEW_FAILED).await?;
                    return Ok(());
                }
            }
        }
        Err(err) if is_moov_error(&err) => {
            // ── 3. fallback: download to temp file ───────────────────────
            tracing::info!("moov not found remotely for {video_id}, downloading to temp file");
            match attempt_local(seafile, seafile_path, &output_pattern, &tmp_path).await {
                Ok(dur) => {
                    set_duration(db, video_id, (dur * 1000.0) as i32).await?;
                    set_preview_count(db, video_id, N_FRAMES as i32).await?;
                    tracing::info!("previews generated for {video_id} via temp file (duration {dur:.1}s)");
                    return Ok(());
                }
                Err(local_err) => {
                    let _ = tokio::fs::remove_file(&tmp_path).await;
                    tracing::error!("local fallback failed for {video_id}:\n{local_err}");
                    set_preview_count(db, video_id, PREVIEW_FAILED).await?;
                    return Ok(());
                }
            }
        }
        Err(other) => {
            tracing::error!("ffmpeg/ffprobe failed for {video_id}:\n{other}");
            return Err(AppError::Internal("ffmpeg exited with non-zero status".to_string()));
        }
    }
}
