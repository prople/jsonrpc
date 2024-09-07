use std::sync::Arc;

use rst_common::standard::erased_serde::Serialize as ErasedSerialized;
use rst_common::with_http_tokio::axum::extract::{Json, State};
use rst_common::with_http_tokio::axum::http::StatusCode;
use rst_common::with_http_tokio::axum::{self, Router};
use rst_common::with_tokio::tokio::net::TcpListener;
use rst_common::with_tokio::tokio::{self, signal};
use rst_common::with_tracing::tracing;

use prople_jsonrpc_core::objects::{RpcProcessor, RpcRequest, RpcResponse};
use prople_jsonrpc_core::types::*;

use super::RpcError;
use super::RpcConfig;

#[derive(Clone)]
pub struct RpcState {
    processor: Arc<RpcProcessor>,
}

impl RpcState {
    pub fn new(processor: RpcProcessor) -> Self {
        Self {
            processor: Arc::new(processor),
        }
    }
}

pub async fn handler(
    State(state): State<Arc<RpcState>>,
    Json(payload): Json<RpcRequest>,
) -> (StatusCode, Json<RpcResponse<Box<dyn ErasedSerialized>, ()>>) {
    let processor = state.processor.clone();
    let response = processor.execute(payload).await;

    let err = response.error.clone();
    let status_code = err
        .clone()
        .map(|err_obj| err_obj.code)
        .map(|err_code| match err_code {
            INVALID_REQUEST_CODE | INVALID_PARAMS_CODE | PARSE_ERROR_CODE => {
                StatusCode::BAD_REQUEST
            }
            METHOD_NOT_FOUND_CODE => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })
        .unwrap_or_else(|| StatusCode::OK);

    (status_code, Json(response))
}

pub struct Rpc{
    config: RpcConfig,
    state: RpcState,
    svc_app: Router<Arc<RpcState>>,
}

impl Rpc {
    pub fn new(config: RpcConfig, state: RpcState, svc_app: Router<Arc<RpcState>>) -> Self {
        Self {
            config,
            state,
            svc_app,
        }
    }

    pub async fn serve(&self) -> Result<(), RpcError> {
        let (host, port) = self.config.load();
        tracing::info!("listening at: host:{} | port:{}", host, port);

        let listener = TcpListener::bind(format!("{}:{}", host, port))
            .await
            .map_err(|err| RpcError::NetError(err.to_string()))?;

        let app = self
            .svc_app
            .clone()
            .with_state(Arc::new(self.state.clone()));

        let _ = axum::serve(listener, app)
            .with_graceful_shutdown(async {
                let ctrl_c = async { signal::ctrl_c().await.expect("error Ctrl-C handler") };

                #[cfg(unix)]
                let terminate = async {
                    signal::unix::signal(signal::unix::SignalKind::terminate())
                        .expect("failed to install signal handler")
                        .recv()
                        .await;
                };

                #[cfg(not(unix))]
                let terminate = std::future::pending::<()>();

                tokio::select! {
                    _ = ctrl_c => {},
                    _ = terminate => {},
                }
            })
            .await
            .map_err(|err| RpcError::AxumError(err.to_string()))?;

        Ok(())
    }
}