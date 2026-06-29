use serde::{Deserialize, Serialize};

/// Dev IPC address. Swap to Unix socket / named pipe later.
pub const DAEMON_TCP_ADDR: &str = "127.0.0.1:42069";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method", rename_all = "snake_case")]
pub enum Request {
    HelloWorld,
    Status,
    Shutdown,
    SetStaticWallpaper { path: String },
    SetVideoWallpaper { path: String },
    ClearWallpaper,
    GetActiveWallpaper,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonStatus {
    pub version: String,
    pub uptime_secs: u64,
    pub pid: u32,
    pub platform: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Response {
    pub fn success<T: Serialize>(data: T) -> Self {
        Self {
            ok: true,
            data: Some(serde_json::to_value(data).expect("serialize response data")),
            error: None,
        }
    }

    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            ok: false,
            data: None,
            error: Some(message.into()),
        }
    }
}
