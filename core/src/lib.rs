#![doc = include_str!("../README.md")]

mod errors;
mod handler;
mod id;
mod processor;
mod request;
mod response;

pub mod objects {
    use super::*;

    pub use errors::RpcErrorBuilder;
    pub use processor::RpcProcessor;
    pub use request::RpcRequest;
    pub use response::RpcResponse;
}

pub mod handlers {
    use super::*;

    pub use handler::agent_ping::{AgentPingHandler, PING_RPC_METHOD};
}

pub mod types {
    use super::*;

    pub use errors::*;
    pub use id::RpcId;
    pub use processor::types::{
        RpcController, RpcHandler, RpcHandlerBoxed, RpcHandlerOutput, RpcMethod,
        RpcResponseSerialized, RpcRoute,
    };
}

pub mod prelude {
    use super::*;

    pub use handlers::*;
    pub use objects::*;
    pub use types::*;
}
