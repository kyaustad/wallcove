use std::path::Path;

use crate::error::{Error, Result};

pub fn set_static_image(path: &Path) -> Result<()> {
    wallpaper_ng::set_from_path(path.display().to_string())
        .map_err(|err| Error::Static(err.to_string()))
}

pub fn clear_static_image() -> Result<()> {
    Ok(())
}
