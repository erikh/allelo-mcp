use super::broker::{CHANNEL_SIZE, GLOBAL_BROKER};
use super::{AppError, Auth, ServerState};
use anyhow::anyhow;
use axum::{
    extract::{Json, State},
    response::sse::{Event, KeepAlive, Sse},
};
use futures_util::stream::Stream;
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};
use tokio::sync::mpsc::channel;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};

type Result<T> = core::result::Result<T, AppError>;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpRequest {
    pub connection_id: String,
    pub command: String,
    pub mcp_route: String,
    pub mcp_route_id: String,
}

// input struct for prompt API
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Prompt {
    pub connection_id: Option<uuid::Uuid>,
    pub prompt: Option<String>,
}

// Response enum for prompt SSE events. Ingested by client which proxies to MCP, or directly to
// the client depending on what response is sent. Server should always send Connection first and
// client should expect that. From there, until the connection is interrupted, all connections are
// assumed to be from the same transaction ID (a UUID).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PromptResponse {
    Connection(uuid::Uuid),
    PromptResponse(String),
    McpRequest(McpRequest),
}

pub(crate) async fn prompt(
    Auth(authed): Auth,
    State(_state): State<Arc<ServerState>>,
    Json(prompt): Json<Prompt>,
) -> Result<Sse<impl Stream<Item = std::result::Result<Event, std::convert::Infallible>>>> {
    if !authed {
        return Err(anyhow!("unauthenticated").into());
    }

    let mut lock = GLOBAL_BROKER.lock().into_future().await;
    let id = if let Some(id) = prompt.connection_id {
        tracing::info!("resuming prompt: {}", id);
        id
    } else {
        let id = lock.create_prompt()?;
        tracing::info!("created new prompt: {}", id);
        id
    };

    if let Some(proxy) = lock.get_prompt(id) {
        tracing::debug!("retreived prompt: {}", id);
        let (s, r) = channel(CHANNEL_SIZE);
        drop(lock);

        let send = proxy.clone();

        if let Some(msg) = prompt.prompt {
            tokio::spawn(async move {
                loop {
                    // FIXME: replace this with actual LLM code
                    tokio::select! {
                        mut lock = send.lock() => {
                            tracing::debug!("send lock acquired for: {}", id);
                            tokio::select! {
                                _ = lock.send_message(msg.clone()) => {}
                        _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => {
                        },
                                else => {}
                            }
                        }
                        else => {}
                    }
                    tracing::debug!("freeing send lock for: {}", id);
                }
            });
        }

        tokio::spawn(async move {
            s.send(PromptResponse::Connection(id)).await.unwrap();

            loop {
                tokio::select! {
                    mut lock = proxy.lock() => {
                        tracing::debug!("recv lock acquired for: {}", id);
                        tokio::select! {
                            Some(output) = lock.next_message() => {
                                s.send(PromptResponse::PromptResponse(output)).await.unwrap();
                            },
                        _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => {
                        },
                            else => {
                                if s.is_closed() || lock.check_timeout() {
                                    tracing::debug!("freeing recv lock for: {}", id);
                                    return;
                                }
                            }
                        }
                    }
                    else => {
                        if s.is_closed() {
                            tracing::debug!("freeing recv lock for: {}", id);
                            return;
                        }
                    }
                }
                tracing::debug!("freeing recv lock for: {}", id);
            }
        });

        let stream = ReceiverStream::new(r)
            .map(|x| Event::default().data(&serde_json::to_string(&x).unwrap()))
            .map(Ok)
            .throttle(Duration::from_millis(10));
        Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
    } else {
        Err(anyhow!("stream closed").into())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {}

pub(crate) async fn metrics(State(_state): State<Arc<ServerState>>) -> Result<Json<Metrics>> {
    Ok(Json::from(Metrics {}))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Status {}

pub(crate) async fn status(State(_state): State<Arc<ServerState>>) -> Result<Json<Status>> {
    Ok(Json::from(Status {}))
}
