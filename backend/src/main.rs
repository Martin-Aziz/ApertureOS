use pixelforge_backend::config::AppConfig;
use pixelforge_backend::{build_app, init_tracing, AppState};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let config = AppConfig::from_env()?;
    let bind_addr = config.bind_addr;

    let app = build_app(AppState::new(config));
    let listener = tokio::net::TcpListener::bind(bind_addr).await?;

    info!(%bind_addr, "pixelforge backend listening");
    axum::serve(listener, app).await?;

    Ok(())
}
