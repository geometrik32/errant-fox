use std::path::Path;
use std::process::Stdio;

use tokio::process::Command;

use crate::{db::DbPool, errors::AppError};

const N_FRAMES: u32 = 10;

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

    let output_pattern = output_dir
        .join("%d.jpg")
        .to_string_lossy()
        .into_owned();

    // Get video duration to compute even frame interval
    let duration = get_duration(download_url).await;
    // interval = duration / N so frames land at 0, interval, 2*interval, ...
    let interval = (duration / N_FRAMES as f64).max(0.5);
    // fps=1/interval means one output frame every `interval` seconds
    let vf = format!("fps=1/{interval:.3},scale=480:-1");

    let output = Command::new("ffmpeg")
        .arg("-y")
        .arg("-i")
        .arg(download_url)
        .arg("-vf")
        .arg(&vf)
        .arg("-frames:v")
        .arg(N_FRAMES.to_string())
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

    tracing::info!("previews generated for {log_id} (duration={duration:.1}s, interval={interval:.3}s)");
    Ok(())
}
