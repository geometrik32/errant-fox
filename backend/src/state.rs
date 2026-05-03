use std::sync::Arc;

use crate::{db::DbPool, s3::S3Client, ws::WsHub};

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub jwt_secret: String,
    pub avatars_dir: String,
    pub previews_dir: String,
    pub s3: Arc<S3Client>,
    pub ws_hub: WsHub,
}
