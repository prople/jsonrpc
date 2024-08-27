use rst_common::standard::async_trait::async_trait;
use rst_common::standard::dyn_clone::{self, DynClone};
use rst_common::standard::erased_serde::Serialize as ErasedSerialized;
use rst_common::standard::serde_json::Value;

use crate::types::RpcError;

/// `HandlerBoxed` is an alias type used as shortcut to the boxed handler type
pub type HandlerBoxed = Box<dyn Handler + Send + Sync>;

impl Clone for HandlerBoxed {
    fn clone(&self) -> Self {
        dyn_clone::clone_box(&**self)
    }
}

// `ResponseSerialized` is an alias type used as shortcut to the serialized response
pub type ResponseSerialized = Box<dyn ErasedSerialized>;

/// `Handler` is the only main trait that designed to be implemented by any
/// request handlers
///
/// This trait must implement the [`DynClone`] which is a custom object to make it possible
/// for any implementers to implement clone behaviors
#[async_trait]
pub trait Handler: DynClone {
    /// `call` means we will start *call* the handler to execute it's logic
    ///
    /// > **WARNING**
    /// >
    /// > The given resut MUST BE any data types that already implement `serde::Serialize`.
    /// > The problem is, `serde` doesn't provide (or even already remove the feature) to this
    /// > kind of traits, so that's the reason why we're using `erased_serde::Serialize`
    async fn call(&self, params: Value) -> Result<Option<ResponseSerialized>, RpcError>;
}

#[derive(Clone, PartialEq, Hash, Eq)]
pub struct Method(String);

impl From<&str> for Method {
    fn from(value: &str) -> Self {
        Method(String::from(value))
    }
}

impl From<String> for Method {
    fn from(value: String) -> Self {
        Method(value)
    }
}

impl ToString for Method {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

/// `Controller` is an object used to wrap the [`HandlerBoxed`]
///
/// The wrapped value actually is just a boxed version of [`Handler`]
#[derive(Clone)]
pub struct Controller(HandlerBoxed);

impl Controller {
    pub fn new(handler: HandlerBoxed) -> Self {
        Self(handler)
    }

    pub fn handler(self) -> HandlerBoxed {
        self.0
    }
}

/// `Route` used to register an [`Method`] with its [`Controller`]
pub struct Route {
    method: Method,
    controller: Controller,
}

impl Route {
    pub fn new(method: Method, handler: HandlerBoxed) -> Self {
        Self {
            method,
            controller: Controller(handler),
        }
    }

    pub fn method(&self) -> Method {
        self.method.clone()
    }

    pub fn controller(&self) -> HandlerBoxed {
        self.controller.clone().handler()
    }
}
