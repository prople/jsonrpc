# prople/jsonrpc/axum

An implementation of `JSON-RPC` server using `Tokio Axum`.

Usages:

```rust
use tokio;
use prople_jsonrpc_axum::{Config, RpcState, run_rpc, build_canceller};

#[tokio:main]
async fn main() {
    let cfg = Config { ... };
    let state = RpcState { ... };
    let (_, rx) = build_canceller(0);

    run_rpc(cfg, state, rx).await;
}
```