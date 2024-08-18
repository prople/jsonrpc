use prople_jsonrpc_core::objects::RpcResponse;
use prople_jsonrpc_core::types::RpcId;

use rst_common::standard::async_trait::async_trait;
use rst_common::standard::serde::de::DeserializeOwned;
use rst_common::standard::serde_json::Value;
use rst_common::with_errors::thiserror::{self, Error};

#[derive(Debug, Clone, Copy, Error)]
pub enum ExecutorError<'life0> {
    #[error("executor error: request error: url: {url} | code: {code}")]
    RequestError { url: &'life0 str, code: u16 },

    #[error("executor error: parse response error: {0}")]
    ParseResponseError(&'life0 str),

    #[error("executor error: build value error: {0}")]
    BuildValueError(&'life0 str),
}

/// `RpcValue` used to convert any value types to the
/// [`Value`]
pub trait RpcValue<'life0>: Send + Sync + Clone {
    fn build_serde_value<'life1>(&self) -> Result<Value, ExecutorError<'life1>>
    where
        'life1: 'life0;
}

/// `Executor` is a main interface that need to implement by
/// all HTTP client executor
#[async_trait]
pub trait Executor<'life0, T>
where
    T: DeserializeOwned + Send + Sync,
{
    type ErrorData: DeserializeOwned + Send + Sync;

    async fn call<'life1>(
        &'life0 self,
        endpoint: &'life0 str,
        params: impl RpcValue<'life1>,
        method: &'life0 str,
        id: Option<RpcId>,
    ) -> Result<RpcResponse<T, Self::ErrorData>, ExecutorError>
    where
        'life1: 'life0;
}
