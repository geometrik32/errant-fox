pub mod auth;
pub mod bouts;
pub mod comments;
pub mod techniques;
pub mod users;
pub mod videos;

use axum::{
    routing::{delete, get, patch, post},
    Router,
};

use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        // Auth
        .route("/api/auth/login", post(auth::login))
        // Current user profile
        .route("/api/users/me", get(auth::get_me).patch(users::patch_me))
        .route("/api/users/me/avatar", post(users::upload_avatar))
        .route("/api/users/{id}/avatar", get(users::get_avatar))
        // Fighters
        .route("/api/fighters", get(users::list_fighters))
        .route("/api/fighters/{id}/bouts", get(users::fighter_bouts))
        // Admin — user management
        .route("/api/admin/users", post(users::create_user))
        .route(
            "/api/admin/users/{id}",
            patch(users::patch_admin_user).delete(users::delete_user),
        )
        .route("/api/admin/users/{id}/avatar", post(users::upload_avatar_for))
        // Techniques
        .route("/api/techniques", get(techniques::list_techniques))
        .route("/api/admin/techniques", post(techniques::create_technique))
        .route(
            "/api/admin/techniques/{id}",
            patch(techniques::patch_technique).delete(techniques::delete_technique),
        )
        // Videos
        .route("/api/videos", get(videos::list_videos))
        .route(
            "/api/videos/{id}",
            get(videos::get_video).patch(videos::patch_video),
        )
        .route("/api/videos/{id}/stream", get(videos::stream_video))
        .route("/api/videos/{id}/download", get(videos::download_video))
        .route(
            "/api/videos/{id}/previews/{frame}",
            get(videos::get_preview_frame),
        )
        .route(
            "/api/videos/{id}/previews/regenerate",
            post(videos::regenerate_preview),
        )
        // Bouts
        .route("/api/bouts", post(bouts::post_bout))
        .route(
            "/api/bouts/{id}",
            patch(bouts::patch_bout).delete(bouts::delete_bout),
        )
        .route("/api/bouts/{id}/download", get(bouts::download_bout))
        .route("/api/bouts/{id}/history", get(bouts::get_bout_history))
        // Comments
        .route("/api/comments", post(comments::post_comment))
        .route("/api/comments/search", get(comments::search_comments))
        .route(
            "/api/comments/{id}",
            patch(comments::patch_comment).delete(comments::delete_comment),
        )
        .route(
            "/api/comments/{id}/react",
            post(comments::react_comment).delete(comments::delete_react),
        )
        // WebSocket
        .route("/ws", get(crate::services::ws::ws_handler))
        .with_state(state)
}
