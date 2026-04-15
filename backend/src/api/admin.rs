use axum::extract::State;
use axum::http::HeaderMap;
use axum::routing::get;
use axum::{Json, Router};

use crate::middleware::auth::authenticate_request;
use crate::models::audit::AuditLogEntry;
use crate::models::auth::UserRole;
use crate::utils::errors::{AppError, AppResult};
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/audit-logs", get(list_audit_logs))
}

async fn list_audit_logs(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<Json<Vec<AuditLogEntry>>> {
    let user = authenticate_request(&headers, state.auth_service.as_ref())?;

    if user.role != UserRole::Admin {
        return Err(AppError::Forbidden(
            "Only admins can access audit logs".to_owned(),
        ));
    }

    let logs = state.project_service.list_audit_logs().await;
    Ok(Json(logs))
}
