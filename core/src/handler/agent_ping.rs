use rst_common::standard::async_trait::async_trait;
use rst_common::standard::erased_serde::Serialize as ErasedSerialized;
use rst_common::standard::serde::{self, Serialize};
use rst_common::standard::serde_json::Value;

use crate::types::{RpcError, RpcHandler, RpcMethod};

pub const PING_RPC_METHOD: &str = "prople.vessel.ping";

/// `AgentPingResponse` used as main response object for the RPC method `ping`
#[derive(Debug, Serialize, Clone)]
#[serde(crate = "self::serde")]
pub struct AgentPingResponse {
    message: String,
}

#[derive(Clone)]
pub struct AgentPingHandler;

#[async_trait]
impl RpcHandler for AgentPingHandler {
    async fn call(
        &self,
        _: RpcMethod,
        _: Value,
    ) -> Result<Option<Box<dyn ErasedSerialized>>, RpcError> {
        let output = AgentPingResponse {
            message: String::from("pong!"),
        };
        Ok(Some(Box::new(output)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rst_common::standard::serde_json;
    use rst_common::with_tokio::tokio;

    #[tokio::test]
    async fn test_agent_ping_call() {
        let handler = AgentPingHandler;
        let response = handler
            .call(RpcMethod::from(PING_RPC_METHOD), Value::Null)
            .await;

        match response {
            Ok(resp) => match resp {
                Some(out) => {
                    let jsonstr = serde_json::to_string(out.as_ref());
                    assert_eq!(r#"{"message":"pong!"}"#, jsonstr.unwrap())
                }
                None => assert!(false),
            },
            Err(err) => assert_eq!(err.to_string(), String::from("")),
        }
    }
}
