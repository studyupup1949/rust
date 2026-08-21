use a_rs_jsonrpc::{RpcError, response::JsonRpcError};
use tracing::Level;

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

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .try_init()
        .ok();

    a_rs_jsonrpc::init_rpc_service();

    let app = axum::Router::new().route(
        "/",
        axum::routing::post(|body: axum::body::Bytes| async move {
            match a_rs_jsonrpc::dispatch_rpc_request(&body).await {
                Ok(resp_body) => resp_body,
                Err(err) => response_error(&body, err),
            }
        }),
    );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("RPC Server running on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}

fn response_error(req_body: &axum::body::Bytes, err: RpcError) -> String {
    let Ok((id, version)) = serde_json::from_slice::<serde_json::Value>(req_body).map(|v| {
        (
            v.get("id").cloned().unwrap_or(serde_json::Value::Null),
            v.get("jsonrpc").cloned().unwrap_or(serde_json::Value::Null),
        )
    }) else {
        return serde_json::to_string(&serde_json::json!({
            "jsonrpc": null,
            "error": { "code": -32603, "message": err.to_string() },
            "id": null
        }))
        .unwrap_or_default();
    };
    serde_json::to_string(&serde_json::json!({
        "jsonrpc": version,
        "error": JsonRpcError::from(err),
        "id": id
    }))
    .unwrap_or_default()
}
