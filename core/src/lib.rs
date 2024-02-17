mod errors;
mod handler;
mod id;
mod processor;
mod request;
mod response;

pub mod objects {
    use super::*;

    pub use errors::RpcErrorObject;
    pub use processor::RpcProcessorObject;
    pub use request::RpcRequestObject;
    pub use response::RpcResponseObject;
}

pub mod handlers {
    use super::*;

    pub use handler::AgentPingHandler;
}

pub mod types {
    use super::*;

    pub use errors::{RpcError, RpcErrorCode, RpcErrorMessage};
    pub use id::RpcId;
    pub use processor::types::{RpcHandler, RpcMethod};
}

pub mod prelude {
    use super::*;

    pub use handlers::*;
    pub use objects::*;
    pub use types::*;
}
