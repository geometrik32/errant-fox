from pydantic_settings import BaseSettings
from pydantic import Field

class Settings(BaseSettings):
    database_url: str = Field(
        default="postgresql://playnite_user:playnite_password@localhost:5432/playnite_db",
        env="DATABASE_URL"
    )
    secret_key: str = Field(
        default="SUPER_SECRET_PLAYNITE_KEY_CHANGE_THIS_IN_PRODUCTION",
        env="SECRET_KEY"
    )
    algorithm: str = "HS256"
    access_token_expire_minutes: int = 60 * 24 * 30  # 30 days

    # MinIO / S3 Settings
    minio_endpoint: str = Field(default="localhost:9000", env="MINIO_ENDPOINT")
    minio_access_key: str = Field(default="minio_admin", env="MINIO_ACCESS_KEY")
    minio_secret_key: str = Field(default="minio_password", env="MINIO_SECRET_KEY")
    minio_secure: bool = False
    
    bucket_games: str = "playnite-games"
    bucket_saves: str = "playnite-saves"

    class Config:
        env_file = ".env"
        env_file_encoding = "utf-8"

settings = Settings()
