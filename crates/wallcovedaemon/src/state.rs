use std::time::Instant;
use tokio_util::sync::CancellationToken;

pub struct DaemonState {
    pub started_at: Instant,
    pub shutdown: CancellationToken,
}
