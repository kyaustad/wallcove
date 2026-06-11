use std::sync::Arc;
use std::time::Instant;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tracing::{info, warn};
use wallcove_core::protocol::{Request, Response, DAEMON_TCP_ADDR};

use crate::handlers::handle_request;

pub async fn run(started_at: Instant) -> anyhow::Result<()> {
    let listener = TcpListener::bind(DAEMON_TCP_ADDR).await?;
    info!("wallcovedaemon listening on {DAEMON_TCP_ADDR}");

    let started_at = Arc::new(started_at);

    loop {
        let (stream, addr) = listener.accept().await?;
        let started_at = Arc::clone(&started_at);

        tokio::spawn(async move {
            if let Err(err) = handle_connection(stream, *started_at).await {
                warn!("client {addr} disconnected with error: {err}");
            }
        });
    }
}

async fn handle_connection(stream: TcpStream, started_at: Instant) -> anyhow::Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();

    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }

        let response = match serde_json::from_str::<Request>(&line) {
            Ok(req) => handle_request(req, started_at),
            Err(err) => Response::failure(format!("invalid request: {err}")),
        };

        let out = serde_json::to_string(&response)? + "\n";
        writer.write_all(out.as_bytes()).await?;
        writer.flush().await?;
    }

    Ok(())
}
