#![doc = include_str!("../README.md")]

use std::sync::Arc;
use std::time::Duration;

use rst_common::standard::erased_serde::Serialize as ErasedSerialized;
use rst_common::with_tracing::tracing::{self, info_span, Level};

use rst_common::with_tokio::tokio::sync::watch::{self, Sender, Receiver};
use rst_common::with_tokio::tokio::task;
use rst_common::with_tokio::tokio::{self, select};

use rst_common::with_http_tokio::axum::extract::Request;
use rst_common::with_http_tokio::axum::{
    extract::MatchedPath, extract::State, http::StatusCode, routing::post, Json, Router,
};
use rst_common::with_http_tokio::hyper;
use rst_common::with_http_tokio::hyper::body::Incoming;
use rst_common::with_http_tokio::hyper_util::rt::TokioIo;
use rst_common::with_http_tokio::tower::Service;
use rst_common::with_http_tokio::tower_http::timeout::TimeoutLayer;
use rst_common::with_http_tokio::tower_http::trace::{self, TraceLayer};

use prople_jsonrpc_core::objects::{RpcProcessor, RpcRequest, RpcResponse};
use prople_jsonrpc_core::types::*;

/// `RpcState` used to setup our internal state object that will be used for the 
/// request logic
#[derive(Clone)]
pub struct RpcState {
    pub processor: Arc<RpcProcessor>,
}

/// `Config` used to modify our RPC HTTP engine behaviors
pub struct Config {
    pub rpc_path: String,
    pub rpc_port: String,
    pub timeout: u64,
}

async fn rpc_handler(
    State(state): State<Arc<RpcState>>,
    Json(payload): Json<RpcRequest>,
) -> (
    StatusCode,
    Json<RpcResponse<Box<dyn ErasedSerialized>, ()>>,
) {
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

/// `build_canceller` used to build watch canceller. The `canceller` used to
/// cancel all processing context based on some signal
///
/// This method actually is a wrapper of `watch::channel`
pub fn build_canceller(init: i32) -> (Sender<i32>, Receiver<i32>) {
    watch::channel(init)
}

/// `run_rpc` used to starting `Axum` HTTP framework
///
/// Because we are building the `JSON-RPC`, the request handler will automatically assigned
/// to `rpc_handler`. This helper designed as private function because there is no need to modify
/// it's behaviors 
pub async fn run_rpc(cfg: Config, rpc_state: RpcState, canceller: Receiver<i32>) {
    let app = Router::new()
        .route(&cfg.rpc_path, post(rpc_handler))
        .layer((
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request| {
                    let matched_path = request
                        .extensions()
                        .get::<MatchedPath>()
                        .map(MatchedPath::as_str);

                    info_span!(
                        "http_request",
                        method = ?request.method(),
                        matched_path,
                        some_other_field = tracing::field::Empty,
                    )
                })
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
            TimeoutLayer::new(Duration::from_secs(cfg.timeout)),
        ))
        .with_state(Arc::new(rpc_state));

    let rpc_addr = format!("0.0.0.0:{}", cfg.rpc_port);
    tracing::info!("listening on: {}", rpc_addr);

    let listener = tokio::net::TcpListener::bind(rpc_addr).await.unwrap();
    let (close_tx, close_rx) = watch::channel(());

    let mut canceller = canceller.clone();
    loop {
        let (socket, remote_addr) = select! {
            result = listener.accept() => {
                result.unwrap()
            }

            _ = canceller.changed() => {
                tracing::warn!("canceller catched! stopping tcp listener to receive request...");
                break;
            }
        };

        let tower_svc = app.clone();
        let close_rx = close_rx.clone();

        let mut canceller = canceller.clone();
        task::spawn(async move {
            let socket = TokioIo::new(socket);
            let hyper_svc = hyper::service::service_fn(move |request: Request<Incoming>| {
                tower_svc.clone().call(request)
            });

            let conn = hyper::server::conn::http1::Builder::new()
                .serve_connection(socket, hyper_svc)
                .with_upgrades();

            let mut conn = std::pin::pin!(conn);
            loop {
                select! {
                    result = conn.as_mut() => {
                        if let Err(err) = result {
                            tracing::debug!("failed to serve connection: {}", err)
                        }

                        break;
                    }

                    _ = canceller.changed() => {
                        tracing::warn!("canceller catched! starting hyper connection to gracefully shutdown");
                        conn.as_mut().graceful_shutdown();
                    }
                }
            }

            tracing::debug!("connection {remote_addr} closed");
            drop(close_rx);
        });
    }

    drop(close_rx);
    drop(listener);
    tracing::debug!("waiting for {} tasks to finish", close_tx.receiver_count());
    close_tx.closed().await;
}
