#[cfg(test)]
use crate::api::server::QueryType;

use crate::{api::server::Search, mcp::service::Service};
use rmcp::ServiceExt;

use super::server::{Input, McpResponse, Metrics, Prompt, SearchResults, Status};
use anyhow::{anyhow, Result};
use futures_util::StreamExt;
use reqwest_eventsource::Event;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

#[derive(Debug, Clone)]
pub struct Client {
    base_url: url::Url,
    #[cfg(test)]
    #[allow(dead_code)]
    query_type: Option<QueryType>,
}

pub type SseResult = Result<UnboundedReceiver<Result<Event>>>;

impl Client {
    pub fn new(base_url: url::Url) -> Self {
        Self {
            base_url,
            #[cfg(test)]
            query_type: None,
        }
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub fn new_testing(base_url: url::Url, query_type: QueryType) -> Self {
        Self {
            base_url,
            query_type: Some(query_type),
        }
    }

    pub async fn mcp_response(&self, _input: McpResponse) -> Result<()> {
        return Err(anyhow!("unimplemented"));
    }

    pub async fn search(&self, _input: Search) -> Result<SearchResults> {
        return Err(anyhow!("unimplemented"));
    }

    pub async fn input(&self, _input: Input) -> Result<bool> {
        return Err(anyhow!("unimplemented"));
    }

    pub async fn metrics(&self) -> Result<Metrics> {
        return Err(anyhow!("unimplemented"));
    }

    pub async fn status(&self) -> Result<Status> {
        return Err(anyhow!("unimplemented"));
    }

    pub async fn prompt(&self, input: Prompt) -> SseResult {
        #[cfg(test)]
        let mut url = self.base_url.join("/prompt")?;
        #[cfg(not(test))]
        let url = self.base_url.join("/prompt")?;
        #[cfg(test)]
        match self.query_type {
            Some(QueryType::RepeatPrompt) => url.set_query(Some("query_type=repeat_prompt")),
            _ => {}
        }

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

    #[allow(dead_code)]
    async fn init_mcp<T, R>(&self) -> Result<(UnboundedSender<T>, UnboundedReceiver<R>)> {
        let (in_s, _in_r) = unbounded_channel();
        let (_out_s, out_r) = unbounded_channel();

        let (stdin_r, _stdin_w) = tokio::io::simplex(4096);
        let (_stdout_r, stdout_w) = tokio::io::simplex(4096);

        let service = Service::default()
            .serve((stdin_r, stdout_w))
            .await
            .inspect_err(|e| {
                tracing::error!("serving error: {:?}", e);
            })?;

        tokio::spawn(async move { service.waiting().await.unwrap() });

        Ok((in_s, out_r))
    }
}
