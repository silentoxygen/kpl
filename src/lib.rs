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
    let (cmd_tx, mut cmd_rx) = mpsc::channel::<PodCommand>(128);
    let (log_tx, log_rx) = mpsc::channel::<LogEvent>(config.buffer);

    let watcher_handle = if config.dev_mode {
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

    // We'll await this exactly once.
    let mut watcher = Some(watcher_handle);

    let mut supervisor = crate::stream::supervisor::StreamSupervisor::new(
        log_tx,
        config.dev_rate_ms,
        config.dev_lines,
    );

    // Output runs on the current task (stdout lock is !Send).
    let output_cfg = config.output;

    // Run three things concurrently; whichever finishes first triggers shutdown.
    tokio::select! {
        // Merger owns stdout; exits on broken pipe or channel close.
        res = crate::merge::output::run_merger(log_rx, output_cfg) => {
            if let Err(e) = res {
                return Err(AppError::Other(format!("output writer failed: {e}")));
            }
        }

        // Supervisor consumes pod lifecycle commands and spawns/cancels stream tasks.
        _ = async {
            while let Some(cmd) = cmd_rx.recv().await {
                supervisor.handle_command(cmd);
            }
        } => { }

        // Watcher: surface errors if it fails.
        join = async {
            watcher.take().unwrap().await
        } => {
            match join {
                Ok(Ok(())) => {}
                Ok(Err(e)) => return Err(e),
                Err(e) => return Err(AppError::Other(format!("pod watcher task failed: {e}"))),
            }
        }
    }

    // If select! exited due to merger/supervisor finishing, still await watcher to surface errors.
    if let Some(w) = watcher.take() {
        match w.await {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(e),
            Err(e) => Err(AppError::Other(format!("pod watcher task failed: {e}"))),
        }
    } else {
        Ok(())
    }
}
