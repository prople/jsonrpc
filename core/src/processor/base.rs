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

#[derive(Clone, PartialEq, Hash, Eq)]
pub struct Method(pub String);

/// `Controller` is an object used to wrap between the RPC method and its controller
/// 
/// By using this object, we can create a standalone object which then register it to the
/// processor
pub struct Controller<T>
where
    T: Handler + Clone + Send + Sync,
{
    method: Method,
    handler: T,
}

impl<T> Controller<T>
where
    T: Handler + Clone + Send + Sync + 'static,
{
    pub fn new(method: String, handler: T) -> Self {
        Self {
            method: Method(method),
            handler,
        }
    }

    pub fn method(&self) -> Method {
        self.method.clone()
    }

    pub fn handler_boxed(&self) -> Box<dyn Handler> {
        let handler = self.handler.clone();
        Box::new(handler)
    }
}
