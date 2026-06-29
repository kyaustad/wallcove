mod daemon;

pub use daemon::{
    daemon_clear_wallpaper, daemon_get_active_wallpaper, daemon_hello_world, daemon_set_static_wallpaper,
    daemon_set_video_wallpaper, daemon_shutdown, daemon_status, pick_and_set_static_wallpaper,
    pick_and_set_video_wallpaper,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            daemon_hello_world,
            daemon_status,
            daemon_shutdown,
            daemon_set_static_wallpaper,
            daemon_set_video_wallpaper,
            daemon_clear_wallpaper,
            daemon_get_active_wallpaper,
            pick_and_set_static_wallpaper,
            pick_and_set_video_wallpaper,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
