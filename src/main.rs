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
    let addr = config.http_addr;

    tracing::info!("HTTP server listening on {}", addr);

    orchestrator_service::infra::http_server::run(addr).await?;

    Ok(())
}
