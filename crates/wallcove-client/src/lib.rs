use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use wallcove_core::protocol::{DaemonStatus, Request, Response, DAEMON_TCP_ADDR};
use wallcove_core::{Error, Result};

pub struct DaemonClient {
    reader: BufReader<tokio::net::tcp::OwnedReadHalf>,
    writer: tokio::net::tcp::OwnedWriteHalf,
}

impl DaemonClient {
    pub async fn connect() -> Result<Self> {
        let stream = TcpStream::connect(DAEMON_TCP_ADDR).await.map_err(|e| {
            Error::Connection(format!("daemon not running at {DAEMON_TCP_ADDR}: {e}"))
        })?;

        let (reader, writer) = stream.into_split();

        Ok(Self {
            reader: BufReader::new(reader),
            writer,
        })
    }

    async fn call_raw(&mut self, request: &Request) -> Result<Response> {
        let payload = serde_json::to_string(request).map_err(|e| Error::Protocol(e.to_string()))?;
        self.writer
            .write_all(format!("{payload}\n").as_bytes())
            .await?;
        self.writer.flush().await?;

        let mut line = String::new();
        self.reader.read_line(&mut line).await?;

        serde_json::from_str(&line).map_err(|e| Error::Protocol(format!("invalid response: {e}")))
    }

    async fn call<T: serde::de::DeserializeOwned>(&mut self, request: Request) -> Result<T> {
        let response = self.call_raw(&request).await?;

        if !response.ok {
            return Err(Error::Daemon(
                response
                    .error
                    .unwrap_or_else(|| "unknown daemon error".into()),
            ));
        }

        let value = response
            .data
            .ok_or_else(|| Error::Protocol("response missing data".into()))?;

        serde_json::from_value(value).map_err(|e| Error::Protocol(e.to_string()))
    }

    pub async fn hello_world(&mut self) -> Result<String> {
        self.call(Request::HelloWorld).await
    }

    pub async fn status(&mut self) -> Result<DaemonStatus> {
        self.call(Request::Status).await
    }
}
