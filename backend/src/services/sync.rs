use std::sync::Arc;

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
    previews_dir: String,
) {
    tracing::info!("running initial seafile sync...");
    match import_new_videos_once(&seafile, &db, &ws_tx).await {
        Ok(_) => {
            tracing::info!("initial seafile sync completed successfully");
        }
        Err(e) => {
            tracing::error!("initial seafile sync error: {e:#}");
        }
    }
}

pub async fn check_stale_videos(
    seafile: &SeafileClient,
    db: &DbPool,
) -> anyhow::Result<Vec<crate::db::models::Video>> {
    let date_re = Regex::new(r"(\d{4}[.\-]\d{2}[.\-]\d{2})").unwrap();
    let folders = seafile.list_folders().await?;

    let mut seen_paths = std::collections::HashSet::new();

    for folder in folders {
        if !date_re.is_match(&folder.name) {
            continue;
        }

        let files = seafile.list_files(&folder.name).await?;
        for file in files {
            if file.entry_type == "file" {
                let seafile_path = format!("{}/{}", folder.name, file.name);
                seen_paths.insert(seafile_path);
            }
        }
    }

    let db_clone = db.clone();
    let stale_videos = tokio::task::spawn_blocking(move || {
        use crate::db::schema::videos;
        let mut conn = db_clone.get()?;
        
        let all_videos = videos::table
            .load::<crate::db::models::Video>(&mut conn)
            .map_err(anyhow::Error::from)?;
            
        let stale: Vec<crate::db::models::Video> = all_videos
            .into_iter()
            .filter(|v| !seen_paths.contains(&v.seafile_path))
            .collect();
            
        Ok::<_, anyhow::Error>(stale)
    })
    .await??;

    Ok(stale_videos)
}

