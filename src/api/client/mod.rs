use super::server::Prompt;
use anyhow::Result;
use futures_util::StreamExt;
use reqwest_eventsource::Event;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

#[derive(Debug, Clone)]
pub struct Client {
    base_url: url::Url,
}

pub type SseResult = Result<UnboundedReceiver<Result<Event>>>;

impl Client {
    pub fn new(base_url: url::Url) -> Self {
        Self { base_url }
    }

    pub async fn prompt(&self, input: Prompt) -> SseResult {
        let url = self.base_url.join("/prompt")?;
        let mut es = reqwest_eventsource::EventSource::post(
            url.to_string(),
            "application/json",
            reqwest::Body::from(serde_json::to_vec(&input)?),
        );

        let (s, r) = unbounded_channel();

        tokio::spawn(async move {
            while let Some(event) = es.next().await {
                match event {
                    Ok(m) => match s.send(Ok(m)) {
                        // try to send an error if we get one trying to send. We probably won't
                        // succeed, so don't try to handle further errors.
                        Err(e) => {
                            let _ = s.send(Err(e.into()));
                            return;
                        }
                        _ => {}
                    },
                    Err(e) => {
                        // if we already have an error, just try to fire off the event before
                        // dying, don't try to process any trouble with firing it to avoid
                        // complexity.
                        let _ = s.send(Err(e.into()));
                        return;
                    }
                }
            }
        });

        Ok(r)
    }
}
