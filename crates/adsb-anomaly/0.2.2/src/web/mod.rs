// ABOUTME: Web server module using Axum for HTTP API and dashboard
// ABOUTME: Provides health endpoint, REST API, dashboard pages, and WebSocket alerts

pub mod dashboard; // Re-enabled for enhanced dashboard
pub mod dashboard_simple;
pub mod simple_dashboard;
pub mod websocket;

use anyhow::Result;
use axum::extract::ws::{Message, WebSocket};
use axum::{
    extract::{Path, Query, State, WebSocketUpgrade},
    http::StatusCode,
    response::{Json, Response},
    routing::get,
    Router,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};

use crate::alerts::AlertManager;
use crate::config::{AnalysisConfig, WebConfig};
use crate::metrics::AppMetrics;
use crate::model::AnomalyType;
use crate::store::{observations, sessions};
use crate::web::websocket::WebSocketManager;

#[derive(Debug, Deserialize)]
struct SessionsQuery {
    limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct ObservationsQuery {
    limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct AlertsQuery {
    #[serde(rename = "type")]
    anomaly_type: Option<String>,
    since_ms: Option<i64>,
    limit: Option<i64>,
}

pub async fn serve(
    pool: SqlitePool,
    web_cfg: WebConfig,
    analysis_cfg: AnalysisConfig,
) -> Result<()> {
    let alert_manager = AlertManager::new(pool.clone());
    let app = create_app(pool, alert_manager, analysis_cfg, web_cfg.clone());

    let addr = format!("0.0.0.0:{}", web_cfg.port);
    info!("Starting web server on {}", addr);

    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Shared application state
#[derive(Clone)]
pub(crate) struct AppState {
    pub pool: SqlitePool,
    pub alert_manager: Arc<AlertManager>,
    pub ws_manager: Arc<WebSocketManager>,
    pub analysis_config: AnalysisConfig,
    pub web_config: WebConfig,
}

fn create_app(
    pool: SqlitePool,
    alert_manager: AlertManager,
    analysis_config: AnalysisConfig,
    web_config: WebConfig,
) -> Router {
    let ws_manager = Arc::new(WebSocketManager::new());
    let state = AppState {
        pool: pool.clone(),
        alert_manager: Arc::new(alert_manager),
        ws_manager: ws_manager.clone(),
        analysis_config,
        web_config,
    };

    Router::new()
        .route("/healthz", get(health_handler))
        .route("/metrics", get(metrics_handler))
        // Dashboard pages (HTML) - use full dashboard with enhanced features
        .merge(dashboard::router().with_state(state.clone()))
        .merge(simple_dashboard::router().with_state(state.pool.clone()))
        // WebSocket alerts
        .route("/ws/alerts", get(websocket_alerts_handler))
        // WebSocket aircraft data
        .route("/ws/aircraft", get(websocket_aircraft_handler))
        // WebSocket aircraft detail data
        .route("/ws/aircraft/:hex", get(websocket_aircraft_detail_handler))
        // API endpoints (JSON)
        .route("/api/sessions", get(sessions_handler))
        .route("/api/aircraft/:hex/observations", get(observations_handler))
        .route("/api/alerts", get(alerts_handler))
        // Legacy WebSocket (keep for compatibility)
        .route("/ws", get(websocket_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health_handler() -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({"ok": true})))
}

async fn metrics_handler() -> Result<String, StatusCode> {
    match AppMetrics::global().export() {
        Ok(metrics) => Ok(metrics),
        Err(e) => {
            warn!("Failed to export metrics: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn sessions_handler(
    State(app_state): State<AppState>,
    Query(params): Query<SessionsQuery>,
) -> Result<Json<Value>, StatusCode> {
    let limit = params.limit.unwrap_or(50); // Default to 50, cap at 1000
    let limit = std::cmp::min(limit, 1000);

    // Use the active sessions with complete data filtering
    match sessions::list_active_sessions_with_complete_data(
        &app_state.pool,
        limit,
        app_state.analysis_config.max_session_gap_seconds,
    )
    .await
    {
        Ok(sessions) => Ok(Json(json!({
            "sessions": sessions,
            "count": sessions.len()
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn observations_handler(
    State(app_state): State<AppState>,
    Path(hex): Path<String>,
    Query(params): Query<ObservationsQuery>,
) -> Result<Json<Value>, StatusCode> {
    let limit = params.limit.unwrap_or(100); // Default to 100, cap at 1000
    let limit = std::cmp::min(limit, 1000);

    // Normalize hex to uppercase
    let hex = hex.to_uppercase();

    match observations::list_observations_by_hex(&app_state.pool, &hex, limit).await {
        Ok(observations) => Ok(Json(json!({
            "observations": observations,
            "hex": hex,
            "count": observations.len()
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn alerts_handler(
    State(app_state): State<AppState>,
    Query(params): Query<AlertsQuery>,
) -> Result<Json<Value>, StatusCode> {
    let anomaly_type = if let Some(type_str) = params.anomaly_type {
        match type_str.parse::<AnomalyType>() {
            Ok(atype) => Some(atype),
            Err(_) => return Err(StatusCode::BAD_REQUEST),
        }
    } else {
        None
    };

    match app_state
        .alert_manager
        .get_alerts(anomaly_type, params.since_ms, params.limit)
        .await
    {
        Ok(alerts) => Ok(Json(json!({
            "alerts": alerts,
            "count": alerts.len()
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn websocket_alerts_handler(
    State(app_state): State<AppState>,
    ws: WebSocketUpgrade,
) -> Response {
    // Pass both the WebSocket manager and alert manager to the handler
    websocket::handle_alerts_websocket(ws, app_state).await
}

async fn websocket_aircraft_handler(
    State(app_state): State<AppState>,
    ws: WebSocketUpgrade,
) -> Response {
    websocket::handle_aircraft_websocket(ws, app_state).await
}

async fn websocket_aircraft_detail_handler(
    State(app_state): State<AppState>,
    Path(hex): Path<String>,
    ws: WebSocketUpgrade,
) -> Response {
    websocket::handle_aircraft_detail_websocket_with_path(Path(hex), ws, app_state).await
}

async fn websocket_handler(State(app_state): State<AppState>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(|socket| handle_websocket(socket, app_state))
}

async fn handle_websocket(socket: WebSocket, app_state: AppState) {
    let mut alert_receiver = app_state.alert_manager.broadcaster().subscribe();
    let (mut sender, mut socket_receiver) = socket.split();

    // Send initial greeting
    if sender
        .send(Message::Text(json!({"type": "connected"}).to_string()))
        .await
        .is_err()
    {
        return;
    }

    // Spawn task to handle incoming messages (for potential future use)
    let _receive_task = tokio::spawn(async move {
        while let Some(msg) = socket_receiver.next().await {
            if let Ok(Message::Close(_)) = msg {
                break;
            }
            // For now, we ignore incoming messages
        }
    });

    // Handle alert broadcasting
    while let Ok(alert) = alert_receiver.recv().await {
        // Convert Alert to WebSocketMessage
        let ws_alert = websocket::AlertMessage {
            ts_ms: alert.ts_ms,
            hex: alert.hex.clone(),
            anomaly_type: alert.anomaly_type.to_string(),
            confidence: alert.confidence,
            details_json: alert.details_json.clone(),
        };

        // Also notify the WebSocket manager for HTMX clients
        let _ = app_state.ws_manager.broadcast_alert(ws_alert.clone());

        // Send JSON for legacy clients
        let message = json!({
            "type": "alert",
            "data": alert
        });

        if sender
            .send(Message::Text(message.to_string()))
            .await
            .is_err()
        {
            warn!("Failed to send alert to WebSocket client");
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use tempfile::TempDir;

    async fn create_test_pool() -> (SqlitePool, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = crate::store::connect_and_migrate(db_path.to_str().unwrap(), false)
            .await
            .unwrap();
        (pool, temp_dir)
    }

    fn create_test_app(pool: SqlitePool) -> Router {
        let alert_manager = AlertManager::new(pool.clone());
        let analysis_config = AnalysisConfig::default();
        let web_config = WebConfig::default();
        create_app(pool, alert_manager, analysis_config, web_config)
    }

    async fn insert_test_data(pool: &SqlitePool) {
        use crate::model::AircraftObservation;

        // Use recent timestamps so sessions are considered "active"
        let now_ms = chrono::Utc::now().timestamp_millis();

        // Insert some test observations
        let observations = vec![
            AircraftObservation {
                id: None,
                ts_ms: now_ms - 120_000, // 2 minutes ago
                hex: "ABC123".to_string(),
                flight: Some("UAL456".to_string()),
                lat: Some(40.7128),
                lon: Some(-74.0060),
                altitude: Some(35000),
                gs: Some(450.0),
                rssi: Some(-45.5),
                msg_count_total: Some(1000),
                raw_json: r#"{"hex":"ABC123"}"#.to_string(),
                msg_rate_hz: None,
            },
            AircraftObservation {
                id: None,
                ts_ms: now_ms - 60_000, // 1 minute ago
                hex: "ABC123".to_string(),
                flight: Some("UAL456".to_string()),
                lat: Some(40.7129),
                lon: Some(-74.0061),
                altitude: Some(35100),
                gs: Some(451.0),
                rssi: Some(-45.0),
                msg_count_total: Some(1010),
                raw_json: r#"{"hex":"ABC123"}"#.to_string(),
                msg_rate_hz: None,
            },
            AircraftObservation {
                id: None,
                ts_ms: now_ms - 30_000, // 30 seconds ago
                hex: "DEF456".to_string(),
                flight: Some("DAL789".to_string()),
                lat: Some(34.0522),
                lon: Some(-118.2437),
                altitude: Some(28000),
                gs: Some(380.0),
                rssi: Some(-52.1),
                msg_count_total: Some(750),
                raw_json: r#"{"hex":"DEF456"}"#.to_string(),
                msg_rate_hz: None,
            },
        ];

        // Insert observations
        observations::insert_observations(pool, &observations)
            .await
            .unwrap();

        // Upsert sessions (this will create session records)
        for mut obs in observations {
            sessions::upsert_session_from_observation(pool, &mut obs)
                .await
                .unwrap();
        }
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let (pool, _temp_dir) = create_test_pool().await;
        let app = create_test_app(pool);
        let server = TestServer::new(app).unwrap();

        let response = server.get("/healthz").await;

        assert_eq!(response.status_code(), StatusCode::OK);

        let json: serde_json::Value = response.json();
        assert_eq!(json["ok"], true);
    }

    #[tokio::test]
    async fn test_health_endpoint_with_reqwest() {
        let (pool, _temp_dir) = create_test_pool().await;
        let app = create_test_app(pool);

        // Start server on random port
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // Spawn server in background
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        // Give server a moment to start
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // Make request with reqwest
        let client = reqwest::Client::new();
        let response = client
            .get(&format!("http://{}/healthz", addr))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);

        let json: serde_json::Value = response.json().await.unwrap();
        assert_eq!(json["ok"], true);
    }

    #[tokio::test]
    async fn test_sessions_api() {
        let (pool, _temp_dir) = create_test_pool().await;
        insert_test_data(&pool).await;

        let app = create_test_app(pool);
        let server = TestServer::new(app).unwrap();

        // Test default limit
        let response = server.get("/api/sessions").await;
        assert_eq!(response.status_code(), StatusCode::OK);

        let json: serde_json::Value = response.json();
        assert!(json["sessions"].is_array());
        assert_eq!(json["count"], 2); // We have 2 unique aircraft

        let sessions = json["sessions"].as_array().unwrap();
        assert_eq!(sessions.len(), 2);

        // Check ordering - should be by last_seen_ms DESC
        // DEF456 has the latest timestamp (1641024002000)
        assert_eq!(sessions[0]["hex"], "DEF456");
        assert_eq!(sessions[1]["hex"], "ABC123");

        // Check session fields are present
        let first_session = &sessions[0];
        assert!(first_session["hex"].is_string());
        assert!(first_session["first_seen_ms"].is_number());
        assert!(first_session["last_seen_ms"].is_number());
        assert!(first_session["message_count"].is_number());
        assert!(first_session["has_position"].is_boolean());
    }

    #[tokio::test]
    async fn test_sessions_api_with_limit() {
        let (pool, _temp_dir) = create_test_pool().await;
        insert_test_data(&pool).await;

        let app = create_test_app(pool);
        let server = TestServer::new(app).unwrap();

        // Test with limit parameter
        let response = server.get("/api/sessions?limit=1").await;
        assert_eq!(response.status_code(), StatusCode::OK);

        let json: serde_json::Value = response.json();
        assert_eq!(json["count"], 1);

        let sessions = json["sessions"].as_array().unwrap();
        assert_eq!(sessions.len(), 1);
        // Should be the most recent one (DEF456)
        assert_eq!(sessions[0]["hex"], "DEF456");
    }

    #[tokio::test]
    async fn test_observations_api() {
        let (pool, _temp_dir) = create_test_pool().await;
        insert_test_data(&pool).await;

        let app = create_test_app(pool);
        let server = TestServer::new(app).unwrap();

        // Test observations for ABC123
        let response = server.get("/api/aircraft/ABC123/observations").await;
        assert_eq!(response.status_code(), StatusCode::OK);

        let json: serde_json::Value = response.json();
        assert_eq!(json["hex"], "ABC123");
        assert_eq!(json["count"], 2); // ABC123 has 2 observations

        let observations = json["observations"].as_array().unwrap();
        assert_eq!(observations.len(), 2);

        // Check ordering - should be by ts_ms DESC (most recent first)
        let first_obs = &observations[0];
        let second_obs = &observations[1];
        assert!(first_obs["ts_ms"].as_i64().unwrap() > second_obs["ts_ms"].as_i64().unwrap());

        // Check observation fields are present
        assert_eq!(first_obs["hex"], "ABC123");
        assert!(first_obs["ts_ms"].is_number());
        assert!(first_obs["flight"].is_string());
        assert!(first_obs["lat"].is_number());
        assert!(first_obs["altitude"].is_number());
    }

    #[tokio::test]
    async fn test_observations_api_with_limit() {
        let (pool, _temp_dir) = create_test_pool().await;
        insert_test_data(&pool).await;

        let app = create_test_app(pool);
        let server = TestServer::new(app).unwrap();

        // Test with limit parameter
        let response = server
            .get("/api/aircraft/ABC123/observations?limit=1")
            .await;
        assert_eq!(response.status_code(), StatusCode::OK);

        let json: serde_json::Value = response.json();
        assert_eq!(json["count"], 1);

        let observations = json["observations"].as_array().unwrap();
        assert_eq!(observations.len(), 1);
        // Should be the most recent observation (within the last few minutes)
        let obs_ts = observations[0]["ts_ms"].as_i64().unwrap();
        let now_ms = chrono::Utc::now().timestamp_millis();
        assert!(
            obs_ts > now_ms - 300_000,
            "Observation should be within last 5 minutes"
        );
    }

    #[tokio::test]
    async fn test_observations_api_nonexistent_hex() {
        let (pool, _temp_dir) = create_test_pool().await;
        insert_test_data(&pool).await;

        let app = create_test_app(pool);
        let server = TestServer::new(app).unwrap();

        // Test with non-existent hex
        let response = server.get("/api/aircraft/NONEXISTENT/observations").await;
        assert_eq!(response.status_code(), StatusCode::OK);

        let json: serde_json::Value = response.json();
        assert_eq!(json["hex"], "NONEXISTENT");
        assert_eq!(json["count"], 0);
        assert_eq!(json["observations"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_observations_api_case_insensitive() {
        let (pool, _temp_dir) = create_test_pool().await;
        insert_test_data(&pool).await;

        let app = create_test_app(pool);
        let server = TestServer::new(app).unwrap();

        // Test with lowercase hex (should be converted to uppercase)
        let response = server.get("/api/aircraft/abc123/observations").await;
        assert_eq!(response.status_code(), StatusCode::OK);

        let json: serde_json::Value = response.json();
        assert_eq!(json["hex"], "ABC123"); // Should be normalized to uppercase
        assert_eq!(json["count"], 2);
    }

    #[tokio::test]
    async fn test_alerts_api() {
        let (pool, _temp_dir) = create_test_pool().await;
        let app = create_test_app(pool.clone());
        let server = TestServer::new(app).unwrap();

        // First insert a test alert via AlertManager
        let mut alert_manager = AlertManager::new(pool);
        let test_alert = crate::alerts::Alert {
            ts_ms: 1641024000000,
            hex: "ABC123".to_string(),
            anomaly_type: AnomalyType::Temporal,
            subtype: "rapid_transmission".to_string(),
            confidence: 0.9,
            details_json: Some(json!({"rate": 15.5}).to_string()),
        };
        alert_manager.record_alert(test_alert).await.unwrap();

        // Test alerts endpoint
        let response = server.get("/api/alerts").await;
        assert_eq!(response.status_code(), StatusCode::OK);

        let json: serde_json::Value = response.json();
        assert!(json["alerts"].is_array());
        assert_eq!(json["count"], 1);

        let alerts = json["alerts"].as_array().unwrap();
        assert_eq!(alerts.len(), 1);

        let alert = &alerts[0];
        assert_eq!(alert["hex"], "ABC123");
        assert_eq!(alert["anomaly_type"], "temporal");
        assert_eq!(alert["confidence"], 0.9);
    }

    #[tokio::test]
    async fn test_alerts_api_with_filter() {
        let (pool, _temp_dir) = create_test_pool().await;
        let app = create_test_app(pool.clone());
        let server = TestServer::new(app).unwrap();

        // Insert multiple alerts of different types
        let mut alert_manager = AlertManager::new(pool);
        let temporal_alert = crate::alerts::Alert {
            ts_ms: 1641024000000,
            hex: "ABC123".to_string(),
            anomaly_type: AnomalyType::Temporal,
            subtype: "rapid_transmission".to_string(),
            confidence: 0.9,
            details_json: None,
        };
        let signal_alert = crate::alerts::Alert {
            ts_ms: 1641024001000,
            hex: "DEF456".to_string(),
            anomaly_type: AnomalyType::Signal,
            subtype: "signal_outlier".to_string(),
            confidence: 0.7,
            details_json: None,
        };

        alert_manager.record_alert(temporal_alert).await.unwrap();
        alert_manager.record_alert(signal_alert).await.unwrap();

        // Test filtering by type
        let response = server.get("/api/alerts?type=temporal").await;
        assert_eq!(response.status_code(), StatusCode::OK);

        let json: serde_json::Value = response.json();
        assert_eq!(json["count"], 1);

        let alerts = json["alerts"].as_array().unwrap();
        assert_eq!(alerts[0]["hex"], "ABC123");
        assert_eq!(alerts[0]["anomaly_type"], "temporal");
    }

    #[tokio::test]
    async fn test_alerts_api_with_invalid_type() {
        let (pool, _temp_dir) = create_test_pool().await;
        let app = create_test_app(pool);
        let server = TestServer::new(app).unwrap();

        // Test with invalid anomaly type
        let response = server.get("/api/alerts?type=invalid_type").await;
        assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    }
}
