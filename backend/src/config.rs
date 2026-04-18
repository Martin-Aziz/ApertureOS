use std::env;
use std::net::SocketAddr;

use thiserror::Error;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub bind_addr: SocketAddr,
    pub frontend_origin: String,
    pub database_url: String,
    pub redis_url: String,
    pub jwt_access_secret: String,
    pub jwt_refresh_secret: String,
    pub jwt_access_ttl_seconds: i64,
    pub jwt_refresh_ttl_seconds: i64,
    pub demo_admin_email: String,
    pub demo_admin_password: String,
    pub ai_shared_secret: String,
    pub ai_service_base_url: String,
    pub ai_request_timeout_seconds: i64,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("invalid socket address in BACKEND_BIND: {0}")]
    InvalidSocketAddress(String),
    #[error("invalid integer value for {name}: {value}")]
    InvalidInteger { name: String, value: String },
    #[error("invalid secret value for {0}: must be at least 32 characters")]
    WeakSecret(&'static str),
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let bind_addr_raw = read_env("BACKEND_BIND", "0.0.0.0:8080");
        let bind_addr = bind_addr_raw
            .parse::<SocketAddr>()
            .map_err(|_| ConfigError::InvalidSocketAddress(bind_addr_raw))?;

        let jwt_access_secret =
            read_env("JWT_ACCESS_SECRET", "development-access-secret-change-before-production-1");
        let jwt_refresh_secret =
            read_env("JWT_REFRESH_SECRET", "development-refresh-secret-change-before-production-1");

        if jwt_access_secret.len() < 32 {
            return Err(ConfigError::WeakSecret("JWT_ACCESS_SECRET"));
        }

        if jwt_refresh_secret.len() < 32 {
            return Err(ConfigError::WeakSecret("JWT_REFRESH_SECRET"));
        }

        Ok(Self {
            bind_addr,
            frontend_origin: read_env("FRONTEND_ORIGIN", "http://localhost:5173"),
            database_url: read_env(
                "DATABASE_URL",
                "postgresql://aperture:aperture@localhost:5432/aperture",
            ),
            redis_url: read_env("REDIS_URL", "redis://localhost:6379"),
            jwt_access_secret,
            jwt_refresh_secret,
            jwt_access_ttl_seconds: read_env_i64("JWT_ACCESS_TTL_SECONDS", 900)?,
            jwt_refresh_ttl_seconds: read_env_i64("JWT_REFRESH_TTL_SECONDS", 60 * 60 * 24 * 7)?,
            demo_admin_email: read_env("DEMO_ADMIN_EMAIL", "admin@pixelforge.local"),
            demo_admin_password: read_env("DEMO_ADMIN_PASSWORD", "ChangeMe123!"),
            ai_shared_secret: read_env(
                "AI_SHARED_SECRET",
                "development-service-secret-change-before-production",
            ),
            ai_service_base_url: read_env("AI_SERVICE_BASE_URL", "http://127.0.0.1:8001"),
            ai_request_timeout_seconds: read_env_i64("AI_REQUEST_TIMEOUT_SECONDS", 20)?,
        })
    }
}

fn read_env(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_owned())
}

fn read_env_i64(name: &str, default: i64) -> Result<i64, ConfigError> {
    let value = env::var(name).unwrap_or_else(|_| default.to_string());
    value.parse::<i64>().map_err(|_| ConfigError::InvalidInteger {
        name: name.to_owned(),
        value,
    })
}
