use rst_common::standard::serde::{self, Serialize};

use crate::objects::RpcErrorObject;
use crate::types::RpcId;

#[derive(Debug, Serialize)]
#[serde(crate = "self::serde")]
pub struct RpcResponseObject<T, E> {
    pub jsonrpc: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<T>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcErrorObject<E>>,

    pub id: Option<RpcId>,
}

impl<T, E> RpcResponseObject<T, E> {
    pub fn with_success(result: Option<T>, id: Option<RpcId>) -> Self {
        RpcResponseObject {
            jsonrpc: String::from("2.0"),
            result,
            error: None,
            id,
        }
    }

    pub fn with_error(error: Option<RpcErrorObject<E>>, id: Option<RpcId>) -> Self {
        RpcResponseObject {
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
    use rst_common::standard::serde_json;
    use crate::types::RpcError;

    #[derive(Serialize, Clone)]
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

        let response: RpcResponseObject<FakeParam, String> =
            RpcResponseObject::with_success(Some(result), None);
        let jsonstr = serde_json::to_string(&response);
        assert!(!jsonstr.is_err());
        assert_eq!(
            jsonstr.unwrap(),
            r#"{"jsonrpc":"2.0","result":{"key":"testkey","value":"testvalue"},"id":null}"#
        )
    }

    #[test]
    fn test_serialize_response_object_with_error() {
        let err = RpcErrorObject::build(RpcError::MethodNotFound, None);
        let response: RpcResponseObject<FakeParam, String> =
            RpcResponseObject::with_error(Some(err), None);
        let jsonstr = serde_json::to_string(&response);
        assert!(!jsonstr.is_err());
        assert_eq!(
            jsonstr.unwrap(),
            r#"{"jsonrpc":"2.0","error":{"code":-32601,"message":"Method not found"},"id":null}"#
        )
    }
}
