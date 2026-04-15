use axum::Router;

use crate::AppState;

pub mod admin;
pub mod auth;
pub mod health;
pub mod metrics;
pub mod projects;

pub fn router() -> Router<AppState> {
    Router::new()
        .nest("/health", health::routes())
    .route("/metrics", axum::routing::get(metrics::export))
        .nest("/api/v1/auth", auth::routes())
        .nest("/api/v1/projects", projects::routes())
        .nest("/api/v1/admin", admin::routes())
}
