mod daemon;
mod wallpapers;
pub use daemon::{daemon_hello_world, daemon_shutdown, daemon_status};
pub use wallpapers::set_static_image_wallpaper_from_path;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            daemon_hello_world,
            daemon_status,
            daemon_shutdown,
            set_static_image_wallpaper_from_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
