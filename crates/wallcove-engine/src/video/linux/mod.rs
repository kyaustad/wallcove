mod decoder;
mod renderer;
mod wayland;

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::thread::{self, JoinHandle};

use tracing::info;

use crate::error::{Error, Result};

pub struct VideoThreadHandle {
    pub thread: JoinHandle<()>,
    pub exited_rx: mpsc::Receiver<()>,
}

pub fn start(
    path: &Path,
    shutdown: mpsc::Receiver<()>,
    ready: mpsc::SyncSender<std::result::Result<(), String>>,
) -> Result<VideoThreadHandle> {
    if std::env::var("WAYLAND_DISPLAY").is_err() {
        return Err(Error::Video(
            "native video wallpapers require a Wayland session on Linux".into(),
        ));
    }

    let path = path.to_path_buf();
    let stop_flag = Arc::new(AtomicBool::new(false));
    let (exited_tx, exited_rx) = mpsc::sync_channel(0);

    let handle = thread::Builder::new()
        .name("wallcove-video".into())
        .spawn(move || {
            info!(path = %path.display(), "starting native wayland video session");
            wayland::run_session(path, shutdown, ready, stop_flag, exited_tx);
        })
        .map_err(|err| Error::Video(format!("failed to spawn video thread: {err}")))?;

    Ok(VideoThreadHandle {
        thread: handle,
        exited_rx,
    })
}

pub fn wait_for_ready(ready: mpsc::Receiver<std::result::Result<(), String>>) -> Result<()> {
    match ready.recv_timeout(std::time::Duration::from_secs(15)) {
        Ok(Ok(())) => Ok(()),
        Ok(Err(message)) => Err(Error::Video(message)),
        Err(mpsc::RecvTimeoutError::Timeout) => Err(Error::Video(
            "timed out waiting for video wallpaper to start (check GStreamer VA-API plugins)"
                .into(),
        )),
        Err(mpsc::RecvTimeoutError::Disconnected) => Err(Error::Video(
            "video wallpaper thread exited before becoming ready".into(),
        )),
    }
}
