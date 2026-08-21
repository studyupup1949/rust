use crate::config::settings::Config;
use crate::swarm::coordinator::Coordinator;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

pub struct DashboardState {
    pub config: Arc<Config>,
    pub coordinator: Option<Arc<Coordinator>>,
}

pub fn router(state: Arc<DashboardState>) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .route("/ready", get(readiness))
        .route("/api/status", get(api_status))
        .route("/api/config", get(api_get_config).post(api_update_config))
        .route("/api/history", get(api_history))
        .route("/api/patterns", get(api_patterns))
        .route("/api/predictions", get(api_predictions))
        .route("/api/trigger/{agent}", post(api_trigger_agent))
        .route("/api/agents", get(api_agents))
        .layer(CorsLayer::permissive())
        .nest_service("/static", ServeDir::new("src/dashboard/static"))
        .with_state(state)
}

async fn index() -> Html<&'static str> {
    Html(include_str!("static/index.html"))
}

async fn api_status(State(state): State<Arc<DashboardState>>) -> Json<Value> {
    if let Some(ref coordinator) = state.coordinator {
        let statuses = coordinator.get_statuses().await;
        Json(json!({
            "running": coordinator.is_running(),
            "agents": statuses,
            "agent_count": coordinator.agent_count(),
        }))
    } else {
        Json(json!({
            "running": false,
            "agents": [],
            "agent_count": 0,
            "message": "Swarm not started. Run 'aas run' first."
        }))
    }
}

async fn api_get_config(State(state): State<Arc<DashboardState>>) -> Json<Value> {
    let config_json = serde_json::to_value(&*state.config).unwrap_or_default();
    Json(config_json)
}

async fn api_update_config(
    State(_state): State<Arc<DashboardState>>,
    axum::extract::Json(body): axum::extract::Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let new_config: Config = serde_json::from_value(body.clone())
        .map_err(|e| {
            tracing::warn!("Config parse error: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    let validation_errors = new_config.validate();
    if !validation_errors.is_empty() {
        tracing::warn!("Config validation failed: {:?}", validation_errors);
        return Ok(Json(json!({
            "status": "validation_error",
            "errors": validation_errors
        })));
    }

    new_config.save().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json!({"status": "saved"})))
}

async fn api_history(State(state): State<Arc<DashboardState>>) -> Json<Value> {
    if let Some(ref coordinator) = state.coordinator {
        let decisions = coordinator.context().memory.get_recent_issues(50).await;
        Json(json!(decisions))
    } else {
        Json(json!([]))
    }
}

async fn api_patterns(State(state): State<Arc<DashboardState>>) -> Json<Value> {
    if let Some(ref coordinator) = state.coordinator {
        let patterns = coordinator.context().memory.get_patterns(None).await;
        Json(json!(patterns))
    } else {
        Json(json!([]))
    }
}

async fn api_predictions(State(state): State<Arc<DashboardState>>) -> Json<Value> {
    if let Some(ref coordinator) = state.coordinator {
        let predictions = coordinator.context().memory.get_predictions(None, None).await;
        Json(json!(predictions))
    } else {
        Json(json!([]))
    }
}

async fn api_trigger_agent(
    State(state): State<Arc<DashboardState>>,
    axum::extract::Path(agent_name): axum::extract::Path<String>,
) -> Json<Value> {
    if let Some(ref coordinator) = state.coordinator {
        match coordinator.trigger_agent(&agent_name).await {
            Ok(msg) => Json(json!({"status": "triggered", "message": msg})),
            Err(e) => Json(json!({"status": "error", "message": e})),
        }
    } else {
        Json(json!({"status": "error", "message": "Swarm not running"}))
    }
}

async fn api_agents(State(state): State<Arc<DashboardState>>) -> Json<Value> {
    Json(json!({
        "available": ["repository", "logs", "metrics", "health", "task", "trace"],
        "enabled": state.config.get_enabled_agents(),
    }))
}

async fn health(State(state): State<Arc<DashboardState>>) -> (StatusCode, &'static str) {
    if let Some(ref coordinator) = state.coordinator {
        if coordinator.is_running() {
            (StatusCode::OK, "healthy")
        } else {
            (StatusCode::SERVICE_UNAVAILABLE, "not running")
        }
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "not initialized")
    }
}

async fn readiness(State(state): State<Arc<DashboardState>>) -> (StatusCode, &'static str) {
    if let Some(ref coordinator) = state.coordinator {
        if coordinator.is_running() && coordinator.agent_count() > 0 {
            (StatusCode::OK, "ready")
        } else {
            (StatusCode::SERVICE_UNAVAILABLE, "not ready")
        }
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "not ready")
    }
}
