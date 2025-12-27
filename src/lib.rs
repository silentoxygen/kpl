pub mod cli;
pub mod config;
pub mod errors;
pub mod tracing;
pub mod types;

use crate::config::Config;
use crate::errors::AppResult;

pub async fn run(_config: Config) -> AppResult<()> {
    // Slice 2 will implement:
    // - Kubernetes client init
    // - pod watcher
    // - streamers
    // - merger/output
    Ok(())
}
