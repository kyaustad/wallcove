use std::path::Path;

use crate::error::{Error, Result};

pub fn set_static_image(path: &Path) -> Result<()> {
    let path_str = path
        .to_str()
        .ok_or_else(|| Error::Static("invalid path".into()))?;
    wallpaper::set_from_path(path_str).map_err(|err| Error::Static(err.to_string()))
}

pub fn clear_static_image() -> Result<()> {
    Ok(())
}
