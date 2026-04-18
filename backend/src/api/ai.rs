use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine as _;
use serde::{Deserialize, Serialize};

use crate::utils::errors::{AppError, AppResult};
use crate::AppState;

#[derive(Debug, Deserialize)]
struct RemoveBackgroundRequest {
    image_base64: String,
}

#[derive(Debug, Serialize)]
struct RemoveBackgroundResponse {
    image_base64: String,
    provider: String,
    processing_ms: u64,
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/remove-background", post(remove_background))
}

async fn remove_background(
    State(state): State<AppState>,
    Json(payload): Json<RemoveBackgroundRequest>,
) -> AppResult<Json<RemoveBackgroundResponse>> {
    let encoded = payload.image_base64.trim();
    if encoded.is_empty() {
        return Err(AppError::BadRequest("image_base64 is required".to_owned()));
    }

    let input_image = BASE64_STANDARD
        .decode(encoded)
        .map_err(|_| AppError::BadRequest("image_base64 must be valid base64".to_owned()))?;

    let result = state.ai_service.remove_background(&input_image).await?;

    Ok(Json(RemoveBackgroundResponse {
        image_base64: BASE64_STANDARD.encode(result.image_bytes),
        provider: result.provider,
        processing_ms: result.processing_ms,
    }))
}
