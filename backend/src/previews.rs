use std::path::Path;
use std::process::Stdio;

use tokio::process::Command;

use crate::{db::DbPool, errors::AppError};

const N_FRAMES: u32 = 1;

async fn get_duration(url: &str) -> f64 {
    let out = Command::new("ffprobe")
        .args([
            "-v", "error",
            "-show_entries", "format=duration",
            "-of", "csv=p=0",
            url,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .await;
    out.ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.trim().parse::<f64>().ok())
        .unwrap_or(60.0)
}

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

    // We'll use 1.jpg as the filename (ffmpeg %d starts from 1)
    let output_pattern = output_dir
        .join("%d.jpg")
        .to_string_lossy()
        .into_owned();

    // Get video duration to seek to the middle
    let duration = get_duration(download_url).await;
    let seek_time = duration / 2.0;

    let output = Command::new("ffmpeg")
        .arg("-y")
        .arg("-i")
        .arg(download_url)
        .arg("-ss")
        .arg(format!("{:.3}", seek_time))
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

    tracing::info!("previews generated for {log_id} (duration={duration:.1}s)");
    Ok(())
}