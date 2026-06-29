use wallcove_core::protocol::{DaemonStatus, Request, Response};
use wallcove_engine::WallpaperEngine;

use crate::state::DaemonState;

pub fn handle_request(req: Request, state: &DaemonState) -> Response {
    match req {
        Request::HelloWorld => Response::success("hello from wallcovedaemon"),
        Request::Status => Response::success(DaemonStatus {
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_secs: state.started_at.elapsed().as_secs(),
            pid: std::process::id(),
            platform: std::env::consts::OS.to_string(),
        }),
        Request::Shutdown => {
            tracing::info!("shutdown requested via IPC");
            shutdown_engine(state);
            state.shutdown.cancel();
            Response::success("shutting down wallcovedaemon")
        }
        Request::SetStaticWallpaper { path } => {
            with_engine(state, |engine| engine.set_static_image(&path))
        }
        Request::SetVideoWallpaper { path } => {
            with_engine(state, |engine| engine.set_video(&path))
        }
        Request::ClearWallpaper => with_engine(state, |engine| engine.clear()),
        Request::GetActiveWallpaper => {
            with_engine(state, |engine| Ok(engine.active()))
        }
    }
}

pub fn shutdown_engine(state: &DaemonState) {
    use std::time::{Duration, Instant};

    let deadline = Instant::now() + Duration::from_secs(4);
    while Instant::now() < deadline {
        if let Ok(mut engine) = state.engine.try_lock() {
            engine.shutdown();
            return;
        }
        std::thread::sleep(Duration::from_millis(25));
    }

    tracing::warn!("timed out waiting for wallpaper engine lock during shutdown");
}

fn with_engine<T, F>(state: &DaemonState, operation: F) -> Response
where
    T: serde::Serialize,
    F: FnOnce(&mut WallpaperEngine) -> wallcove_engine::Result<T>,
{
    let mut engine = state
        .engine
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    match operation(&mut engine) {
        Ok(data) => Response::success(data),
        Err(err) => Response::failure(err.to_string()),
    }
}
