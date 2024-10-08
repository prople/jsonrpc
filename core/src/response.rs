use rst_common::standard::serde::{self, Deserialize, Serialize};

use crate::objects::RpcErrorBuilder;
use crate::types::RpcId;

/// `RpcResponseObject` used as modeling of `JSON-RPC` response model
///
/// Ref: <https://www.jsonrpc.org/specification#response_object>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(crate = "self::serde")]
pub struct RpcResponse<T> {
    pub jsonrpc: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<T>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcErrorBuilder>,

    pub id: Option<RpcId>,
}

impl<T> RpcResponse<T> {
    /// `with_success` is a helper function used to build [`RpcResponse`]
    /// but only if in success condition
    pub fn with_success(result: Option<T>, id: Option<RpcId>) -> Self {
        RpcResponse {
            jsonrpc: String::from("2.0"),
            result,
            error: None,
            id,
        }
    }

    /// `with_error` is a helper function used to build [`RpcResponse`]
    /// used in error condition
    pub fn with_error(error: Option<RpcErrorBuilder>, id: Option<RpcId>) -> Self {
        RpcResponse {
            jsonrpc: String::from("2.0"),
            result: None,
            error,
            id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::RpcError;
    use rst_common::standard::serde_json;

    #[derive(Serialize, Deserialize, Clone)]
    #[serde(crate = "self::serde")]
    struct FakeParam {
        key: String,
        value: String,
    }

    #[test]
    fn test_serialize_response_object_with_success() {
        let result = FakeParam {
            key: String::from("testkey"),
            value: String::from("testvalue"),
        };

        let response: RpcResponse<FakeParam> =
            RpcResponse::with_success(Some(result), None);
        let jsonstr = serde_json::to_string(&response);
        assert!(!jsonstr.is_err());
        assert_eq!(
            jsonstr.unwrap(),
            r#"{"jsonrpc":"2.0","result":{"key":"testkey","value":"testvalue"},"id":null}"#
        )
    }

    #[test]
    fn test_deserialize_response_success() {
        let result = FakeParam {
            key: String::from("testkey"),
            value: String::from("testvalue"),
        };

        let response: RpcResponse<FakeParam> =
            RpcResponse::with_success(Some(result.clone()), None);
        let jsonstr = serde_json::to_string(&response);
        assert!(!jsonstr.is_err());

        let output: Result<RpcResponse<FakeParam>, serde_json::Error> =
            serde_json::from_str(jsonstr.unwrap().as_str());

        assert!(!output.is_err());

        let output_obj = output.unwrap();
        assert_eq!(output_obj.clone().result.unwrap().key, result.key);
        assert_eq!(output_obj.result.unwrap().value, result.value)
    }

    #[test]
    fn test_serialize_response_object_with_error() {
        let err = RpcErrorBuilder::build(RpcError::MethodNotFound);
        let response: RpcResponse<FakeParam> = RpcResponse::with_error(Some(err), None);
        let jsonstr = serde_json::to_string(&response);
        assert!(!jsonstr.is_err());
        assert_eq!(
            jsonstr.unwrap(),
            r#"{"jsonrpc":"2.0","error":{"code":-32601,"message":"Method not found"},"id":null}"#
        )
    }
}
