mod handlers;
mod server;
mod state;

use std::time::Instant;

use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "wallcovedaemon=info".into()),
        )
        .init();

    let shutdown = CancellationToken::new();

    // Ctrl+C → same token as Request::Shutdown
    let signal_shutdown = shutdown.clone();
    tokio::spawn(async move {
        match tokio::signal::ctrl_c().await {
            Ok(()) => tracing::info!("received Ctrl+C, shutting down"),
            Err(err) => tracing::warn!("failed to listen for Ctrl+C: {err}"),
        }
        signal_shutdown.cancel();
    });

    let started_at = Instant::now();
    server::run(started_at, shutdown).await
}
