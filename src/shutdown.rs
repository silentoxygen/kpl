use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Copy)]
pub enum ShutdownReason {
    CtrlC,
    Sigterm,
    OutputClosed,
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

pub async fn wait_ctrl_c(shutdown: &Shutdown) -> ShutdownReason {
    let _ = tokio::signal::ctrl_c().await;
    shutdown.cancel();
    ShutdownReason::CtrlC
}

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
