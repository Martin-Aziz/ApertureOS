use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};

use crate::models::auth::{LoginRequest, LoginResponse, RefreshRequest, TokenPair};
use crate::utils::errors::AppResult;
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/refresh", post(refresh))
}

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<Json<LoginResponse>> {
    let response = state.auth_service.login(payload).await?;
    Ok(Json(response))
}

async fn refresh(
    State(state): State<AppState>,
    Json(payload): Json<RefreshRequest>,
) -> AppResult<Json<TokenPair>> {
    let payload = payload.validate()?;
    let token_pair = state.auth_service.refresh(payload.refresh_token).await?;
    Ok(Json(token_pair))
}
