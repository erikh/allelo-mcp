#![allow(dead_code)]

// NOTE: the underlying LLM client's abstraction is not much different than this one. I chose to
// NIH this so I'd have control of the inner workings. Don't get mad, modifying it to support new
// APIs should not be very complicated if the llm crate supports it already.

pub enum LLMClientType {
    // NOTE: please provide diverse clients for different models, so they can be pre-programmed
    // with appropriate parameters independently without forcing this to be a part of the client
    // user's implementation.
    Ollama,
    Claude,
}

#[derive(Debug, Clone)]
pub struct LLMClient {
    params: LLMClientParams,
}

#[derive(Debug, Clone)]
pub struct LLMClientParams {
    base_url: String,
    api_key: Option<String>,
    max_tokens: Option<u32>,
    timeout: Option<std::time::Duration>,
    // FIXME: json schema response support
}

impl LLMClient {
    pub fn new(params: LLMClientParams) -> Self {
        Self { params }
    }
}
