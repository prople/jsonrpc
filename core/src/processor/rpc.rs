use std::collections::HashMap;

use rst_common::with_logging::log::error;

use crate::handlers::AgentPingHandler;
use crate::objects::{RpcErrorBuilder, RpcRequest, RpcResponse};
use crate::types::{RpcError, RpcHandlerBoxed, RpcMethod, RpcResponseSerialized, RpcRoute};

/// `RpcProcessor` is primary object to manage request method handlers including
/// for its handler execution
pub struct RpcProcessor {
    handlers: HashMap<RpcMethod, RpcHandlerBoxed>,
}

impl Default for RpcProcessor {
    fn default() -> Self {
        let mut handlers: HashMap<RpcMethod, RpcHandlerBoxed> = HashMap::new();
        handlers.insert(
            RpcMethod::from("prople.agent.ping"),
            Box::new(AgentPingHandler),
        );

        RpcProcessor { handlers }
    }
}

impl RpcProcessor {
    pub fn new() -> Self {
        let handlers: HashMap<RpcMethod, RpcHandlerBoxed> = HashMap::new();
        Self { handlers }
    }

    /// `register_controller` used to register given [`RpcController`] to the current registry
    pub fn register_route(&mut self, route: RpcRoute) -> &mut Self {
        let controller = route.controller();
        self.handlers.insert(route.method(), controller);
        self
    }

    /// `handlers` used to get current saved hash map
    ///
    /// The return value will be in shared reference without any mutability capability
    pub fn handlers(&self) -> &HashMap<RpcMethod, RpcHandlerBoxed> {
        &self.handlers
    }

    /// `execute` used to process incoming [`RpcRequest]
    ///
    /// The internal flow is, for each time incoming request object
    /// it will fetch the handler based on RPC method.
    /// If it have a handler, it will *call* the handler.
    /// If not, it will build the [`RpcErrorObject`] and put it into the [`RpcResponse`]
    pub async fn execute(&self, request: RpcRequest) -> RpcResponse<RpcResponseSerialized, ()> {
        let method = RpcMethod::from(request.method.clone());
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
                let err_obj: RpcErrorBuilder<()> = RpcErrorBuilder::build(err, None);
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

    use rst_common::standard::async_trait::async_trait;
    use rst_common::standard::serde_json::{self, Value};

    use rst_common::with_tokio::tokio;

    use crate::processor::types::RpcHandler;
    use crate::types::RpcId;

    mock! {
        Handler {}

        #[async_trait]
        impl RpcHandler for Handler {
            async fn call(&self, params: Value) -> Result<Option<RpcResponseSerialized>, RpcError> {
                let output = FakeParam{
                    key: String::from("test-key"),
                    value: String::from("test-value")
                };

                Ok(Some(Box::new(output)))
            }
        }

        impl Clone for Handler {
            fn clone(&self) -> Self;
        }
    }

    #[tokio::test]
    async fn test_processor_register_controller_ping() {
        let request = RpcRequest {
            id: Some(RpcId::IntegerVal(1)),
            jsonrpc: String::from("2.0"),
            method: String::from("prople.agent.ping"),
            params: Value::Null,
        };

        let ping_controller = Box::new(AgentPingHandler);
        let mut processor = RpcProcessor::new();
        let response = processor
            .register_route(RpcRoute::new(
                RpcMethod::from(String::from("prople.agent.ping")),
                ping_controller,
            ))
            .execute(request)
            .await;

        let jsonstr = serde_json::to_string(&response);
        assert!(!jsonstr.is_err());
        assert_eq!(
            r#"{"jsonrpc":"2.0","result":{"message":"pong!"},"id":1}"#,
            jsonstr.unwrap()
        )
    }

    #[tokio::test]
    async fn test_processor_register_multiple_controllers() {
        let mut mock_handler = MockHandler::new();
        mock_handler
            .expect_clone()
            .times(1)
            .returning(|| MockHandler::new());

        let ping_controller = Box::new(AgentPingHandler);
        let mock_controller = Box::new(mock_handler);

        let mut processor = RpcProcessor::new();
        processor
            .register_route(RpcRoute::new(
                RpcMethod::from("prople.agent.ping"),
                ping_controller,
            ))
            .register_route(RpcRoute::new(
                RpcMethod::from("mock.handler"),
                mock_controller,
            ));

        let handlers = processor.handlers();
        assert_eq!(handlers.len(), 2)
    }

    #[tokio::test]
    async fn test_processor_execute_success() {
        let processor = RpcProcessor::default();
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

        handler.expect_clone().returning(|| {
            let mut copied = MockHandler::new();
            copied
                .expect_call()
                .with(predicate::eq(Value::Null))
                .times(1)
                .returning(|_| Err(RpcError::InvalidParams));

            copied
        });

        let mut processor = RpcProcessor::default();
        processor.register_route(RpcRoute::new(
            RpcMethod::from("test.mock"),
            Box::new(handler),
        ));

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
