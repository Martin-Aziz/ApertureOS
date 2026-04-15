use axum::http::header::AUTHORIZATION;
use axum::http::HeaderMap;

use crate::models::auth::AuthenticatedUser;
use crate::services::auth_service::AuthService;
use crate::utils::errors::{AppError, AppResult};

pub fn authenticate_request(
    headers: &HeaderMap,
    auth_service: &AuthService,
) -> AppResult<AuthenticatedUser> {
    let token = extract_bearer_token(headers)?;
    auth_service.validate_access_token(&token)
}

fn extract_bearer_token(headers: &HeaderMap) -> AppResult<String> {
    let authorization = headers
        .get(AUTHORIZATION)
        .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".to_owned()))?;

    let value = authorization
        .to_str()
        .map_err(|_| AppError::Unauthorized("Invalid Authorization header".to_owned()))?;

    let token = value
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Unauthorized("Expected Bearer token".to_owned()))?;

    if token.trim().is_empty() {
        return Err(AppError::Unauthorized("Bearer token cannot be empty".to_owned()));
    }

    Ok(token.to_owned())
}
