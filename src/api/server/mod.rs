mod axum_support;
pub(crate) mod broker;
mod config;
mod handlers;
#[cfg(test)]
mod tests;
pub use self::config::*;
pub use axum_support::*;
pub use handlers::*;

use axum::{
    Router,
    routing::{get, post, put},
};
use http::{Method, header::*};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::cors::{Any as CorsAny, CorsLayer};
use tower_http::trace::{DefaultMakeSpan, DefaultOnFailure, DefaultOnRequest};
use tracing::Level;

#[derive(Debug, Clone)]
pub struct Server {
    router: Router,
    config: Config,
}

impl Server {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        Ok(Self {
            router: Router::new()
                .route("/prompt", post(prompt))
                .route("/mcp_response", post(mcp_response))
                .route("/search", post(search))
                .route("/input", put(input))
                .route("/status", get(status))
                .route("/metrics", get(metrics))
                .with_state(Arc::new(ServerState {
                    config: config.clone(),
                }))
                .layer(
                    ServiceBuilder::new()
                        .layer(
                            tower_http::trace::TraceLayer::new_for_http()
                                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                                .on_request(DefaultOnRequest::new().level(Level::INFO))
                                .on_failure(DefaultOnFailure::new().level(Level::ERROR)),
                        )
                        .layer(
                            CorsLayer::new()
                                .allow_methods([
                                    Method::GET,
                                    Method::POST,
                                    Method::DELETE,
                                    Method::PUT,
                                    Method::PATCH,
                                    Method::HEAD,
                                    Method::TRACE,
                                    Method::OPTIONS,
                                ])
                                .allow_origin(CorsAny)
                                .allow_headers([CONTENT_TYPE, ACCEPT, AUTHORIZATION])
                                .allow_private_network(true),
                        ),
                ),
            config: config.clone(),
        })
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        let handle = axum_server::Handle::new();
        self.start_with_handle(handle).await
    }

    pub async fn start_with_handle(&self, handle: axum_server::Handle) -> anyhow::Result<()> {
        tokio::spawn(shutdown_signal(handle.clone()));
        Ok(axum_server::bind(self.config.listen)
            .handle(handle)
            .serve(self.router.clone().into_make_service())
            .await?)
    }
}

async fn shutdown_signal(handle: axum_server::Handle) {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install CTRL+C signal handler");
    };

    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::warn!("signal received, starting graceful shutdown");
    handle.graceful_shutdown(Some(std::time::Duration::from_secs(10)));
}
