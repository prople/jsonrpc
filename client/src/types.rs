use prople_jsonrpc_core::types::{RpcErrorBuilder, RpcId};

use rst_common::standard::async_trait::async_trait;
use rst_common::standard::serde::{self, de::DeserializeOwned, Deserialize, Serialize};
use rst_common::standard::serde_json::Value;
use rst_common::with_errors::thiserror::{self, Error};

#[derive(Debug, Clone, Error)]
pub enum ExecutorError {
    #[error("executor error: request error: url: {url} | code: {code}")]
    RequestError { url: String, code: u16 },

    #[error("executor error: parse response error: {0}")]
    ParseResponseError(String),

    #[error("executor error: build value error: {0}")]
    BuildValueError(String),

    #[error("executor error: missing error")]
    MissingError,

    #[error("executor error: missing result")]
    MissingResult,
}

/// `RpcValue` used to convert any value types to the
/// [`Value`]
pub trait RpcValue: Send + Sync + Clone {
    fn build_serde_value(&self) -> Result<Value, ExecutorError>;
}

#[derive(Clone)]
pub struct NullValue;

impl RpcValue for NullValue {
    fn build_serde_value(&self) -> Result<Value, ExecutorError> {
        Ok(Value::Null)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "self::serde")]
pub struct JSONResponse<T>
where
    T: Clone,
{
    pub jsonrpc: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<T>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcErrorBuilder>,

    pub id: Option<RpcId>,
}

/// `Executor` is a main interface that need to implement by
/// all HTTP client executor
#[async_trait]
pub trait Executor<T>
where
    T: DeserializeOwned + Send + Sync + Clone,
{
    async fn call(
        &self,
        endpoint: String,
        params: Option<impl RpcValue>,
        method: String,
        id: Option<RpcId>,
    ) -> Result<JSONResponse<T>, ExecutorError>;
}
