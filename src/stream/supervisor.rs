use std::collections::HashMap;

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::types::{LogEvent, PodCommand, StreamKey};

pub struct StreamSupervisor {
    streams: HashMap<StreamKey, CancellationToken>,
    log_tx: mpsc::Sender<LogEvent>,
}

impl StreamSupervisor {
    pub fn new(log_tx: mpsc::Sender<LogEvent>) -> Self {
        Self {
            streams: HashMap::new(),
            log_tx,
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

                    let cancel = CancellationToken::new();
                    let child = cancel.child_token();
                    let tx = self.log_tx.clone();
                    let pod_clone = pod.clone();

                    tokio::spawn(async move {
                        tokio::select! {
                            _ = child.cancelled() => {}
                            _ = crate::stream::dev::dev_stream(pod_clone, container, tx) => {}
                        }
                    });

                    self.streams.insert(key, cancel);
                }
            }

            PodCommand::StopPod { pod } => {
                self.streams.retain(|key, cancel| {
                    if key.pod == pod {
                        cancel.cancel();
                        false
                    } else {
                        true
                    }
                });
            }
        }
    }
}
