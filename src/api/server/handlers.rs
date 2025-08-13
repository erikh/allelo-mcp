use crate::api::server::{PromptClient, PromptRepeaterClient};

use super::broker::{CHANNEL_SIZE, GLOBAL_BROKER};
use super::{AppError, Auth, ServerState, ServiceAuth};
use anyhow::anyhow;
#[cfg(test)]
use axum::extract::Query;
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpResponse {
    pub connection_id: String,
    pub response: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptType {
    query_type: QueryType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryType {
    #[serde(rename = "repeat_prompt")]
    RepeatPrompt,
}

pub(crate) async fn prompt(
    Auth(authed): Auth,
    State(_state): State<Arc<ServerState>>,
    #[cfg(test)]
    #[allow(unused)]
    Query(params): Query<PromptType>,
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
            #[cfg(test)]
            {
                match params.query_type {
                    QueryType::RepeatPrompt => {
                        let prc = &PromptRepeaterClient;
                        tokio::spawn(prc.prompt(id, send, msg));
                    }
                }
            }

            #[cfg(not(test))]
            {
                let prc = &PromptRepeaterClient;
                tokio::spawn(prc.prompt(id, send, msg));
            }
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
                                let timeout = lock.check_timeout();
                                if s.is_closed() || timeout {
                                    tracing::debug!("freeing recv lock for: {}", id);
                                    if timeout {
                                        let mut global = GLOBAL_BROKER.lock().into_future().await;
                                        global.expire_prompt(id);
                                        drop(global);
                                    }
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

pub(crate) async fn mcp_response(
    Auth(authed): Auth,
    State(_state): State<Arc<ServerState>>,
    Json(_response): Json<McpResponse>,
) -> Result<()> {
    if !authed {
        return Err(anyhow!("unauthenticated").into());
    }

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Search {
    input: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchResults {
    results: Vec<String>,
}

pub(crate) async fn search(
    Auth(authed): Auth,
    State(_state): State<Arc<ServerState>>,
    Json(_search): Json<Search>,
) -> Result<Json<SearchResults>> {
    if !authed {
        return Err(anyhow!("unauthenticated").into());
    }

    return Ok(Default::default());
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Input {
    input: String,
}

pub(crate) async fn input(
    Auth(authed): Auth,
    State(_state): State<Arc<ServerState>>,
    Json(_input): Json<Input>,
) -> Result<Json<bool>> {
    if !authed {
        return Err(anyhow!("unauthenticated").into());
    }

    return Ok(axum::Json(true));
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {}

pub(crate) async fn metrics(
    ServiceAuth(authed): ServiceAuth,
    State(_state): State<Arc<ServerState>>,
) -> Result<Json<Metrics>> {
    if !authed {
        return Err(anyhow!("unauthenticated").into());
    }
    Ok(Json::from(Metrics {}))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Status {}

pub(crate) async fn status(
    ServiceAuth(authed): ServiceAuth,
    State(_state): State<Arc<ServerState>>,
) -> Result<Json<Status>> {
    if !authed {
        return Err(anyhow!("unauthenticated").into());
    }

    Ok(Json::from(Status {}))
}
