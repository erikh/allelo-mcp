use allelo_mcp::mcp::service;
use anyhow::Result;
use rmcp::{transport::sse_server::SseServer, ServiceExt};

#[tokio::main]
async fn main() -> Result<()> {
    let args = std::env::args().skip(1).next();

    match args {
        Some(x) => match x.as_str() {
            "stdio" => {
                tracing::info!("Using stdio transport");
                let service = service::Service::default()
                    .serve((tokio::io::stdin(), tokio::io::stdout()))
                    .await
                    .inspect_err(|e| {
                        tracing::error!("serving error: {:?}", e);
                    })?;
                service.waiting().await?;
            }
            "sse" => {
                tracing::info!("Using SSE transport");
                let ct = SseServer::serve("0.0.0.0:3000".parse()?)
                    .await?
                    .with_service(move || service::Service::default());
                tokio::signal::ctrl_c().await?;
                ct.cancel();
            }
            _ => {
                tracing::error!("Invalid transport type: {}", x);
                return Err(anyhow::anyhow!("Invalid transport type: {}", x));
            }
        },
        None => {
            tracing::error!("unspecified transport");
            return Err(anyhow::anyhow!(
                r#"unspecified transport, please provide "stdio" or "sse""#
            ));
        }
    }

    Ok(())
}
