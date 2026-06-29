use std::sync::Mutex;
use std::time::Instant;

use tokio_util::sync::CancellationToken;
use wallcove_engine::WallpaperEngine;

pub struct DaemonState {
    pub started_at: Instant,
    pub shutdown: CancellationToken,
    pub engine: Mutex<WallpaperEngine>,
}

impl DaemonState {
    pub fn new(started_at: Instant, shutdown: CancellationToken) -> Self {
        Self {
            started_at,
            shutdown,
            engine: Mutex::new(WallpaperEngine::new()),
        }
    }
}
