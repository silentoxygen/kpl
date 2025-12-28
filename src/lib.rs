pub mod cli;
pub mod config;
pub mod dev;
pub mod errors;
pub mod kube;
pub mod logging;
pub mod merge;
pub mod podwatch;
pub mod stream;
pub mod types;

use tokio::sync::mpsc;

use crate::config::Config;
use crate::errors::{AppError, AppResult};
use crate::types::{LogEvent, PodCommand};

pub async fn run(config: Config) -> AppResult<()> {
    // Control-plane: pod lifecycle commands
    let (cmd_tx, mut cmd_rx) = mpsc::channel::<PodCommand>(128);

    // Data-plane: log events (bounded for backpressure safety)
    let (log_tx, log_rx) = mpsc::channel::<LogEvent>(config.buffer);

    // Start the appropriate "pod source" depending on mode.
    let watcher = if config.dev_mode {
        crate::dev::pods::spawn_dev_pods(config.namespace.clone(), cmd_tx)
    } else {
        let client = crate::kube::client::make_client().await?;
        crate::podwatch::watcher::spawn_pod_watcher(
            client,
            config.namespace.clone(),
            config.selector.clone(),
            cmd_tx,
        )
    };

    // Single stdout writer / merger task
    tokio::spawn(crate::merge::output::run_merger(log_rx, config.output));

    // Supervisor: turns PodCommand into per-container stream tasks
    let mut supervisor = crate::stream::supervisor::StreamSupervisor::new(log_tx);

    while let Some(cmd) = cmd_rx.recv().await {
        supervisor.handle_command(cmd);
    }

    // If cmd_rx closes, watcher may still be running; await it so errors surface.
    match watcher.await {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(e) => Err(AppError::Other(format!("pod watcher task failed: {e}"))),
    }
}
