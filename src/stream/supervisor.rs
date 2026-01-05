use std::collections::HashMap;

use kube::Client;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::config::KubeLogOpts;
use crate::errors::AppError;
use crate::types::{LogEvent, PodCommand, PodKey, StreamKey};

#[derive(Clone)]
pub enum StreamBackend {
    Dev {
        rate_ms: u64,
        max_lines: Option<u64>,
    },
    Kube {
        client: Client,
        opts: KubeLogOpts,
    },
}

pub struct StreamSupervisor {
    backend: StreamBackend,
    log_tx: mpsc::Sender<LogEvent>,
    fatal_tx: mpsc::Sender<AppError>,
    shutdown: CancellationToken,

    streams: HashMap<StreamKey, CancellationToken>,
}

impl StreamSupervisor {
    pub fn new(
        log_tx: mpsc::Sender<LogEvent>,
        fatal_tx: mpsc::Sender<AppError>,
        backend: StreamBackend,
        shutdown: CancellationToken,
    ) -> Self {
        Self {
            backend,
            log_tx,
            fatal_tx,
            shutdown,
            streams: HashMap::new(),
        }
    }

    pub fn handle_command(&mut self, cmd: PodCommand) {
        match cmd {
            PodCommand::StartPod { pod, containers } => self.start_pod(pod, containers),
            PodCommand::StopPod { pod } => self.stop_pod(pod),
        }
    }

    fn start_pod(&mut self, pod: PodKey, containers: Vec<String>) {
        for container in containers {
            let key = StreamKey {
                pod: pod.clone(),
                container: container.clone(),
            };

            if self.streams.contains_key(&key) {
                continue;
            }

            let token = self.shutdown.child_token();
            self.streams.insert(key.clone(), token.clone());

            let log_tx = self.log_tx.clone();
            let fatal_tx = self.fatal_tx.clone();

            match self.backend.clone() {
                StreamBackend::Dev { rate_ms, max_lines } => {
                    let pod_clone = pod.clone();
                    let container_clone = container.clone();

                    tokio::spawn(async move {
                        crate::stream::dev::dev_stream(
                            pod_clone,
                            container_clone,
                            log_tx,
                            rate_ms,
                            max_lines,
                        )
                        .await;

                        let _ = token;
                    });
                }

                StreamBackend::Kube { client, opts } => {
                    let pod_clone = pod.clone();
                    let container_clone = container.clone();

                    tokio::spawn(async move {
                        if let Err(e) = crate::stream::kube::kube_stream(
                            client,
                            pod_clone,
                            container_clone,
                            opts,
                            log_tx,
                            token,
                        )
                        .await
                        {
                            let _ = fatal_tx.send(e).await;
                        }
                    });
                }
            }
        }
    }

    fn stop_pod(&mut self, pod: PodKey) {
        self.streams.retain(|k, token| {
            if k.pod == pod {
                token.cancel();
                false
            } else {
                true
            }
        });
    }

    pub fn shutdown_all(&mut self) {
        for (_, token) in self.streams.drain() {
            token.cancel();
        }
    }
}
