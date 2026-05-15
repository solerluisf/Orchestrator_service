use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "orchestrator_service=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    tracing::info!("Starting Orchestrator Service");

    let config = orchestrator_service::config::OrchestratorConfig::load()?;

    let broadcaster = orchestrator_service::infra::ws_server::TelemetryBroadcaster::new(256);

    let http_addr = config.http_addr;
    let ws_addr = config.ws_addr;

    tracing::info!("HTTP server listening on {}", http_addr);
    tracing::info!("WebSocket server listening on {}", ws_addr);

    let ws_broadcaster = broadcaster.clone();
    let ws_handle = tokio::spawn(async move {
        if let Err(e) = orchestrator_service::infra::ws_server::run_ws_server(ws_addr, ws_broadcaster).await {
            tracing::error!("WebSocket server error: {}", e);
        }
    });

    orchestrator_service::infra::http_server::run(http_addr).await?;

    ws_handle.abort();

    Ok(())
}
