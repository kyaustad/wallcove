use wallcove_core::protocol::{DaemonStatus, Request, Response};

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
            state.shutdown.cancel();
            Response::success("shutting down wallcovedaemon")
        }
    }
}
