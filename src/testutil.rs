use crate::api::server::*;
use anyhow::Result;

const DEFAULT_API_URL: &str = "http://localhost:8999";

pub fn default_api_url() -> url::Url {
    DEFAULT_API_URL.parse().unwrap()
}

pub async fn start_api_server(config: Config) -> Result<axum_server::Handle> {
    let handle = axum_server::Handle::new();
    let server = Server::new(config).await?;
    let h = handle.clone();
    tokio::spawn(async move { server.start_with_handle(handle).await.unwrap() });
    Ok(h)
}

pub fn shutdown_handle(handle: axum_server::Handle) {
    handle.graceful_shutdown(Some(std::time::Duration::from_secs(10)));
}
