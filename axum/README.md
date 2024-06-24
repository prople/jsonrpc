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

Example

```rust
fn build_rpc_processor(args: AnaArgs) -> RpcState {
    let mut cf_names: Vec<String> = Vec::new();
    cf_names.push(types::STORAGE_COLUMN_DID.to_string());
    cf_names.push(types::STORAGE_COLUMN_DID_DOC.to_string());
    cf_names.push(types::STORAGE_COLUMN_DID_KEY_SECURES.to_string());

    let mut rocksdb_opts = RocksDBOptions::default();
    rocksdb_opts.max_write_buffer_number = 6;

    let storage_path = args.db_path;
    let storage = RocksDB::new(&storage_path, cf_names, rocksdb_opts)
        .expect("Failed to initiate RocksDB instance");
    let storage_did = StorageDID::new(storage);
    let handler_did = IdentityRPCHandler::new(Box::new(storage_did));

    let mut processor = RpcProcessorObject::build();
    processor.register_handler(
        String::from("prople.agent.controller.setup"),
        Box::new(handler_did),
    );

    RpcState {
        processor: Arc::new(processor),
    }
}
```