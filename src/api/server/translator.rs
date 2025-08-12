use super::broker::BrokerProxy;
use anyhow::Result;
use std::sync::{atomic::AtomicBool, Arc};
use tokio::sync::mpsc::Receiver;

#[allow(dead_code)]
#[async_trait::async_trait]
pub trait Translator<T, R>
where
    T: Send + Sync + 'static,
    R: Send + Sync + 'static,
{
    async fn serve(
        self,
        mut r: Receiver<T>,
        done: Arc<AtomicBool>,
        mut proxy: BrokerProxy<T, R>,
    ) -> Result<()>;
}
