use std::path::{Path, PathBuf};

const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp", "bmp", "gif"];
const VIDEO_EXTENSIONS: &[&str] = &["mp4", "webm", "mkv", "mov", "avi", "ogv", "m4v"];

pub fn validate_image_path(path: impl AsRef<Path>) -> Result<PathBuf, crate::Error> {
    validate_media_path(path, IMAGE_EXTENSIONS, "image")
}

pub fn validate_video_path(path: impl AsRef<Path>) -> Result<PathBuf, crate::Error> {
    validate_media_path(path, VIDEO_EXTENSIONS, "video")
}

fn validate_media_path(
    path: impl AsRef<Path>,
    extensions: &[&str],
    label: &str,
) -> Result<PathBuf, crate::Error> {
    let path = path.as_ref();
    let canonical = path
        .canonicalize()
        .map_err(|_| crate::Error::NotFound(path.display().to_string()))?;

    if !canonical.is_file() {
        return Err(crate::Error::NotFound(canonical.display().to_string()));
    }

    let extension = canonical
        .extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| {
            crate::Error::UnsupportedType(format!(
                "expected a {} file with a known extension",
                label
            ))
        })?;

    if !extensions.contains(&extension.as_str()) {
        return Err(crate::Error::UnsupportedType(format!(
            ".{extension} is not a supported {label} format"
        )));
    }

    Ok(canonical)
}
