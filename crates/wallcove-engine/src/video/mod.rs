mod player;

#[cfg(target_os = "linux")]
mod linux;

pub use player::VideoPlayer;
