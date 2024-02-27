use rst_common::standard::async_trait::async_trait;
use rst_common::standard::erased_serde::Serialize as ErasedSerialized;
use rst_common::standard::serde_json::Value;

use crate::types::RpcError;

/// `Handler` is the only main trait that designed to be implemented by any
/// request handlers
#[async_trait]
pub trait Handler {
    /// `call` means we will start *call* the handler to execute it's logic
    ///
    /// > **WARNING**
    /// >
    /// > The given resut MUST BE any data types that already implement `serde::Serialize`.
    /// > The problem is, `serde` doesn't provide (or even already remove the feature) to this 
    /// > kind of traits, so that's the reason why we're using `erased_serde::Serialize` 
    async fn call(&self, params: Value) -> Result<Option<Box<dyn ErasedSerialized>>, RpcError>;
}

pub type Method = String;
