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
    output_format: Option<String>,
}

#[derive(Debug, Serialize)]
struct RemoveBackgroundResponse {
    image_base64: String,
    provider: String,
    processing_ms: u64,
}

const SUPPORTED_OUTPUT_FORMAT: &str = "png";
const BASE64_EXPANSION_NUMERATOR: usize = 4;
const BASE64_EXPANSION_DENOMINATOR: usize = 3;

fn max_base64_length(max_image_bytes: usize) -> usize {
    max_image_bytes
        .saturating_add(BASE64_EXPANSION_DENOMINATOR - 1)
        .saturating_div(BASE64_EXPANSION_DENOMINATOR)
        .saturating_mul(BASE64_EXPANSION_NUMERATOR)
}

fn validate_output_format(output_format: Option<String>) -> AppResult<()> {
    if let Some(raw_output_format) = output_format {
        let normalized = raw_output_format.trim().to_lowercase();
        if normalized != SUPPORTED_OUTPUT_FORMAT {
            return Err(AppError::BadRequest(format!(
                "output_format must be {SUPPORTED_OUTPUT_FORMAT}"
            )));
        }
    }

    Ok(())
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/remove-background", post(remove_background))
}

async fn remove_background(
    State(state): State<AppState>,
    Json(payload): Json<RemoveBackgroundRequest>,
) -> AppResult<Json<RemoveBackgroundResponse>> {
    let RemoveBackgroundRequest {
        image_base64,
        output_format,
    } = payload;

    validate_output_format(output_format)?;

    let encoded = image_base64.trim();
    if encoded.is_empty() {
        return Err(AppError::BadRequest("image_base64 is required".to_owned()));
    }

    let max_payload_size = state.config.ai_max_image_bytes;
    if encoded.len() > max_base64_length(max_payload_size) {
        return Err(AppError::PayloadTooLarge(format!(
            "image payload exceeds the configured limit of {max_payload_size} bytes"
        )));
    }

    let input_image = BASE64_STANDARD
        .decode(encoded)
        .map_err(|_| AppError::BadRequest("image_base64 must be valid base64".to_owned()))?;

    if input_image.len() > max_payload_size {
        return Err(AppError::PayloadTooLarge(format!(
            "decoded image exceeds the configured limit of {max_payload_size} bytes"
        )));
    }

    let result = state.ai_service.remove_background(&input_image).await?;

    Ok(Json(RemoveBackgroundResponse {
        image_base64: BASE64_STANDARD.encode(result.image_bytes),
        provider: result.provider,
        processing_ms: result.processing_ms,
    }))
}
