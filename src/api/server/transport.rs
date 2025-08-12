#![allow(dead_code)]
use super::broker::{BrokerProxy, CHANNEL_SIZE};
use anyhow::Result;
use std::{
    marker::PhantomData,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tokio::sync::mpsc::{channel, Receiver, Sender};

#[derive(Debug, Clone)]
pub struct Connection<T, R>
where
    T: Clone + Sync + Send + 'static,
    R: Clone + Sync + Send + 'static,
{
    input: Sender<T>,
    done: Arc<AtomicBool>,
    r: PhantomData<R>,
}

impl<T, R> Connection<T, R>
where
    T: Clone + Sync + Send + 'static,
    R: Clone + From<T> + Sync + Send + 'static,
{
    pub fn new(proxy: BrokerProxy<T, R>) -> Self {
        let (s, r) = channel(CHANNEL_SIZE);

        let done = Arc::new(AtomicBool::default());
        let done2 = done.clone();
        let this = Self {
            input: s,
            done,
            r: Default::default(),
        };

        let ret = this.clone();
        tokio::spawn(async move { this.serve(r, done2, proxy).await.unwrap() });
        ret
    }

    pub fn shutdown(&mut self) {
        self.done.store(true, Ordering::Relaxed);
    }

    async fn serve(
        self,
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
