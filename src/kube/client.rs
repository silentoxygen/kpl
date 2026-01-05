use crate::errors::AppResult;

pub async fn make_client() -> AppResult<kube::Client> {
    let client = kube::Client::try_default().await?;
    Ok(client)
}
