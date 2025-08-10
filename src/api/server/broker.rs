#![allow(unused)]

use super::McpRequest;
use anyhow::{anyhow, Result};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc::{channel, Receiver, Sender};

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
#[derive(Debug, Clone)]
pub struct Broker {
    mcp: HashMap<uuid::Uuid, Arc<BrokerPipe<McpRequest>>>,
    prompt: HashMap<uuid::Uuid, Arc<BrokerPipe<String>>>,
}
