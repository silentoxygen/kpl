use tokio_util::sync::CancellationToken;

/// Why we are shutting down (useful for logs + tests).
#[derive(Debug, Clone, Copy)]
pub enum ShutdownReason {
    CtrlC,
    Sigterm,
    OutputClosed, // e.g. broken pipe / downstream closed
    WatcherEnded,
    WatcherError,
    OutputError,
}

pub struct Shutdown {
    token: CancellationToken,
}

impl Shutdown {
    pub fn new() -> Self {
        Self {
            token: CancellationToken::new(),
        }
    }

    pub fn token(&self) -> CancellationToken {
        self.token.clone()
    }

    pub fn cancel(&self) {
        self.token.cancel();
    }
}

impl Default for Shutdown {
    fn default() -> Self {
        Self::new()
    }
}

/// Waits for Ctrl+C (SIGINT) and cancels the token.
/// Returns the reason so caller can log it.
pub async fn wait_ctrl_c(shutdown: &Shutdown) -> ShutdownReason {
    let _ = tokio::signal::ctrl_c().await;
    shutdown.cancel();
    ShutdownReason::CtrlC
}

/// Wait for SIGTERM on Unix (Linux/macOS). On non-Unix, this future never completes.
#[cfg(unix)]
pub async fn wait_sigterm(shutdown: &Shutdown) -> ShutdownReason {
    use tokio::signal::unix::{signal, SignalKind};

    match signal(SignalKind::terminate()) {
        Ok(mut sig) => {
            sig.recv().await;
            shutdown.cancel();
            ShutdownReason::Sigterm
        }
        Err(_) => {
            // If we can't register, just never fire.
            shutdown.token().cancelled().await;
            ShutdownReason::Sigterm
        }
    }
}

#[cfg(not(unix))]
pub async fn wait_sigterm(shutdown: &Shutdown) -> ShutdownReason {
    shutdown.token().cancelled().await;
    ShutdownReason::Sigterm
}
