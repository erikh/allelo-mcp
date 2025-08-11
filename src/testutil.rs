use crate::api::server::{broker::BrokerProxy, *};
use anyhow::Result;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::mpsc::{channel, Receiver, Sender};

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

#[allow(dead_code)]
#[derive(Debug)]
pub struct TestConnection<T>
where
    T: Sync + Send + 'static,
{
    input: Sender<T>,
    done: Arc<AtomicBool>,
}

#[allow(dead_code)]
impl<T> TestConnection<T>
where
    T: Sync + Send + 'static,
{
    pub fn new(proxy: BrokerProxy<T>) -> Self {
        let (s, r) = channel(broker::CHANNEL_SIZE);

        let done = Arc::new(AtomicBool::default());
        let done2 = done.clone();
        tokio::spawn(async move { Self::serve(r, done2, proxy).await.unwrap() });
        Self { input: s, done }
    }

    pub fn shutdown(&mut self) {
        self.done.store(true, Ordering::Relaxed);
    }

    async fn serve(
        mut r: Receiver<T>,
        done: Arc<AtomicBool>,
        mut proxy: BrokerProxy<T>,
    ) -> Result<()> {
        while let Some(x) = r.recv().await {
            if done.load(Ordering::Relaxed) {
                return Ok(());
            }

            proxy.send_message(x).await?;
        }

        Ok(())
    }
}
