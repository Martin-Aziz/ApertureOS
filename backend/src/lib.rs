use std::sync::Arc;
use std::sync::Once;

use axum::http::HeaderValue;
use axum::Router;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

pub mod api;
pub mod config;
pub mod middleware;
pub mod models;
pub mod repositories;
pub mod services;
pub mod utils;

use config::AppConfig;
use repositories::project_repository::InMemoryProjectRepository;
use services::auth_service::AuthService;
use services::project_service::ProjectService;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub auth_service: Arc<AuthService>,
    pub project_service: Arc<ProjectService>,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        let repository = Arc::new(InMemoryProjectRepository::default());
        let auth_service = Arc::new(AuthService::new(config.clone()));
        let project_service = Arc::new(ProjectService::new(repository));

        Self {
            config,
            auth_service,
            project_service,
        }
    }
}

pub fn build_app(state: AppState) -> Router {
    let allow_origin = HeaderValue::from_str(&state.config.frontend_origin)
        .map(AllowOrigin::exact)
        .unwrap_or_else(|_| AllowOrigin::any());

    let cors = CorsLayer::new()
        .allow_origin(allow_origin)
        .allow_headers(Any)
        .allow_methods(Any);

    api::router()
        .with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}

pub fn init_tracing() {
    static ONCE: Once = Once::new();

    ONCE.call_once(|| {
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(
                EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
            )
            .with_current_span(false)
            .with_span_list(false)
            .init();
    });
}
