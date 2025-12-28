use std::collections::HashMap;

use k8s_openapi::api::core::v1::Pod;
use kube::{Api, Client};
use tokio::sync::mpsc;

use crate::errors::AppResult;
use crate::types::{PodCommand, PodKey};

use futures::{pin_mut, StreamExt};

#[derive(Debug, Clone)]
struct PodState {
    uid: String,
    containers: Vec<String>,
}

/// Watches pods matching (namespace, selector) and emits StartPod/StopPod commands.
///
/// v1 behavior:
/// - Only uses spec.containers (excludes initContainers)
/// - Uses pod UID to detect replacement (rollouts)
pub fn spawn_pod_watcher(
    client: Client,
    namespace: String,
    selector: String,
    tx: mpsc::Sender<PodCommand>,
) -> tokio::task::JoinHandle<AppResult<()>> {
    tokio::spawn(async move {
        let pods: Api<Pod> = Api::namespaced(client, &namespace);

        // kube-runtime watcher takes its own Config (NOT ListParams)
        let wc = kube_runtime::watcher::Config::default().labels(&selector);

        tracing::info!(%namespace, %selector, "starting pod watcher");

        // Keyed by (namespace, pod name) -> state (uid + containers)
        let mut active: HashMap<(String, String), PodState> = HashMap::new();

        let stream = kube_runtime::watcher(pods, wc);
        pin_mut!(stream);

        while let Some(item) = stream.next().await {
            let ev = item?; // watcher::Error -> AppError via From impl (we'll add it)

            match ev {
                kube_runtime::watcher::Event::Applied(pod) => {
                    if let Some((ns, name, uid, containers)) = extract_pod_identity(&pod) {
                        let key = (ns.clone(), name.clone());

                        match active.get(&key) {
                            None => {
                                active.insert(
                                    key,
                                    PodState {
                                        uid: uid.clone(),
                                        containers: containers.clone(),
                                    },
                                );
                                emit_start(&tx, ns, name, uid, containers).await;
                            }
                            Some(prev) if prev.uid != uid => {
                                tracing::warn!(
                                    namespace = %ns,
                                    pod = %name,
                                    old_uid = %prev.uid,
                                    new_uid = %uid,
                                    "pod replaced (uid changed)"
                                );

                                emit_stop(&tx, ns.clone(), name.clone(), prev.uid.clone()).await;

                                active.insert(
                                    key,
                                    PodState {
                                        uid: uid.clone(),
                                        containers: containers.clone(),
                                    },
                                );
                                emit_start(&tx, ns, name, uid, containers).await;
                            }
                            Some(prev) => {
                                // Same UID: containers might change; update state (no stop/start yet).
                                if prev.containers != containers {
                                    tracing::debug!(
                                        namespace = %ns,
                                        pod = %name,
                                        uid = %uid,
                                        "container set changed; updating state"
                                    );
                                    active.insert(key, PodState { uid, containers });
                                }
                            }
                        }
                    }
                }

                kube_runtime::watcher::Event::Deleted(pod) => {
                    if let Some((ns, name, uid, _)) = extract_pod_identity(&pod) {
                        let key = (ns.clone(), name.clone());

                        if let Some(prev) = active.remove(&key) {
                            let stop_uid = if prev.uid.is_empty() { uid } else { prev.uid };
                            emit_stop(&tx, ns, name, stop_uid).await;
                        }
                    }
                }

                kube_runtime::watcher::Event::Restarted(pod_list) => {
                    tracing::info!("watcher restarted; resyncing pod set");

                    let mut new_active: HashMap<(String, String), PodState> = HashMap::new();

                    for pod in pod_list {
                        if let Some((ns, name, uid, containers)) = extract_pod_identity(&pod) {
                            new_active.insert((ns, name), PodState { uid, containers });
                        }
                    }

                    // Stop missing/changed
                    for ((ns, name), old_state) in active.iter() {
                        match new_active.get(&(ns.clone(), name.clone())) {
                            None => {
                                emit_stop(&tx, ns.clone(), name.clone(), old_state.uid.clone())
                                    .await;
                            }
                            Some(new_state) if new_state.uid != old_state.uid => {
                                emit_stop(&tx, ns.clone(), name.clone(), old_state.uid.clone())
                                    .await;
                            }
                            _ => {}
                        }
                    }

                    // Start new/changed
                    for ((ns, name), new_state) in new_active.iter() {
                        match active.get(&(ns.clone(), name.clone())) {
                            None => {
                                emit_start(
                                    &tx,
                                    ns.clone(),
                                    name.clone(),
                                    new_state.uid.clone(),
                                    new_state.containers.clone(),
                                )
                                .await;
                            }
                            Some(old_state) if old_state.uid != new_state.uid => {
                                emit_start(
                                    &tx,
                                    ns.clone(),
                                    name.clone(),
                                    new_state.uid.clone(),
                                    new_state.containers.clone(),
                                )
                                .await;
                            }
                            _ => {}
                        }
                    }

                    active = new_active;
                }
            }
        }

        Ok(())
    })
}

fn extract_pod_identity(pod: &Pod) -> Option<(String, String, String, Vec<String>)> {
    let ns = pod
        .metadata
        .namespace
        .clone()
        .unwrap_or_else(|| "default".to_string());
    let name = pod.metadata.name.clone()?;
    let uid = pod.metadata.uid.clone().unwrap_or_default();

    let spec = pod.spec.as_ref()?;
    let containers: Vec<String> = spec.containers.iter().map(|c| c.name.clone()).collect();

    Some((ns, name, uid, containers))
}

async fn emit_start(
    tx: &mpsc::Sender<PodCommand>,
    namespace: String,
    pod: String,
    uid: String,
    containers: Vec<String>,
) {
    let cmd = PodCommand::StartPod {
        pod: PodKey {
            namespace,
            name: pod,
            uid,
        },
        containers,
    };

    let _ = tx.send(cmd).await;
}

async fn emit_stop(tx: &mpsc::Sender<PodCommand>, namespace: String, pod: String, uid: String) {
    let cmd = PodCommand::StopPod {
        pod: PodKey {
            namespace,
            name: pod,
            uid,
        },
    };

    let _ = tx.send(cmd).await;
}
