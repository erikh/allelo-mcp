use axum::{
    extract::FromRequestParts,
    http::request::Parts,
    response::{IntoResponse, Response},
};
use problem_details::ProblemDetails;
use std::{
    any::{Any, TypeId},
    sync::Arc,
};
use tokio::sync::Mutex;

use super::broker::BrokerPipe;

#[async_trait::async_trait]
pub trait PromptClient {
    async fn prompt(&self, id: uuid::Uuid, proxy: Arc<Mutex<BrokerPipe<String>>>, prompt: String);
}

pub struct PromptRepeaterClient;

#[async_trait::async_trait]
impl PromptClient for PromptRepeaterClient {
    async fn prompt(&self, id: uuid::Uuid, send: Arc<Mutex<BrokerPipe<String>>>, msg: String) {
        loop {
            tokio::select! {
                mut lock = send.lock() => {
                    tracing::debug!("send lock acquired for: {}", id);
                    tokio::select! {
                        _ = lock.send_message(msg.clone()) => {}
                    }
                }
                _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => { },
            }
            tracing::debug!("freeing send lock for: {}", id);
        }
    }
}

#[derive(Debug, Clone)]
pub struct ServerState {}

#[derive(Debug, Clone, Default)]
pub struct AppError(pub ProblemDetails);

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error> + Any,
{
    fn from(value: E) -> Self {
        // hack around type specialization
        if TypeId::of::<E>() == TypeId::of::<ProblemDetails>() {
            Self(
                <(dyn Any + 'static)>::downcast_ref::<ProblemDetails>(&value)
                    .unwrap()
                    .clone(),
            )
        } else {
            Self(
                ProblemDetails::new()
                    .with_detail(value.into().to_string())
                    .with_title("Uncategorized Error"),
            )
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        self.0.into_response()
    }
}

#[derive(Debug, Clone, Default)]
pub struct Auth(pub bool);

impl FromRequestParts<Arc<ServerState>> for Auth {
    type Rejection = AppError;

    async fn from_request_parts(
        _parts: &mut Parts,
        _state: &Arc<ServerState>,
    ) -> core::result::Result<Self, Self::Rejection> {
        Ok(Self(true))
    }
}

#[derive(Debug, Clone, Default)]
pub struct ServiceAuth(pub bool);

impl FromRequestParts<Arc<ServerState>> for ServiceAuth {
    type Rejection = AppError;

    async fn from_request_parts(
        _parts: &mut Parts,
        _state: &Arc<ServerState>,
    ) -> core::result::Result<Self, Self::Rejection> {
        Ok(Self(true))
    }
}
