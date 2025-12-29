use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

use time::OffsetDateTime;

use crate::types::{LogEvent, PodKey};

pub async fn dev_stream(
    pod: PodKey,
    container: String,
    tx: mpsc::Sender<LogEvent>,
    rate_ms: u64,
    max_lines: Option<u64>,
) {
    let mut counter: u64 = 0;

    loop {
        counter += 1;

        let event = LogEvent {
            ts: OffsetDateTime::now_utc(),
            namespace: pod.namespace.clone(),
            pod: pod.name.clone(),
            container: container.clone(),
            message: format!("log line {}", counter),
        };

        if tx.send(event).await.is_err() {
            break;
        }

        if let Some(max) = max_lines {
            if counter >= max {
                break;
            }
        }

        sleep(Duration::from_millis(rate_ms)).await;
    }
}
