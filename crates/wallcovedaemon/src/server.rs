use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};
use wallcove_core::protocol::{Request, Response, DAEMON_TCP_ADDR};

use crate::handlers::{handle_request, shutdown_engine};
use crate::state::DaemonState;

pub async fn run(started_at: Instant, shutdown: CancellationToken) -> anyhow::Result<()> {
    let listener = TcpListener::bind(DAEMON_TCP_ADDR).await?;
    info!("wallcovedaemon listening on {DAEMON_TCP_ADDR}");

    let state = Arc::new(DaemonState::new(started_at, shutdown.clone()));

    loop {
        tokio::select! {
            // Prefer shutdown over accepting one more connection
            biased;

            _ = shutdown.cancelled() => {
                info!("shutdown signal received, stopping accept loop");
                break;
            }

            accept_result = listener.accept() => {
                let (stream, addr) = accept_result?;
                let state = Arc::clone(&state);

                tokio::spawn(async move {
                    if let Err(err) = handle_connection(stream, state).await {
                        warn!("client {addr} disconnected with error: {err}");
                    }
                });
            }
        }
    }

    // Brief grace: let in-flight connection handlers finish writing responses
    tokio::time::sleep(Duration::from_millis(250)).await;
    shutdown_engine(&state);
    info!("wallcovedaemon stopped");
    Ok(())
}

async fn handle_connection(stream: TcpStream, state: Arc<DaemonState>) -> anyhow::Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();

    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }

        let response = match serde_json::from_str::<Request>(&line) {
            Ok(req) => handle_request(req, &state),
            Err(err) => Response::failure(format!("invalid request: {err}")),
        };

        let out = serde_json::to_string(&response)? + "\n";
        writer.write_all(out.as_bytes()).await?;
        writer.flush().await?;
    }

    Ok(())
}
