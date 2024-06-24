use rst_common::standard::erased_serde::Serialize as ErasedSerialized;
use rst_common::with_logging::log::error;
use std::collections::HashMap;

use crate::handlers::AgentPingHandler;
use crate::objects::{RpcErrorBuilder, RpcRequest, RpcResponse};
use crate::types::{RpcError, RpcHandler, RpcMethod};

/// `RpcProcessor` is primary object to manage request method handlers including
/// for its handler execution 
pub struct RpcProcessor {
    handlers: HashMap<RpcMethod, Box<dyn RpcHandler + Send + Sync>>,
}

impl RpcProcessor {
    pub fn build() -> Self {
        let mut handlers: HashMap<String, Box<dyn RpcHandler + Send + Sync>> = HashMap::new();
        handlers.insert("prople.agent.ping".to_string(), Box::new(AgentPingHandler));

        RpcProcessor { handlers }
    }

    /// `register_handler` used to register a RPC method's handler
    ///
    /// A `handler` is any object that MUST BE implement the [`RpcHandler`]
    /// Besides of implement the trait, we also need to make sure that the handler itself
    /// implement `Send` and `Sync` implicitly, because the handler will be thrown to some
    /// background process asynchronously
    pub fn register_handler(
        &mut self,
        method: String,
        handler: Box<dyn RpcHandler + Send + Sync>,
    ) -> () {
        self.handlers.insert(method, handler);
    }

    /// `execute` used to process incoming [`RpcRequest]
    ///
    /// The internal flow is, for each time incoming request object
    /// it will fetch the handler based on RPC method. 
    /// If it have a handler, it will *call* the handler.
    /// If not, it will build the [`RpcErrorObject`] and put it into the [`RpcResponse`]
    pub async fn execute(
        &self,
        request: RpcRequest,
    ) -> RpcResponse<Box<dyn ErasedSerialized>, ()> {
        let method = request.method.clone();
        let params = request.params.clone();

        let handler = match self.handlers.get(&method) {
            Some(caller) => caller,
            None => {
                let err_obj: RpcErrorBuilder<()> =
                    RpcErrorBuilder::build(RpcError::MethodNotFound, None);
                let response = RpcResponse::with_error(Some(err_obj), request.id);
                return response;
            }
        };

        match handler.call(params).await {
            Ok(success) => {
                let response = RpcResponse::with_success(success, request.id);
                response
            }
            Err(err) => {
                error!("error from handler: {}", err.to_string());
                let err_obj: RpcErrorBuilder<()> =
                    RpcErrorBuilder::build(err, None);
                let response = RpcResponse::with_error(Some(err_obj), request.id);
                response
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    use mockall::*;
    use crate::types::RpcId;

    use rst_common::standard::async_trait::async_trait;
    use rst_common::standard::serde_json::{self, Value};
    
    use rst_common::with_tokio::tokio;

    mock! {
        Handler {}

        #[async_trait]
        impl RpcHandler for Handler {
            async fn call(&self, params: Value) -> Result<Option<Box<dyn ErasedSerialized>>, RpcError> {
                let output = FakeParam{
                    key: String::from("test-key"),
                    value: String::from("test-value")
                };

                Ok(Some(Box::new(output)))
            }
        }
    }

    #[tokio::test]
    async fn test_processor_execute_success() {
        let processor = RpcProcessor::build();
        let request = RpcRequest {
            id: Some(RpcId::IntegerVal(1)),
            jsonrpc: String::from("2.0"),
            method: String::from("prople.agent.ping"),
            params: Value::Null,
        };

        let response = processor.execute(request).await;

        let jsonstr = serde_json::to_string(&response);
        assert!(!jsonstr.is_err());
        assert_eq!(
            r#"{"jsonrpc":"2.0","result":{"message":"pong!"},"id":1}"#,
            jsonstr.unwrap()
        )
    }

    #[tokio::test]
    async fn test_processor_handler_error() {
        let mut handler = MockHandler::new();

        handler
            .expect_call()
            .with(predicate::eq(Value::Null))
            .times(1)
            .returning(|_| Err(RpcError::InvalidParams));

        let mut processor = RpcProcessor::build();
        processor.register_handler(String::from("test.mock"), Box::new(handler));

        let request = RpcRequest {
            id: Some(RpcId::IntegerVal(1)),
            jsonrpc: String::from("2.0"),
            method: String::from("test.mock"),
            params: Value::Null,
        };

        let response = processor.execute(request).await;
        let jsonstr = serde_json::to_string(&response);
        assert!(!jsonstr.is_err());
        assert_eq!(
            r#"{"jsonrpc":"2.0","error":{"code":-32602,"message":"Invalid params"},"id":1}"#,
            jsonstr.unwrap()
        )
    }
}
