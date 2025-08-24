use anyhow::Result;
use futures_util::StreamExt;
use llm::{builder::LLMBuilder, chat::ChatMessageBuilder};
use std::sync::Arc;
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedReceiver},
    Mutex,
};

// NOTE: the underlying LLM client's abstraction is not much different than this one. I chose to
// NIH this so I'd have control of the inner workings. Don't get mad, modifying it to support new
// APIs should not be very complicated if the llm crate supports it already.

// NOTE: copy of llm::chat::ReasoningEffort type; it's not clone or debug and I want that.
#[derive(Debug, Clone, PartialEq)]
// FIXME: we don't use reasoning for anything yet
#[allow(dead_code)]
enum ReasoningEffort {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, PartialEq)]
struct ReasoningOptions {
    effort: ReasoningEffort,
    token_budget: Option<u32>,
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
    reasoning: Option<ReasoningOptions>,
    // NOTE: if this is set to None,
    top_p: f32,
    top_k: u32,
    max_tokens: u32,
}

#[derive(Debug, Clone)]
pub enum LLMClientType {
    // NOTE: please provide diverse clients for different models, so they can be pre-programmed
    // with appropriate parameters independently without forcing this to be a part of the client
    // user's implementation.

    // Qwen production model
    OllamaQwen,
    // Vicuna low-memory model for integration testing
    OllamaVicuna,
}

impl LLMClientType {
    pub(crate) fn to_model(&self) -> String {
        match self {
            // NOTE: each enum corresponds to both a PLATFORM and MODEL. See `build_client` in
            // LLMClient below.
            LLMClientType::OllamaQwen => "qwen3:30b".into(),
            LLMClientType::OllamaVicuna => "vicuna:7b".into(),
        }
    }

    pub(crate) fn to_options(&self) -> LLMClientOptions {
        match self {
            // NOTE: each enum corresponds to both a PLATFORM and MODEL. See `build_client` in
            // LLMClient below.
            LLMClientType::OllamaQwen => LLMClientOptions {
                max_tokens: 65536,
                temperature: 0.7,
                top_p: 0.8,
                top_k: 20,
                system_prompt: None,
                reasoning: None,
            },
            LLMClientType::OllamaVicuna => LLMClientOptions {
                max_tokens: 512,
                temperature: 0.7,
                top_p: 0.95,
                top_k: 40,
                system_prompt: None,
                reasoning: None,
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

    pub async fn prompt(&self, prompt: String) -> Result<UnboundedReceiver<String>> {
        // FIXME: I can't seem to get this library to stream with ollama
        let mut stream = self
            .client
            .lock()
            .await
            .chat_stream(&[ChatMessageBuilder::new(llm::chat::ChatRole::User)
                .content(prompt)
                .build()])
            .await?;

        let (s, r) = unbounded_channel();

        tokio::spawn(async move {
            while let Some(Ok(item)) = stream.next().await {
                s.send(item).unwrap()
            }
        });

        Ok(r)
    }

    fn build_client(
        client_type: LLMClientType,
        params: LLMClientParams,
    ) -> Result<Box<dyn llm::LLMProvider>> {
        // FIXME: this should probably be simplified. maybe the client type can configure the
        // builder directly.
        let mut builder = LLMBuilder::new();

        builder = match client_type {
            LLMClientType::OllamaQwen | LLMClientType::OllamaVicuna => {
                builder.backend(llm::builder::LLMBackend::Ollama)
            }
        };

        builder = builder
            .stream(false) // FIXME: I can't seem to get this library to stream with ollama
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

        builder = if let Some(reasoning) = options.reasoning {
            builder = builder
                .reasoning(true)
                .reasoning_effort(reasoning.effort.into());

            if let Some(token_budget) = reasoning.token_budget {
                builder = builder.reasoning_budget_tokens(token_budget);
            }

            builder
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
        assert_eq!(LLMClientType::OllamaQwen.to_model(), "qwen3:30b");
        assert_eq!(
            LLMClientType::OllamaQwen.to_options(),
            LLMClientOptions {
                max_tokens: 65536,
                reasoning: None,
                system_prompt: None,
                top_p: 0.8,
                top_k: 20,
                temperature: 0.7
            }
        );
    }
}
