use std::path::Path;
use std::process::Stdio;

use tokio::process::Command;

use crate::{db::DbPool, errors::AppError};

/// Generate 10 evenly-spaced preview frames (0.jpg … 9.jpg) via FFmpeg,
/// then update preview_count=10 in the database.
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

    // %d is 0-based thanks to -start_number 0
    let output_pattern = output_dir
        .join("%d.jpg")
        .to_string_lossy()
        .into_owned();

    let status = Command::new("ffmpeg")
        .arg("-y")
        .arg("-i")
        .arg(download_url)
        .arg("-vf")
        // select every 300th frame (≈10 s at 30 fps) then scale to 480 px wide
        .arg("select=not(mod(n\\,300)),scale=480:-1")
        .arg("-vsync")
        .arg("0")
        .arg("-frames:v")
        .arg("10")
        .arg("-start_number")
        .arg("0")
        .arg(&output_pattern)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .map_err(|e| AppError::Internal(format!("ffmpeg spawn: {e}")))?;

    if !status.success() {
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
            .set(videos::preview_count.eq(10))
            .execute(&mut conn)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok::<(), AppError>(())
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    tracing::info!("previews generated for {log_id}");
    Ok(())
}
