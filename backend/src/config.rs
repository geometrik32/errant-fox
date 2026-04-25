use std::env;

pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub seafile_url: String,
    pub seafile_token: String,
    pub seafile_repo_id: String,
    pub previews_dir: String,
    pub avatars_dir: String,
    pub server_port: u16,
    pub frontend_origin: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        Config {
            database_url: required("DATABASE_URL"),
            jwt_secret: required("JWT_SECRET"),
            seafile_url: required("SEAFILE_URL"),
            seafile_token: required("SEAFILE_TOKEN"),
            seafile_repo_id: required("SEAFILE_REPO_ID"),
            previews_dir: required("PREVIEWS_DIR"),
            avatars_dir: required("AVATARS_DIR"),
            server_port: required("SERVER_PORT")
                .parse()
                .expect("SERVER_PORT must be a valid port number"),
            frontend_origin: required("FRONTEND_ORIGIN"),
        }
    }
}

fn required(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| panic!("{key} must be set in .env"))
}
