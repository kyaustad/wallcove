use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WallpaperKind {
    None,
    StaticImage,
    Video,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActiveWallpaper {
    pub kind: WallpaperKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

impl ActiveWallpaper {
    pub fn none() -> Self {
        Self {
            kind: WallpaperKind::None,
            path: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WallpaperApplied {
    pub kind: WallpaperKind,
    pub path: String,
}
