# prople/jsonrpc

It's a `Rust Workspace` project that will contains two sub-projects (or crates):

- `prople-jsonrpc-core`
- `prople-jsonrpc-axum`

## prople-jsonrpc-core

Provides a core abstraction to maintain a `JSON-RPC` request and response data including for the controller or handler. Please refer to it's standalone [README.md](./core/README.md) for more detail information.

## prople-jsonrpc-axum

An implementation of RPC server using `Tokio Axum`. This server will using the `prople-jsonrpc-core` to maintain request handler, payload and response.