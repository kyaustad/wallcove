mod handlers;
mod server;

use std::time::Instant;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "wallcovedaemon=info".into()),
        )
        .init();

    let started_at = Instant::now();
    server::run(started_at).await
}
