use rst_common::standard::serde::{self, Deserialize, Serialize};
use rst_common::with_errors::thiserror::Error;

pub type RpcErrorCode = i64;
pub type RpcErrorMessage = &'static str;

pub const PARSE_ERROR_CODE: RpcErrorCode = -32700;
pub const INVALID_REQUEST_CODE: RpcErrorCode = -32600;
pub const METHOD_NOT_FOUND_CODE: RpcErrorCode = -32601;
pub const INVALID_PARAMS_CODE: RpcErrorCode = -32602;
pub const INTERNAL_ERROR_CODE: RpcErrorCode = -32603;

pub const PARSE_ERROR_MESSAGE: RpcErrorMessage = "Parse error";
pub const INVALID_REQUEST_MESSAGE: RpcErrorMessage = "Invalid request";
pub const METHOD_NOT_FOUND_MESSAGE: RpcErrorMessage = "Method not found";
pub const INVALID_PARAMS_MESSAGE: RpcErrorMessage = "Invalid params";
pub const INTERNAL_ERROR_MESSAGE: RpcErrorMessage = "Internal error";

/// `RpcError` is the only error data structures, that should be 
/// cover all required error types based on the `JSON-RPC` specification
#[derive(Debug, Clone, Copy, Error)]
pub enum RpcError {
    #[error("something went wrong with parsing data")]
    ParseError,
    
    #[error("error invalid request")]
    InvalidRequest,
    
    #[error("error unknown method or method not found")]
    MethodNotFound,
    
    #[error("error invalid params")]
    InvalidParams,

    #[error("internal error")]
    InternalError,
}

impl RpcError {
    pub fn build(&self) -> (RpcErrorCode, RpcErrorMessage) {
        match self {
            RpcError::ParseError => (PARSE_ERROR_CODE, PARSE_ERROR_MESSAGE),
            RpcError::MethodNotFound => (METHOD_NOT_FOUND_CODE, METHOD_NOT_FOUND_MESSAGE),
            RpcError::InvalidRequest => (INVALID_REQUEST_CODE, INVALID_REQUEST_MESSAGE),
            RpcError::InvalidParams => (INVALID_PARAMS_CODE, INVALID_PARAMS_MESSAGE),
            RpcError::InternalError => (INTERNAL_ERROR_CODE, INTERNAL_ERROR_MESSAGE),
        }
    }
}

/// `RpcErrorObject` is an object designed to build the error response object
/// 
/// This method will only parse a [`RpcError`] enum variants, parse the error codes
/// including for it's error message
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(crate = "self::serde")]
pub struct RpcErrorObject<T> {
    pub code: RpcErrorCode,
    pub message: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T> RpcErrorObject<T> {
    pub fn build(err: RpcError, data: Option<T>) -> Self {
        let (code, message) = err.build();
        RpcErrorObject {
            code,
            message: message.to_string(),
            data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    use rst_common::standard::serde_json;
    use rst_common::with_tests::table_test::table_test;

    #[test]
    fn test_build_error_enum() {
        let table = vec![
            (
                RpcError::ParseError,
                (PARSE_ERROR_CODE, PARSE_ERROR_MESSAGE),
            ),
            (
                RpcError::InvalidRequest,
                (INVALID_REQUEST_CODE, INVALID_REQUEST_MESSAGE),
            ),
            (
                RpcError::MethodNotFound,
                (METHOD_NOT_FOUND_CODE, METHOD_NOT_FOUND_MESSAGE),
            ),
            (
                RpcError::InvalidParams,
                (INVALID_PARAMS_CODE, INVALID_PARAMS_MESSAGE),
            ),
            (
                RpcError::InternalError,
                (INTERNAL_ERROR_CODE, INTERNAL_ERROR_MESSAGE),
            ),
        ];

        for (validator, input, expected) in table_test!(table) {
            let result = input.build();
            validator
                .given(&format!("{:?}", input))
                .when("build error")
                .then(&format!("it should be: {:?}", expected))
                .assert_eq(expected, result);
        }
    }

    #[test]
    fn test_serialize_error_object() {
        let table = vec![
            (
                RpcError::ParseError,
                String::from(r#"{"code":-32700,"message":"Parse error"}"#),
            ),
            (
                RpcError::InvalidRequest,
                String::from(r#"{"code":-32600,"message":"Invalid request"}"#),
            ),
            (
                RpcError::MethodNotFound,
                String::from(r#"{"code":-32601,"message":"Method not found"}"#),
            ),
            (
                RpcError::InvalidParams,
                String::from(r#"{"code":-32602,"message":"Invalid params"}"#),
            ),
            (
                RpcError::InternalError,
                String::from(r#"{"code":-32603,"message":"Internal error"}"#),
            ),
        ];

        for (validator, input, expected) in table_test!(table) {
            let err: RpcErrorObject<String> = RpcErrorObject::build(input, None);
            let errobj = serde_json::to_string(&err);
            assert!(!errobj.is_err());

            validator
                .given(&format!("{:?}", input))
                .when("build error")
                .then(&format!("it should be: {:?}", expected))
                .assert_eq(expected, format!("{}", errobj.unwrap()));
        }
    }
}
