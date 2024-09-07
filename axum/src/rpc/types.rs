use rst_common::with_errors::thiserror::{self, Error};

#[derive(Debug, Error)]
pub enum RpcError {
    #[error("net error: {0}")]
    NetError(String),

    #[error("axum error: {0}")]
    AxumError(String)
} 