use std::path::Path;

use tracing::info;
use wallcove_core::{ActiveWallpaper, WallpaperApplied, WallpaperKind};

use crate::error::Result;
use crate::media::{validate_image_path, validate_video_path};
use crate::static_image;
use crate::video::VideoPlayer;

/// Owns the active wallpaper session (static image or video).
///
/// Platform backends live here so the daemon stays a thin IPC shell.
/// Future Wallcove packages will deserialize into instructions for this engine.
pub struct WallpaperEngine {
    active: ActiveWallpaper,
    video_player: VideoPlayer,
}

impl WallpaperEngine {
    pub fn new() -> Self {
        Self {
            active: ActiveWallpaper::none(),
            video_player: VideoPlayer::new(),
        }
    }

    pub fn active(&self) -> ActiveWallpaper {
        self.active.clone()
    }

    pub fn set_static_image(&mut self, path: impl AsRef<Path>) -> Result<WallpaperApplied> {
        let path = validate_image_path(path)?;
        self.video_player.stop();

        static_image::set_static_image(&path)?;

        let applied = WallpaperApplied {
            kind: WallpaperKind::StaticImage,
            path: path.display().to_string(),
        };

        self.active = ActiveWallpaper {
            kind: WallpaperKind::StaticImage,
            path: Some(applied.path.clone()),
        };

        info!(path = %applied.path, "static image wallpaper applied");
        Ok(applied)
    }

    pub fn set_video(&mut self, path: impl AsRef<Path>) -> Result<WallpaperApplied> {
        let path = validate_video_path(path)?;
        self.video_player.start(&path)?;

        let applied = WallpaperApplied {
            kind: WallpaperKind::Video,
            path: path.display().to_string(),
        };

        self.active = ActiveWallpaper {
            kind: WallpaperKind::Video,
            path: Some(applied.path.clone()),
        };

        info!(path = %applied.path, "video wallpaper applied");
        Ok(applied)
    }

    pub fn clear(&mut self) -> Result<ActiveWallpaper> {
        self.video_player.stop();
        static_image::clear_static_image()?;
        self.active = ActiveWallpaper::none();
        info!("wallpaper cleared");
        Ok(self.active.clone())
    }

    pub fn shutdown(&mut self) {
        self.video_player.stop();
        self.active = ActiveWallpaper::none();
    }
}

impl Default for WallpaperEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for WallpaperEngine {
    fn drop(&mut self) {
        self.shutdown();
    }
}
