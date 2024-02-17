use rst_common::standard::erased_serde::Serialize as ErasedSerialized;
use rst_common::with_logging::log::error;
use std::collections::HashMap;

use crate::handlers::AgentPingHandler;
use crate::objects::{RpcErrorObject, RpcRequestObject, RpcResponseObject};
use crate::types::{RpcError, RpcHandler, RpcMethod};

pub struct RpcProcessorObject {
    pub handlers: HashMap<RpcMethod, Box<dyn RpcHandler + Send + Sync>>,
}

impl RpcProcessorObject {
    pub fn build() -> Self {
        let mut handlers: HashMap<String, Box<dyn RpcHandler + Send + Sync>> = HashMap::new();
        handlers.insert("prople.agent.ping".to_string(), Box::new(AgentPingHandler));

        RpcProcessorObject { handlers }
    }

    pub fn register_handler(
        &mut self,
        method: String,
        handler: Box<dyn RpcHandler + Send + Sync>,
    ) -> () {
        self.handlers.insert(method, handler);
    }

    pub async fn execute(
        &self,
        request: RpcRequestObject,
    ) -> RpcResponseObject<Box<dyn ErasedSerialized>, ()> {
        let method = request.method.clone();
        let params = request.params.clone();

        let handler = match self.handlers.get(&method) {
            Some(caller) => caller,
            None => {
                let err_obj: RpcErrorObject<()> =
                    RpcErrorObject::build(RpcError::MethodNotFound, None);
                let response = RpcResponseObject::with_error(Some(err_obj), request.id);
                return response;
            }
        };

        match handler.call(params).await {
            Ok(success) => {
                let response = RpcResponseObject::with_success(success, request.id);
                response
            }
            Err(err) => {
                error!("error from handler: {}", err.to_string());
                let err_obj: RpcErrorObject<()> =
                    RpcErrorObject::build(RpcError::InternalError, None);
                let response = RpcResponseObject::with_error(Some(err_obj), request.id);
                response
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::types::RpcId;

    use rst_common::with_errors::anyhow::{anyhow, Result};
    use rst_common::standard::async_trait::async_trait;
    use rst_common::with_tests::mockall::{mock, predicate};
    use rst_common::standard::serde_json::Value;
    use rst_common::with_tokio::tokio;

    use super::*;

    mock! {
        Handler {}

        #[async_trait]
        impl RpcHandler for Handler {
            async fn call(&self, params: Value) -> Result<Option<Box<dyn ErasedSerialized>>> {
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
        let processor = RpcProcessorObject::build();
        let request = RpcRequestObject {
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
            .returning(|_| Err(anyhow!("error")));

        let mut processor = RpcProcessorObject::build();
        processor.register_handler(String::from("test.mock"), Box::new(handler));

        let request = RpcRequestObject {
            id: Some(RpcId::IntegerVal(1)),
            jsonrpc: String::from("2.0"),
            method: String::from("test.mock"),
            params: Value::Null,
        };

        let response = processor.execute(request).await;

        let jsonstr = serde_json::to_string(&response);
        assert!(!jsonstr.is_err());
        assert_eq!(
            r#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"Internal error"},"id":1}"#,
            jsonstr.unwrap()
        )
    }
}
