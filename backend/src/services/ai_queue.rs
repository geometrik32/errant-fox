use tokio::sync::mpsc;
use crate::state::AppState;
use crate::api::videos::execute_ai_label_for_video;

pub fn start_ai_queue_processor(state: AppState, mut rx: mpsc::UnboundedReceiver<String>) {
    tokio::spawn(async move {
        while let Some(video_id) = rx.recv().await {
            let db = state.db.clone();
            let vid = video_id.clone();
            let is_still_queued = tokio::task::spawn_blocking(move || {
                use crate::db::schema::videos;
                use diesel::prelude::*;
                if let Ok(mut conn) = db.get() {
                    videos::table
                        .filter(videos::id.eq(&vid))
                        .select(videos::is_queued)
                        .first::<bool>(&mut conn)
                        .unwrap_or(false)
                } else {
                    false
                }
            })
            .await
            .unwrap_or(false);

            if is_still_queued {
                let _ = execute_ai_label_for_video(state.clone(), video_id).await;
            }
        }
    });
}
