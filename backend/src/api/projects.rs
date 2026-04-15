use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{delete, get};
use axum::{Json, Router};
use uuid::Uuid;

use crate::middleware::auth::authenticate_request;
use crate::models::project::{CreateProjectRequest, Project};
use crate::utils::errors::AppResult;
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_projects).post(create_project))
        .route("/:project_id", delete(delete_project))
}

async fn list_projects(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<Json<Vec<Project>>> {
    let user = authenticate_request(&headers, state.auth_service.as_ref())?;
    let projects = state.project_service.list_projects(&user).await;
    Ok(Json(projects))
}

async fn create_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateProjectRequest>,
) -> AppResult<Json<Project>> {
    let user = authenticate_request(&headers, state.auth_service.as_ref())?;
    let created = state.project_service.create_project(&user, payload).await?;
    Ok(Json(created))
}

async fn delete_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
) -> AppResult<StatusCode> {
    let user = authenticate_request(&headers, state.auth_service.as_ref())?;
    state
        .project_service
        .soft_delete_project(&user, project_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
