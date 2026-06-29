use wallcove_client::DaemonClient;
use wallcove_core::protocol::DaemonStatus;
use wallcove_core::{ActiveWallpaper, WallpaperApplied};

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

#[tauri::command]
pub async fn daemon_set_static_wallpaper(path: String) -> Result<WallpaperApplied, String> {
    let mut client = DaemonClient::connect().await.map_err(|e| e.to_string())?;
    client
        .set_static_wallpaper(path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn daemon_set_video_wallpaper(path: String) -> Result<WallpaperApplied, String> {
    let mut client = DaemonClient::connect().await.map_err(|e| e.to_string())?;
    client
        .set_video_wallpaper(path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn daemon_clear_wallpaper() -> Result<ActiveWallpaper, String> {
    let mut client = DaemonClient::connect().await.map_err(|e| e.to_string())?;
    client.clear_wallpaper().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn daemon_get_active_wallpaper() -> Result<ActiveWallpaper, String> {
    let mut client = DaemonClient::connect().await.map_err(|e| e.to_string())?;
    client
        .get_active_wallpaper()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn pick_and_set_static_wallpaper() -> Result<WallpaperApplied, String> {
    let path = rfd::FileDialog::new()
        .add_filter("Image", &["png", "jpg", "jpeg", "webp", "bmp", "gif"])
        .pick_file()
        .ok_or_else(|| "file selection cancelled".to_string())?;

    daemon_set_static_wallpaper(path.display().to_string()).await
}

#[tauri::command]
pub async fn pick_and_set_video_wallpaper() -> Result<WallpaperApplied, String> {
    let path = rfd::FileDialog::new()
        .add_filter(
            "Video",
            &["mp4", "webm", "mkv", "mov", "avi", "ogv", "m4v"],
        )
        .pick_file()
        .ok_or_else(|| "file selection cancelled".to_string())?;

    daemon_set_video_wallpaper(path.display().to_string()).await
}
