use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread::JoinHandle;
use std::time::Duration;

use tracing::{info, warn};

use crate::error::{Error, Result};

#[cfg(target_os = "linux")]
use crate::video::linux;

const STOP_TIMEOUT: Duration = Duration::from_secs(3);

struct VideoSession {
    shutdown_tx: mpsc::Sender<()>,
    stop_flag: Arc<AtomicBool>,
    exited_rx: mpsc::Receiver<()>,
    thread: Option<JoinHandle<()>>,
}

/// Native GPU video wallpaper session.
///
/// Linux/Wayland uses an in-process GStreamer + EGL + glow renderer on a
/// wlr-layer-shell surface (Phonto-inspired, owned by Wallcove).
pub struct VideoPlayer {
    session: Option<VideoSession>,
}

impl VideoPlayer {
    pub fn new() -> Self {
        Self { session: None }
    }

    pub fn start(&mut self, path: &Path) -> Result<()> {
        self.stop();

        let (shutdown_tx, shutdown_rx) = mpsc::channel();
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);
        let stop_flag = Arc::new(AtomicBool::new(false));

        #[cfg(target_os = "linux")]
        let video_thread = linux::start(path, shutdown_rx, ready_tx)?;

        #[cfg(not(target_os = "linux"))]
        let video_thread = {
            let _ = (path, shutdown_rx, ready_tx);
            return Err(Error::Video(
                "native video wallpapers are not yet implemented on this platform".into(),
            ));
        };

        #[cfg(target_os = "linux")]
        linux::wait_for_ready(ready_rx)?;

        info!(path = %path.display(), "native video wallpaper running");

        self.session = Some(VideoSession {
            shutdown_tx,
            stop_flag,
            exited_rx: video_thread.exited_rx,
            thread: Some(video_thread.thread),
        });

        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(mut session) = self.session.take() {
            session.stop_flag.store(true, Ordering::Relaxed);
            let _ = session.shutdown_tx.send(());

            match session.exited_rx.recv_timeout(STOP_TIMEOUT) {
                Ok(()) => {
                    if let Some(thread) = session.thread.take() {
                        if thread.join().is_err() {
                            warn!("video wallpaper thread panicked during join");
                        }
                    }
                    info!("stopped native video wallpaper");
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    warn!(
                        ?STOP_TIMEOUT,
                        "video wallpaper thread stop timed out; releasing engine lock"
                    );
                    // Drop the join handle without joining so the daemon stays responsive.
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    if let Some(thread) = session.thread.take() {
                        let _ = thread.join();
                    }
                    info!("stopped native video wallpaper");
                }
            }
        }
    }
}

impl Default for VideoPlayer {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for VideoPlayer {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stop_is_idempotent() {
        let mut player = VideoPlayer::new();
        player.stop();
    }
}
