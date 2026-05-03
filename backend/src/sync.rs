use std::sync::Arc;
use std::time::Duration;

use chrono::NaiveDate;
use diesel::prelude::*;
use regex::Regex;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::{
    db::{models::NewVideo, DbPool},
    s3::S3Client,
    ws::WsEvent,
};

pub async fn run_sync(
    s3: Arc<S3Client>,
    db: DbPool,
    ws_tx: broadcast::Sender<WsEvent>,
) {
    let date_re = Regex::new(r"(\d{4}[.\-]\d{2}[.\-]\d{2})").unwrap();
    let mut interval = tokio::time::interval(Duration::from_secs(60));

    loop {
        interval.tick().await;
        if let Err(e) = sync_once(&s3, &db, &ws_tx, &date_re).await {
            tracing::error!("s3 sync error: {e:#}");
        }
    }
}

async fn sync_once(
    s3: &S3Client,
    db: &DbPool,
    ws_tx: &broadcast::Sender<WsEvent>,
    date_re: &Regex,
) -> anyhow::Result<()> {
    let keys = s3.list_objects().await?;

    for key in keys {
        // Only process video files
        let lower = key.to_lowercase();
        if !lower.ends_with(".mp4") && !lower.ends_with(".mkv") && !lower.ends_with(".mov") && !lower.ends_with(".avi") {
            continue;
        }

        // Extract date from the key (anywhere in the path)
        let date_str = match date_re.find(&key) {
            Some(m) => m.as_str().to_string(),
            None => continue,
        };

        let date = match NaiveDate::parse_from_str(&date_str, "%Y.%m.%d")
            .or_else(|_| NaiveDate::parse_from_str(&date_str, "%Y-%m-%d"))
        {
            Ok(d) => d,
            Err(_) => continue,
        };

        let key_clone = key.clone();
        let db_clone = db.clone();
        let exists: bool = tokio::task::spawn_blocking(move || {
            use crate::db::schema::videos;
            let mut conn = db_clone.get()?;
            diesel::select(diesel::dsl::exists(
                videos::table.filter(videos::seafile_path.eq(&key_clone)),
            ))
            .get_result::<bool>(&mut conn)
            .map_err(anyhow::Error::from)
        })
        .await??;

        if exists {
            continue;
        }

        let new_id = Uuid::new_v4().to_string();
        let new_video = NewVideo {
            id: new_id.clone(),
            seafile_path: key.clone(),
            fighter_a_id: None,
            fighter_b_id: None,
            date,
            duration_ms: None,
            preview_count: 0,
        };

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
            tracing::error!("failed to insert {key}: {e}");
            continue;
        }

        let _ = ws_tx.send(WsEvent::NewVideo {
            id: new_id.clone(),
            date: date_str.clone(),
            preview_url: format!("/api/videos/{new_id}/previews/0"),
        });
        tracing::info!("synced new video: {key}");
    }

    Ok(())
}
