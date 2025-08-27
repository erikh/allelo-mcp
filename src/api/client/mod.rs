use super::server::{
	Input, McpResponse, Metrics, Prompt, SearchResults, Status,
};
#[cfg(test)]
use crate::api::server::QueryType;
use crate::{api::server::Search, mcp::service::Service};

use anyhow::{Result, anyhow};
use futures_util::StreamExt;
use reqwest_eventsource::Event;
use rmcp::ServiceExt;
use std::{ops::Deref, sync::Arc};
use tokio::{
	io::{AsyncReadExt, AsyncWriteExt},
	sync::mpsc::{
		UnboundedReceiver, UnboundedSender, unbounded_channel,
	},
};

type McpPipe = (UnboundedSender<Vec<u8>>, UnboundedReceiver<Vec<u8>>);

#[derive(Debug, Clone)]
pub struct Client {
	base_url: url::Url,
	#[cfg(test)]
	#[allow(dead_code)]
	query_type: Option<QueryType>,
	mcp: Arc<McpPipe>,
}

pub type SseResult = Result<UnboundedReceiver<Result<Event>>>;

impl Client {
	pub async fn new(base_url: url::Url) -> Result<Self> {
		Ok(Self {
			base_url,
			#[cfg(test)]
			query_type: None,
			mcp: Arc::new(Self::init_mcp().await?),
		})
	}

	#[cfg(test)]
	#[allow(dead_code)]
	pub async fn new_testing(
		base_url: url::Url, query_type: QueryType,
	) -> Result<Self> {
		Ok(Self {
			base_url,
			query_type: Some(query_type),
			mcp: Arc::new(Self::init_mcp().await?),
		})
	}

	pub async fn mcp_response(
		&self, _input: McpResponse,
	) -> Result<()> {
		return Err(anyhow!("unimplemented"));
	}

	pub async fn search(
		&self, _input: Search,
	) -> Result<SearchResults> {
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
			Some(QueryType::RepeatPrompt) => {
				url.set_query(Some("query_type=repeat_prompt"))
			}
			_ => {}
		}

		let mut es = reqwest_eventsource::EventSource::post(
			url.to_string(),
			"application/json",
			reqwest::Body::from(serde_json::to_vec(&input)?),
		);

		let (s, r) = unbounded_channel();
		let (_mcp_in, _mcp_out) = self.mcp.clone().deref();

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

	async fn init_mcp() -> Result<McpPipe> {
		let (in_s, mut in_r) = unbounded_channel::<Vec<u8>>();
		let (out_s, out_r) = unbounded_channel();

		let (stdin_r, mut stdin_w) = tokio::io::simplex(4096);
		let (mut stdout_r, stdout_w) = tokio::io::simplex(4096);

		tokio::spawn(async move {
			if let Ok(service) =
				Service::default().serve((stdin_r, stdout_w)).await
			{
				let _ = service.waiting().await;
			}
		});

		tokio::spawn(async move {
			while let Some(x) = in_r.recv().await {
				if let Err(_) = stdin_w.write_all(x.as_slice()).await {
					return;
				}
			}
		});

		tokio::spawn(async move {
			loop {
				let mut dst = String::new();
				if let Err(_) = stdout_r.read_to_string(&mut dst).await
				{
					return;
				}

				if let Err(_) = out_s.send(dst.as_bytes().to_vec()) {
					return;
				}
			}
		});

		Ok((in_s, out_r))
	}
}
