use std::time::Duration;
use tokio::sync::mpsc;
use crate::state::AppState;
use crate::api::videos::execute_ai_label_for_video;

pub fn start_ai_queue_processor(state: AppState, mut rx: mpsc::UnboundedReceiver<String>) {
    tokio::spawn(async move {
        loop {
            // 1. Fetch next queued video directly from DB
            let db = state.db.clone();
            let next_video_id: Option<String> = tokio::task::spawn_blocking(move || {
                use crate::db::schema::videos;
                use diesel::prelude::*;
                if let Ok(mut conn) = db.get() {
                    videos::table
                        .filter(videos::is_queued.eq(true))
                        .order(videos::created_at.asc())
                        .select(videos::id)
                        .first::<String>(&mut conn)
                        .ok()
                } else {
                    None
                }
            })
            .await
            .unwrap_or(None);

            if let Some(video_id) = next_video_id {
                println!("[ai_queue] Picked queued video {} from DB for AI analysis", video_id);
                let _ = execute_ai_label_for_video(state.clone(), video_id).await;
                // Immediately check for next queued item without sleeping
                continue;
            }

            // 2. If no items in DB, wait for a signal on rx OR poll after 3 seconds
            let _ = tokio::time::timeout(Duration::from_secs(3), rx.recv()).await;
        }
    });
}
