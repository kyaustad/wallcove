use wallcove_client::DaemonClient;
use wallcove_core::protocol::DaemonStatus;

#[tauri::command]
pub async fn daemon_hello_world() -> Result<String, String> {
    let mut client = DaemonClient::connect().await.map_err(|e| e.to_string())?;
    client.hello_world().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn daemon_status() -> Result<DaemonStatus, String> {
    let mut client = DaemonClient::connect().await.map_err(|e| e.to_string())?;
    client.status().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn daemon_shutdown() -> Result<String, String> {
    let mut client = DaemonClient::connect().await.map_err(|e| e.to_string())?;
    client.shutdown().await.map_err(|e| e.to_string())
}
