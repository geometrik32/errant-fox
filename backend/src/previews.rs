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
async fn get_duration(video_id: &str, input: &str) -> Result<f64, String> {
    let is_remote = input.starts_with("http");
    tracing::info!("[{video_id}] ffprobe starting (remote={is_remote})");

    let mut cmd = Command::new("ffprobe");
    if is_remote {
        cmd.arg("-user_agent").arg(FAKE_USER_AGENT);
    }
    let output = cmd
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
        tracing::warn!("[{video_id}] ffprobe FAILED (exit={code})");
        return Err(format!("ffprobe exit={code}: {stderr}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: FfprobeOutput = serde_json::from_str(&stdout)
        .map_err(|e| format!("ffprobe json: {e} (stdout: {stdout:.200})"))?;
    let secs: f64 = parsed.format.duration.parse().unwrap_or(60.0);
    tracing::info!("[{video_id}] ffprobe OK duration={secs:.1}s");
    Ok(secs)
}

/// Run ffmpeg on `input`, extract one frame at `seek_secs` to `output_pattern`.
async fn extract_frame(
    video_id: &str,
    input: &str,
    seek_secs: f64,
    output_pattern: &str,
) -> Result<(), String> {
    let is_remote = input.starts_with("http");
    tracing::info!("[{video_id}] ffmpeg starting  seek={seek_secs:.1}s  remote={is_remote}");

    let mut cmd = Command::new("ffmpeg");
    if is_remote {
        cmd.arg("-user_agent").arg(FAKE_USER_AGENT);
    }
    let output = cmd
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
        tracing::warn!("[{video_id}] ffmpeg FAILED (exit={code})");
        return Err(format!("ffmpeg exit={code}: {stderr}"));
    }

    let size = tokio::fs::metadata(
        output_pattern.replacen("%d", "0", 1)
    ).await.map(|m| m.len()).unwrap_or(0);
    tracing::info!("[{video_id}] ffmpeg OK  frame_size={size}B");
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

async fn attempt_remote(
    video_id: &str,
    seafile: &SeafileClient,
    seafile_path: &str,
    output_pattern: &str,
) -> Result<f64, String> {
    tracing::info!("[{video_id}] strategy=remote  seafile_path={seafile_path}");

    tracing::info!("[{video_id}] seafile get_download_url (probe)");
    let probe_url = seafile
        .get_download_url(seafile_path)
        .await
        .map_err(|e| format!("seafile url (probe): {e}"))?;
    tracing::info!("[{video_id}] seafile download_url_ok (probe)");

    let duration_secs = get_duration(video_id, &probe_url).await?;
    let midpoint = duration_secs / 2.0;

    tracing::info!("[{video_id}] seafile get_download_url (frame)");
    let frame_url = seafile
        .get_download_url(seafile_path)
        .await
        .map_err(|e| format!("seafile url (frame): {e}"))?;
    tracing::info!("[{video_id}] seafile download_url_ok (frame)");

    extract_frame(video_id, &frame_url, midpoint, output_pattern).await?;

    Ok(duration_secs)
}

async fn attempt_local(
    video_id: &str,
    seafile: &SeafileClient,
    seafile_path: &str,
    output_pattern: &str,
    tmp_path: &Path,
) -> Result<f64, String> {
    tracing::info!("[{video_id}] strategy=local  downloading to temp file");

    tracing::info!("[{video_id}] seafile get_download_url (download)");
    let download_url = seafile
        .get_download_url(seafile_path)
        .await
        .map_err(|e| format!("seafile url (download): {e}"))?;
    tracing::info!("[{video_id}] seafile download_url_ok (download)");

    tracing::info!("[{video_id}] reqwest GET start");
    let response = reqwest::get(&download_url)
        .await
        .map_err(|e| format!("reqwest GET: {e}"))?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("reqwest returned HTTP {}", status.as_u16()));
    }
    let content_length = response.content_length();
    tracing::info!("[{video_id}] reqwest response  status={status}  content_length={content_length:?}");

    tracing::info!("[{video_id}] downloading body ...");
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("reqwest download: {e}"))?;
    tracing::info!("[{video_id}] downloaded  size={}B  ({:.1}MB)", bytes.len(), bytes.len() as f64 / 1_048_576.0);

    tracing::info!("[{video_id}] writing temp file  path={}", tmp_path.display());
    tokio::fs::write(tmp_path, &bytes)
        .await
        .map_err(|e| format!("tmp write: {e}"))?;

    let tmp_str = tmp_path.to_string_lossy();
    let duration_secs = get_duration(video_id, &tmp_str).await?;
    let midpoint = duration_secs / 2.0;
    extract_frame(video_id, &tmp_str, midpoint, output_pattern).await?;

    tracing::info!("[{video_id}] removing temp file");
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
    tracing::info!("[{video_id}] generate_previews START  seafile_path={seafile_path}");

    let output_dir = previews_dir.join(video_id);
    tokio::fs::create_dir_all(&output_dir)
        .await
        .map_err(|e| AppError::Internal(format!("create_dir_all: {e}")))?;
    tracing::debug!("[{video_id}] output_dir={}", output_dir.display());

    let output_pattern = output_dir
        .join("%d.jpg")
        .to_string_lossy()
        .into_owned();
    let tmp_path = output_dir.join("video.tmp");

    // ── 1. ffmpeg: try remote ─────────────────────────────────────────────
    match attempt_remote(video_id, seafile, seafile_path, &output_pattern).await {
        Ok(dur) => {
            let dur_ms = (dur * 1000.0) as i32;
            tracing::info!("[{video_id}] DB set_duration({dur_ms})");
            set_duration(db, video_id, dur_ms).await?;
            tracing::info!("[{video_id}] DB set_preview_count({})", N_FRAMES);
            set_preview_count(db, video_id, N_FRAMES as i32).await?;
            tracing::info!("[{video_id}] DONE  duration={dur:.1}s  strategy=remote");
            return Ok(());
        }
        Err(err) if is_http_error(&err) => {
            let detail = err.lines().find(|l| l.contains("HTTP error")).unwrap_or("");
            tracing::warn!("[{video_id}] HTTP error, retrying  detail={detail:.200}");
            match attempt_remote(video_id, seafile, seafile_path, &output_pattern).await {
                Ok(dur) => {
                    let dur_ms = (dur * 1000.0) as i32;
                    tracing::info!("[{video_id}] DB set_duration({dur_ms})");
                    set_duration(db, video_id, dur_ms).await?;
                    tracing::info!("[{video_id}] DB set_preview_count({})", N_FRAMES);
                    set_preview_count(db, video_id, N_FRAMES as i32).await?;
                    tracing::info!("[{video_id}] DONE  duration={dur:.1}s  strategy=remote(retry)");
                    return Ok(());
                }
                Err(retry_err) => {
                    tracing::error!("[{video_id}] FAILED  reason=double_http_error  detail:\n{retry_err}");
                    tracing::info!("[{video_id}] DB set_preview_count({PREVIEW_FAILED})");
                    set_preview_count(db, video_id, PREVIEW_FAILED).await?;
                    return Ok(());
                }
            }
        }
        Err(err) if is_moov_error(&err) => {
            // ── 2. fallback: download to temp file ─────────────────────────
            tracing::info!("[{video_id}] moov not found remotely, switching to strategy=local");
            match attempt_local(video_id, seafile, seafile_path, &output_pattern, &tmp_path).await {
                Ok(dur) => {
                    let dur_ms = (dur * 1000.0) as i32;
                    tracing::info!("[{video_id}] DB set_duration({dur_ms})");
                    set_duration(db, video_id, dur_ms).await?;
                    tracing::info!("[{video_id}] DB set_preview_count({})", N_FRAMES);
                    set_preview_count(db, video_id, N_FRAMES as i32).await?;
                    tracing::info!("[{video_id}] DONE  duration={dur:.1}s  strategy=local");
                    return Ok(());
                }
                Err(local_err) => {
                    let _ = tokio::fs::remove_file(&tmp_path).await;
                    tracing::error!("[{video_id}] FAILED  reason=local_fallback  detail:\n{local_err}");
                    tracing::info!("[{video_id}] DB set_preview_count({PREVIEW_FAILED})");
                    set_preview_count(db, video_id, PREVIEW_FAILED).await?;
                    return Ok(());
                }
            }
        }
        Err(other) => {
            tracing::error!("[{video_id}] FAILED  reason=ffmpeg_error  detail:\n{other}");
            return Err(AppError::Internal("ffmpeg exited with non-zero status".to_string()));
        }
    }
}
