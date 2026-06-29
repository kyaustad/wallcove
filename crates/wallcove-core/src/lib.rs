pub mod error;
pub mod protocol;
pub mod wallpaper;

pub use error::{Error, Result};
pub use protocol::{DaemonStatus, Request, Response, DAEMON_TCP_ADDR};
pub use wallpaper::{ActiveWallpaper, WallpaperApplied, WallpaperKind};
