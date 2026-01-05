use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

use crate::errors::AppResult;
use crate::types::{PodCommand, PodKey};

pub fn spawn_dev_pods(
    namespace: String,
    tx: mpsc::Sender<PodCommand>,
) -> tokio::task::JoinHandle<AppResult<()>> {
    tokio::spawn(async move {
        tracing::info!("starting dev-mode pod source");

        let pod = PodKey {
            namespace: namespace.clone(),
            name: "dev-pod-1".to_string(),
            uid: "dev-uid-1".to_string(),
        };

        tx.send(PodCommand::StartPod {
            pod: pod.clone(),
            containers: vec!["app".to_string(), "sidecar".to_string()],
        })
        .await
        .ok();

        sleep(Duration::from_secs(5)).await;

        tracing::info!("simulating pod restart");

        tx.send(PodCommand::StopPod { pod: pod.clone() }).await.ok();

        let pod2 = PodKey {
            uid: "dev-uid-2".to_string(),
            ..pod
        };

        tx.send(PodCommand::StartPod {
            pod: pod2,
            containers: vec!["app".to_string(), "sidecar".to_string()],
        })
        .await
        .ok();

        sleep(Duration::from_secs(5)).await;

        tracing::info!("dev-mode finished");

        Ok(())
    })
}
