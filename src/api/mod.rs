use axum::{
    extract::{FromRequestParts, State},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Router,
};
use http::{header::*, Method};
use problem_details::ProblemDetails;
use serde::Deserialize;
use std::{
    any::{Any, TypeId},
    net::SocketAddr,
    path::PathBuf,
    sync::Arc,
};

use tower::ServiceBuilder;
use tower_http::cors::{Any as CorsAny, CorsLayer};
use tower_http::trace::{DefaultMakeSpan, DefaultOnFailure, DefaultOnRequest};
use tracing::Level;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    listen: SocketAddr,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            listen: "127.0.0.1:8999".parse().unwrap(),
        }
    }
}

impl Config {
    pub fn from_file(filename: PathBuf) -> anyhow::Result<Self> {
        let r = std::fs::OpenOptions::new().read(true).open(filename)?;
        Ok(serde_yaml_ng::from_reader(r)?)
    }
}

pub(crate) type Result<T> = core::result::Result<T, AppError>;

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

#[derive(Debug, Clone)]
pub struct ServerState {}

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
                .with_state(Arc::new(ServerState {}))
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

    #[cfg(unix)]
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

pub(crate) async fn prompt(State(state): State<Arc<ServerState>>) -> Result<()> {
    Ok(())
}
