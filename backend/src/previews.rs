use std::path::Path;
use std::process::Stdio;

use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::{db::DbPool, errors::AppError, seafile::SeafileClient};

const N_FRAMES: u32 = 1;

// Seek 10 seconds into the video for the thumbnail.
const PREVIEW_SEEK_SECS: f64 = 10.0;

// Limit how much video data we download through reqwest (≈ 10 s at ~8 Mbps).
// Once ffmpeg has decoded enough frames to hit PREVIEW_SEEK_SECS it will exit,
// so we cap the pipe to avoid downloading the whole file.
const MAX_DOWNLOAD_BYTES: u64 = 20_000_000;

/// Download video data through reqwest and pipe it to ffmpeg's stdin.
///
/// We use reqwest instead of letting ffmpeg fetch the URL directly because
/// ffmpeg's internal HTTP client (Lavf) can make multiple probe requests
/// that exhaust Seafile's one-time download token, resulting in HTTP 403.
async fn run_ffmpeg_piped(
    video_id: &str,
    download_url: &str,
    output_pattern: &str,
) -> Result<(), String> {
    // Start downloading via reqwest (proven to work with Seafile).
    let mut response = reqwest::get(download_url)
        .await
        .map_err(|e| format!("reqwest GET: {e}"))?;

    let status = response.status();
    if !status.is_success() {
        return Err(format!("reqwest returned HTTP {}", status.as_u16()));
    }
    tracing::debug!("reqwest GET {} for {video_id}", status);

    // Spawn ffmpeg reading from stdin.
    let mut child = Command::new("ffmpeg")
        .arg("-y")
        .arg("-ss")
        .arg(format!("{:.3}", PREVIEW_SEEK_SECS))
        .arg("-i")
        .arg("pipe:0")
        .arg("-vf")
        .arg("scale=480:-1")
        .arg("-vframes")
        .arg("1")
        .arg("-start_number")
        .arg("0")
        .arg(output_pattern)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("ffmpeg spawn: {e}"))?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| "ffmpeg stdin already taken".to_string())?;

    // Pipe reqwest → ffmpeg, stopping early once we've sent enough data
    // for ffmpeg to decode through PREVIEW_SEEK_SECS.
    let mut sent: u64 = 0;
    loop {
        let chunk = response
            .chunk()
            .await
            .map_err(|e| format!("download chunk: {e}"))?;
        match chunk {
            Some(bytes) => {
                stdin
                    .write_all(&bytes)
                    .await
                    .map_err(|e| format!("pipe write: {e}"))?;
                sent += bytes.len() as u64;
                if sent >= MAX_DOWNLOAD_BYTES {
                    break;
                }
            }
            None => break, // EOF
        }
    }
    // Close stdin so ffmpeg knows input is done.
    drop(stdin);

    let output = child
        .wait_with_output()
        .await
        .map_err(|e| format!("ffmpeg wait: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(stderr);
    }
    Ok(())
}

/// Returns true when the ffmpeg stderr suggests a permanent HTTP-level
/// rejection that won't be fixed by a fresh download URL.
fn is_permanent_http_error(stderr: &str) -> bool {
    stderr.contains("HTTP error 403")
        || stderr.contains("HTTP error 404")
        || stderr.contains("HTTP error 410")
        || stderr.contains("reqwest returned HTTP 403")
        || stderr.contains("reqwest returned HTTP 404")
        || stderr.contains("reqwest returned HTTP 410")
}

/// Sentinel value written to `videos.preview_count` when both the initial
/// ffmpeg attempt and the retry fail with an HTTP error (403/404/410).
/// This prevents the server from re-spawning ffmpeg on every gallery load
/// for videos whose Seafile download links are permanently broken.
const PREVIEW_FAILED: i32 = -1;

/// Update the preview_count column for a video.  `value` is typically
/// `N_FRAMES` (success) or `PREVIEW_FAILED` (permanent HTTP error).
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

    match run_ffmpeg_piped(video_id, &download_url, &output_pattern).await {
        Ok(()) => {
            set_preview_count(db, video_id, N_FRAMES as i32).await?;
            tracing::info!("previews generated for {video_id}");
            Ok(())
        }
        Err(first_stderr) => {
            if !is_permanent_http_error(&first_stderr) {
                // Non-HTTP error (e.g. corrupt video, codec issue) — don't retry.
                tracing::error!("ffmpeg failed for {video_id}:\n{first_stderr}");
                return Err(AppError::Internal(
                    "ffmpeg exited with non-zero status".to_string(),
                ));
            }

            // HTTP error — the download URL may have expired. Fetch a fresh one
            // and retry once.
            tracing::warn!(
                "ffmpeg HTTP error for {video_id}, retrying with fresh download URL \
                 (first attempt: {:.200})",
                first_stderr.lines().find(|l| l.contains("HTTP error") || l.contains("reqwest returned HTTP")).unwrap_or("")
            );

            let retry_url = match seafile.get_download_url(seafile_path).await {
                Ok(url) => url,
                Err(e) => {
                    tracing::error!("seafile retry url failed for {video_id}: {e}");
                    return Err(AppError::Internal(format!("seafile retry url: {e}")));
                }
            };

            match run_ffmpeg_piped(video_id, &retry_url, &output_pattern).await {
                Ok(()) => {
                    set_preview_count(db, video_id, N_FRAMES as i32).await?;
                    tracing::info!("previews generated for {video_id} (after retry)");
                    Ok(())
                }
                Err(retry_stderr) => {
                    if is_permanent_http_error(&retry_stderr) {
                        // Both attempts got HTTP errors — the file is permanently
                        // inaccessible.  Mark it so we never retry again.
                        tracing::error!(
                            "ffmpeg double HTTP failure for {video_id}, \
                             marking as permanently failed:\n{retry_stderr}"
                        );
                        set_preview_count(db, video_id, PREVIEW_FAILED).await?;
                        // Return Ok — the failure has been persisted; no need to
                        // bubble up an error.
                        Ok(())
                    } else {
                        tracing::error!("ffmpeg retry also failed for {video_id}:\n{retry_stderr}");
                        Err(AppError::Internal(
                            "ffmpeg exited with non-zero status".to_string(),
                        ))
                    }
                }
            }
        }
    }
}