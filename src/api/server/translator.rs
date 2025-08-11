use anyhow::Result;

#[allow(dead_code)]
#[async_trait::async_trait]
pub trait Translator<T, R>
where
    T: Send + Sync + 'static,
    R: Send + Sync + 'static,
{
    async fn send(&self, input: T) -> Result<()>;
    async fn recv(&self) -> Result<R>;
}
