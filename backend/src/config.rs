use std::env;

pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub s3_endpoint: String,
    pub s3_bucket: String,
    pub s3_access_key: String,
    pub s3_secret_key: String,
    pub s3_region: String,
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
            s3_endpoint: required("S3_ENDPOINT"),
            s3_bucket: required("S3_BUCKET"),
            s3_access_key: required("S3_ACCESS_KEY"),
            s3_secret_key: required("S3_SECRET_KEY"),
            s3_region: env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
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
