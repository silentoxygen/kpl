use k8s_openapi::api::core::v1::Pod;
use kube::{api::LogParams, Api, Client};
use time::OffsetDateTime;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use tokio_util::sync::CancellationToken;

use futures::AsyncBufReadExt; // <-- IMPORTANT: futures, not tokio

use crate::types::{LogEvent, PodKey};

/// Stream logs for a single (pod, container) until cancelled.
/// Reconnects on transient errors/EOF with backoff.
/// For v1, we do NOT backfill old logs; we follow from "now".
pub async fn kube_stream(
    client: Client,
    pod: PodKey,
    container: String,
    tx: mpsc::Sender<LogEvent>,
    cancel: CancellationToken,
) {
    let mut backoff = Backoff::new(Duration::from_millis(200), Duration::from_secs(5));

    loop {
        if cancel.is_cancelled() {
            return;
        }

        let api: Api<Pod> = Api::namespaced(client.clone(), &pod.namespace);

        let lp = LogParams {
            follow: true,
            timestamps: false,
            container: Some(container.clone()),
            ..Default::default()
        };

        let mut reader = match api.log_stream(&pod.name, &lp).await {
            Ok(r) => {
                backoff.reset();
                r
            }
            Err(e) => {
                tracing::warn!(
                    namespace = %pod.namespace,
                    pod = %pod.name,
                    container = %container,
                    error = %e,
                    "log_stream failed; backing off"
                );
                let d = backoff.next_delay();
                sleep_or_cancel(d, &cancel).await;
                continue;
            }
        };

        let mut buf: Vec<u8> = Vec::with_capacity(8 * 1024);

        loop {
            buf.clear();

            // futures::AsyncBufReadExt::read_until is an async method returning a Future.
            // We select on cancellation.
            let read_fut = reader.read_until(b'\n', &mut buf);

            let res = tokio::select! {
                _ = cancel.cancelled() => return,
                r = read_fut => r,
            };

            match res {
                Ok(0) => {
                    // EOF -> reconnect
                    break;
                }
                Ok(_n) => {
                    let line = trim_newline(&buf);
                    if line.is_empty() {
                        continue;
                    }

                    let msg = String::from_utf8_lossy(line).to_string();

                    let ev = LogEvent {
                        ts: OffsetDateTime::now_utc(),
                        namespace: pod.namespace.clone(),
                        pod: pod.name.clone(),
                        container: container.clone(),
                        message: msg,
                    };

                    if tx.send(ev).await.is_err() {
                        return; // merger is gone
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        namespace = %pod.namespace,
                        pod = %pod.name,
                        container = %container,
                        error = %e,
                        "log reader error; reconnecting"
                    );
                    break;
                }
            }
        }

        let d = backoff.next_delay();
        sleep_or_cancel(d, &cancel).await;
    }
}

async fn sleep_or_cancel(d: Duration, cancel: &CancellationToken) {
    tokio::select! {
        _ = sleep(d) => {}
        _ = cancel.cancelled() => {}
    }
}

fn trim_newline(bytes: &[u8]) -> &[u8] {
    let mut end = bytes.len();
    while end > 0 && (bytes[end - 1] == b'\n' || bytes[end - 1] == b'\r') {
        end -= 1;
    }
    &bytes[..end]
}

struct Backoff {
    cur: Duration,
    min: Duration,
    max: Duration,
}

impl Backoff {
    fn new(min: Duration, max: Duration) -> Self {
        Self { cur: min, min, max }
    }

    fn reset(&mut self) {
        self.cur = self.min;
    }

    fn next_delay(&mut self) -> Duration {
        let d = self.cur;
        self.cur = std::cmp::min(self.cur * 2, self.max);
        d
    }
}
