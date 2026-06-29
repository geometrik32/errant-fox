use std::env;

pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub seafile_url: String,
    pub seafile_token: String,
    pub previews_dir: String,
    pub avatars_dir: String,
    pub server_port: u16,
    pub frontend_origin: String,
    pub frontend_url: String,
    pub vk_group_token: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        let origin = required("FRONTEND_ORIGIN");
        let url = env::var("FRONTEND_URL").unwrap_or_else(|_| origin.clone());
        Config {
            database_url: required("DATABASE_URL"),
            jwt_secret: required("JWT_SECRET"),
            seafile_url: required("SEAFILE_URL"),
            seafile_token: required("SEAFILE_TOKEN"),
            previews_dir: required("PREVIEWS_DIR"),
            avatars_dir: required("AVATARS_DIR"),
            server_port: required("SERVER_PORT")
                .parse()
                .expect("SERVER_PORT must be a valid port number"),
            frontend_origin: origin,
            frontend_url: url,
            vk_group_token: env::var("VK_GROUP_TOKEN").ok(),
        }
    }
}

fn required(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| panic!("{key} must be set in .env"))
}
