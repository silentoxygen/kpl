use futures::AsyncBufReadExt;
use kube::api::LogParams;
use kube::Client;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::config::KubeLogOpts;
use crate::errors::AppResult;
use crate::types::{LogEvent, PodKey};

pub async fn kube_stream(
    client: Client,
    pod: PodKey,
    container: String,
    _opts: KubeLogOpts,
    tx: mpsc::Sender<LogEvent>,
    shutdown: CancellationToken,
) -> AppResult<()> {
    let pods: kube::Api<k8s_openapi::api::core::v1::Pod> =
        kube::Api::namespaced(client, &pod.namespace);

    let lp = LogParams {
        follow: true,
        timestamps: false,
        container: Some(container.clone()),
        ..Default::default()
    };

    let mut reader = pods.log_stream(&pod.name, &lp).await?;

    let mut line = String::new();

    loop {
        line.clear();

        tokio::select! {
            _ = shutdown.cancelled() => {
                return Ok(());
            }

            res = reader.read_line(&mut line) => {
                let n = res?;
                if n == 0 {
                    return Ok(());
                }
                while line.ends_with('\n') || line.ends_with('\r') {
                    line.pop();
                }

                let _ = tx.send(LogEvent {
                    ts: time::OffsetDateTime::now_utc(),
                    namespace: pod.namespace.clone(),
                    pod: pod.name.clone(),
                    container: container.clone(),
                    message: line.clone(),
                }).await;
            }
        }
    }
}
