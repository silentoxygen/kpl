use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

use crate::types::{LogEvent, PodKey};
use time::OffsetDateTime;

pub async fn dev_stream(pod: PodKey, container: String, tx: mpsc::Sender<LogEvent>) {
    let mut counter = 0u64;

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
            // merger is gone → stop
            break;
        }

        sleep(Duration::from_millis(500)).await;
    }
}
