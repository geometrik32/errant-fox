use std::sync::Arc;
use std::time::Duration;

use chrono::NaiveDate;
use diesel::prelude::*;
use regex::Regex;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::db::{models::NewVideo, DbPool};
use super::{seafile::SeafileClient, ws::WsEvent};

pub async fn run_sync(
    seafile: Arc<SeafileClient>,
    db: DbPool,
    ws_tx: broadcast::Sender<WsEvent>,
) {
    let date_re = Regex::new(r"(\d{4}[.\-]\d{2}[.\-]\d{2})").unwrap();
    let mut interval = tokio::time::interval(Duration::from_secs(60));

    loop {
        interval.tick().await;
        if let Err(e) = sync_once(&seafile, &db, &ws_tx, &date_re).await {
            tracing::error!("seafile sync error: {e:#}");
        }
    }
}

async fn sync_once(
    seafile: &SeafileClient,
    db: &DbPool,
    ws_tx: &broadcast::Sender<WsEvent>,
    date_re: &Regex,
) -> anyhow::Result<()> {
    let folders = seafile.list_folders().await?;

    let mut seen_paths: std::collections::HashSet<String> = std::collections::HashSet::new();

    for folder in folders {
        let date_str = match date_re.find(&folder.name) {
            Some(m) => m.as_str().to_string(),
            None => continue,
        };

        let date = match NaiveDate::parse_from_str(&date_str, "%Y.%m.%d")
            .or_else(|_| NaiveDate::parse_from_str(&date_str, "%Y-%m-%d"))
        {
            Ok(d) => d,
            Err(_) => continue,
        };

        let files = match seafile.list_files(&folder.name).await {
            Ok(f) => f,
            Err(e) => {
                tracing::warn!("failed to list '{}': {e}", folder.name);
                continue;
            }
        };

        for file in files {
            // seafile_path: "FolderName/filename.mp4" (no leading slash)
            let seafile_path = format!("{}/{}", folder.name, file.name);
            seen_paths.insert(seafile_path.clone());

            let path_clone = seafile_path.clone();
            let db_clone = db.clone();
            let exists: bool = tokio::task::spawn_blocking(move || {
                use crate::db::schema::videos;
                let mut conn = db_clone.get()?;
                diesel::select(diesel::dsl::exists(
                    videos::table.filter(videos::seafile_path.eq(&path_clone)),
                ))
                .get_result::<bool>(&mut conn)
                .map_err(anyhow::Error::from)
            })
            .await??;

            if exists {
                continue;
            }

            let new_id = Uuid::new_v4().to_string();
            let mut new_video = NewVideo {
                id: new_id.clone(),
                seafile_path: seafile_path.clone(),
                fighter_a_id: None,
                fighter_b_id: None,
                date,
                duration_ms: None,
                preview_count: 0,
                fps: None,
            };

            // Try to extract FPS from the moov atom (first 1 MB of the file)
            match seafile
                .fetch_range(&seafile_path, Some("bytes=0-1048576"))
                .await
            {
                Ok(resp) => match resp.bytes().await {
                    Ok(data) => match super::moov::parse_fps(&data) {
                        Ok(info) => {
                            tracing::info!(
                                "extracted fps={:.2} for {seafile_path}",
                                info.fps
                            );
                            new_video.fps = Some(info.fps);
                        }
                        Err(e) => {
                            tracing::warn!("moov parse failed for {seafile_path}: {e:#}");
                        }
                    },
                    Err(e) => {
                        tracing::warn!("failed to read body for {seafile_path}: {e:#}");
                    }
                },
                Err(e) => {
                    tracing::warn!("fetch_range failed for {seafile_path}: {e:#}");
                }
            }

            let db_clone = db.clone();
            if let Err(e) = tokio::task::spawn_blocking(move || {
                use crate::db::schema::videos;
                let mut conn = db_clone.get()?;
                diesel::insert_into(videos::table)
                    .values(&new_video)
                    .execute(&mut conn)
                    .map_err(anyhow::Error::from)
            })
            .await?
            {
                tracing::error!("failed to insert {seafile_path}: {e}");
                continue;
            }

            let _ = ws_tx.send(WsEvent::NewVideo {
                id: new_id.clone(),
                date: date_str.clone(),
                preview_url: format!("/api/videos/{new_id}/previews/0"),
            });
            tracing::info!("synced new video: {seafile_path}");
        }
    }

    // Delete videos whose seafile_path is no longer present in Seafile
    let seen_vec: Vec<String> = seen_paths.into_iter().collect();
    let db_clone = db.clone();
    let removed: Vec<(String, String)> = tokio::task::spawn_blocking(move || {
        use crate::db::schema::videos;
        let mut conn = db_clone.get()?;
        // Find all videos not in the current seen set
        let stale = videos::table
            .select((videos::id, videos::seafile_path))
            .filter(videos::seafile_path.ne_all(&seen_vec))
            .load::<(String, String)>(&mut conn)
            .map_err(anyhow::Error::from)?;
        if !stale.is_empty() {
            let stale_ids: Vec<&str> = stale.iter().map(|(id, _)| id.as_str()).collect();
            diesel::delete(videos::table.filter(videos::id.eq_any(&stale_ids)))
                .execute(&mut conn)
                .map_err(anyhow::Error::from)?;
        }
        Ok::<_, anyhow::Error>(stale)
    })
    .await??;

    for (id, path) in removed {
        let _ = ws_tx.send(WsEvent::VideoRemoved { id: id.clone() });
        tracing::info!("removed stale video: {path} (id={id})");
    }

    Ok(())
}
