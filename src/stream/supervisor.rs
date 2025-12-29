use std::collections::HashMap;

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::types::{LogEvent, PodCommand, StreamKey};

pub struct StreamSupervisor {
    streams: HashMap<StreamKey, CancellationToken>,
    log_tx: mpsc::Sender<LogEvent>,
    dev_rate_ms: u64,
    dev_lines: Option<u64>,
}

impl StreamSupervisor {
    pub fn new(log_tx: mpsc::Sender<LogEvent>, dev_rate_ms: u64, dev_lines: Option<u64>) -> Self {
        Self {
            streams: HashMap::new(),
            log_tx,
            dev_rate_ms,
            dev_lines,
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
                    let rate_ms = self.dev_rate_ms;
                    let max_lines = self.dev_lines;

                    tokio::spawn(async move {
                        tokio::select! {
                            _ = child.cancelled() => {}
                            _ = crate::stream::dev::dev_stream(pod_clone, container, tx, rate_ms, max_lines) => {}
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
