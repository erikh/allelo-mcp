#![allow(unused)]

use std::{collections::HashMap, sync::Arc};

use tokio::sync::mpsc::{channel, Receiver, Sender};

use super::McpRequest;

const CHANNEL_SIZE: usize = 1000;

#[derive(Debug)]
pub struct BrokerPipe<T> {
    sender: Sender<T>,
    receiver: Receiver<T>,
}

#[derive(Debug, Clone)]
pub struct Broker {
    mcp_routes: HashMap<uuid::Uuid, Arc<BrokerPipe<McpRequest>>>,
}
