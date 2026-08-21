use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::sse::{Event, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use futures_util::stream;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::net::TcpListener;
use uuid::Uuid;

use crate::{
    ExecutionState, FlowEngine, FlowError, FlowEvent, FlowService, NodeDescriptor, NodeFactory,
    NodeFieldDescriptor, ValidationIssue,
};

#[derive(Clone)]
struct AppState {
    service: Arc<FlowService>,
}

pub fn build_router(engine: Arc<FlowEngine>) -> Router {
    build_router_with_service(Arc::new(FlowService::new(engine)))
}

pub fn build_router_with_factories(
    engine: Arc<FlowEngine>,
    node_factories: HashMap<String, NodeFactory>,
) -> Router {
    build_router_with_service(Arc::new(FlowService::with_factories(
        engine,
        node_factories,
    )))
}

pub fn build_router_with_service(service: Arc<FlowService>) -> Router {
    Router::new()
        .route("/api/flow/info", get(info))
        .route("/api/flow/node-types", get(node_types))
        .route("/api/flow/node-types", post(register_node_type))
        .route("/api/flow/nodes", get(nodes))
        .route("/api/flow/validate", post(validate))
        .route("/api/flow/executions", post(start_execution))
        .route("/api/flow/execution/:id", get(get_execution))
        .route("/api/flow/events/:id", get(get_events))
        .route("/api/flow/pause/:id", post(pause_execution))
        .route("/api/flow/resume/:id", post(resume_execution))
        .route("/api/flow/terminate/:id", post(terminate_execution))
        .route("/api/flow/context/:id", get(get_context))
        .route(
            "/api/flow/context/:id/:key",
            put(set_context_entry).delete(delete_context_entry),
        )
        .route("/api/flow/run/:name", post(run_named_flow))
        .route("/api/flow/node-type/:node_type", delete(delete_node_type))
        .with_state(AppState { service })
}

pub async fn serve(addr: SocketAddr, engine: Arc<FlowEngine>) -> std::io::Result<()> {
    serve_with_service(addr, Arc::new(FlowService::new(engine))).await
}

pub async fn serve_with_factories(
    addr: SocketAddr,
    engine: Arc<FlowEngine>,
    node_factories: HashMap<String, NodeFactory>,
) -> std::io::Result<()> {
    serve_with_service(
        addr,
        Arc::new(FlowService::with_factories(engine, node_factories)),
    )
    .await
}

pub async fn serve_with_service(
    addr: SocketAddr,
    service: Arc<FlowService>,
) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    serve_listener(listener, service).await
}

pub async fn serve_listener(
    listener: TcpListener,
    service: Arc<FlowService>,
) -> std::io::Result<()> {
    axum::serve(listener, build_router_with_service(service))
        .await
        .map_err(std::io::Error::other)
}

#[derive(Debug, Serialize)]
struct InfoResponse {
    engine: &'static str,
    version: String,
    progressive_disclosure: bool,
    summary: String,
    node_count: usize,
    capabilities: crate::FlowCapabilities,
}

async fn info(State(state): State<AppState>) -> Json<InfoResponse> {
    let capabilities = state.service.capabilities();
    Json(InfoResponse {
        engine: "a3s-flow",
        version: capabilities.version.clone(),
        progressive_disclosure: capabilities.progressive_disclosure,
        summary: capabilities.summary.clone(),
        node_count: capabilities.nodes.len(),
        capabilities,
    })
}

#[derive(Debug, Serialize)]
struct NodeTypesResponse {
    node_types: Vec<String>,
}

async fn node_types(State(state): State<AppState>) -> Json<NodeTypesResponse> {
    Json(NodeTypesResponse {
        node_types: state.service.node_types(),
    })
}

#[derive(Debug, Serialize)]
struct NodesResponse {
    nodes: Vec<crate::NodeDescriptor>,
}

async fn nodes(State(state): State<AppState>) -> Json<NodesResponse> {
    Json(NodesResponse {
        nodes: state.service.node_descriptors(),
    })
}

#[derive(Debug, Deserialize)]
struct ValidateRequest {
    definition: Value,
}

#[derive(Debug, Serialize)]
struct ValidateResponse {
    valid: bool,
    issues: Vec<ValidationIssue>,
}

