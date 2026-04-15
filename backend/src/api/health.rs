use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;

use crate::utils::errors::AppResult;
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/live", get(live)).route("/ready", get(ready))
}

async fn live() -> Json<serde_json::Value> {
    Json(json!({
        "status": "live",
        "service": "pixelforge-backend"
    }))
}

async fn ready(State(state): State<AppState>) -> AppResult<Json<serde_json::Value>> {
    let checks = json!({
        "database_configured": !state.config.database_url.trim().is_empty(),
        "redis_configured": !state.config.redis_url.trim().is_empty(),
        "ai_shared_secret_configured": !state.config.ai_shared_secret.trim().is_empty()
    });

    Ok(Json(json!({
        "status": "ready",
        "service": "pixelforge-backend",
        "checks": checks
    })))
}
