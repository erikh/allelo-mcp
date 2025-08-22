#![allow(dead_code)]

pub enum LLMClientType {
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
    model: String,
    max_tokens: Option<u32>,
    timeout: Option<std::time::Duration>,
    // FIXME: json schema response support
}

impl LLMClient {
    pub fn new(params: LLMClientParams) -> Self {
        Self { params }
    }
}
