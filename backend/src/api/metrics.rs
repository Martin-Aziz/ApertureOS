use axum::extract::State;
use axum::response::IntoResponse;

use crate::AppState;

pub async fn export(State(state): State<AppState>) -> impl IntoResponse {
    let database_configured = if state.config.database_url.trim().is_empty() {
        0
    } else {
        1
    };

    let redis_configured = if state.config.redis_url.trim().is_empty() {
        0
    } else {
        1
    };

    let body = format!(
        concat!(
            "# HELP pixelforge_backend_info Static service metadata.\n",
            "# TYPE pixelforge_backend_info gauge\n",
            "pixelforge_backend_info{{service=\"pixelforge-backend\"}} 1\n",
            "# HELP pixelforge_backend_dependency_configured Whether dependency endpoints are configured.\n",
            "# TYPE pixelforge_backend_dependency_configured gauge\n",
            "pixelforge_backend_dependency_configured{{dependency=\"database\"}} {database_configured}\n",
            "pixelforge_backend_dependency_configured{{dependency=\"redis\"}} {redis_configured}\n"
        ),
        database_configured = database_configured,
        redis_configured = redis_configured,
    );

    (
        [(
            "content-type",
            "text/plain; version=0.0.4; charset=utf-8",
        )],
        body,
    )
}
