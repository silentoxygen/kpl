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
use crate::shutdown::{Shutdown, ShutdownReason};
use crate::stream::supervisor::{StreamBackend, StreamSupervisor};
use crate::types::{LogEvent, PodCommand};

pub async fn run(config: Config) -> AppResult<()> {
    let shutdown = Shutdown::new();
    let shutdown_token: CancellationToken = shutdown.token();

    let (cmd_tx, mut cmd_rx) = mpsc::channel::<PodCommand>(128);
    let (log_tx, log_rx) = mpsc::channel::<LogEvent>(config.buffer);

    // Kube client (if not dev)
    let client_opt = if config.dev_mode {
        None
    } else {
        Some(crate::kube::client::make_client().await?)
    };

    // Watcher (pod source)
    let watcher_handle = if config.dev_mode {
        crate::dev::pods::spawn_dev_pods(config.namespace.clone(), cmd_tx)
    } else {
        let client = client_opt.clone().expect("client must exist when not dev");
        crate::podwatch::watcher::spawn_pod_watcher(
            client,
            config.namespace.clone(),
            config.selector.clone(),
            cmd_tx,
        )
    };
    let mut watcher = Some(watcher_handle);

    // Stream backend
    let backend = if config.dev_mode {
        StreamBackend::Dev {
            rate_ms: config.dev_rate_ms,
            max_lines: config.dev_lines,
        }
    } else {
        StreamBackend::Kube {
            client: client_opt.clone().unwrap(),
        }
    };

    let mut supervisor = StreamSupervisor::new(log_tx, backend, shutdown_token.clone());

    // Output runs in the current task (stdout lock is !Send)
    let output_cfg = config.output;

    // Supervisor loop future (runs until cmd channel closes or shutdown)
    let cmd_loop_shutdown = shutdown_token.clone();
    let supervisor_loop = async move {
        while let Some(cmd) = cmd_rx.recv().await {
            if cmd_loop_shutdown.is_cancelled() {
                break;
            }
            supervisor.handle_command(cmd);
        }
        supervisor
    };

    // Signal waiters
    let ctrl_c_fut = crate::shutdown::wait_ctrl_c(&shutdown);
    let sigterm_fut = crate::shutdown::wait_sigterm(&shutdown);

    // Main orchestration: whichever finishes first triggers shutdown.
    let reason = tokio::select! {
        r = ctrl_c_fut => r,
        r = sigterm_fut => r,

        // Output ended (broken pipe or channel closed) OR output error.
        res = crate::merge::output::run_merger(log_rx, output_cfg) => {
            match res {
                Ok(()) => {
                    shutdown.cancel();
                    ShutdownReason::OutputClosed
                }
                Err(e) => {
                    shutdown.cancel();
                    tracing::error!(error = %e, "output writer failed");
                    ShutdownReason::OutputError
                }
            }
        }

        // Watcher ended (success/error)
        join = async { watcher.take().unwrap().await } => {
            match join {
                Ok(Ok(())) => {
                    shutdown.cancel();
                    ShutdownReason::WatcherEnded
                }
                Ok(Err(e)) => {
                    shutdown.cancel();
                    tracing::error!(error = %e, "pod watcher error");
                    ShutdownReason::WatcherError
                }
                Err(e) => {
                    shutdown.cancel();
                    tracing::error!(error = %e, "pod watcher task failed");
                    ShutdownReason::WatcherError
                }
            }
        }

        // Supervisor loop ends (cmd channel closed)
        _ = shutdown_token.cancelled() => {
            ShutdownReason::WatcherEnded
        }
    };

    tracing::info!(reason = ?reason, "shutting down");

    // Teardown order:
    // 1) ensure global shutdown
    shutdown.cancel();

    // 2) stop all streams via supervisor (retrieve it back from the loop)
    let mut supervisor = supervisor_loop.await;
    supervisor.shutdown_all();

    // 3) abort watcher if still running
    if let Some(w) = watcher.take() {
        w.abort();
    }

    Ok(())
}
