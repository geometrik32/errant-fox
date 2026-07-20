use axum::http::{header, Method};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::{AllowOrigin, CorsLayer};

mod api;
mod config;
mod db;
mod errors;
mod middleware;
mod services;
mod state;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = config::Config::from_env();

    let seafile_client = services::seafile::SeafileClient::new(
        config.seafile_url.clone(),
        config.seafile_token.clone(),
    );

    let db_pool = db::init_pool(&config.database_url);

    // Reset any stuck is_analyzing flags in the database on startup, and ensure system AI user exists
    let db_pool_clone = db_pool.clone();
    let _ = tokio::task::spawn_blocking(move || {
        use crate::db::schema::{videos, users};
        use crate::db::models::NewUser;
        use diesel::prelude::*;
        if let Ok(mut conn) = db_pool_clone.get() {
            // 1. Reset analyzing & queued state
            let _ = diesel::update(videos::table)
                .set((
                    videos::is_analyzing.eq(false),
                    videos::is_queued.eq(false),
                ))
                .execute(&mut conn);

            // 2. Ensure AI user exists
            let ai_exists = users::table
                .filter(users::id.eq("ai"))
                .first::<crate::db::models::User>(&mut conn)
                .is_ok();

            if !ai_exists {
                let new_ai_user = NewUser {
                    id: "ai".to_string(),
                    username: "ai".to_string(),
                    display_name: "Нейросеть".to_string(),
                    password_hash: "".to_string(),
                    is_admin: false,
                    avatar_path: None,
                    color: Some("#a855f7".to_string()),
                    vk_id: None,
                    role: "user".to_string(),
                };
                let _ = diesel::insert_into(users::table)
                    .values(&new_ai_user)
                    .execute(&mut conn);
            }

            // 3. Recalculate AI status for all videos in database
            if let Ok(video_ids) = videos::table.select(videos::id).load::<String>(&mut conn) {
                for vid in video_ids {
                    let _ = api::bouts::recalculate_video_ai_status(&mut conn, &vid);
                }
            }
        }
    });

    let (ws_tx, _ws_rx) = tokio::sync::broadcast::channel::<services::ws::WsEvent>(256);
    let (ai_queue_tx, ai_queue_rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    tokio::spawn(services::sync::run_sync(
        seafile_client.clone(),
        db_pool.clone(),
        ws_tx.clone(),
        config.previews_dir.clone(),
    ));

    let app_state = state::AppState {
        db: db_pool,
        jwt_secret: config.jwt_secret.clone(),
        avatars_dir: config.avatars_dir.clone(),
        previews_dir: config.previews_dir.clone(),
        transcripts_dir: config.transcripts_dir.clone(),
        seafile: seafile_client,
        ws_hub: ws_tx,
        presence: std::sync::Arc::new(tokio::sync::RwLock::new(services::ws::PresenceRegistry::default())),
        server_port: config.server_port,
        frontend_url: config.frontend_url.clone(),
        vk_notifier: std::sync::Arc::new(services::vk::VkNotificationService::new(config.vk_group_token.clone())),
        vk_app_id: config.vk_app_id.clone(),
        vk_app_secret: config.vk_app_secret.clone(),
        ai_queue_tx,
    };

    services::ai_queue::start_ai_queue_processor(app_state.clone(), ai_queue_rx);

    let origin = config
        .frontend_origin
        .parse::<axum::http::HeaderValue>()
        .expect("FRONTEND_ORIGIN must be a valid HTTP origin");

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::exact(origin))
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);

    let app = api::router(app_state).layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    let listener = TcpListener::bind(addr)
        .await
        .expect("failed to bind to port");

    tracing::info!("listening on {addr}");
    axum::serve(listener, app)
        .await
        .expect("server stopped unexpectedly");
}
