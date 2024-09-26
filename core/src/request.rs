use rst_common::standard::serde::{self, Deserialize, Serialize};
use rst_common::standard::serde_json::Value;

use crate::types::RpcId;

/// `RpcRequestObject` used to modeling `JSON-RPC` request spc model
///
/// Ref: <https://www.jsonrpc.org/specification#request_object>
#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RpcId>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rst_common::standard::serde_json::{self, json, Error};

    #[test]
    fn test_serialize_request_object() {
        let payload = RpcRequest {
            id: Some(RpcId::IntegerVal(1)),
            jsonrpc: String::from("2.0"),
            params: Some(json!([1, 2])),
            method: String::from("testing"),
        };

        let jsonstr = serde_json::to_string(&payload);
        assert_eq!(
            jsonstr.unwrap(),
            r#"{"jsonrpc":"2.0","method":"testing","params":[1,2],"id":1}"#
        );
    }

    #[test]
    fn test_serialize_request_object_optional_id() {
        let payload = RpcRequest {
            id: None,
            jsonrpc: String::from("2.0"),
            params: Some(json!([1, 2])),
            method: String::from("testing"),
        };

        let jsonstr = serde_json::to_string(&payload);
        assert_eq!(
            jsonstr.unwrap(),
            r#"{"jsonrpc":"2.0","method":"testing","params":[1,2]}"#
        )
    }

    #[test]
    fn test_deserialize_with_id() {
        let jsonstr = r#"{"jsonrpc":"2.0","method":"testing","params":[1,2], "id": 1}"#;
        let jsonobj: Result<RpcRequest, Error> = serde_json::from_str(jsonstr);

        assert!(!jsonobj.is_err());

        let jsonreq = jsonobj.unwrap();
        assert_eq!(jsonreq.jsonrpc.as_str(), "2.0");
        assert_eq!(jsonreq.method.as_str(), "testing");

        let params_unwrapped = jsonreq.params.unwrap();
        let params = params_unwrapped.as_array().unwrap();
      
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].as_u64().unwrap(), 1);
        assert_eq!(params[1].as_u64().unwrap(), 2)
    }

    #[test]
    fn test_deserialize_with_fake_param() {
        let jsonstr = r#"{"jsonrpc":"2.0","method":"testing","params": {"key": "testkey", "value": "testvalue"}, "id": 1}"#;
        let jsonobj: Result<RpcRequest, Error> = serde_json::from_str(jsonstr);

        assert!(!jsonobj.is_err());
        let payload = jsonobj.unwrap();

        let params_unwrapped = payload.params.unwrap();
        assert_eq!("testkey", params_unwrapped.get("key").unwrap());
        assert_eq!("testvalue", params_unwrapped.get("value").unwrap())
    }

    #[test]
    fn test_deserialize_without_id() {
        let jsonstr = r#"{"jsonrpc":"2.0","method":"testing","params":[1,2]}"#;
        let jsonobj: Result<RpcRequest, Error> = serde_json::from_str(jsonstr);
        assert!(!jsonobj.is_err());

        let jsonreq = jsonobj.unwrap();
        assert!(jsonreq.id.is_none())
    }
}
