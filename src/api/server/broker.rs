use super::McpRequest;
use anyhow::Result;
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
    time::Instant,
};
use tokio::sync::{
    Mutex,
    mpsc::{Receiver, Sender, channel},
};
use uuid::Uuid;

pub type GlobalBroker = Arc<Mutex<Broker>>;
pub static GLOBAL_BROKER: LazyLock<GlobalBroker> = LazyLock::new(|| Default::default());
pub(crate) const CHANNEL_SIZE: usize = 1000;
const TIMEOUT_SECS: u64 = 600;

#[derive(Debug)]
pub struct BrokerPipe<T> {
    last_message: Instant,
    sender: Sender<T>,
    receiver: Receiver<T>,
}

impl<T> BrokerPipe<T>
where
    T: Sync + Send + 'static,
{
    pub fn new() -> Self {
        let (sender, receiver) = channel(CHANNEL_SIZE);
        Self {
            sender,
            receiver,
            last_message: Instant::now(),
        }
    }

    pub async fn next_message(&mut self) -> Option<T> {
        self.receiver.recv().await.map(|x| {
            self.last_message = Instant::now();
            x
        })
    }

    pub async fn send_message(&mut self, msg: T) -> Result<()> {
        Ok(self.sender.send(msg).await.map(|x| {
            self.last_message = Instant::now();
            x
        })?)
    }

    pub fn last_message(&self) -> Instant {
        self.last_message.clone()
    }

    pub fn check_timeout(&self) -> bool {
        std::time::Instant::now() - std::time::Duration::from_secs(TIMEOUT_SECS) > self.last_message
    }
}

// NOTE: this is probably not a long-term solution, but it should route requests between the API
// service and the various AI services / MCPs involved in the process. It will likely use a lot of
// memory and will likely need to be replaced with a dedicated queue before using in production.
#[derive(Debug, Clone, Default)]
pub struct Broker {
    mcp: HashMap<uuid::Uuid, Arc<Mutex<BrokerPipe<McpRequest>>>>,
    prompt: HashMap<uuid::Uuid, Arc<Mutex<BrokerPipe<String>>>>,
}

pub(crate) type PromptPipe = Arc<Mutex<BrokerPipe<String>>>;
pub(crate) type McpPipe = Arc<Mutex<BrokerPipe<McpRequest>>>;

impl Broker {
    // FIXME: replace anyhow with thiserror here
    pub fn create(&mut self) -> Result<uuid::Uuid> {
        let uuid = Uuid::new_v4();
        let prompt_proxy = Arc::new(Mutex::new(BrokerPipe::new()));
        let mcp_proxy = Arc::new(Mutex::new(BrokerPipe::new()));
        self.prompt.insert(uuid, prompt_proxy);
        self.mcp.insert(uuid, mcp_proxy);

        Ok(uuid)
    }

    pub fn get_mcp(&self, id: uuid::Uuid) -> Option<McpPipe> {
        self.mcp.get(&id).cloned()
    }

    pub fn get_prompt(&self, id: uuid::Uuid) -> Option<PromptPipe> {
        self.prompt.get(&id).cloned()
    }

    pub fn expire(&mut self, id: uuid::Uuid) {
        self.prompt.remove(&id);
        self.mcp.remove(&id);
    }
}

#[cfg(test)]
mod tests {
    use crate::api::server::broker::{Broker, CHANNEL_SIZE};
    use anyhow::anyhow;
    use tokio::sync::mpsc::channel;

    #[tokio::test]
    async fn test_broker_modify_last_message_on_send() {
        let mut broker = Broker::default();

        let id = broker.create().unwrap();
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

    #[tokio::test]
    async fn test_broker_modify_last_message_on_recv() {
        let mut broker = Broker::default();

        let id = broker.create().unwrap();
        let proxy = broker.get_prompt(id).unwrap();
        let lock = proxy.lock().await;
        let start = lock.last_message.clone();
        drop(lock);

        let (s, mut r) = channel(2);

        // cheat a little for the sender so we know it doesn't modify last message
        let b = broker.clone();
        let s2 = s.clone();
        tokio::spawn(async move {
            let proxy = b.get_prompt(id).unwrap();
            let lock = proxy.lock().await;
            for _ in 0..CHANNEL_SIZE {
                match lock.sender.send("hello, world!".into()).await {
                    Err(e) => s2.send(Err(anyhow!(e))).await.unwrap(),
                    _ => {}
                }
            }

            s2.send(Ok(())).await.unwrap();
        });

        let b = broker.clone();
        tokio::spawn(async move {
            let proxy = b.get_prompt(id).unwrap();
            let mut lock = proxy.lock().await;
            for _ in 0..CHANNEL_SIZE {
                match lock.next_message().await {
                    Some(x) => {
                        if x != "hello, world!" {
                            s.send(Err(anyhow!("input and output didn't match")))
                                .await
                                .unwrap();
                            return;
                        }
                    }
                    None => {
                        s.send(Err(anyhow!("message was not returned")))
                            .await
                            .unwrap();
                        return;
                    }
                }
            }

            s.send(Ok(())).await.unwrap();
        });

        // two futures, two potential errors
        for _ in 0..2 {
            if let Some(Err(e)) = r.recv().await {
                assert!(false, "{}", e);
            }
        }

        let proxy = broker.get_prompt(id).unwrap();
        let lock = proxy.lock().await;
        assert_ne!(lock.last_message, start);
    }
}
