# prople/jsonrpc/axum

An implementation of `JSON-RPC` server using `Tokio Axum`.

Usages:

```rust
use tokio;
use tokio::sync::watch::{self, Receiver, Sender};
use prople_jsonrpc_axum::{Config, RpcState, run_rpc};

#[tokio:main]
async fn main() {
    let cfg = Config { ... };
    let state = RpcState { ... };
    let (_, rx) = watch::channel(0);

    run_rpc(cfg, state, rx).await;
}
```