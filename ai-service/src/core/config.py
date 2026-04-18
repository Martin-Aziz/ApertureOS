from functools import lru_cache

from pydantic import Field
from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    app_env: str = "development"
    log_level: str = "INFO"
    ai_shared_secret: str = (
        "development-service-secret-change-before-production"
    )
    ai_max_requests_per_minute: int = Field(default=100, ge=1)
    ai_max_image_bytes: int = Field(default=8_000_000, ge=1)

    model_config = SettingsConfigDict(
        env_file=".env",
        env_file_encoding="utf-8",
        extra="ignore",
    )


@lru_cache(maxsize=1)
def get_settings() -> Settings:
    return Settings()
