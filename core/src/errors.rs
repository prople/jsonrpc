use rst_common::standard::serde::{self, Deserialize, Serialize};
use rst_common::with_errors::thiserror::{self, Error};

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
#[derive(Debug, Clone, Error)]
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

    #[error("handler error: {0}")]
    HandlerError(String)
}

impl RpcError {
    pub fn build(&self) -> (RpcErrorCode, String) {
        match self {
            RpcError::ParseError => (PARSE_ERROR_CODE, PARSE_ERROR_MESSAGE.to_string()),
            RpcError::MethodNotFound => (METHOD_NOT_FOUND_CODE, METHOD_NOT_FOUND_MESSAGE.to_string()),
            RpcError::InvalidRequest => (INVALID_REQUEST_CODE, INVALID_REQUEST_MESSAGE.to_string()),
            RpcError::InvalidParams => (INVALID_PARAMS_CODE, INVALID_PARAMS_MESSAGE.to_string()),
            RpcError::InternalError => (INTERNAL_ERROR_CODE, INTERNAL_ERROR_MESSAGE.to_string()),
            RpcError::HandlerError(herr) => (INTERNAL_ERROR_CODE, herr.clone()) 
        }
    }
}

/// `RpcErrorBuilder` is an object designed to build the error response object
///
/// This method will only parse a [`RpcError`] enum variants, parse the error codes
/// including for it's error message
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(crate = "self::serde")]
pub struct RpcErrorBuilder {
    pub code: RpcErrorCode,
    pub message: String,
}

impl RpcErrorBuilder {
    pub fn build(err: RpcError) -> Self {
        let (code, message) = err.build();
        RpcErrorBuilder {
            code,
            message: message.to_string(),
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
                (PARSE_ERROR_CODE, PARSE_ERROR_MESSAGE.to_string()),
            ),
            (
                RpcError::InvalidRequest,
                (INVALID_REQUEST_CODE, INVALID_REQUEST_MESSAGE.to_string()),
            ),
            (
                RpcError::MethodNotFound,
                (METHOD_NOT_FOUND_CODE, METHOD_NOT_FOUND_MESSAGE.to_string()),
            ),
            (
                RpcError::InvalidParams,
                (INVALID_PARAMS_CODE, INVALID_PARAMS_MESSAGE.to_string()),
            ),
            (
                RpcError::InternalError,
                (INTERNAL_ERROR_CODE, INTERNAL_ERROR_MESSAGE.to_string()),
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
            let err: RpcErrorBuilder = RpcErrorBuilder::build(input.clone());
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
