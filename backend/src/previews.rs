use std::path::Path;
use std::process::Stdio;

use tokio::process::Command;

use crate::{db::DbPool, errors::AppError, seafile::SeafileClient};

const N_FRAMES: u32 = 1;

// Seek 10 seconds into the video for the thumbnail.
// A fixed offset avoids a separate ffprobe round-trip (which would consume
// the Seafile one-time download token before ffmpeg can use it).
const PREVIEW_SEEK_SECS: f64 = 10.0;

/// Run ffmpeg with the given download URL; return stderr on failure.
async fn run_ffmpeg(
    _video_id: &str,
    download_url: &str,
    output_pattern: &str,
) -> Result<(), String> {
    let output = Command::new("ffmpeg")
        .arg("-y")
        .arg("-ss")
        .arg(format!("{:.3}", PREVIEW_SEEK_SECS))
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
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(stderr);
    }
    Ok(())
}

/// Returns true when the ffmpeg stderr suggests the download URL is stale or
/// the server rejected the request (HTTP 403 / 404 / 410).
fn is_http_error(stderr: &str) -> bool {
    stderr.contains("HTTP error 403")
        || stderr.contains("HTTP error 404")
        || stderr.contains("HTTP error 410")
}

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

    // Fetch a fresh download URL and attempt ffmpeg extraction.
    let download_url = seafile
        .get_download_url(seafile_path)
        .await
        .map_err(|e| AppError::Internal(format!("seafile download url: {e}")))?;

    let stderr = match run_ffmpeg(video_id, &download_url, &output_pattern).await {
        Ok(()) => {
            // Success – fall through to DB update.
            String::new()
        }
        Err(stderr) => {
            // If the error looks like an expired / rejected download URL, get a
            // fresh URL and retry once.
            if is_http_error(&stderr) {
                tracing::warn!(
                    "ffmpeg HTTP error for {video_id}, retrying with fresh download URL"
                );
                let retry_url = seafile
                    .get_download_url(seafile_path)
                    .await
                    .map_err(|e| AppError::Internal(format!("seafile retry url: {e}")))?;
                match run_ffmpeg(video_id, &retry_url, &output_pattern).await {
                    Ok(()) => String::new(),
                    Err(retry_stderr) => {
                        tracing::error!("ffmpeg retry also failed for {video_id}:\n{retry_stderr}");
                        return Err(AppError::Internal(
                            "ffmpeg exited with non-zero status".to_string(),
                        ));
                    }
                }
            } else {
                tracing::error!("ffmpeg failed for {video_id}:\n{stderr}");
                return Err(AppError::Internal(
                    "ffmpeg exited with non-zero status".to_string(),
                ));
            }
        }
    };

    // Quiet clippy: if we reach here without error the stderr is meaningless.
    let _ = stderr;

    let video_id = video_id.to_string();
    let log_id = video_id.clone();
    let db = db.clone();
    tokio::task::spawn_blocking(move || {
        use crate::db::schema::videos;
        use diesel::prelude::*;
        let mut conn = db.get().map_err(|e| AppError::Internal(e.to_string()))?;
        diesel::update(videos::table.filter(videos::id.eq(&video_id)))
            .set(videos::preview_count.eq(N_FRAMES as i32))
            .execute(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok::<(), AppError>(())
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    tracing::info!("previews generated for {log_id}");
    Ok(())
}