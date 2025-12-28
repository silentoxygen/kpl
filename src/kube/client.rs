use crate::errors::AppResult;

pub async fn make_client() -> AppResult<kube::Client> {
    // Uses KUBECONFIG / ~/.kube/config out-of-cluster, or in-cluster config.
    let client = kube::Client::try_default().await?;
    Ok(client)
}
