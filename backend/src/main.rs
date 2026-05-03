use axum::http::{header, Method};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::{AllowOrigin, CorsLayer};

mod api;
mod config;
mod db;
mod errors;
mod middleware;
mod previews;
mod s3;
mod state;
mod sync;
mod ws;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = config::Config::from_env();

    let s3_client = s3::S3Client::new(
        config.s3_endpoint.clone(),
        config.s3_bucket.clone(),
        config.s3_access_key.clone(),
        config.s3_secret_key.clone(),
        config.s3_region.clone(),
    );

    let db_pool = db::init_pool(&config.database_url);

    let (ws_tx, _ws_rx) = tokio::sync::broadcast::channel::<ws::WsEvent>(256);

    tokio::spawn(sync::run_sync(
        s3_client.clone(),
        db_pool.clone(),
        ws_tx.clone(),
    ));

    let app_state = state::AppState {
        db: db_pool,
        jwt_secret: config.jwt_secret.clone(),
        avatars_dir: config.avatars_dir.clone(),
        previews_dir: config.previews_dir.clone(),
        s3: s3_client,
        ws_hub: ws_tx,
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
