pub mod cli;
pub mod config;
pub mod dev;
pub mod errors;
pub mod kube;
pub mod logging;
pub mod podwatch;
pub mod types;

use tokio::sync::mpsc;

use crate::config::Config;
use crate::errors::{AppError, AppResult};
use crate::types::PodCommand;

pub async fn run(config: Config) -> AppResult<()> {
    let (tx, mut rx) = mpsc::channel::<PodCommand>(128);

    // Start the appropriate "pod source" depending on mode.
    let watcher = if config.dev_mode {
        crate::dev::pods::spawn_dev_pods(config.namespace.clone(), tx)
    } else {
        let client = crate::kube::client::make_client().await?;
        crate::podwatch::watcher::spawn_pod_watcher(
            client,
            config.namespace.clone(),
            config.selector.clone(),
            tx,
        )
    };

    // For Slice 2/Slice 2.1: just print lifecycle decisions.
    // Slice 3 will turn StartPod/StopPod into stream task orchestration.
    while let Some(cmd) = rx.recv().await {
        match cmd {
            PodCommand::StartPod { pod, containers } => {
                tracing::info!(
                    namespace = %pod.namespace,
                    pod = %pod.name,
                    uid = %pod.uid,
                    containers = ?containers,
                    "StartPod"
                );
            }
            PodCommand::StopPod { pod } => {
                tracing::info!(
                    namespace = %pod.namespace,
                    pod = %pod.name,
                    uid = %pod.uid,
                    "StopPod"
                );
            }
        }
    }

    // If rx closes, watcher may still be running; await it so errors surface.
    match watcher.await {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(e) => Err(AppError::Other(format!("pod watcher task failed: {e}"))),
    }
}
