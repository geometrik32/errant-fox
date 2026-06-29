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

    let (ws_tx, _ws_rx) = tokio::sync::broadcast::channel::<services::ws::WsEvent>(256);

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
        seafile: seafile_client,
        ws_hub: ws_tx,
        presence: std::sync::Arc::new(tokio::sync::RwLock::new(services::ws::PresenceRegistry::default())),
        server_port: config.server_port,
        frontend_url: config.frontend_url.clone(),
        vk_notifier: std::sync::Arc::new(services::vk::VkNotificationService::new(config.vk_group_token.clone())),
    };

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
