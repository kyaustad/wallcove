#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
mod windows;

use std::path::Path;

use crate::error::{Error, Result};

pub fn set_static_image(path: &Path) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        return linux::set_static_image(path);
    }

    #[cfg(target_os = "windows")]
    {
        return windows::set_static_image(path);
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        let _ = path;
        Err(Error::Static(
            "static wallpapers are not supported on this platform".into(),
        ))
    }
}

pub fn clear_static_image() -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        return linux::clear_static_image();
    }

    #[cfg(target_os = "windows")]
    {
        return windows::clear_static_image();
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        Ok(())
    }
}
