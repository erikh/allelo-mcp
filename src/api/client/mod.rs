use super::server::Prompt;
use anyhow::Result;
use futures_util::StreamExt;
use reqwest_eventsource::Event;

#[derive(Debug, Clone)]
pub struct Client {
    base_url: url::Url,
}

impl Client {
    pub fn new(base_url: url::Url) -> Self {
        Self { base_url }
    }

    pub async fn prompt(&self, input: Prompt) -> Result<()> {
        let url = self.base_url.join("/prompt")?;
        let mut es = reqwest_eventsource::EventSource::post(
            url.to_string(),
            "application/json",
            reqwest::Body::from(serde_json::to_vec(&input)?),
        );

        while let Some(event) = es.next().await {
            match event {
                Ok(Event::Open) => {}
                Ok(Event::Message(_m)) => {}
                Err(e) => return Err(e.into()),
            }
        }

        Ok(())
    }
}
