pub mod cli;
pub mod config;
pub mod dev;
pub mod errors;
pub mod kube;
pub mod logging;
pub mod merge;
pub mod podwatch;
pub mod shutdown;
pub mod stream;
pub mod types;

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::config::Config;
use crate::errors::AppResult;
use crate::types::PodCommand;

pub async fn run(config: Config) -> AppResult<()> {
    let shutdown = crate::shutdown::Shutdown::new();
    let shutdown_token: CancellationToken = shutdown.token();
    let monitor_shutdown: CancellationToken = shutdown_token.clone();

    // Command channel (pod lifecycle)
    let (cmd_tx, mut cmd_rx) = mpsc::channel::<PodCommand>(128);

    // Log event channel
    let (log_tx, log_rx) = mpsc::channel(config.runtime.buffer);

    // Spawn pod source (dev or kube)
    let watcher_handle = if config.dev_mode {
        crate::dev::pods::spawn_dev_pods(config.namespace.clone(), cmd_tx)
    } else {
        let client = crate::kube::client::make_client().await?;
        crate::podwatch::watcher::spawn_pod_watcher(
            client,
            config.namespace.clone(),
            config.selector.clone(),
            config.kube.containers.clone(),
            cmd_tx,
        )
    };

    // Output merger config
    let output_cfg = config.output.clone();

    // Stream supervisor backend
    let backend = if config.dev_mode {
        crate::stream::supervisor::StreamBackend::Dev {
            rate_ms: config.dev.rate_ms,
            max_lines: config.dev.lines,
        }
    } else {
        crate::stream::supervisor::StreamBackend::Kube {
            client: crate::kube::client::make_client().await?,
            opts: config.kube.clone(),
        }
    };

    let (fatal_tx, _fatal_rx) = mpsc::channel(1);

    let mut supervisor = crate::stream::supervisor::StreamSupervisor::new(
        log_tx,
        fatal_tx,
        backend,
        shutdown_token.clone(),
    );

    // Supervisor command loop (background)
    let cmd_loop_shutdown = shutdown_token.clone();
    let supervisor_task = tokio::spawn(async move {
        while let Some(cmd) = cmd_rx.recv().await {
            if cmd_loop_shutdown.is_cancelled() {
                break;
            }
            supervisor.handle_command(cmd);
        }
        supervisor
    });

    // SIGTERM future (portable)
    #[cfg(unix)]
    let sigterm_fut = async {
        use tokio::signal::unix::{signal, SignalKind};
        let mut sigterm =
            signal(SignalKind::terminate()).expect("failed to register SIGTERM handler");
        sigterm.recv().await;
    };

    #[cfg(not(unix))]
    let sigterm_fut = async { std::future::pending::<()>().await };

    // Monitor task: triggers shutdown
    let monitor_task = tokio::spawn(async move {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                monitor_shutdown.cancel();
                tracing::info!(reason="CtrlC", "shutdown requested");
            }
            _ = sigterm_fut => {
                monitor_shutdown.cancel();
                tracing::info!(reason="Sigterm", "shutdown requested");
            }
            join = watcher_handle => {
                match join {
                    Ok(Ok(())) => tracing::info!(reason="WatcherEnded", "shutdown requested"),
                    Ok(Err(e)) => tracing::error!(error=%e, reason="WatcherError", "shutdown requested"),
                    Err(e) => tracing::error!(error=%e, reason="WatcherTaskJoinError", "shutdown requested"),
                }
                monitor_shutdown.cancel();
            }
        }
    });

    // Foreground output merger (stdout lock is !Send)
    let merger_res =
        crate::merge::output::run_merger(log_rx, output_cfg, shutdown_token.clone()).await;

    // Cleanup
    shutdown_token.cancel();

    let mut supervisor = match supervisor_task.await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(error=%e, "supervisor task failed");
            let _ = monitor_task.await;
            return merger_res;
        }
    };
    supervisor.shutdown_all();

    let _ = monitor_task.await;

    merger_res
}
