use a_rs_jsonrpc::{JsonRpcClient, JsonRpcResponse, RpcError, request::JsonRpcVersion};
use a_rs_jsonrpc_macros::rpc_method;
use serde::Serialize;

const TEST_URL: &str = "http://localhost:3000/";
const APP_JSON: &str = "application/json";

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();
}

// runing all zero params tests with `cargo run --example unit`

/// Client tests for no parameter RPC to the `ping` service.
/// for no parameter, you can use unit, empty slice, None or rpc_method to send request.
#[tokio::test]
async fn test_zero_params_request_with_unit() {
    init_tracing();
    let resp: JsonRpcResponse<String> =
        ().send_v2_request(TEST_URL, APP_JSON, "ping")
            .await
            .unwrap();
    assert_eq!(resp.jsonrpc, JsonRpcVersion::V2_0);
    assert_eq!(resp.result, Some("pong".to_string()));
}

#[tokio::test]
async fn test_zero_params_request_with_empty_slice() {
    init_tracing();
    let empty: Vec<()> = vec![];
    let resp: JsonRpcResponse<String> = empty
        .send_v2_request(TEST_URL, APP_JSON, "ping")
        .await
        .unwrap();
    assert_eq!(resp.jsonrpc, JsonRpcVersion::V2_0);
    assert_eq!(resp.result, Some("pong".to_string()));
}

#[tokio::test]
async fn test_zero_params_request_with_none() {
    init_tracing();
    let none: Option<Vec<()>> = None;
    let resp: JsonRpcResponse<String> = none
        .send_v2_request(TEST_URL, APP_JSON, "ping")
        .await
        .unwrap();
    assert_eq!(resp.jsonrpc, JsonRpcVersion::V2_0);
    assert_eq!(resp.result, Some("pong".to_string()));
}

#[tokio::test]
async fn test_zero_params_request_with_rpc_method() {
    init_tracing();
    #[rpc_method(
        url = "http://localhost:3000/",
        content_type = "application/json",
        method = "ping",
        version = "v2",
        mode = "array"
    )]
    async fn reload() -> Result<JsonRpcResponse<String>, RpcError> {}
    let resp = reload().await.unwrap();
    assert_eq!(resp.jsonrpc, JsonRpcVersion::V2_0);
    assert_eq!(resp.result, Some("pong".to_string()));
}

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

// running all two params tests with `cargo run --example arith`

/// Client tests for two parameter RPC requests to the arith service.
/// for two parameters, you can use tuple, array, struct, or rpc_method
/// mode to send the request.
#[tokio::test]
async fn test_two_params_request_with_tuple() {
    init_tracing();
    let resp: JsonRpcResponse<i32> = (10, 20)
        .send_v1_request(TEST_URL, APP_JSON, "addArray")
        .await
        .unwrap();
    assert_eq!(resp.jsonrpc, JsonRpcVersion::V1_0);
    assert_eq!(resp.result, Some(30));
}

#[tokio::test]
async fn test_two_params_request_with_array() {
    init_tracing();
    let params = vec![10, 20];
    let resp: JsonRpcResponse<i32> = params
        .send_v1_request(TEST_URL, APP_JSON, "addArray")
        .await
        .unwrap();
    assert_eq!(resp.jsonrpc, JsonRpcVersion::V1_0);
    assert_eq!(resp.result, Some(30));
}

#[tokio::test]
async fn test_two_params_request_with_struct_array() {
    init_tracing();
    #[derive(Clone, Serialize, JsonRpcClient)]
    #[jsonrpc(
        url = "http://localhost:3000/",
        content_type = "application/json",
        method = "addArray"
    )]
    struct AddParams {
        a: i32,
        b: i32,
    }
    let params = AddParams { a: 10, b: 20 };
    use a_rs_jsonrpc::JsonRpcClientCall;
    let resp: JsonRpcResponse<i32> = params.call_rpc_v1().await.unwrap();
    assert_eq!(resp.jsonrpc, JsonRpcVersion::V1_0);
    assert_eq!(resp.result, Some(30));
}

/// construct a two parameter request send with array params with a struct object.
/// then you can easily use the generated `call_rpc_v1` method to send the request.
/// be sure to import `a_rs_jsonrpc::JsonRpcClientCall` trait into namespace.
#[tokio::test]
async fn test_two_params_request_with_rpc_method_array() {
    init_tracing();
    #[rpc_method(
        url = "http://localhost:3000/",
        content_type = "application/json",
        method = "addArray",
        version = "v1",
        mode = "array"
    )]
    async fn add(a: i32, b: i32) -> Result<JsonRpcResponse<i32>, RpcError> {}
    let resp = add(10, 20).await.unwrap();
    assert_eq!(resp.jsonrpc, JsonRpcVersion::V1_0);
    assert_eq!(resp.result, Some(30));
}

/// construct a two parameter request send with object params with a struct object.
/// then you can easily use the generated `call_rpc_v1_obj` method to send the request.
/// be sure to import `a_rs_jsonrpc::JsonRpcClientCall` trait into namespace.
#[tokio::test]
async fn test_two_params_request_with_struct_obj() {
    init_tracing();
    #[derive(Clone, Serialize, JsonRpcClient)]
    #[jsonrpc(
        url = "http://localhost:3000/",
        content_type = "application/json",
        method = "addObj"
    )]
    struct AddParams {
        lhs: i32,
        rhs: i32,
    }
    let params = AddParams { lhs: 10, rhs: 20 };
    use a_rs_jsonrpc::JsonRpcClientCall;
    let resp: JsonRpcResponse<i32> = params.call_rpc_v1_obj().await.unwrap();
    assert_eq!(resp.jsonrpc, JsonRpcVersion::V1_0);
    assert_eq!(resp.result, Some(30));
}

#[tokio::test]
async fn test_two_params_request_with_rpc_method_obj() {
    init_tracing();
    #[rpc_method(
        url = "http://localhost:3000/",
        content_type = "application/json",
        method = "addObj",
        version = "v1",
        mode = "obj"
    )]
    async fn add(lhs: i32, rhs: i32) -> Result<JsonRpcResponse<i32>, RpcError> {}
    let resp = add(10, 20).await.unwrap();
    assert_eq!(resp.jsonrpc, JsonRpcVersion::V1_0);
    assert_eq!(resp.result, Some(30));
}
