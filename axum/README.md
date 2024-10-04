# prople/jsonrpc/axum

An implementation of `JSON-RPC` server using `Tokio Axum`.

> WARNING!
>
> There is a breaking changes from `0.1.x` to latest version: `0.2.0`
>
> Please always use the latest version which provides more nicer API

## Usages

```rust
use rst_common::with_tokio::tokio;
use prople_jsonrpc_axum::rpc::{RpcConfig, RpcError, Rpc};

#[tokio::main]
async fn main() -> Result<(), RpcError> {
    // you need to configure your `RpcProcessor`
    // assumed you have already set the object
    let state = RpcState::new(processor);
    let config = RpcConfig::new(String::from("host"), String::from("port"));

    // assumed you've already set your Axum's endpoint Router
    // the `app` variable defined here should be an instance of axum::Router
    let rpc = Rpc::new(config, state, app);
    let _ = rpc.serve()?;
}
```

## Installation

```toml
[dependencies]
prople-jsonrpc-axum = {version = "0.2.1"}
```