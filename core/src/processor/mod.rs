mod base;
mod rpc;

pub use rpc::RpcProcessor;

pub mod types {
    use super::*;

    pub use base::Controller as RpcController;
    pub use base::Handler as RpcHandler;
    pub use base::HandlerBoxed as RpcHandlerBoxed;
    pub use base::Method as RpcMethod;
    pub use base::ResponseSerialized as RpcResponseSerialized;
}
