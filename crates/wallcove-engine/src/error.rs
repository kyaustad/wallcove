use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("wallpaper file not found: {0}")]
    NotFound(String),

    #[error("unsupported file type: {0}")]
    UnsupportedType(String),

    #[error("failed to set static wallpaper: {0}")]
    Static(String),

    #[error("failed to start video wallpaper: {0}")]
    Video(String),

    #[error("video player not available: {0}")]
    PlayerUnavailable(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
