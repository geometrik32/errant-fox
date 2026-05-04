use std::path::Path;
use std::process::Stdio;

use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::{db::DbPool, errors::AppError, seafile::SeafileClient};

const SEEK_SECS: f64 = 10.0;
const PREVIEW_FAILED: i32 = -1;

/// Download `url` to `tmp_path` using streaming (no full-file RAM buffer).
async fn download_to_file(video_id: &str, url: &str, tmp_path: &Path) -> Result<(), String> {
    let response = reqwest::get(url)
        .await
        .map_err(|e| format!("GET: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status().as_u16()));
    }

    let size_hint = response.content_length();
    tracing::info!(
        "[{video_id}] downloading  size={:.0}MB",
        size_hint.unwrap_or(0) as f64 / 1_048_576.0
    );

    let mut file = tokio::fs::File::create(tmp_path)
        .await
        .map_err(|e| format!("create tmp: {e}"))?;

    let mut stream = response.bytes_stream();
    let mut total: u64 = 0;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("stream: {e}"))?;
        total += chunk.len() as u64;
        file.write_all(&chunk).await.map_err(|e| format!("write: {e}"))?;
    }

    tracing::info!("[{video_id}] downloaded {total}B ({:.1}MB)", total as f64 / 1_048_576.0);
    Ok(())
}

/// Extract one frame at SEEK_SECS from a local file path.
async fn extract_frame(video_id: &str, input: &str, output: &str) -> Result<(), String> {
    tracing::info!("[{video_id}] ffmpeg  seek={SEEK_SECS}s  input={input}");

    let result = Command::new("ffmpeg")
        .arg("-y")
        .arg("-ss").arg(format!("{:.3}", SEEK_SECS))
        .arg("-i").arg(input)
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
        tracing::warn!("[{video_id}] ffmpeg FAILED (exit={code}):\n{stderr:.600}");
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
) -> Result<(), AppError> {
    tracing::info!("[{video_id}] generate_previews  path={seafile_path}");

    let output_dir = previews_dir.join(video_id);
    tokio::fs::create_dir_all(&output_dir)
        .await
        .map_err(|e| AppError::Internal(format!("mkdir: {e}")))?;

    let tmp_path = output_dir.join("video.tmp");
    let output_pattern = output_dir.join("%d.jpg").to_string_lossy().into_owned();

    let result: Result<(), String> = async {
        let url = seafile
            .get_download_url(seafile_path)
            .await
            .map_err(|e| format!("seafile url: {e}"))?;

        download_to_file(video_id, &url, &tmp_path).await?;

        let tmp_str = tmp_path.to_string_lossy().into_owned();
        extract_frame(video_id, &tmp_str, &output_pattern).await?;

        Ok(())
    }
    .await;

    let _ = tokio::fs::remove_file(&tmp_path).await;

    match result {
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
