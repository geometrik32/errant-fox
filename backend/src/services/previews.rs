use std::path::Path;
use std::process::Stdio;

use tokio::process::Command;

use crate::{db::DbPool, errors::AppError};
use super::seafile::SeafileClient;

const FAKE_USER_AGENT: &str = "Mozilla/5.0";
const PREVIEW_FAILED: i32 = -1;

async fn extract_frame(video_id: &str, url: &str, output: &str) -> Result<(), String> {
    tracing::info!("[{video_id}] ffmpeg extract first frame");

    let result = Command::new("ffmpeg")
        .arg("-y")
        .arg("-user_agent").arg(FAKE_USER_AGENT)
        .arg("-i").arg(url)
        .arg("-vf").arg("scale=480:-1")
        .arg("-vframes").arg("1")
        .arg("-start_number").arg("0")
        .arg(output)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("ffmpeg spawn: {e}"))?;

    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        let code = result.status.code().unwrap_or(-1);
        tracing::warn!("[{video_id}] ffmpeg FAILED (exit={code}):\n{stderr:.500}");
        return Err(format!("ffmpeg exit={code}"));
    }

    let frame_path = output.replacen("%d", "0", 1);
    let size = tokio::fs::metadata(&frame_path)
        .await
        .map(|m| m.len())
        .unwrap_or(0);

    if size == 0 {
        return Err("ffmpeg produced empty frame".to_string());
    }

    tracing::info!("[{video_id}] ffmpeg OK  frame={size}B");
    Ok(())
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

pub async fn generate_previews(
    video_id: &str,
    seafile: &SeafileClient,
    seafile_path: &str,
    previews_dir: &Path,
    db: &DbPool,
    server_port: u16,
) -> Result<(), AppError> {
    tracing::info!("[{video_id}] generate_previews  path={seafile_path}");

    let output_dir = previews_dir.join(video_id);
    tokio::fs::create_dir_all(&output_dir)
        .await
        .map_err(|e| AppError::Internal(format!("mkdir: {e}")))?;

    let output_pattern = output_dir.join("%d.jpg").to_string_lossy().into_owned();

    let local_stream_url = format!("http://127.0.0.1:{}/api/videos/{}/stream", server_port, video_id);

    match extract_frame(video_id, &local_stream_url, &output_pattern).await {
        Ok(()) => {
            set_preview_count(db, video_id, 1).await?;
            tracing::info!("[{video_id}] DONE");
        }
        Err(e) => {
            tracing::error!("[{video_id}] FAILED: {e}");
            set_preview_count(db, video_id, PREVIEW_FAILED).await?;
        }
    }

    Ok(())
}
