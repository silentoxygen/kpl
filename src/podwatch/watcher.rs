use std::collections::HashSet;

use futures::{pin_mut, StreamExt};
use k8s_openapi::api::core::v1::Pod;
use kube::{Api, Client, Resource, ResourceExt};
use kube_runtime::watcher;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::errors::{AppError, AppResult};
use crate::types::{PodCommand, PodKey};

pub fn spawn_pod_watcher(
    client: Client,
    namespace: String,
    selector: String,
    containers_filter: Vec<String>,
    tx: mpsc::Sender<PodCommand>,
) -> JoinHandle<AppResult<()>> {
    tokio::spawn(async move {
        let api: Api<Pod> = Api::namespaced(client, &namespace);

        let stream = watcher(api, watcher::Config::default().labels(&selector));
        pin_mut!(stream);

        while let Some(item) = stream.next().await {
            let ev = item.map_err(|e| AppError::Other(format!("watcher error: {e}")))?;

            match ev {
                watcher::Event::Applied(pod) => {
                    let Some(uid) = pod.meta().uid.clone() else {
                        continue;
                    };
                    let name = pod.name_any();

                    let pod_key = PodKey {
                        namespace: namespace.clone(),
                        name,
                        uid,
                    };

                    let containers = pick_containers(&pod, &containers_filter);

                    if !containers.is_empty() {
                        let _ = tx
                            .send(PodCommand::StartPod {
                                pod: pod_key,
                                containers,
                            })
                            .await;
                    }
                }

                watcher::Event::Deleted(pod) => {
                    let Some(uid) = pod.meta().uid.clone() else {
                        continue;
                    };
                    let name = pod.name_any();

                    let pod_key = PodKey {
                        namespace: namespace.clone(),
                        name,
                        uid,
                    };

                    let _ = tx.send(PodCommand::StopPod { pod: pod_key }).await;
                }

                watcher::Event::Restarted(pods) => {
                    for pod in pods {
                        let Some(uid) = pod.meta().uid.clone() else {
                            continue;
                        };
                        let name = pod.name_any();

                        let pod_key = PodKey {
                            namespace: namespace.clone(),
                            name,
                            uid,
                        };

                        let containers = pick_containers(&pod, &containers_filter);
                        if !containers.is_empty() {
                            let _ = tx
                                .send(PodCommand::StartPod {
                                    pod: pod_key,
                                    containers,
                                })
                                .await;
                        }
                    }
                }
            }
        }

        Ok(())
    })
}

fn pick_containers(pod: &Pod, filter: &[String]) -> Vec<String> {
    let mut names: Vec<String> = pod
        .spec
        .as_ref()
        .map(|s| s.containers.iter().map(|c| c.name.clone()).collect())
        .unwrap_or_default();

    if filter.is_empty() {
        return names;
    }

    let present: HashSet<String> = names.iter().cloned().collect();
    for want in filter {
        if !present.contains(want) {
            tracing::warn!(
                pod = %pod.name_any(),
                container = %want,
                "requested container not found in pod spec"
            );
        }
    }

    names.retain(|c| filter.contains(c));
    names
}