async fn validate(
    State(state): State<AppState>,
    Json(req): Json<ValidateRequest>,
) -> Json<ValidateResponse> {
    let issues = state.service.validate(&req.definition);
    Json(ValidateResponse {
        valid: issues.is_empty(),
        issues,
    })
}

#[derive(Debug, Deserialize)]
struct StartExecutionRequest {
    definition: Value,
    #[serde(default)]
    variables: HashMap<String, Value>,
}

#[derive(Debug, Serialize)]
struct ExecutionSnapshot {
    execution_id: Uuid,
    state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<crate::FlowResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
}

fn execution_snapshot(execution_id: Uuid, state: ExecutionState) -> ExecutionSnapshot {
    match state {
        ExecutionState::Running => ExecutionSnapshot {
            execution_id,
            state: "running".to_string(),
            result: None,
            reason: None,
        },
        ExecutionState::Paused => ExecutionSnapshot {
            execution_id,
            state: "paused".to_string(),
            result: None,
            reason: None,
        },
        ExecutionState::Completed(result) => ExecutionSnapshot {
            execution_id,
            state: "completed".to_string(),
            result: Some(result),
            reason: None,
        },
        ExecutionState::Failed(reason) => ExecutionSnapshot {
            execution_id,
            state: "failed".to_string(),
            result: None,
            reason: Some(reason),
        },
        ExecutionState::Terminated => ExecutionSnapshot {
            execution_id,
            state: "terminated".to_string(),
            result: None,
            reason: None,
        },
    }
}

async fn start_execution(
    State(state): State<AppState>,
    Json(req): Json<StartExecutionRequest>,
) -> Result<(StatusCode, Json<ExecutionSnapshot>), HttpError> {
    let execution_id = state
        .service
        .start_execution(&req.definition, req.variables)
        .await?;
    Ok((
        StatusCode::ACCEPTED,
        Json(ExecutionSnapshot {
            execution_id,
            state: "running".to_string(),
            result: None,
            reason: None,
        }),
    ))
}

