use wallpaper_ng;

#[tauri::command]
pub fn set_static_image_wallpaper_from_path(path: String) -> Result<bool, String> {
    wallpaper_ng::set_from_path(path)
        .map(|_| true)
        .map_err(|e| e.to_string())
}
