use std::sync::Arc;

use evobase_auth::JwtAuthService;
use evobase_core::AppConfig;
use evobase_db::PostgresStorage;
use evobase_gateway::{AppState, build_router};
use evobase_messaging::InMemoryMessagingHub;
use tokio::{net::TcpListener, signal};
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let config = AppConfig::from_env()?;
    let storage = Arc::new(PostgresStorage::connect(&config.database.url).await?);
    let auth_service = Arc::new(JwtAuthService::new(storage.clone(), config.tokens.clone()));
    let messaging_service = Arc::new(InMemoryMessagingHub::default());
    let state = AppState::new(auth_service, messaging_service, storage);
    let router = build_router(state);

    let listener = TcpListener::bind(config.server.addr).await?;
    info!(address = %config.server.addr, "evobase server listening");

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,evobase_server=debug")),
        )
        .with(fmt::layer().with_target(true))
        .init();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(error) = signal::ctrl_c().await {
            tracing::error!(%error, "failed to install CTRL+C handler");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        let mut signal = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler");
        signal.recv().await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
