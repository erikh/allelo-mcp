use crate::api::server::{broker::BrokerProxy, translator::Translator, *};
use anyhow::Result;
use std::{
    marker::PhantomData,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
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
pub struct TestConnection<T, R>
where
    T: Sync + Send + 'static,
    R: Sync + Send + 'static,
{
    input: Sender<T>,
    done: Arc<AtomicBool>,
    r: PhantomData<R>,
}

#[allow(dead_code)]
impl<T, R> TestConnection<T, R>
where
    T: Sync + Send + 'static,
    R: From<T> + Sync + Send + 'static,
{
    pub fn new(proxy: BrokerProxy<T, R>) -> Self {
        let (s, r) = channel(broker::CHANNEL_SIZE);

        let done = Arc::new(AtomicBool::default());
        let done2 = done.clone();
        tokio::spawn(async move { Self::serve(r, done2, proxy).await.unwrap() });
        Self {
            input: s,
            done,
            r: Default::default(),
        }
    }

    pub fn shutdown(&mut self) {
        self.done.store(true, Ordering::Relaxed);
    }
}

#[async_trait::async_trait]
impl<T, R> Translator<T, R> for TestConnection<T, R>
where
    T: Sync + Send + 'static,
    R: From<T> + Sync + Send + 'static,
{
    async fn serve(
        mut r: Receiver<T>,
        done: Arc<AtomicBool>,
        mut proxy: BrokerProxy<T, R>,
    ) -> Result<()> {
        loop {
            tokio::select! {
                Some(_x) = proxy.input().next_message() => {
                    // discard input
                },
                Some(x) = r.recv() => {
                    proxy.output().send_message(x.into()).await?;
                }
                else => {
                    if done.load(Ordering::Relaxed) || proxy.check_timeout() {
                        return Ok(());
                    }
                }
            }
        }
    }
}