pub async fn delete_videos_cascade(
    db: &DbPool,
    previews_dir: &str,
    stale_ids: &[String],
) -> anyhow::Result<()> {
    if stale_ids.is_empty() {
        return Ok(());
    }

    let stale_ids_clone = stale_ids.to_vec();
    let db_clone = db.clone();
    
    tokio::task::spawn_blocking(move || {
        use crate::db::schema::{bouts, comment_reactions, comments, videos};
        let mut conn = db_clone.get()?;
        
        let ids: Vec<&str> = stale_ids_clone.iter().map(|s| s.as_str()).collect();
        
        conn.transaction::<_, anyhow::Error, _>(|conn| {
            // Delete comment reactions for comments of these videos
            let comment_ids = comments::table
                .select(comments::id)
                .filter(comments::video_id.eq_any(&ids))
                .load::<i32>(conn)?;
            if !comment_ids.is_empty() {
                diesel::delete(comment_reactions::table.filter(comment_reactions::comment_id.eq_any(&comment_ids)))
                    .execute(conn)?;
            }

            // Delete comments
            diesel::delete(comments::table.filter(comments::video_id.eq_any(&ids)))
                .execute(conn)?;

            // Delete bouts
            diesel::delete(bouts::table.filter(bouts::video_id.eq_any(&ids)))
                .execute(conn)?;

            // Delete videos
            diesel::delete(videos::table.filter(videos::id.eq_any(&ids)))
                .execute(conn)?;

            Ok(())
        })
    })
    .await??;

    // Delete preview dirs
    for id in stale_ids {
        let dir = std::path::Path::new(previews_dir).join(id);
        if dir.exists() {
            if let Err(e) = tokio::fs::remove_dir_all(&dir).await {
                tracing::error!("failed to delete preview dir for {id}: {e}");
            } else {
                tracing::info!("deleted preview dir for {id}");
            }
        }
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub struct ParsedFolderInfo {
    pub date: NaiveDate,
    pub is_tournament: bool,
    pub tournament_name: Option<String>,
}

pub fn parse_folder_name(name: &str) -> Option<ParsedFolderInfo> {
    let date_re = Regex::new(r"(\d{4}[.\-]\d{2}[.\-]\d{2})").ok()?;
    let date_match = date_re.find(name)?;
    let date_str = date_match.as_str();
    let date = NaiveDate::parse_from_str(date_str, "%Y.%m.%d")
        .or_else(|_| NaiveDate::parse_from_str(date_str, "%Y-%m-%d"))
        .ok()?;

    let tour_re = Regex::new(r"(?i)(\d{4}[.\-]\d{2}[.\-]\d{2})[ _\-\[]*(?:турнир|tournament)[\] _\-]*(.*)").ok()?;
    if let Some(caps) = tour_re.captures(name) {
        let raw_name = caps.get(2).map(|m| m.as_str().trim()).unwrap_or("");
        let clean_name = raw_name
            .trim_start_matches(|c| c == '-' || c == '_' || c == ' ' || c == ']' || c == ':')
            .trim();
        let tournament_name = if clean_name.is_empty() {
            None
        } else {
            Some(clean_name.to_string())
        };
        Some(ParsedFolderInfo {
            date,
            is_tournament: true,
            tournament_name,
        })
    } else {
        Some(ParsedFolderInfo {
            date,
            is_tournament: false,
            tournament_name: None,
        })
    }
}

pub async fn import_new_videos(
    seafile: &SeafileClient,
    db: &DbPool,
    ws_tx: &broadcast::Sender<WsEvent>,
) -> anyhow::Result<Vec<String>> {
    import_new_videos_once(seafile, db, ws_tx).await
}

async fn import_new_videos_once(
    seafile: &SeafileClient,
    db: &DbPool,
    ws_tx: &broadcast::Sender<WsEvent>,
) -> anyhow::Result<Vec<String>> {
    let folders = seafile.list_folders().await?;
    let mut imported_ids = Vec::new();

    for folder in folders {
        let folder_info = match parse_folder_name(&folder.name) {
            Some(info) => info,
            None => continue,
        };

        let files = match seafile.list_files(&folder.name).await {
            Ok(f) => f,
            Err(e) => {
                tracing::warn!("failed to list '{}': {e}", folder.name);
                continue;
            }
        };

        // Загрузим все существующие в БД seafile_path для этой папки
        let folder_prefix = format!("{}/", folder.name);
        let db_clone = db.clone();
        let folder_prefix_clone = folder_prefix.clone();
        let existing_paths: std::collections::HashSet<String> = tokio::task::spawn_blocking(move || {
            use crate::db::schema::videos;
            let mut conn = db_clone.get()?;
            let paths = videos::table
                .select(videos::seafile_path)
                .filter(videos::seafile_path.like(format!("{}%", folder_prefix_clone)))
                .load::<String>(&mut conn)?;
            Ok::<_, anyhow::Error>(paths.into_iter().collect())
        })
        .await??;

        if folder_info.is_tournament && !existing_paths.is_empty() {
            let folder_prefix_update = folder_prefix.clone();
            let tour_name = folder_info.tournament_name.clone();
            let db_update = db.clone();
            let _ = tokio::task::spawn_blocking(move || {
                use crate::db::schema::videos;
                let mut conn = db_update.get()?;
                diesel::update(videos::table.filter(videos::seafile_path.like(format!("{}%", folder_prefix_update))))
                    .set((
                        videos::is_tournament.eq(true),
                        videos::tournament_name.eq(tour_name),
                    ))
                    .execute(&mut conn)?;
                Ok::<_, anyhow::Error>(())
            })
            .await;
        }

        for file in files {
            if file.entry_type != "file" {
                continue;
            }
            let seafile_path = format!("{}/{}", folder.name, file.name);
            
            if existing_paths.contains(&seafile_path) {
                continue;
            }

            let new_id = Uuid::new_v4().to_string();
            let mut new_video = NewVideo {
                id: new_id.clone(),
                seafile_path: seafile_path.clone(),
                fighter_a_id: None,
                fighter_b_id: None,
                date: folder_info.date,
                duration_ms: None,
                preview_count: 0,
                fps: None,
                is_tournament: folder_info.is_tournament,
                tournament_name: folder_info.tournament_name.clone(),
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
                date: folder_info.date.format("%Y-%m-%d").to_string(),
                preview_url: format!("/api/videos/{new_id}/previews/0"),
                is_tournament: folder_info.is_tournament,
                tournament_name: folder_info.tournament_name.clone(),
            });
            tracing::info!("synced new video: {seafile_path}");
            imported_ids.push(new_id);
        }
    }

    Ok(imported_ids)
}
