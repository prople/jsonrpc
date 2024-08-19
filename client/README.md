# prople/jsonrpc/client

It's an HTTP JSON-RPC client implementation. This package will depends on `prople-jsonrpc-core`, to make sure that both of
components will use same data structure, especially for the:

- `RpcRequest`
- `RpcError`

> WARNING
>
> Although this package depends on `prople-jsonrpc-core`, there is some known bug for the `serde` implementation `Deserialized`
> which unable to detect the trait implementation.
>
> Ref: https://github.com/serde-rs/serde-rs.github.io/commit/0009ee2ed4e8083d0a450bb387bbbf17eadbc018