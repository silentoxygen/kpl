use futures::AsyncBufReadExt;
use k8s_openapi::api::core::v1::Pod;
use kube::{api::LogParams, Api, Client};
use time::OffsetDateTime;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use tokio_util::sync::CancellationToken;

use crate::errors::AppError;
use crate::types::{KubeLogOpts, LogEvent, PodKey};

pub async fn kube_stream(
    client: Client,
    pod: PodKey,
    container: String,
    opts: KubeLogOpts,
    tx: mpsc::Sender<LogEvent>,
    fatal_tx: mpsc::Sender<AppError>,
    cancel: CancellationToken,
) {
    let mut backoff = Backoff::new(
        Duration::from_millis(opts.reconnect_min_ms),
        Duration::from_millis(opts.reconnect_max_ms),
    );

    loop {
        if cancel.is_cancelled() {
            return;
        }

        let api: Api<Pod> = Api::namespaced(client.clone(), &pod.namespace);

        let lp = LogParams {
            follow: true,
            timestamps: false, // v1 output timestamps are local unless we later parse kube timestamps
            container: Some(container.clone()),
            since_seconds: opts.since_seconds,
            tail_lines: opts.tail_lines,
            ..Default::default()
        };

        let mut reader = match api.log_stream(&pod.name, &lp).await {
            Ok(r) => {
                backoff.reset();
                r
            }
            Err(e) => {
                match classify_kube_error(&e) {
                    KubeErrClass::Auth => {
                        let _ = fatal_tx
                            .send(AppError::Other(format!(
                                "auth error streaming logs ({} {} {}): {e}",
                                pod.namespace, pod.name, container
                            )))
                            .await;
                        return;
                    }
                    KubeErrClass::NotFound => {
                        // Pod doesn't exist: watcher should drive lifecycle; stop this stream.
                        tracing::info!(
                            namespace = %pod.namespace,
                            pod = %pod.name,
                            container = %container,
                            "pod/container not found; stopping stream"
                        );
                        return;
                    }
                    KubeErrClass::Retry => {
                        tracing::warn!(
                            namespace = %pod.namespace,
                            pod = %pod.name,
                            container = %container,
                            error = %e,
                            "log_stream failed; backing off"
                        );
                        let d = backoff.next_delay_jittered();
                        sleep_or_cancel(d, &cancel).await;
                        continue;
                    }
                }
            }
        };

        let mut buf: Vec<u8> = Vec::with_capacity(8 * 1024);

        loop {
            buf.clear();

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
                        return; // output/merger gone
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

        let d = backoff.next_delay_jittered();
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

#[derive(Debug, Clone, Copy)]
enum KubeErrClass {
    Auth,
    NotFound,
    Retry,
}

fn classify_kube_error(e: &kube::Error) -> KubeErrClass {
    match e {
        kube::Error::Api(ae) => match ae.code {
            401 | 403 => KubeErrClass::Auth,
            404 => KubeErrClass::NotFound,
            429 => KubeErrClass::Retry,
            c if c >= 500 => KubeErrClass::Retry,
            _ => KubeErrClass::Retry,
        },
        _ => KubeErrClass::Retry,
    }
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

    fn next_delay_jittered(&mut self) -> Duration {
        // deterministic-ish jitter without adding rand:
        // uses current time nanos modulo a window.
        let base = self.cur;
        self.cur = std::cmp::min(self.cur * 2, self.max);

        let nanos = (time::OffsetDateTime::now_utc().nanosecond() % 250_000_000) as u64; // 0..250ms
        base + Duration::from_nanos(nanos)
    }
}
