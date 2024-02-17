# prople/jsonrpc/core

In `Prople`, default `CCP (Client Communication Protocol)`, which is, a communication protocol from client's device or user's application, will be `JSONRPC`.

The reason why `JSONRPC` over `REST API` is because the `JSONRPC` more simpler than `REST` itself. We only need to maintain a single endpoint and its handler. As it said on its official website:

> A light weight remote procedure call protocol. It is designed to be simple!

Source: https://www.jsonrpc.org/

Example of rpc call:

```json
{"jsonrpc": "2.0", "method": "subtract", "params": [42, 23], "id": 1}
```

Example of response:

```json
{"jsonrpc": "2.0", "result": 19, "id": 1}
```

## Prople JSONRPC

This library provides base abstraction to working with `JSONRPC` itself. The base abstraction will provides:

- Request abstraction 
- Response abstraction
- Error abstraction

These abstractions designed following [JSON-RPC 2.0 Spec](https://www.jsonrpc.org/specification).

Besides of these abstractions, the `prople/jsonrpc/core` also provides the *rpc processor* too. This abstraction designed to handle each of rpc methods and its handler. 

`Handler` abstraction

```rust
#[async_trait]
pub trait Handler {
    async fn call(&self, params: Value) -> Result<Option<Box<dyn ErasedSerialized>>>;
}

pub type Method = String;
```

`RpcProcessor` abstraction

```rust
pub struct RpcProcessorObject {
    pub handlers: HashMap<RpcMethod, Box<dyn RpcHandler + Send + Sync>>,
}
```

These abstractions designed as a core `JSONRPC`, which we should not care about any of HTTP framework implementations. 

By using core rpc abstraction, all abstraction and logic handler, will be separated from any specific frameworks, and it means, we can change the HTTP framework easily.

Example of these abstraction's implementation:

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

Example in `Tokio Axum`:

```rust
pub async fn run_rpc(args: AnaArgs, canceller: Receiver<i32>) {
    let rpc_state = build_rpc_processor(args.clone());
    let app = Router::new()
        .route("/rpc", post(rpc_handler))
        .layer((
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request| {
                    let matched_path = request
                        .extensions()
                        .get::<MatchedPath>()
                        .map(MatchedPath::as_str);

                    info_span!(
                        "http_request",
                        method = ?request.method(),
                        matched_path,
                        some_other_field = tracing::field::Empty,
                    )
                })
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
            TimeoutLayer::new(Duration::from_secs(5)),
        ))
        .with_state(Arc::new(rpc_state));

    let rpc_addr = format!("0.0.0.0:{}", args.rpc_port);
    tracing::info!("listening on: {}", rpc_addr);

    let listener = tokio::net::TcpListener::bind(rpc_addr).await.unwrap();
    let (close_tx, close_rx) = watch::channel(());

    let mut canceller = canceller.clone();
    loop {
        let (socket, remote_addr) = select! {
            result = listener.accept() => {
                result.unwrap()
            }

            _ = canceller.changed() => {
                tracing::warn!("canceller catched! stopping tcp listener to receive request...");
                break;
            }
        };

        let tower_svc = app.clone();
        let close_rx = close_rx.clone();

        let mut canceller = canceller.clone();
        task::spawn(async move {
            let socket = TokioIo::new(socket);
            let hyper_svc = hyper::service::service_fn(move |request: Request<Incoming>| {
                tower_svc.clone().call(request)
            });

            let conn = hyper::server::conn::http1::Builder::new()
                .serve_connection(socket, hyper_svc)
                .with_upgrades();

            let mut conn = std::pin::pin!(conn);
            loop {
                select! {
                    result = conn.as_mut() => {
                        if let Err(err) = result {
                            tracing::debug!("failed to serve connection: {}", err)
                        }

                        break;
                    }

                    _ = canceller.changed() => {
                        tracing::warn!("canceller catched! starting hyper connection to gracefully shutdown");
                        conn.as_mut().graceful_shutdown();
                    }
                }
            }

            tracing::debug!("connection {remote_addr} closed");
            drop(close_rx);
        });
    }

    drop(close_rx);
    drop(listener);
    tracing::debug!("waiting for {} tasks to finish", close_tx.receiver_count());
    close_tx.closed().await;
}
```

The *rpc processor* really separated for the framework's implementation. The HTTP framework (`Axum`), only take responsibilities, to open a TCP port, get the request, route the request payload to the *rpc processor*, got the response, and forward it back to the request caller.

### Default Handler

There is a default *rpc method* and its handler.

```rust
    let mut handlers: HashMap<String, Box<dyn RpcHandler + Send + Sync>> = HashMap::new();
    handlers.insert("prople.agent.ping".to_string(), Box::new(AgentPingHandler));
```

This default method and handler used to send a *ping-pong* request-response. The handler itself look like this:

```rust
#[async_trait]
impl RpcHandler for AgentPingHandler {
    async fn call(&self, _: Value) -> Result<Option<Box<dyn ErasedSerialized>>> {
        let output = AgentPingResponse {
            message: String::from("pong!"),
        };
        Ok(Some(Box::new(output)))
    }
}
```