use std::path::Path;
use std::process::Stdio;

use tokio::process::Command;

use crate::{db::DbPool, errors::AppError};

const N_FRAMES: u32 = 1;

// Seek 10 seconds into the video for the thumbnail.
// A fixed offset avoids a separate ffprobe round-trip (which would consume
// the Seafile one-time download token before ffmpeg can use it).
const PREVIEW_SEEK_SECS: f64 = 10.0;

pub async fn generate_previews(
    video_id: &str,
    download_url: &str,
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
        .arg(&output_pattern)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| AppError::Internal(format!("ffmpeg spawn: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::error!("ffmpeg failed for {video_id}:\n{stderr}");
        return Err(AppError::Internal("ffmpeg exited with non-zero status".to_string()));
    }

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