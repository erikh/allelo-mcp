use super::server::Prompt;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Client {
    base_url: url::Url,
}

impl Client {
    pub fn new(base_url: url::Url) -> Self {
        Self { base_url }
    }

    pub fn prompt(&self, _input: Prompt) -> Result<()> {
        let _url = self.base_url.join("/prompt")?;
        Ok(())
    }
}
