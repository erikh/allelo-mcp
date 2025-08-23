#![allow(dead_code)]
use anyhow::Result;
use llm::builder::LLMBuilder;
use std::sync::Arc;
use tokio::sync::Mutex;

// NOTE: the underlying LLM client's abstraction is not much different than this one. I chose to
// NIH this so I'd have control of the inner workings. Don't get mad, modifying it to support new
// APIs should not be very complicated if the llm crate supports it already.

// NOTE: copy of ReasoningEffort type; it's not clone or debug and I want that.
#[derive(Debug, Clone, PartialEq)]
enum ReasoningEffort {
    Low,
    Medium,
    High,
}

impl Into<llm::chat::ReasoningEffort> for ReasoningEffort {
    fn into(self) -> llm::chat::ReasoningEffort {
        match self {
            ReasoningEffort::Low => llm::chat::ReasoningEffort::Low,
            ReasoningEffort::Medium => llm::chat::ReasoningEffort::Medium,
            ReasoningEffort::High => llm::chat::ReasoningEffort::High,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LLMClientOptions {
    temperature: f32,
    system_prompt: Option<String>,
    // NOTE: if this is "some", the reasoning flag is also set; otherwise, it is false.
    // this might need to change in the future.
    reasoning_effort: Option<ReasoningEffort>,
    top_p: f32,
    top_k: u32,
    max_tokens: u32,
}

#[derive(Debug, Clone)]
pub enum LLMClientType {
    // NOTE: please provide diverse clients for different models, so they can be pre-programmed
    // with appropriate parameters independently without forcing this to be a part of the client
    // user's implementation.
    Ollama,
}

impl LLMClientType {
    pub(crate) fn to_model(&self) -> String {
        match self {
            // NOTE: each enum corresponds to both a PLATFORM and MODEL. See `build_client` in
            // LLMClient below.
            LLMClientType::Ollama => "qwen3:30b".into(),
        }
    }

    pub(crate) fn to_options(&self) -> LLMClientOptions {
        match self {
            // NOTE: each enum corresponds to both a PLATFORM and MODEL. See `build_client` in
            // LLMClient below.
            LLMClientType::Ollama => LLMClientOptions {
                max_tokens: 65536,
                temperature: 0.7,
                top_p: 0.8,
                top_k: 20,
                system_prompt: None,
                reasoning_effort: None,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct LLMClientParams {
    pub base_url: String,
    pub api_key: Option<String>,
    pub timeout: Option<std::time::Duration>,
    // FIXME: json schema response support
}

pub type LLMProvider = Arc<Mutex<Box<dyn llm::LLMProvider>>>;

#[derive(Clone)]
pub struct LLMClient {
    params: LLMClientParams,
    client: LLMProvider,
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

    pub fn into_inner(&self) -> LLMProvider {
        self.client.clone()
    }

    fn build_client(
        client_type: LLMClientType,
        params: LLMClientParams,
    ) -> Result<Box<dyn llm::LLMProvider>> {
        // FIXME: this should probably be simplified. maybe the client type can configure the
        // builder directly.
        let mut builder = LLMBuilder::new();

        builder = match client_type {
            LLMClientType::Ollama => builder.backend(llm::builder::LLMBackend::Ollama),
        };

        builder = builder
            .stream(true)
            .model(client_type.to_model())
            .base_url(params.base_url);

        if let Some(key) = params.api_key {
            builder = builder.api_key(key);
        }

        if let Some(timeout) = params.timeout {
            builder = builder.timeout_seconds(timeout.as_secs());
        }

        let options = client_type.to_options();

        builder = builder
            .max_tokens(options.max_tokens)
            .top_p(options.top_p)
            .top_k(options.top_k)
            .temperature(options.temperature);

        if let Some(system_prompt) = options.system_prompt {
            builder = builder.system(system_prompt);
        }

        builder = if let Some(reasoning_effort) = options.reasoning_effort {
            builder
                .reasoning(true)
                .reasoning_effort(reasoning_effort.into())
        } else {
            builder.reasoning(false)
        };

        Ok(builder.build()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_configuration() {
        assert_eq!(LLMClientType::Ollama.to_model(), "qwen3:30b");
        assert_eq!(
            LLMClientType::Ollama.to_options(),
            LLMClientOptions {
                max_tokens: 65536,
                reasoning_effort: None,
                system_prompt: None,
                top_p: 0.8,
                top_k: 20,
                temperature: 0.7
            }
        );
    }
}
