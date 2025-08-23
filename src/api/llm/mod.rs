#![allow(dead_code)]

use anyhow::Result;

// NOTE: the underlying LLM client's abstraction is not much different than this one. I chose to
// NIH this so I'd have control of the inner workings. Don't get mad, modifying it to support new
// APIs should not be very complicated if the llm crate supports it already.

use std::sync::Arc;

use llm::builder::LLMBuilder;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub enum LLMClientType {
    // NOTE: please provide diverse clients for different models, so they can be pre-programmed
    // with appropriate parameters independently without forcing this to be a part of the client
    // user's implementation.
    Ollama,
}

impl LLMClientType {
    pub fn to_model(&self) -> String {
        match self {
            // NOTE: each enum corresponds to both a PLATFORM and MODEL. See `build_client` in
            // LLMClient below.
            LLMClientType::Ollama => "qwen3:235b".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LLMClientParams {
    pub base_url: String,
    pub api_key: Option<String>,
    pub max_tokens: Option<u32>,
    pub timeout: Option<std::time::Duration>,
    // FIXME: json schema response support
}

#[derive(Clone)]
pub struct LLMClient {
    params: LLMClientParams,
    client: Arc<Mutex<Box<dyn llm::LLMProvider>>>,
}

impl std::fmt::Debug for LLMClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?}", self.params))
    }
}

impl LLMClient {
    pub fn new(client_type: LLMClientType, params: LLMClientParams) -> Result<Self> {
        Ok(Self {
            params: params.clone(),
            client: Arc::new(Mutex::new(Self::build_client(client_type, params)?)),
        })
    }

    fn build_client(
        client_type: LLMClientType,
        params: LLMClientParams,
    ) -> Result<Box<dyn llm::LLMProvider>> {
        let mut builder = LLMBuilder::new();

        builder = match client_type {
            LLMClientType::Ollama => builder.backend(llm::builder::LLMBackend::Ollama),
        };

        builder = builder
            .model(client_type.to_model())
            .base_url(params.base_url);

        if let Some(key) = params.api_key {
            builder = builder.api_key(key);
        }

        if let Some(max_tokens) = params.max_tokens {
            builder = builder.max_tokens(max_tokens);
        }

        if let Some(timeout) = params.timeout {
            builder = builder.timeout_seconds(timeout.as_secs());
        }

        Ok(builder.build()?)
    }
}
