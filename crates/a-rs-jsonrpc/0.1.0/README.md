# A-RS-JSONRPC

A battery-included JSON-RPC library for Rust.

## Features
- Server and Client implementations
- JSON-RPC 1.0/2.0 support
- Asynchronous support but not bound to any specific async runtime
- Procedural macros for easy method definition

## Usage

```bash
cargo add a-rs-jsonrpc
```

## Example

see `examples` folder for more service examples. see `tests` folder for more client examples.

### Server Example

```rust
use a_rs_jsonrpc::{RpcError, response::JsonRpcError};

/// A simple echo RPC service example using a-rs-jsonrpc and receives parameters as an array.
/// just setup with `jsonrpc_service_fn_array` macro.
/// configured with method name "echoArray" which is used in the request `method` field,
/// and version "v2" which is used in the request `jsonrpc` field.
/// the function takes a single parameter `msg` of type `String` which is used in the request `params` field.
/// it returns a `Result<String, RpcError>` which is serialized as the response `result` field.
#[a_rs_jsonrpc::jsonrpc_service_fn_array(method = "echoArray", version = "v2")]
async fn echo_array(msg: String) -> Result<String, RpcError> {
    tracing::debug!("got client request message: {}", msg);
    Ok(msg)
}

/// A simple echo RPC service example using a-rs-jsonrpc and receives parameters as an object.
#[a_rs_jsonrpc::jsonrpc_service_fn_obj(method = "echoObj", version = "v2")]
async fn echo_obj(msg: String) -> Result<serde_json::Value, RpcError> {
    tracing::debug!("got client request message: {}", msg);
    Ok(serde_json::json!({"msg": msg}))
}

// you can use any async HTTP server framework, such as axum, to serve the JSON-RPC service
```

Then you can use the client to call the server for testing:
```bash
cargo test -- --nocapture one_params # or
cargo nextest run --nocapture -- one_params
```

### Client Example

```rust
const HELLO: &str = "hello";

// running all one param tests with `cargo run --example echo`

/// Client tests for one parameter RPC requests to the echo service.
/// for one parameter, you can use scalar, singleton array, option, singleton tuple, or rpc_method
/// mode to send the request.
#[tokio::test]
async fn test_one_params_request_with_scalar() {
    init_tracing();
    let resp: JsonRpcResponse<String> = HELLO
        .send_v2_request(TEST_URL, APP_JSON, "echoArray")
        .await
        .unwrap();
    assert_eq!(resp.jsonrpc, JsonRpcVersion::V2_0);
    assert_eq!(resp.result, Some(HELLO.to_string()));
}

#[tokio::test]
async fn test_one_params_request_with_singleton_array() {
    init_tracing();
    let params = vec![HELLO];
    let resp: JsonRpcResponse<String> = params
        .send_v2_request(TEST_URL, APP_JSON, "echoArray")
        .await
        .unwrap();
    assert_eq!(resp.jsonrpc, JsonRpcVersion::V2_0);
    assert_eq!(resp.result, Some(HELLO.to_string()));
}

#[tokio::test]
async fn test_one_params_request_with_option() {
    init_tracing();
    let resp: JsonRpcResponse<String> = Some(HELLO)
        .send_v2_request(TEST_URL, APP_JSON, "echoArray")
        .await
        .unwrap();
    assert_eq!(resp.jsonrpc, JsonRpcVersion::V2_0);
    assert_eq!(resp.result, Some(HELLO.to_string()));
}

#[tokio::test]
async fn test_one_params_request_with_tuple() {
    init_tracing();
    let resp: JsonRpcResponse<String> = (HELLO,)
        .send_v2_request(TEST_URL, APP_JSON, "echoArray")
        .await
        .unwrap();
    assert_eq!(resp.jsonrpc, JsonRpcVersion::V2_0);
    assert_eq!(resp.result, Some(HELLO.to_string()));
}

#[tokio::test]
async fn test_one_params_request_with_rpc_method() {
    init_tracing();
    #[rpc_method(
        url = "http://localhost:3000/",
        content_type = "application/json",
        method = "echoArray",
        version = "v2",
        mode = "array"
    )]
    async fn test_echo(msg: &str) -> Result<JsonRpcResponse<String>, RpcError> {}
    let resp = test_echo(HELLO).await.unwrap();
    assert_eq!(resp.jsonrpc, JsonRpcVersion::V2_0);
    assert_eq!(resp.result, Some(HELLO.to_string()))
}
```
