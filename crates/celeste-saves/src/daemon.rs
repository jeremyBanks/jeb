use std::process::exit;

use fork::fork;
use fork::Fork;
use tracing::instrument;
use tracing::trace;
use tracing_unwrap::ResultExt;

#[instrument]
pub fn forked_daemon() -> bool {
    trace!("Forking daemon from PID {}...", std::process::id());
    if matches!(fork().unwrap_or_log(), Fork::Child) {
        fork::close_fd().ok();
        fork::setsid().unwrap_or_log();
        if matches!(fork().unwrap_or_log(), Fork::Child) {
            trace!("Continuing as daemon with PID {}...", std::process::id());
            true
        } else {
            exit(0)
        }
    } else {
        false
    }
}

#[instrument]
pub fn daemonize() {
    if !forked_daemon() {
        trace!("Exiting as parent with PID {}...", std::process::id());
        exit(0)
    }
}
