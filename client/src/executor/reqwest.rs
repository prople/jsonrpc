use std::fmt::Debug;
use std::marker::PhantomData;

use rst_common::standard::async_trait::async_trait;
use rst_common::standard::reqwest::{Client, StatusCode};
use rst_common::standard::serde::de::DeserializeOwned;

use prople_jsonrpc_core::objects::RpcRequest;
use prople_jsonrpc_core::types::RpcId;

use crate::types::{Executor, ExecutorError, JSONResponse, RpcValue};

#[derive(Clone)]
pub struct Reqwest<T, E>
where
    T: Clone,
    E: Clone,
{
    client: Client,
    _phantom0: PhantomData<T>,
    _phantom1: PhantomData<E>,
}

impl<T, E> Reqwest<T, E>
where
    T: Clone,
    E: Clone,
{
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            _phantom0: PhantomData::default(),
            _phantom1: PhantomData::default(),
        }
    }
}

#[async_trait]
impl<T, E> Executor<T> for Reqwest<T, E>
where
    T: DeserializeOwned + Send + Sync + Debug + Clone,
    E: DeserializeOwned + Send + Sync + Clone,
{
    type ErrorData = E;

    async fn call(
        &self,
        endpoint: String,
        params: impl RpcValue,
        method: String,
        id: Option<RpcId>,
    ) -> Result<JSONResponse<T, E>, ExecutorError> {
        let value_params = params.build_serde_value()?;

        let request = RpcRequest {
            jsonrpc: String::from("2.0"),
            method: String::from(method),
            params: value_params,
            id,
        };

        let res = self
            .client
            .post(endpoint.clone())
            .json(&request)
            .send()
            .await
            .map_err(|err| {
                let code = {
                    match err.status() {
                        Some(code) => code.as_u16(),
                        _ => StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                    }
                };

                ExecutorError::RequestError {
                    url: endpoint,
                    code,
                }
            })?;

        let resp_json = res
            .json::<JSONResponse<T, Self::ErrorData>>()
            .await
            .map_err(|_| {
                ExecutorError::ParseResponseError("unable to parse json response".to_string())
            })?;

        Ok(resp_json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{Matcher, Server};

    use prople_jsonrpc_core::types::{RpcError, RpcErrorBuilder};
    use rst_common::standard::serde::{self, Deserialize, Serialize};
    use rst_common::standard::serde_json::{self, Value};
    use rst_common::with_errors::thiserror::{self, Error};
    use rst_common::with_tokio::tokio;

    #[derive(Serialize, Deserialize, Clone, Debug)]
    #[serde(crate = "self::serde")]
    struct FakePayload {
        msg: String,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    #[serde(crate = "self::serde")]
    struct FakeErrorData {
        err_msg: String,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    #[serde(crate = "self::serde")]
    struct FakeResponse {
        msg: String,
    }

    impl RpcValue for FakePayload {
        fn build_serde_value(&self) -> Result<Value, ExecutorError> {
            serde_json::to_value(self)
                .map_err(|_| ExecutorError::BuildValueError("unable to build value".to_string()))
        }
    }

    #[derive(Serialize, Deserialize, Error, Debug)]
    #[serde(crate = "self::serde")]
    enum FakeError {
        #[error("error: {0}")]
        ErrorMsg(String),
    }

    #[tokio::test]
    async fn test_call_success() {
        let payload = FakePayload {
            msg: "hello world".to_string(),
        };

        let try_jsonvalue = serde_json::to_value(payload.clone());
        assert!(!try_jsonvalue.is_err());

        let jsonvalue = try_jsonvalue.unwrap();
        let request_payload = RpcRequest {
            jsonrpc: String::from("2.0"),
            method: String::from("test.rpc"),
            params: jsonvalue,
            id: Some(RpcId::IntegerVal(1)),
        };

        let request_payload_value = serde_json::to_value(request_payload).unwrap();
        let jsonresp: JSONResponse<FakeResponse, FakeErrorData> = JSONResponse {
            id: Some(RpcId::IntegerVal(1)),
            result: Some(FakeResponse {
                msg: "hello response".to_string(),
            }),
            error: None,
            jsonrpc: String::from("2.0"),
        };

        let jsonresp_str_builder = serde_json::to_string(&jsonresp);
        assert!(!jsonresp_str_builder.is_err());

        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/rpc")
            .match_body(Matcher::Json(request_payload_value))
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(jsonresp_str_builder.unwrap())
            .create_async()
            .await;

        let url = server.url();
        let endpoint = format!("{}/rpc", url);

        let client = Reqwest::<FakeResponse, FakeErrorData>::new();
        let resp = client
            .call(
                endpoint,
                payload,
                "test.rpc".to_string(),
                Some(RpcId::IntegerVal(1)),
            )
            .await;

        assert!(!resp.is_err());
        mock.assert();

        let resp_json = resp.unwrap().result;
        assert!(resp_json.is_some());

        let fake_resp = resp_json.unwrap();
        assert_eq!(fake_resp.msg, "hello response".to_string())
    }

    #[tokio::test]
    async fn test_call_error() {
        let payload = FakePayload {
            msg: "hello world".to_string(),
        };

        let try_jsonvalue = serde_json::to_value(payload.clone());
        assert!(!try_jsonvalue.is_err());

        let jsonvalue = try_jsonvalue.unwrap();
        let request_payload = RpcRequest {
            jsonrpc: String::from("2.0"),
            method: String::from("test.rpc"),
            params: jsonvalue,
            id: Some(RpcId::IntegerVal(1)),
        };

        let request_payload_value = serde_json::to_value(request_payload).unwrap();
        let error_response = RpcErrorBuilder::build(
            RpcError::InvalidRequest,
            Some(FakeErrorData {
                err_msg: "hello error".to_string(),
            }),
        );

        let jsonresp: JSONResponse<FakeResponse, FakeErrorData> = JSONResponse {
            error: Some(error_response),
            id: Some(RpcId::IntegerVal(1)),
            jsonrpc: String::from("2.0"),
            result: None,
        };

        let jsonresp_str_builder = serde_json::to_string(&jsonresp);
        assert!(!jsonresp_str_builder.is_err());

        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/rpc")
            .match_body(Matcher::Json(request_payload_value))
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(jsonresp_str_builder.unwrap())
            .create_async()
            .await;

        let url = server.url();
        let endpoint = format!("{}/rpc", url);

        let client = Reqwest::<FakeResponse, FakeErrorData>::new();
        let resp = client
            .call(
                endpoint,
                payload,
                "test.rpc".to_string(),
                Some(RpcId::IntegerVal(1)),
            )
            .await;

        assert!(!resp.is_err());
        mock.assert();

        let error_resp = resp.unwrap().error;
        assert!(error_resp.is_some());

        let error_rpc = error_resp.unwrap();
        let (error_code, error_msg) = RpcError::InvalidRequest.build();

        assert_eq!(error_code, error_rpc.code);
        assert_eq!(error_msg, error_rpc.message);
        assert_eq!("hello error".to_string(), error_rpc.data.unwrap().err_msg);
    }

    #[tokio::test]
    async fn test_call_parse_invalid_response() {
        let payload = FakePayload {
            msg: "hello world".to_string(),
        };

        let try_jsonvalue = serde_json::to_value(payload.clone());
        assert!(!try_jsonvalue.is_err());

        let jsonvalue = try_jsonvalue.unwrap();
        let request_payload = RpcRequest {
            jsonrpc: String::from("2.0"),
            method: String::from("test.rpc"),
            params: jsonvalue,
            id: Some(RpcId::IntegerVal(1)),
        };

        let request_payload_value = serde_json::to_value(request_payload).unwrap();

        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/rpc")
            .match_body(Matcher::Json(request_payload_value))
            .with_status(500)
            .with_header("content-type", "application/json")
            .with_body(r#"{"msg": "error"}"#)
            .create_async()
            .await;

        let url = server.url();
        let endpoint = format!("{}/rpc", url);

        let client = Reqwest::<FakeResponse, FakeErrorData>::new();
        let resp = client
            .call(
                endpoint,
                payload,
                "test.rpc".to_string(),
                Some(RpcId::IntegerVal(1)),
            )
            .await;

        mock.assert();

        assert!(resp.is_err());
        assert!(matches!(
            resp.unwrap_err(),
            ExecutorError::ParseResponseError { .. }
        ))
    }
}
