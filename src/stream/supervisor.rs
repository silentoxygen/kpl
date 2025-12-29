use std::collections::HashMap;

use kube::Client;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::types::{LogEvent, PodCommand, StreamKey};

#[derive(Clone)]
pub enum StreamBackend {
    Dev {
        rate_ms: u64,
        max_lines: Option<u64>,
    },
    Kube {
        client: Client,
    },
}

pub struct StreamSupervisor {
    streams: HashMap<StreamKey, CancellationToken>,
    log_tx: mpsc::Sender<LogEvent>,
    backend: StreamBackend,
    shutdown: CancellationToken,
}

impl StreamSupervisor {
    pub fn new(
        log_tx: mpsc::Sender<LogEvent>,
        backend: StreamBackend,
        shutdown: CancellationToken,
    ) -> Self {
        Self {
            streams: HashMap::new(),
            log_tx,
            backend,
            shutdown,
        }
    }

    pub fn handle_command(&mut self, cmd: PodCommand) {
        match cmd {
            PodCommand::StartPod { pod, containers } => {
                for container in containers {
                    let key = StreamKey {
                        pod: pod.clone(),
                        container: container.clone(),
                    };

                    if self.streams.contains_key(&key) {
                        continue;
                    }

                    let token = self.shutdown.child_token();
                    let stream_token = token.child_token();

                    let tx = self.log_tx.clone();
                    let pod_clone = pod.clone();

                    match self.backend.clone() {
                        StreamBackend::Dev { rate_ms, max_lines } => {
                            tokio::spawn(async move {
                                tokio::select! {
                                    _ = stream_token.cancelled() => {}
                                    _ = crate::stream::dev::dev_stream(
                                        pod_clone,
                                        container,
                                        tx,
                                        rate_ms,
                                        max_lines,
                                    ) => {}
                                }
                            });
                        }
                        StreamBackend::Kube { client } => {
                            tokio::spawn(async move {
                                crate::stream::kube::kube_stream(
                                    client,
                                    pod_clone,
                                    container,
                                    tx,
                                    stream_token,
                                )
                                .await;
                            });
                        }
                    }

                    self.streams.insert(key, token);
                }
            }

            PodCommand::StopPod { pod } => {
                self.streams.retain(|key, token| {
                    if key.pod == pod {
                        token.cancel();
                        false
                    } else {
                        true
                    }
                });
            }
        }
    }

    pub fn shutdown_all(&mut self) {
        for (_, token) in self.streams.drain() {
            token.cancel();
        }
    }
}
