use std::time::Instant;

use wallcove_core::protocol::{DaemonStatus, Request, Response};

pub fn handle_request(req: Request, started_at: Instant) -> Response {
    match req {
        Request::HelloWorld => Response::success("hello from wallcovedaemon"),
        Request::Status => Response::success(DaemonStatus {
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_secs: started_at.elapsed().as_secs(),
            pid: std::process::id(),
            platform: std::env::consts::OS.to_string(),
        }),
    }
}
