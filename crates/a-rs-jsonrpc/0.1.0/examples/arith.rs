use a_rs_jsonrpc::RpcError;
use tracing::Level;

#[a_rs_jsonrpc::jsonrpc_service_fn_array(method = "addArray", version = "v1")]
async fn add_array(a: i64, b: i64) -> Result<i64, RpcError> {
    tracing::debug!("got client request to add: {} + {}", a, b);
    a.checked_add(b)
        .ok_or(RpcError::CustomError("add overflow".to_string()))
}

#[a_rs_jsonrpc::jsonrpc_service_fn_obj(method = "addObj", version = "v1")]
async fn add_obj(lhs: i64, rhs: i64) -> Result<i64, RpcError> {
    tracing::debug!("got client request to add: {} + {}", lhs, rhs);
    lhs.checked_add(rhs)
        .ok_or(RpcError::CustomError("add overflow".to_string()))
}

#[a_rs_jsonrpc::jsonrpc_service_fn_array(method = "substractArray", version = "v1")]
async fn substract_array(a: i64, b: i64) -> Result<i64, RpcError> {
    tracing::debug!("got client request to substract: {} - {}", a, b);
    a.checked_sub(b)
        .ok_or(RpcError::CustomError("substract overflow".to_string()))
}

#[a_rs_jsonrpc::jsonrpc_service_fn_obj(method = "substractObj", version = "v1")]
async fn substract_obj(lhs: i64, rhs: i64) -> Result<i64, RpcError> {
    tracing::debug!("got client request to substract: {} - {}", lhs, rhs);
    lhs.checked_sub(rhs)
        .ok_or(RpcError::CustomError("substract overflow".to_string()))
}

#[a_rs_jsonrpc::jsonrpc_service_fn_array(method = "multiplyArray", version = "v1")]
async fn multiply_array(a: i64, b: i64) -> Result<i64, RpcError> {
    tracing::debug!("got client request to multiply: {} * {}", a, b);
    a.checked_mul(b)
        .ok_or(RpcError::CustomError("multiply overflow".to_string()))
}

#[a_rs_jsonrpc::jsonrpc_service_fn_obj(method = "multiplyObj", version = "v1")]
async fn multiply_obj(lhs: i64, rhs: i64) -> Result<i64, RpcError> {
    tracing::debug!("got client request to multiply: {} * {}", lhs, rhs);
    lhs.checked_mul(rhs)
        .ok_or(RpcError::CustomError("multiply overflow".to_string()))
}

#[a_rs_jsonrpc::jsonrpc_service_fn_array(method = "divideArray", version = "v1")]
async fn divide_array(a: i64, b: i64) -> Result<i64, RpcError> {
    tracing::debug!("got client request to divide: {} / {}", a, b);
    if b == 0 {
        return Err(RpcError::CustomError("divided by zero".to_string()));
    }
    a.checked_div(b)
        .ok_or(RpcError::CustomError("divide overflow".to_string()))
}

#[a_rs_jsonrpc::jsonrpc_service_fn_obj(method = "divideObj", version = "v1")]
async fn divide_obj(lhs: i64, rhs: i64) -> Result<i64, RpcError> {
    tracing::debug!("got client request to divide: {} / {}", lhs, rhs);
    if rhs == 0 {
        return Err(RpcError::CustomError("divided by zero".to_string()));
    }
    lhs.checked_div(rhs)
        .ok_or(RpcError::CustomError("divide overflow".to_string()))
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

fn response_error(req_body: &axum::body::Bytes, err: impl std::fmt::Display) -> String {
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
        "error": { "code": -32603, "message": err.to_string() },
        "id": id
    }))
    .unwrap_or_default()
}
