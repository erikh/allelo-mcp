#![allow(dead_code)]

use super::McpRequest;
use anyhow::Result;
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
    time::Instant,
};
use tokio::sync::{
    mpsc::{channel, Receiver, Sender},
    Mutex,
};
use uuid::Uuid;

pub type GlobalBroker = Arc<Mutex<Broker>>;
pub static GLOBAL_BROKER: LazyLock<GlobalBroker> = LazyLock::new(|| Default::default());

const CHANNEL_SIZE: usize = 1000;

#[derive(Debug)]
pub struct BrokerProxy<T> {
    last_message: Instant,
    pipe: BrokerPipe<T>,
}

impl<T> BrokerProxy<T>
where
    T: Sync + Send + 'static,
{
    pub fn new() -> Self {
        Self {
            last_message: Instant::now(),
            pipe: BrokerPipe::new(),
        }
    }

    pub async fn next_message(&mut self) -> Option<T> {
        self.pipe.receiver.recv().await.map(|x| {
            self.last_message = Instant::now();
            x
        })
    }

    pub async fn send_message(&mut self, msg: T) -> Result<()> {
        Ok(self.pipe.sender.send(msg).await.map(|x| {
            self.last_message = Instant::now();
            x
        })?)
    }
}

#[derive(Debug)]
pub struct BrokerPipe<T> {
    sender: Sender<T>,
    receiver: Receiver<T>,
}

impl<T> BrokerPipe<T>
where
    T: Sync + Send + 'static,
{
    pub fn new() -> Self {
        let (sender, receiver) = channel(CHANNEL_SIZE);
        Self { sender, receiver }
    }
}

// NOTE: this is probably not a long-term solution, but it should route requests between the API
// service and the various AI services / MCPs involved in the process. It will likely use a lot of
// memory and will likely need to be replaced with a dedicated queue before using in production.
#[derive(Debug, Clone, Default)]
pub struct Broker {
    mcp: HashMap<uuid::Uuid, Arc<Mutex<BrokerProxy<McpRequest>>>>,
    prompt: HashMap<uuid::Uuid, Arc<Mutex<BrokerProxy<String>>>>,
}

impl Broker {
    // FIXME: replace anyhow with thiserror here
    pub fn create_mcp(&mut self) -> Result<uuid::Uuid> {
        let uuid = Uuid::new_v4();
        self.mcp
            .insert(uuid, Arc::new(Mutex::new(BrokerProxy::new())));
        Ok(uuid)
    }

    // FIXME: replace anyhow with thiserror here
    pub fn create_prompt(&mut self) -> Result<uuid::Uuid> {
        let uuid = Uuid::new_v4();
        self.prompt
            .insert(uuid, Arc::new(Mutex::new(BrokerProxy::new())));
        Ok(uuid)
    }

    pub fn get_mcp(&self, id: uuid::Uuid) -> Option<Arc<Mutex<BrokerProxy<McpRequest>>>> {
        self.mcp.get(&id).cloned()
    }

    pub fn get_prompt(&self, id: uuid::Uuid) -> Option<Arc<Mutex<BrokerProxy<String>>>> {
        self.prompt.get(&id).cloned()
    }

    pub fn expire_mcp(&mut self, id: uuid::Uuid) {
        self.mcp.remove(&id);
    }

    pub fn expire_prompt(&mut self, id: uuid::Uuid) {
        self.prompt.remove(&id);
    }
}

#[cfg(test)]
mod tests {
    use crate::api::server::broker::{Broker, CHANNEL_SIZE};
    use tokio::sync::mpsc::channel;

    #[tokio::test]
    async fn test_broker() {
        let mut broker = Broker::default();

        let id = broker.create_prompt().unwrap();
        let proxy = broker.get_prompt(id).unwrap();
        let lock = proxy.lock().await;
        let start = lock.last_message.clone();
        drop(lock);

        let (s, mut r) = channel(1);

        let b = broker.clone();
        tokio::spawn(async move {
            for _ in 0..CHANNEL_SIZE {
                let proxy = b.get_prompt(id).unwrap();
                let mut lock = proxy.lock().await;
                match lock.send_message(Default::default()).await {
                    Err(e) => s.send(Err(e)).await.unwrap(),
                    _ => {}
                }
            }

            s.send(Ok(())).await.unwrap();
        });

        if let Some(Err(e)) = r.recv().await {
            assert!(false, "{}", e);
        }

        let proxy = broker.get_prompt(id).unwrap();
        let lock = proxy.lock().await;
        assert_ne!(lock.last_message, start);
    }
}
