use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::RpcId;

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcRequestObject {
    pub jsonrpc: String,
    pub method: String,
    pub params: Value,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RpcId>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Error};

    #[test]
    fn test_serialize_request_object() {
        let payload = RpcRequestObject {
            id: Some(RpcId::IntegerVal(1)),
            jsonrpc: String::from("2.0"),
            params: json!([1, 2]),
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
        let payload = RpcRequestObject {
            id: None,
            jsonrpc: String::from("2.0"),
            params: json!([1, 2]),
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
        let jsonobj: Result<RpcRequestObject, Error> = serde_json::from_str(jsonstr);

        assert!(!jsonobj.is_err());

        let jsonreq = jsonobj.unwrap();
        assert_eq!(jsonreq.jsonrpc.as_str(), "2.0");
        assert_eq!(jsonreq.method.as_str(), "testing");

        let params = jsonreq.params.as_array().unwrap();
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].as_u64().unwrap(), 1);
        assert_eq!(params[1].as_u64().unwrap(), 2)
    }

    #[test]
    fn test_deserialize_with_fake_param() {
        let jsonstr = r#"{"jsonrpc":"2.0","method":"testing","params": {"key": "testkey", "value": "testvalue"}, "id": 1}"#;
        let jsonobj: Result<RpcRequestObject, Error> = serde_json::from_str(jsonstr);

        assert!(!jsonobj.is_err());
        let payload = jsonobj.unwrap();

        assert_eq!("testkey", payload.params.get("key").unwrap());
        assert_eq!("testvalue", payload.params.get("value").unwrap())
    }

    #[test]
    fn test_deserialize_without_id() {
        let jsonstr = r#"{"jsonrpc":"2.0","method":"testing","params":[1,2]}"#;
        let jsonobj: Result<RpcRequestObject, Error> = serde_json::from_str(jsonstr);
        assert!(!jsonobj.is_err());

        let jsonreq = jsonobj.unwrap();
        assert!(jsonreq.id.is_none())
    }
}
