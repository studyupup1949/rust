use a_rs_jsonrpc::RpcError;
use serde::Serialize;
use tracing::Level;

#[a_rs_jsonrpc::jsonrpc_service_fn_array(method = "ping", version = "v2")]
async fn ping() -> Result<String, RpcError> {
    tracing::debug!("got client ping request");
    Ok("pong".to_string())
}

#[derive(Debug, Serialize)]
pub struct TodoItem {
    id: u32,
    title: String,
    status: String,
}

#[a_rs_jsonrpc::jsonrpc_service_fn_obj(method = "todoList", version = "v2")]
async fn todo_list() -> Result<Vec<TodoItem>, RpcError> {
    tracing::debug!("got client todoList request");
    Ok(vec![
        TodoItem {
            id: 1,
            title: "Leaning Rust".to_string(),
            status: "pending".to_string(),
        },
        TodoItem {
            id: 2,
            title: "Meeting with devs team".to_string(),
            status: "completed".to_string(),
        },
    ])
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
