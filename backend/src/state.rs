use std::sync::Arc;

use crate::{db::DbPool, seafile::SeafileClient, ws::WsHub};

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub jwt_secret: String,
    pub avatars_dir: String,
    pub previews_dir: String,
    pub seafile: Arc<SeafileClient>,
    pub ws_hub: WsHub,
}
