use std::time::Duration;

use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine as _;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};

use crate::config::AppConfig;
use crate::utils::errors::{AppError, AppResult};

#[derive(Clone)]
pub struct AiService {
    client: Client,
    base_url: String,
    shared_secret: String,
    timeout_seconds: u64,
}

#[derive(Debug, Clone)]
pub struct RemoveBackgroundResult {
    pub image_bytes: Vec<u8>,
    pub provider: String,
    pub processing_ms: u64,
}

#[derive(Debug, Serialize)]
struct RemoveBackgroundRequest {
    image_base64: String,
}

#[derive(Debug, Deserialize)]
struct RemoveBackgroundResponse {
    image_base64: String,
    provider: String,
    processing_ms: u64,
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    detail: Option<String>,
}

impl AiService {
    pub fn new(config: AppConfig) -> Self {
        Self {
            client: Client::new(),
            base_url: config.ai_service_base_url.trim_end_matches('/').to_owned(),
            shared_secret: config.ai_shared_secret,
            timeout_seconds: config.ai_request_timeout_seconds.max(1) as u64,
        }
    }

    pub async fn remove_background(&self, image_bytes: &[u8]) -> AppResult<RemoveBackgroundResult> {
        if image_bytes.is_empty() {
            return Err(AppError::BadRequest("Input image cannot be empty".to_owned()));
        }

        let payload = RemoveBackgroundRequest {
            image_base64: BASE64_STANDARD.encode(image_bytes),
        };

        let endpoint = format!("{}/v1/remove-background", self.base_url);
        let response = self
            .client
            .post(&endpoint)
            .header("x-ai-service-secret", &self.shared_secret)
            .timeout(Duration::from_secs(self.timeout_seconds))
            .json(&payload)
            .send()
            .await
            .map_err(|error| {
                AppError::ServiceUnavailable(format!(
                    "Local AI service is unavailable: {error}"
                ))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            let detail = serde_json::from_str::<ErrorResponse>(&body)
                .ok()
                .and_then(|error| error.detail)
                .unwrap_or_else(|| format!("AI service returned status {status}"));

            return Err(match status {
                StatusCode::BAD_REQUEST => AppError::BadRequest(detail),
                StatusCode::PAYLOAD_TOO_LARGE => AppError::PayloadTooLarge(detail),
                _ => AppError::ServiceUnavailable(detail),
            });
        }

        let parsed = response
            .json::<RemoveBackgroundResponse>()
            .await
            .map_err(|error| AppError::Internal(format!("Invalid AI response payload: {error}")))?;

        let output_image = BASE64_STANDARD
            .decode(parsed.image_base64)
            .map_err(|error| AppError::Internal(format!("AI response base64 decode failed: {error}")))?;

        Ok(RemoveBackgroundResult {
            image_bytes: output_image,
            provider: parsed.provider,
            processing_ms: parsed.processing_ms,
        })
    }
}
