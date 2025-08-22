use crate::api::server::broker::{McpPipe, PromptPipe};
use crate::api::server::{CloneableBrokerPipe, PromptClient, PromptRepeaterClient};

use super::broker::{CHANNEL_SIZE, GLOBAL_BROKER};
use super::{AppError, Auth, ServerState, ServiceAuth};
use anyhow::anyhow;
use axum::extract::Query;
use axum::{
    extract::{Json, State},
    response::sse::{Event, KeepAlive, Sse},
};
use futures_util::stream::Stream;
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};
use tokio::sync::mpsc::{channel, Receiver};
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

#[derive(Debug, Clone)]
struct PromptControl {
    id: uuid::Uuid,
    prompt: PromptPipe,
    mcp: McpPipe,
}

async fn get_prompt(id: Option<uuid::Uuid>) -> Result<PromptControl> {
    let mut lock = GLOBAL_BROKER.lock().into_future().await;
    let id = if let Some(id) = id {
        tracing::info!("resuming prompt: {}", id);
        id
    } else {
        let id = lock.create()?;
        tracing::info!("created new prompt: {}", id);
        id
    };

    if let Some(prompt) = lock.get_prompt(id) {
        if let Some(mcp) = lock.get_mcp(id) {
            Ok(PromptControl { id, prompt, mcp })
        } else {
            Err(anyhow!("stream closed").into())
        }
    } else {
        lock.expire(id);
        Err(anyhow!("stream closed").into())
    }
}

async fn prompt_client(
    #[allow(unused)] query_type: QueryType,
    id: uuid::Uuid,
    send: CloneableBrokerPipe,
    msg: String,
) {
    #[cfg(test)]
    {
        // FIXME: this shouldn't fall through
        if matches!(query_type, QueryType::RepeatPrompt) {
            let prc = &PromptRepeaterClient;
            tokio::spawn(prc.prompt(id, send, msg));
        }
    }

    #[cfg(not(test))]
    {
        let prc = &PromptRepeaterClient;
        tokio::spawn(prc.prompt(id, send, msg));
    }
}

async fn prompt_multiplex(control: PromptControl) -> Receiver<PromptResponse> {
    let (s, r) = channel(CHANNEL_SIZE);

    tokio::spawn(async move {
        s.send(PromptResponse::Connection(control.id))
            .await
            .unwrap();

        loop {
            if s.is_closed() {
                return;
            }

            let mut prompt_lock = control.prompt.lock().await;
            if prompt_lock.check_timeout() {
                let mut global = GLOBAL_BROKER.lock().into_future().await;
                global.expire(control.id);
                drop(global);
                tracing::debug!("prompt proxy expired: {}", control.id);
                return;
            }

            let mut mcp_lock = control.mcp.lock().await;
            if mcp_lock.check_timeout() {
                let mut global = GLOBAL_BROKER.lock().into_future().await;
                global.expire(control.id);
                drop(global);
                tracing::debug!("mcp proxy expired: {}", control.id);
                return;
            }

            tracing::debug!("recv lock acquired for: {}", control.id);

            tokio::select! {
                Some(output) = mcp_lock.next_message() => {
                    let _ = s.send(PromptResponse::McpRequest(output)).await;
                },
                Some(output) = prompt_lock.next_message() => {
                    let _ = s.send(PromptResponse::PromptResponse(output)).await;
                }
                _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => {}
            }

            tracing::debug!("freeing prompt recv lock for: {}", control.id);
        }
    });

    r
}

pub(crate) async fn prompt(
    Auth(authed): Auth,
    State(_state): State<Arc<ServerState>>,
    Query(params): Query<PromptType>,
    Json(prompt): Json<Prompt>,
) -> Result<Sse<impl Stream<Item = std::result::Result<Event, std::convert::Infallible>>>> {
    if !authed {
        return Err(anyhow!("unauthenticated").into());
    }

    let control = get_prompt(prompt.connection_id).await?;
    tracing::debug!("retreived prompt: {}", control.id);

    let send = control.prompt.clone();
    if let Some(msg) = prompt.prompt {
        prompt_client(params.query_type, control.id, send, msg).await;
    }

    let r = prompt_multiplex(control).await;
    let stream = ReceiverStream::new(r)
        .map(|x| Event::default().data(&serde_json::to_string(&x).unwrap()))
        .map(Ok)
        .throttle(Duration::from_millis(10));
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
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
