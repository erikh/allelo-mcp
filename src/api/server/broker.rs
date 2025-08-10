#![allow(dead_code)]

use super::McpRequest;
use anyhow::Result;
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
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

    pub async fn next_message(&mut self) -> Option<T> {
        self.receiver.recv().await
    }

    pub async fn send_message(&mut self, msg: T) -> Result<()> {
        Ok(self.sender.send(msg).await?)
    }
}

// NOTE: this is probably not a long-term solution, but it should route requests between the API
// service and the various AI services / MCPs involved in the process. It will likely use a lot of
// memory and will likely need to be replaced with a dedicated queue before using in production.
#[derive(Debug, Clone, Default)]
pub struct Broker {
    mcp: HashMap<uuid::Uuid, Arc<BrokerPipe<McpRequest>>>,
    prompt: HashMap<uuid::Uuid, Arc<BrokerPipe<String>>>,
}

impl Broker {
    // FIXME: replace anyhow with thiserror here
    pub fn create_mcp(&mut self) -> Result<uuid::Uuid> {
        let uuid = Uuid::new_v4();
        self.mcp.insert(uuid, Arc::new(BrokerPipe::new()));
        Ok(uuid)
    }

    // FIXME: replace anyhow with thiserror here
    pub fn create_prompt(&mut self) -> Result<uuid::Uuid> {
        let uuid = Uuid::new_v4();
        self.prompt.insert(uuid, Arc::new(BrokerPipe::new()));
        Ok(Default::default())
    }

    pub fn get_mcp(&self, id: uuid::Uuid) -> Option<Arc<BrokerPipe<McpRequest>>> {
        self.mcp.get(&id).cloned()
    }

    pub fn get_prompt(&self, id: uuid::Uuid) -> Option<Arc<BrokerPipe<String>>> {
        self.prompt.get(&id).cloned()
    }

    pub fn expire_mcp(&mut self, id: uuid::Uuid) {
        self.mcp.remove(&id);
    }

    pub fn expire_prompt(&mut self, id: uuid::Uuid) {
        self.prompt.remove(&id);
    }
}
