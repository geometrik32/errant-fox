use std::sync::Arc;

use crate::{db::DbPool, services::{seafile::SeafileClient, ws::WsHub}};

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub jwt_secret: String,
    pub avatars_dir: String,
    pub previews_dir: String,
    pub seafile: Arc<SeafileClient>,
    pub ws_hub: WsHub,
    pub presence: Arc<tokio::sync::RwLock<crate::services::ws::PresenceRegistry>>,
    pub server_port: u16,
    pub frontend_url: String,
    pub vk_notifier: Arc<crate::services::vk::VkNotificationService>,
    pub vk_app_id: Option<String>,
    pub vk_app_secret: Option<String>,
}
