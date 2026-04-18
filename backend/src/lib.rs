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
use services::ai_service::AiService;
use services::auth_service::AuthService;
use services::project_service::ProjectService;

fn build_allow_origin(frontend_origin: &str) -> AllowOrigin {
    let mut candidates = vec![frontend_origin.to_owned()];

    if frontend_origin.contains("localhost") {
        candidates.push(frontend_origin.replace("localhost", "127.0.0.1"));
    } else if frontend_origin.contains("127.0.0.1") {
        candidates.push(frontend_origin.replace("127.0.0.1", "localhost"));
    }

    let mut header_values = Vec::new();
    for candidate in candidates {
        let already_present = header_values
            .iter()
            .any(|existing: &HeaderValue| existing.as_bytes() == candidate.as_bytes());
        if already_present {
            continue;
        }

        if let Ok(value) = HeaderValue::from_str(&candidate) {
            header_values.push(value);
        }
    }

    match header_values.len() {
        0 => AllowOrigin::any(),
        1 => AllowOrigin::exact(header_values.remove(0)),
        _ => AllowOrigin::list(header_values),
    }
}

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub auth_service: Arc<AuthService>,
    pub project_service: Arc<ProjectService>,
    pub ai_service: Arc<AiService>,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        let repository = Arc::new(InMemoryProjectRepository::default());
        let auth_service = Arc::new(AuthService::new(config.clone()));
        let project_service = Arc::new(ProjectService::new(repository));
        let ai_service = Arc::new(AiService::new(config.clone()));

        Self {
            config,
            auth_service,
            project_service,
            ai_service,
        }
    }
}

pub fn build_app(state: AppState) -> Router {
    let allow_origin = build_allow_origin(&state.config.frontend_origin);

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
