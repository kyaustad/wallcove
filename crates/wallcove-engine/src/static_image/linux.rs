use std::path::Path;
use std::process::Command;

use tracing::debug;

use crate::error::{Error, Result};

pub fn set_static_image(path: &Path) -> Result<()> {
    let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();

    if is_kde(&desktop) {
        if let Ok(()) = try_plasma_apply_wallpaperimage(path) {
            return Ok(());
        }
        if let Ok(()) = try_kde_qdbus6(path) {
            return Ok(());
        }
    }

    match wallpaper_ng::set_from_path(path.display().to_string()) {
        Ok(()) => return Ok(()),
        Err(err) => {
            debug!(%err, desktop = %desktop, "wallpaper-ng failed, trying fallbacks");
        }
    }

    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        if let Ok(()) = try_swaybg(path) {
            return Ok(());
        }
    }

    if let Ok(()) = try_feh(path) {
        return Ok(());
    }

    Err(Error::Static(format!(
        "no working static wallpaper backend found for desktop '{desktop}'. \
         On KDE Plasma 6 install plasma-workspace (plasma-apply-wallpaperimage). \
         On wlroots Wayland compositors install swaybg. On X11 install feh."
    )))
}

fn is_kde(desktop: &str) -> bool {
    let desktop = desktop.to_ascii_lowercase();
    desktop.contains("kde") || desktop.contains("plasma")
}

fn try_plasma_apply_wallpaperimage(path: &Path) -> Result<()> {
    run_success(
        "plasma-apply-wallpaperimage",
        &[path.to_str().ok_or_else(|| Error::Static("invalid path".into()))?],
    )
}

fn try_kde_qdbus6(path: &Path) -> Result<()> {
    let path_str = path
        .to_str()
        .ok_or_else(|| Error::Static("invalid path".into()))?;
    let image_value = enquote::enquote('"', &format!("file://{path_str}"));
    let script = format!(
        r#"
for (const desktop of desktops()) {{
    desktop.currentConfigGroup = ["Wallpaper", "org.kde.image", "General"]
    desktop.writeConfig("Image", {image_value})
}}"#,
    );

    run_success(
        "qdbus6",
        &[
            "org.kde.plasmashell",
            "/PlasmaShell",
            "org.kde.PlasmaShell.evaluateScript",
            &script,
        ],
    )
}

fn try_swaybg(path: &Path) -> Result<()> {
    let path_str = path
        .to_str()
        .ok_or_else(|| Error::Static("invalid path".into()))?;

    Command::new("swaybg")
        .args(["-i", path_str, "-m", "fill"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|err| Error::Static(format!("failed to start swaybg: {err}")))
}

fn try_feh(path: &Path) -> Result<()> {
    let path_str = path
        .to_str()
        .ok_or_else(|| Error::Static("invalid path".into()))?;
    run_success("feh", &["--bg-fill", path_str])
}

pub fn clear_static_image() -> Result<()> {
    let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();

    if is_kde(&desktop) {
        try_kde_clear_solid_color()?;
    }

    Ok(())
}

fn try_kde_clear_solid_color() -> Result<()> {
    let script = r##"
for (const desktop of desktops()) {
    desktop.currentConfigGroup = ["Wallpaper", "org.kde.color", "General"];
    desktop.writeConfig("Color", "#1d232a");
}
"##;

    run_success(
        "qdbus6",
        &[
            "org.kde.plasmashell",
            "/PlasmaShell",
            "org.kde.PlasmaShell.evaluateScript",
            script,
        ],
    )
}

fn run_success(command: &str, args: &[&str]) -> Result<()> {
    let output = Command::new(command)
        .args(args)
        .output()
        .map_err(|err| Error::Static(format!("failed to run {command}: {err}")))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(Error::Static(format!(
            "{command} exited with {}: {}",
            output.status,
            stderr.trim()
        )))
    }
}