async fn get_execution(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ExecutionSnapshot>, HttpError> {
    Ok(Json(execution_snapshot(
        id,
        state.service.get_execution(id).await?,
    )))
}

#[derive(Debug, Serialize)]
struct DeleteNodeTypeResponse {
    node_type: String,
    removed: bool,
}

async fn delete_node_type(
    State(state): State<AppState>,
    Path(node_type): Path<String>,
) -> Result<Json<DeleteNodeTypeResponse>, HttpError> {
    match state.service.unregister_node_type(&node_type) {
        Ok(true) => Ok(Json(DeleteNodeTypeResponse {
            node_type,
            removed: true,
        })),
        Ok(false) => Err(HttpError::node_type_not_found(node_type)),
        Err(err) => Err(HttpError::from(err)),
    }
}

#[derive(Debug, Deserialize)]
struct RegisterNodeTypeRequest {
    factory: String,
    #[serde(default)]
    descriptor: Option<NodeDescriptorInput>,
}

#[derive(Debug, Deserialize)]
struct NodeDescriptorInput {
    display_name: String,
    category: String,
    summary: String,
    #[serde(default)]
    default_data: Value,
    #[serde(default)]
    fields: Vec<NodeFieldDescriptor>,
}

#[derive(Debug, Serialize)]
struct RegisterNodeTypeResponse {
    node_type: String,
    registered: bool,
    replaced: bool,
}

async fn register_node_type(
    State(state): State<AppState>,
    Json(req): Json<RegisterNodeTypeRequest>,
) -> Result<(StatusCode, Json<RegisterNodeTypeResponse>), HttpError> {
    let descriptor = req.descriptor.map(|descriptor| NodeDescriptor {
        node_type: String::new(),
        display_name: descriptor.display_name,
        category: descriptor.category,
        summary: descriptor.summary,
        default_data: descriptor.default_data,
        fields: descriptor.fields,
    });
    let (node_type, replaced) = state.service.register_node_type(&req.factory, descriptor)?;

    Ok((
        StatusCode::CREATED,
        Json(RegisterNodeTypeResponse {
            node_type,
            registered: true,
            replaced,
        }),
    ))
}

async fn get_events(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>>, HttpError> {
    let rx = state.service.subscribe(id).await?;
    let stream = stream::unfold(rx, |mut rx| async move {
        loop {
            match rx.recv().await {
                Ok(flow_event) => {
                    let event = Event::default()
                        .event(flow_event_name(&flow_event))
                        .json_data(flow_event)
                        .expect("serialize flow event");
                    return Some((Ok(event), rx));
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(tokio::sync::broadcast::error::RecvError::Closed) => return None,
            }
        }
    });
    Ok(Sse::new(stream))
}

fn flow_event_name(event: &FlowEvent) -> &'static str {
    match event {
        FlowEvent::FlowStarted { .. } => "flow.started",
        FlowEvent::FlowCompleted { .. } => "flow.completed",
        FlowEvent::FlowFailed { .. } => "flow.failed",
        FlowEvent::FlowTerminated { .. } => "flow.terminated",
        FlowEvent::NodeStarted { .. } => "node.started",
        FlowEvent::NodeCompleted { .. } => "node.completed",
        FlowEvent::NodeSkipped { .. } => "node.skipped",
        FlowEvent::NodeFailed { .. } => "node.failed",
        FlowEvent::NodeCompletedFull { .. } => "node.completed_full",
        FlowEvent::IterationStarted { .. } => "iteration.started",
        FlowEvent::IterationNext { .. } => "iteration.next",
        FlowEvent::IterationCompleted { .. } => "iteration.completed",
        FlowEvent::LoopStarted { .. } => "loop.started",
        FlowEvent::LoopCompleted { .. } => "loop.completed",
        FlowEvent::ParallelBranchStarted { .. } => "parallel_branch.started",
        FlowEvent::ParallelBranchCompleted { .. } => "parallel_branch.completed",
        FlowEvent::NodeRetry { .. } => "node.retry",
    }
}

async fn pause_execution(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ExecutionSnapshot>, HttpError> {
    Ok(Json(execution_snapshot(
        id,
        state.service.pause_execution(id).await?,
    )))
}

async fn resume_execution(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ExecutionSnapshot>, HttpError> {
    Ok(Json(execution_snapshot(
        id,
        state.service.resume_execution(id).await?,
    )))
}

async fn terminate_execution(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<ExecutionSnapshot>), HttpError> {
    state.service.terminate_execution(id).await?;
    Ok((
        StatusCode::ACCEPTED,
        Json(ExecutionSnapshot {
            execution_id: id,
            state: "terminating".to_string(),
            result: None,
            reason: None,
        }),
    ))
}

#[derive(Debug, Deserialize)]
struct RunNamedFlowRequest {
    #[serde(default)]
    variables: HashMap<String, Value>,
}

#[derive(Debug, Serialize)]
struct RunNamedFlowResponse {
    execution_id: Uuid,
    state: String,
    flow_name: String,
}

async fn run_named_flow(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(req): Json<RunNamedFlowRequest>,
) -> Result<(StatusCode, Json<RunNamedFlowResponse>), HttpError> {
    let execution_id = state.service.run_named_flow(&name, req.variables).await?;
    Ok((
        StatusCode::ACCEPTED,
        Json(RunNamedFlowResponse {
            execution_id,
            state: "running".to_string(),
            flow_name: name,
        }),
    ))
}

#[derive(Debug, Serialize)]
struct ContextResponse {
    context: HashMap<String, Value>,
}

async fn get_context(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ContextResponse>, HttpError> {
    Ok(Json(ContextResponse {
        context: state.service.get_context(id).await?,
    }))
}

#[derive(Debug, Deserialize)]
struct SetContextRequest {
    value: Value,
}

#[derive(Debug, Serialize)]
struct ContextMutationResponse {
    key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    updated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    removed: Option<bool>,
}

async fn set_context_entry(
    State(state): State<AppState>,
    Path((id, key)): Path<(Uuid, String)>,
    Json(req): Json<SetContextRequest>,
) -> Result<Json<ContextMutationResponse>, HttpError> {
    state
        .service
        .set_context_entry(id, key.clone(), req.value)
        .await?;
    Ok(Json(ContextMutationResponse {
        key,
        updated: Some(true),
        removed: None,
    }))
}

async fn delete_context_entry(
    State(state): State<AppState>,
    Path((id, key)): Path<(Uuid, String)>,
) -> Result<Json<ContextMutationResponse>, HttpError> {
    let removed = state.service.delete_context_entry(id, &key).await?;
    Ok(Json(ContextMutationResponse {
        key,
        updated: None,
        removed: Some(removed),
    }))
}

#[derive(Debug)]
struct HttpError {
    status: StatusCode,
    code: String,
    message: String,
}

impl HttpError {
    fn node_type_not_found(node_type: String) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: "node_type_not_found".to_string(),
            message: format!("node type not found: {node_type}"),
        }
    }
}

impl From<FlowError> for HttpError {
    fn from(value: FlowError) -> Self {
        match value {
            FlowError::InvalidDefinition(message) => Self {
                status: StatusCode::BAD_REQUEST,
                code: "invalid_definition".to_string(),
                message,
            },
            FlowError::UnknownNode(message) => Self {
                status: StatusCode::BAD_REQUEST,
                code: "unknown_node".to_string(),
                message,
            },
            FlowError::ExecutionNotFound(id) => Self {
                status: StatusCode::NOT_FOUND,
                code: "execution_not_found".to_string(),
                message: format!("execution not found: {id}"),
            },
            FlowError::FlowNotFound(name) => Self {
                status: StatusCode::NOT_FOUND,
                code: "flow_not_found".to_string(),
                message: format!("flow not found: {name}"),
            },
            FlowError::InvalidTransition { action, from } => Self {
                status: StatusCode::CONFLICT,
                code: "invalid_transition".to_string(),
                message: format!("cannot {action} a {from} execution"),
            },
            FlowError::ProtectedNodeType(node_type) => Self {
                status: StatusCode::CONFLICT,
                code: "protected_node_type".to_string(),
                message: format!("node type is protected and cannot be removed: {node_type}"),
            },
            FlowError::Terminated => Self {
                status: StatusCode::CONFLICT,
                code: "terminated".to_string(),
                message: "execution was terminated".to_string(),
            },
            FlowError::Json(err) => Self {
                status: StatusCode::BAD_REQUEST,
                code: "json".to_string(),
                message: err.to_string(),
            },
            FlowError::NodeFailed { reason, .. } => Self {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                code: "node_failed".to_string(),
                message: reason,
            },
            FlowError::CyclicGraph => Self {
                status: StatusCode::BAD_REQUEST,
                code: "cyclic_graph".to_string(),
                message: "flow graph contains a cycle".to_string(),
            },
            FlowError::Internal(message) => Self {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                code: "internal".to_string(),
                message,
            },
        }
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        let body = serde_json::json!({
            "error": {
                "code": self.code,
                "message": self.message,
            }
        });
        (self.status, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FlowStore;
    use crate::Node;
    use async_trait::async_trait;
    use axum::body::Body;
    use axum::http::{Method, Request, StatusCode};
    use serde_json::json;
    use std::time::Duration;
    use tower::util::ServiceExt;

    fn app() -> Router {
        build_router(Arc::new(FlowEngine::new(
            crate::NodeRegistry::with_defaults(),
        )))
    }

    struct SlowNode;

    #[async_trait]
    impl Node for SlowNode {
        fn node_type(&self) -> &str {
            "slow"
        }

        async fn execute(&self, _ctx: crate::ExecContext) -> crate::Result<Value> {
            tokio::time::sleep(Duration::from_millis(5)).await;
            Ok(json!({ "ok": true }))
        }
    }

    async fn app_with_factory_and_store() -> Router {
        let flow_store = Arc::new(crate::MemoryFlowStore::new());
        flow_store
            .save(
                "hello",
                &json!({
                    "nodes": [{ "id": "a", "type": "noop" }],
                    "edges": []
                }),
            )
            .await
            .unwrap();
        let engine = Arc::new(
            FlowEngine::new(crate::NodeRegistry::with_defaults())
                .with_flow_store(flow_store as Arc<dyn crate::FlowStore>),
        );
        let mut factories: HashMap<String, NodeFactory> = HashMap::new();
        factories.insert(
            "slow-test-node".to_string(),
            Arc::new(|| Arc::new(SlowNode)),
        );
        build_router_with_factories(engine, factories)
    }

    async fn json_body(resp: Response) -> Value {
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn info_returns_ok() {
        let resp = app()
            .oneshot(
                Request::builder()
                    .uri("/api/flow/info")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = json_body(resp).await;
        assert_eq!(body["engine"], "a3s-flow");
        assert!(body["capabilities"].is_object());
    }

    #[tokio::test]
    async fn validate_returns_issues() {
        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/flow/validate")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&json!({
                    "definition": {
                        "nodes": [{ "id": "a", "type": "missing" }],
                        "edges": []
                    }
                }))
                .unwrap(),
            ))
            .unwrap();

        let resp = app().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = json_body(resp).await;
        assert_eq!(body["valid"], false);
        assert_eq!(body["issues"].as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn start_and_get_execution_work() {
        let app = app();
        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/flow/executions")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&json!({
                    "definition": {
                        "nodes": [{ "id": "a", "type": "noop" }],
                        "edges": []
                    },
                    "variables": {}
                }))
                .unwrap(),
            ))
            .unwrap();

        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::ACCEPTED);
        let body = json_body(resp).await;
        let id = body["execution_id"].as_str().unwrap();

        let resp = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/flow/execution/{id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn delete_builtin_node_type_returns_conflict() {
        let resp = app()
            .oneshot(
                Request::builder()
                    .method(Method::DELETE)
                    .uri("/api/flow/node-type/noop")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CONFLICT);
        let body = json_body(resp).await;
        assert_eq!(body["error"]["code"], "protected_node_type");
    }

    #[tokio::test]
    async fn delete_missing_node_type_returns_not_found() {
        let resp = app()
            .oneshot(
                Request::builder()
                    .method(Method::DELETE)
                    .uri("/api/flow/node-type/does-not-exist")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        let body = json_body(resp).await;
        assert_eq!(body["error"]["code"], "node_type_not_found");
    }

    #[tokio::test]
    async fn pause_resume_and_context_routes_work() {
        let app = app();
        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/flow/executions")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&json!({
                    "definition": {
                        "nodes": [
                            { "id": "a", "type": "noop" },
                            { "id": "b", "type": "noop" }
                        ],
                        "edges": [{ "source": "a", "target": "b" }]
                    }
                }))
                .unwrap(),
            ))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let body = json_body(resp).await;
        let id = body["execution_id"].as_str().unwrap().to_string();

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(format!("/api/flow/pause/{id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::PUT)
                    .uri(format!("/api/flow/context/{id}/approval"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::to_vec(&json!({ "value": "granted" })).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/flow/context/{id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = json_body(resp).await;
        assert_eq!(body["context"]["approval"], "granted");

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(format!("/api/flow/resume/{id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn terminate_and_delete_context_routes_work() {
        let app = app();
        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/flow/executions")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&json!({
                    "definition": {
                        "nodes": [{ "id": "a", "type": "noop" }],
                        "edges": []
                    }
                }))
                .unwrap(),
            ))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let body = json_body(resp).await;
        let id = body["execution_id"].as_str().unwrap().to_string();

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::PUT)
                    .uri(format!("/api/flow/context/{id}/approval"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::to_vec(&json!({ "value": "granted" })).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::DELETE)
                    .uri(format!("/api/flow/context/{id}/approval"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = json_body(resp).await;
        assert_eq!(body["removed"], true);

        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(format!("/api/flow/terminate/{id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::ACCEPTED);
    }

    #[tokio::test]
    async fn events_route_returns_sse_stream() {
        let app = app();
        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/flow/executions")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&json!({
                    "definition": {
                        "nodes": [{ "id": "a", "type": "noop" }],
                        "edges": []
                    }
                }))
                .unwrap(),
            ))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let body = json_body(resp).await;
        let id = body["execution_id"].as_str().unwrap().to_string();

        let resp = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/flow/events/{id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap(),
            "text/event-stream"
        );
    }

    #[tokio::test]
    async fn register_node_type_route_works() {
        let resp = app_with_factory_and_store()
            .await
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/flow/node-types")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::to_vec(&json!({
                            "factory": "slow-test-node",
                            "descriptor": {
                                "display_name": "Slow Node",
                                "category": "testing",
                                "summary": "Test-only slow node.",
                                "default_data": { "delay_ms": 5 },
                                "fields": []
                            }
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let body = json_body(resp).await;
        assert_eq!(body["node_type"], "slow");
        assert_eq!(body["registered"], true);
    }

    #[tokio::test]
    async fn run_named_flow_route_works() {
        let resp = app_with_factory_and_store()
            .await
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/flow/run/hello")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::to_vec(&json!({ "variables": {} })).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::ACCEPTED);
        let body = json_body(resp).await;
        assert_eq!(body["flow_name"], "hello");
        assert_eq!(body["state"], "running");
    }

    #[tokio::test]
    async fn nodes_route_returns_descriptors() {
        let resp = app()
            .oneshot(
                Request::builder()
                    .uri("/api/flow/nodes")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = json_body(resp).await;
        assert!(body["nodes"].is_array());
        assert!(!body["nodes"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn get_execution_unknown_returns_not_found() {
        let resp = app()
            .oneshot(
                Request::builder()
                    .uri("/api/flow/execution/00000000-0000-0000-0000-000000000000")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        let body = json_body(resp).await;
        assert_eq!(body["error"]["code"], "execution_not_found");
    }

    #[tokio::test]
    async fn run_named_flow_without_store_returns_internal_error() {
        // Without a FlowStore, run_named_flow should return an internal error
        let resp = app()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/flow/run/nonexistent")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::to_vec(&json!({ "variables": {} })).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = json_body(resp).await;
        assert_eq!(body["error"]["code"], "internal");
    }

    #[tokio::test]
    async fn start_execution_with_missing_nodes_field_returns_bad_request() {
        // Empty nodes array should fail validation
        let resp = app()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/flow/executions")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::to_vec(&json!({
                            "definition": { "nodes": [], "edges": [] }
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn pause_completed_execution_returns_conflict() {
        // Start an execution that will complete quickly
        let app = app();
        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/flow/executions")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&json!({
                    "definition": { "nodes": [{ "id": "a", "type": "noop" }], "edges": [] }
                }))
                .unwrap(),
            ))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let body = json_body(resp).await;
        let id = body["execution_id"].as_str().unwrap().to_string();

        // Wait for the execution to reach a terminal state
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Now try to pause - execution is completed so should return conflict
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(format!("/api/flow/pause/{id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn resume_running_execution_returns_conflict() {
        // Start a long-running execution that won't complete immediately
        struct NeverDoneNode;
        #[async_trait]
        impl Node for NeverDoneNode {
            fn node_type(&self) -> &str {
                "never-done"
            }
            async fn execute(&self, _ctx: crate::ExecContext) -> crate::Result<Value> {
                tokio::time::sleep(Duration::from_secs(3600)).await;
                Ok(json!({}))
            }
        }
        let factory = app_with_factory_and_store().await;
        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/flow/executions")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&json!({
                    "definition": { "nodes": [{ "id": "a", "type": "never-done" }], "edges": [] }
                }))
                .unwrap(),
            ))
            .unwrap();
        let resp = factory.clone().oneshot(req).await.unwrap();
        let body = json_body(resp).await;
        let id = body["execution_id"].as_str().unwrap().to_string();

        // Resume a running execution should fail with conflict
        let resp = factory
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(format!("/api/flow/resume/{id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn terminate_completed_execution_returns_conflict() {
        // Start an execution that completes quickly
        let app = app();
        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/flow/executions")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&json!({
                    "definition": { "nodes": [{ "id": "a", "type": "noop" }], "edges": [] }
                }))
                .unwrap(),
            ))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let body = json_body(resp).await;
        let id = body["execution_id"].as_str().unwrap().to_string();

        // Wait for completion
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Try to terminate a completed execution - should return conflict
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(format!("/api/flow/terminate/{id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn context_get_unknown_execution_returns_not_found() {
        let resp = app()
            .oneshot(
                Request::builder()
                    .uri("/api/flow/context/00000000-0000-0000-0000-000000000000")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}
