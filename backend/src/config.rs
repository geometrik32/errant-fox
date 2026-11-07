use std::env;

pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub seafile_url: String,
    pub seafile_internal_url: Option<String>,
    pub seafile_token: String,
    pub previews_dir: String,
    pub avatars_dir: String,
    pub transcripts_dir: String,
    pub temp_dir: String,
    pub server_port: u16,
    pub frontend_origin: String,
    pub frontend_url: String,
    pub vk_group_token: Option<String>,
    pub vk_app_id: Option<String>,
    pub vk_app_secret: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        let origin = required("FRONTEND_ORIGIN");
        let url = env::var("FRONTEND_URL").unwrap_or_else(|_| origin.clone());
        Config {
            database_url: resolve_data_path("DATABASE_URL"),
            jwt_secret: required("JWT_SECRET"),
            seafile_url: required("SEAFILE_URL"),
            seafile_internal_url: env::var("SEAFILE_INTERNAL_URL")
                .ok()
                .filter(|s| !s.trim().is_empty()),
            seafile_token: required("SEAFILE_TOKEN"),
            previews_dir: resolve_data_path("PREVIEWS_DIR"),
            avatars_dir: resolve_data_path("AVATARS_DIR"),
            transcripts_dir: get_transcripts_dir(),
            temp_dir: get_temp_dir(),
            server_port: required("SERVER_PORT")
                .parse()
                .expect("SERVER_PORT must be a valid port number"),
            frontend_origin: origin,
            frontend_url: url,
            vk_group_token: env::var("VK_GROUP_TOKEN").ok(),
            vk_app_id: env::var("VK_APP_ID").ok(),
            vk_app_secret: env::var("VK_APP_SECRET").ok(),
        }
    }
}

fn resolve_data_path(key: &str) -> String {
    let val = required(key);
    if std::path::Path::new(&val).exists() {
        val
    } else if val.starts_with("/data") {
        let relative = val.trim_start_matches('/');
        if std::path::Path::new(relative).exists() {
            relative.to_string()
        } else {
            val
        }
    } else {
        val
    }
}

fn get_temp_dir() -> String {
    if let Ok(dir) = env::var("TEMP_DIR") {
        dir
    } else if std::path::Path::new("/data").exists() {
        "/data/temp".to_string()
    } else {
        "data/temp".to_string()
    }
}

fn get_transcripts_dir() -> String {
    if let Ok(dir) = env::var("TRANSCRIPTS_DIR") {
        dir
    } else if std::path::Path::new("/data").exists() {
        "/data/transcripts".to_string()
    } else {
        "data/transcripts".to_string()
    }
}

fn required(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| panic!("{key} must be set in .env"))
}
