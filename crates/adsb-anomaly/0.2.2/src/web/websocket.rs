// ABOUTME: WebSocket handlers for real-time alert streaming to dashboard clients
// ABOUTME: Manages WebSocket connections and broadcasts new anomaly alerts using HTMX-compatible format

#![allow(dead_code)]

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Response,
};
use chrono::{DateTime, Utc};
use futures::{SinkExt, StreamExt};
use serde_json::{self, Value};
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

/// WebSocket connection manager
#[derive(Debug, Clone)]
pub struct WebSocketManager {
    /// Broadcast channel for sending alerts to all connected clients
    alert_sender: broadcast::Sender<AlertMessage>,
}

/// Alert message format for WebSocket transmission
#[derive(Debug, Clone)]
pub struct AlertMessage {
    pub ts_ms: i64,
    pub hex: String,
    pub anomaly_type: String,
    pub confidence: f64,
    pub details_json: Option<String>,
}

/// Aircraft data message format for WebSocket transmission
#[derive(Debug, Clone)]
pub struct AircraftDataMessage {
    pub aircraft_count: usize,
    pub aircraft: Vec<crate::web::dashboard::AircraftView>,
    pub map_center_lat: f64,
    pub map_center_lon: f64,
}

impl WebSocketManager {
    /// Create a new WebSocket manager
    pub fn new() -> Self {
        let (alert_sender, _) = broadcast::channel(100);
        Self { alert_sender }
    }

    /// Get a subscription to alert broadcasts
    pub fn subscribe(&self) -> broadcast::Receiver<AlertMessage> {
        self.alert_sender.subscribe()
    }

    /// Broadcast an alert to all connected clients
    pub fn broadcast_alert(
        &self,
        alert: AlertMessage,
    ) -> Result<usize, broadcast::error::SendError<AlertMessage>> {
        self.alert_sender.send(alert)
    }
}

/// Aircraft data WebSocket manager
#[derive(Debug, Clone)]
pub struct AircraftDataManager {
    /// Broadcast channel for sending aircraft updates to all connected clients
    aircraft_sender: broadcast::Sender<AircraftDataMessage>,
}

impl AircraftDataManager {
    /// Create a new aircraft data manager
    pub fn new() -> Self {
        let (aircraft_sender, _) = broadcast::channel(10);
        Self { aircraft_sender }
    }

    /// Get a subscription to aircraft data broadcasts
    pub fn subscribe(&self) -> broadcast::Receiver<AircraftDataMessage> {
        self.aircraft_sender.subscribe()
    }

    /// Broadcast aircraft data to all connected clients
    pub fn broadcast_aircraft_data(
        &self,
        data: AircraftDataMessage,
    ) -> Result<usize, broadcast::error::SendError<AircraftDataMessage>> {
        self.aircraft_sender.send(data)
    }
}

/// Handle WebSocket upgrade for alerts stream
#[allow(private_interfaces)]
pub async fn handle_alerts_websocket(
    ws: WebSocketUpgrade,
    app_state: crate::web::AppState,
) -> Response {
    ws.on_upgrade(|socket| handle_alert_socket_from_alerts(socket, app_state))
}

/// Handle WebSocket upgrade for aircraft data stream
#[allow(private_interfaces)]
pub async fn handle_aircraft_websocket(
    ws: WebSocketUpgrade,
    app_state: crate::web::AppState,
) -> Response {
    ws.on_upgrade(|socket| handle_aircraft_socket(socket, app_state))
}

/// Handle WebSocket upgrade for aircraft-specific updates
#[allow(private_interfaces)]
pub async fn handle_aircraft_detail_websocket(
    ws: WebSocketUpgrade,
    hex: String,
    app_state: crate::web::AppState,
) -> Response {
    ws.on_upgrade(|socket| handle_aircraft_detail_socket(socket, hex, app_state))
}

/// Handle individual WebSocket connection for alerts
async fn handle_alert_socket(socket: WebSocket, ws_manager: Arc<WebSocketManager>) {
    let (mut sender, mut receiver) = socket.split();
    let mut alert_receiver = ws_manager.subscribe();

    info!("New WebSocket connection established for alerts");

    // Spawn task to handle incoming messages (mostly just ping/pong)
    let recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Close(_)) => {
                    debug!("WebSocket client disconnected");
                    break;
                }
                Ok(Message::Ping(_data)) => {
                    debug!("Received ping, sending pong");
                    // Pong will be sent automatically by axum
                }
                Ok(Message::Pong(_)) => {
                    debug!("Received pong");
                }
                Ok(Message::Text(text)) => {
                    debug!("Received text message: {}", text);
                    // Could handle client commands here if needed
                }
                Err(e) => {
                    warn!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });

    // Spawn task to handle outgoing alert messages
    let send_task = tokio::spawn(async move {
        while let Ok(alert) = alert_receiver.recv().await {
            let html_row = format_alert_as_html_row(&alert);

            if let Err(e) = sender.send(Message::Text(html_row)).await {
                error!("Failed to send alert to WebSocket client: {}", e);
                break;
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = recv_task => {
            debug!("WebSocket receive task completed");
        }
        _ = send_task => {
            debug!("WebSocket send task completed");
        }
    }

    info!("WebSocket connection closed for alerts");
}

/// Handle individual WebSocket connection for alerts from AlertManager
async fn handle_alert_socket_from_alerts(socket: WebSocket, app_state: crate::web::AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut alert_receiver = app_state.alert_manager.broadcaster().subscribe();

    info!("New WebSocket connection established for alerts");

    // Spawn task to handle incoming messages (mostly just ping/pong)
    let recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Close(_)) => {
                    debug!("WebSocket client disconnected");
                    break;
                }
                Ok(Message::Ping(_data)) => {
                    debug!("Received ping, sending pong");
                    // Pong will be sent automatically by axum
                }
                Ok(Message::Pong(_)) => {
                    debug!("Received pong");
                }
                Ok(Message::Text(text)) => {
                    debug!("Received text message: {}", text);
                    // Could handle client commands here if needed
                }
                Err(e) => {
                    warn!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });

    // Spawn task to handle outgoing alert messages from AlertManager
    let send_task = tokio::spawn(async move {
        while let Ok(alert) = alert_receiver.recv().await {
            // Convert Alert to AlertMessage
            let alert_message = AlertMessage {
                ts_ms: alert.ts_ms,
                hex: alert.hex,
                anomaly_type: alert.anomaly_type.to_string(),
                confidence: alert.confidence,
                details_json: alert.details_json,
            };

            let html_row = format_alert_as_html_row(&alert_message);

            if let Err(e) = sender.send(Message::Text(html_row)).await {
                error!("Failed to send alert to WebSocket client: {}", e);
                break;
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = recv_task => {
            debug!("WebSocket receive task completed");
        }
        _ = send_task => {
            debug!("WebSocket send task completed");
        }
    }

    info!("WebSocket connection closed for alerts");
}

/// Format an alert as HTML table row for HTMX insertion
fn format_alert_as_html_row(alert: &AlertMessage) -> String {
    let time_dt = DateTime::from_timestamp_millis(alert.ts_ms).unwrap_or_else(Utc::now);
    let now = Utc::now();
    let duration = now.signed_duration_since(time_dt);

    let time_ago = if duration.num_seconds() < 60 {
        format!("{}s ago", duration.num_seconds())
    } else if duration.num_minutes() < 60 {
        format!("{}m ago", duration.num_minutes())
    } else {
        format!("{}h ago", duration.num_hours())
    };

    // Extract details summary
    let details_summary = alert
        .details_json
        .as_ref()
        .and_then(|json_str| {
            serde_json::from_str::<Value>(json_str)
                .ok()
                .and_then(|v| v.get("reason").and_then(|r| r.as_str().map(String::from)))
        })
        .unwrap_or_else(|| "—".to_string());

    // Determine badge class based on anomaly type
    let badge_class = match alert.anomaly_type.as_str() {
        "Temporal" => "badge-warning",
        "Signal" | "Identity" | "Behavioral" => "badge-danger",
        _ => "badge",
    };

    // Extract subtype from details
    let subtype = alert
        .details_json
        .as_ref()
        .and_then(|json_str| {
            serde_json::from_str::<Value>(json_str)
                .ok()
                .and_then(|v| v.get("subtype").and_then(|s| s.as_str().map(String::from)))
        })
        .unwrap_or_else(|| "general".to_string());

    format!(
        r#"<tr hx-swap-oob="afterbegin:#alerts-table">
            <td>{}</td>
            <td><a href="/aircraft/{}">{}</a></td>
            <td><span class="badge {}">{}</span></td>
            <td>{}</td>
            <td>{:.1}%</td>
            <td>{}</td>
            <td><a href="/aircraft/{}" class="btn">View Aircraft</a></td>
        </tr>"#,
        time_ago,
        alert.hex,
        alert.hex,
        badge_class,
        alert.anomaly_type,
        subtype,
        alert.confidence * 100.0,
        details_summary,
        alert.hex
    )
}

/// Handle individual WebSocket connection for aircraft data
async fn handle_aircraft_socket(socket: WebSocket, app_state: crate::web::AppState) {
    let (mut sender, mut receiver) = socket.split();
    let pool = app_state.pool.clone();
    let analysis_config = app_state.analysis_config.clone();

    info!("New WebSocket connection established for aircraft data");

    // Spawn task to handle incoming messages (mostly just ping/pong)
    let recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Close(_)) => {
                    debug!("WebSocket client disconnected");
                    break;
                }
                Ok(Message::Ping(_data)) => {
                    debug!("Received ping, sending pong");
                }
                Ok(Message::Pong(_)) => {
                    debug!("Received pong");
                }
                Ok(Message::Text(text)) => {
                    debug!("Received text message: {}", text);
                }
                Err(e) => {
                    warn!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });

    // Spawn task to periodically send aircraft data updates
    let send_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(2));

        loop {
            interval.tick().await;

            // Get latest aircraft data with active sessions filtering
            match get_aircraft_data(&pool, &analysis_config).await {
                Ok(aircraft_data) => {
                    let html_update = format_aircraft_data_as_html(&aircraft_data);

                    if let Err(e) = sender.send(Message::Text(html_update)).await {
                        error!("Failed to send aircraft data to WebSocket client: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    error!("Failed to get aircraft data: {}", e);
                }
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = recv_task => {
            debug!("WebSocket receive task completed");
        }
        _ = send_task => {
            debug!("WebSocket send task completed");
        }
    }

    info!("WebSocket connection closed for aircraft data");
}

/// Get aircraft data from database (similar to dashboard handler)
async fn get_aircraft_data(
    pool: &SqlitePool,
    analysis_config: &crate::config::AnalysisConfig,
) -> Result<AircraftDataMessage, Box<dyn std::error::Error + Send + Sync>> {
    use crate::store::sessions::list_active_sessions_with_complete_data;
    use crate::web::dashboard::AircraftView;

    let limit = 200; // Increased from 50 to show all active aircraft
    let sessions = list_active_sessions_with_complete_data(
        pool,
        limit,
        analysis_config.max_session_gap_seconds,
    )
    .await
    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

    let aircraft: Vec<AircraftView> = sessions
        .into_iter()
        .map(|s| AircraftView {
            hex: s.hex,
            flight: s.flight,
            lat: s.lat,
            lon: s.lon,
            altitude: s.altitude,
            speed: s.speed,
            has_position: s.has_position,
            message_count: s.message_count,
        })
        .collect();

    // Calculate dynamic map center from aircraft with position data
    let positioned_aircraft: Vec<&AircraftView> = aircraft
        .iter()
        .filter(|a| a.has_position && a.lat.is_some() && a.lon.is_some())
        .collect();

    let (map_center_lat, map_center_lon) = if positioned_aircraft.is_empty() {
        (41.8781, -87.6298)
    } else {
        let avg_lat = positioned_aircraft
            .iter()
            .map(|a| a.lat.unwrap())
            .sum::<f64>()
            / positioned_aircraft.len() as f64;
        let avg_lon = positioned_aircraft
            .iter()
            .map(|a| a.lon.unwrap())
            .sum::<f64>()
            / positioned_aircraft.len() as f64;
        (avg_lat, avg_lon)
    };

    Ok(AircraftDataMessage {
        aircraft_count: aircraft.len(),
        aircraft,
        map_center_lat,
        map_center_lon,
    })
}

/// Format aircraft data as HTMX out-of-band HTML for WebSocket updates
fn format_aircraft_data_as_html(data: &AircraftDataMessage) -> String {
    let mut aircraft_table_html = String::new();

    if data.aircraft.is_empty() {
        aircraft_table_html = r#"<div class="text-center text-gray-500 py-12">
            <div class="text-4xl mb-2">✈️</div>
            <div class="text-lg font-medium">No active sessions</div>
            <div class="text-sm">Aircraft will appear here when they start transmitting</div>
        </div>"#
            .to_string();
    } else {
        aircraft_table_html.push_str(
            r#"<table class="w-full table-fixed">
            <thead>
                <tr>
                    <th class="w-20 px-2 py-1 text-left text-xs text-gray-500">Aircraft</th>
                    <th class="px-2 py-1 text-left text-xs text-gray-500">Flight</th>
                </tr>
            </thead>
            <tbody>"#,
        );

        for aircraft in &data.aircraft {
            let flight_display = aircraft.flight.as_deref().unwrap_or("—");

            aircraft_table_html.push_str(&format!(
                r#"<tr class="hover:bg-gray-50 cursor-pointer" onclick="window.location.href='/aircraft/{}'">
                    <td class="px-2 py-1 truncate">
                        <div class="font-mono text-xs">{}</div>
                    </td>
                    <td class="px-2 py-1 text-sm truncate">{}</td>
                </tr>"#,
                aircraft.hex, aircraft.hex, flight_display
            ));
        }

        aircraft_table_html.push_str("</tbody></table>");
    }

    // Build JavaScript aircraft data for map updates
    let mut aircraft_data_js = String::from("var aircraftData = [");
    let mut first = true;

    for aircraft in &data.aircraft {
        if aircraft.has_position {
            if let (Some(lat), Some(lon)) = (aircraft.lat, aircraft.lon) {
                if !first {
                    aircraft_data_js.push(',');
                }
                first = false;

                let flight = aircraft.flight.as_deref().unwrap_or("Unknown");
                let alt = aircraft.altitude.unwrap_or(0);
                let speed = aircraft.speed.unwrap_or(0.0);

                aircraft_data_js.push_str(&format!(
                    r#"{{hex: "{}", flight: "{}", lat: {}, lon: {}, alt: {}, speed: {}, hasPosition: true}}"#,
                    aircraft.hex, flight, lat, lon, alt, speed
                ));
            }
        }
    }
    aircraft_data_js.push_str(
        "]; if (typeof updateAircraftMap === 'function') { updateAircraftMap(aircraftData, ",
    );
    aircraft_data_js.push_str(&format!(
        "{}, {}); }}",
        data.map_center_lat, data.map_center_lon
    ));

    // Create the final HTML with proper HTMX out-of-band swapping
    let final_html = format!(
        r#"<div hx-swap-oob="innerHTML:#aircraft-list">{}</div><div hx-swap-oob="innerHTML:#active-count">{}</div><script>{}</script>"#,
        aircraft_table_html, data.aircraft_count, aircraft_data_js
    );

    // Log the HTML being sent for debugging - first 500 chars to avoid spam
    debug!(
        "WebSocket HTML (truncated): {}",
        &final_html[..final_html.len().min(500)]
    );
    debug!(
        "Aircraft count: {}, Positioned aircraft: {}",
        data.aircraft_count,
        data.aircraft.iter().filter(|a| a.has_position).count()
    );

    final_html
}

/// Handle individual WebSocket connection for aircraft-specific data
async fn handle_aircraft_detail_socket(
    socket: WebSocket,
    hex: String,
    app_state: crate::web::AppState,
) {
    let (mut sender, mut receiver) = socket.split();
    let pool = app_state.pool.clone();
    let hex = hex.to_uppercase(); // Normalize hex code

    info!("New WebSocket connection established for aircraft {}", hex);

    // Spawn task to handle incoming messages (mostly just ping/pong)
    let recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Close(_)) => {
                    debug!("WebSocket client disconnected");
                    break;
                }
                Ok(Message::Ping(_data)) => {
                    debug!("Received ping, sending pong");
                }
                Ok(Message::Pong(_)) => {
                    debug!("Received pong");
                }
                Ok(Message::Text(text)) => {
                    debug!("Received text message: {}", text);
                }
                Err(e) => {
                    warn!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });

    // Spawn task to periodically send aircraft-specific data updates
    let hex_clone = hex.clone();
    let send_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3));

        loop {
            interval.tick().await;

            // Get latest session data for this specific aircraft
            match get_aircraft_detail_data(&pool, &hex_clone).await {
                Ok(detail_data) => {
                    let html_update = format_aircraft_detail_data_as_html(&detail_data);

                    if let Err(e) = sender.send(Message::Text(html_update)).await {
                        error!(
                            "Failed to send aircraft detail data to WebSocket client: {}",
                            e
                        );
                        break;
                    }
                }
                Err(e) => {
                    debug!("No data found for aircraft {}: {}", hex_clone, e);
                    // Send empty update to keep connection alive
                    if let Err(e) = sender
                        .send(Message::Text("<!-- keepalive -->".to_string()))
                        .await
                    {
                        error!("Failed to send keepalive to WebSocket client: {}", e);
                        break;
                    }
                }
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = recv_task => {
            debug!("WebSocket receive task completed");
        }
        _ = send_task => {
            debug!("WebSocket send task completed");
        }
    }

    info!("WebSocket connection closed for aircraft {}", hex);
}

/// Aircraft detail data message for WebSocket transmission
#[derive(Debug, Clone)]
pub struct AircraftDetailData {
    pub hex: String,
    pub session: Option<crate::web::dashboard::SessionView>,
    pub flight_path: Vec<crate::web::dashboard::FlightPathPoint>,
    pub latest_position: Option<(f64, f64)>, // (lat, lon)
}

/// Get aircraft detail data from database
async fn get_aircraft_detail_data(
    pool: &SqlitePool,
    hex: &str,
) -> Result<AircraftDetailData, Box<dyn std::error::Error + Send + Sync>> {
    use crate::web::dashboard::{FlightPathPoint, SessionView};

    // Get current session
    let session_result = sqlx::query_as::<_, crate::web::dashboard::SessionRecord>(
        r#"
        SELECT hex, first_seen_ms, last_seen_ms, message_count,
               has_position, has_altitude, has_callsign,
               flight, lat, lon, altitude, speed
        FROM aircraft_sessions
        WHERE hex = ? AND (last_seen_ms > ? OR first_seen_ms > ?)
        ORDER BY last_seen_ms DESC
        LIMIT 1
        "#,
    )
    .bind(hex)
    .bind(chrono::Utc::now().timestamp_millis() - 24 * 60 * 60 * 1000) // Last 24 hours
    .bind(chrono::Utc::now().timestamp_millis() - 24 * 60 * 60 * 1000)
    .fetch_optional(pool)
    .await?;

    let session = session_result.map(|s| {
        let (speed_mph, speed_kmh) = if let Some(speed_knots) = s.speed {
            let mph = (speed_knots * 1.15078).round() as i32;
            let kmh = (speed_knots * 1.852).round() as i32;
            (Some(mph), Some(kmh))
        } else {
            (None, None)
        };

        SessionView {
            hex: s.hex,
            flight: s.flight,
            lat: s.lat,
            lon: s.lon,
            altitude: s.altitude,
            speed: s.speed,
            speed_mph,
            speed_kmh,
            message_count: s.message_count,
            has_position: s.has_position,
            has_altitude: s.has_altitude,
            has_callsign: s.has_callsign,
            last_seen_ms: s.last_seen_ms,
        }
    });

    // Get recent flight path points (last 50 points)
    let flight_path_result = sqlx::query_as::<_, crate::web::dashboard::FlightPathRecord>(
        r#"
        SELECT ts_ms, lat, lon, altitude
        FROM aircraft_observations
        WHERE hex = ? AND lat IS NOT NULL AND lon IS NOT NULL
        ORDER BY ts_ms DESC
        LIMIT 50
        "#,
    )
    .bind(hex)
    .fetch_all(pool)
    .await?;

    let mut flight_path: Vec<FlightPathPoint> = flight_path_result
        .into_iter()
        .map(|obs| FlightPathPoint {
            lat: obs.lat,
            lon: obs.lon,
            ts_ms: obs.ts_ms,
            altitude: obs.altitude,
        })
        .collect();

    // Reverse to get chronological order
    flight_path.reverse();

    // Get latest position from session or flight path
    let latest_position = if let Some(ref s) = session {
        if let (Some(lat), Some(lon)) = (s.lat, s.lon) {
            Some((lat, lon))
        } else {
            None
        }
    } else {
        flight_path.last().map(|p| (p.lat, p.lon))
    };

    Ok(AircraftDetailData {
        hex: hex.to_string(),
        session,
        flight_path,
        latest_position,
    })
}

/// Format aircraft detail data as HTMX out-of-band HTML for WebSocket updates
fn format_aircraft_detail_data_as_html(data: &AircraftDetailData) -> String {
    let mut updates = Vec::new();

    // Update position info if we have session data
    if let Some(ref session) = data.session {
        if let (Some(lat), Some(lon)) = (session.lat, session.lon) {
            let position_html = format!(
                r#"<div class="grid grid-cols-2 gap-4 text-sm">
                        <div>
                            <span class="text-gray-500">Latitude:</span>
                            <div class="font-mono">{:.6}°</div>
                        </div>
                        <div>
                            <span class="text-gray-500">Longitude:</span>
                            <div class="font-mono">{:.6}°</div>
                        </div>
                    </div>
                    <span class="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-green-100 text-green-800 mt-2">
                        Position Available
                    </span>"#,
                lat, lon
            );
            updates.push(format!(
                r#"<div hx-swap-oob="innerHTML:#position-data">{}</div>"#,
                position_html
            ));
        }

        // Update altitude info
        if let Some(altitude) = session.altitude {
            let altitude_html = format!(
                r#"<div class="font-mono text-lg">{} ft</div>
                    <span class="text-sm text-gray-500">feet above sea level</span>"#,
                altitude
            );
            updates.push(format!(
                r#"<div hx-swap-oob="innerHTML:#altitude-data">{}</div>"#,
                altitude_html
            ));
        }

        // Update speed info
        if let Some(speed) = session.speed {
            let speed_html = format!(
                r#"<div class="font-mono text-lg">{:.1} kt</div>
                    <span class="text-sm text-gray-500">knots</span>"#,
                speed
            );
            updates.push(format!(
                r#"<div hx-swap-oob="innerHTML:#speed-data">{}</div>"#,
                speed_html
            ));
        }
    }

    // Build JavaScript for map and flight path updates
    if let Some((lat, lon)) = data.latest_position {
        let flight_path_js = if !data.flight_path.is_empty() {
            let path_coords: Vec<String> = data
                .flight_path
                .iter()
                .map(|p| format!("[{}, {}]", p.lat, p.lon))
                .collect();
            format!(
                r#"var flightPathCoords = [{}];
                   if (typeof updateAircraftPosition === 'function') {{
                       updateAircraftPosition({}, {}, flightPathCoords);
                   }}"#,
                path_coords.join(", "),
                lat,
                lon
            )
        } else {
            format!(
                r#"if (typeof updateAircraftPosition === 'function') {{
                       updateAircraftPosition({}, {}, []);
                   }}"#,
                lat, lon
            )
        };
        updates.push(format!("<script>{}</script>", flight_path_js));
    }

    // Create the final HTML with all updates
    let final_html = updates.join("");

    // Log for debugging
    debug!(
        "Aircraft detail WebSocket HTML (truncated): {}",
        &final_html[..final_html.len().min(200)]
    );

    final_html
}

// Add the new route handler
pub async fn handle_aircraft_detail_websocket_with_path(
    axum::extract::Path(hex): axum::extract::Path<String>,
    ws: WebSocketUpgrade,
    app_state: crate::web::AppState,
) -> Response {
    handle_aircraft_detail_websocket(ws, hex, app_state).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_manager_creation() {
        let manager = WebSocketManager::new();
        let mut receiver = manager.subscribe();

        // Should be able to subscribe without panicking
        assert!(receiver.try_recv().is_err()); // No messages yet
    }

    #[test]
    fn test_alert_broadcast() {
        let manager = WebSocketManager::new();
        let mut receiver = manager.subscribe();

        let alert = AlertMessage {
            ts_ms: chrono::Utc::now().timestamp_millis(),
            hex: "ABC123".to_string(),
            anomaly_type: "Temporal".to_string(),
            confidence: 0.85,
            details_json: Some(r#"{"reason":"rapid_transmission"}"#.to_string()),
        };

        let result = manager.broadcast_alert(alert.clone());
        assert!(result.is_ok());

        // Should be able to receive the alert
        let received = receiver.try_recv();
        assert!(received.is_ok());
        let received_alert = received.unwrap();
        assert_eq!(received_alert.hex, "ABC123");
        assert_eq!(received_alert.anomaly_type, "Temporal");
    }

    #[test]
    fn test_format_alert_as_html_row() {
        let alert = AlertMessage {
            ts_ms: chrono::Utc::now().timestamp_millis(),
            hex: "DEF456".to_string(),
            anomaly_type: "Signal".to_string(),
            confidence: 0.92,
            details_json: Some(r#"{"reason":"signal_outlier","subtype":"rssi_spike"}"#.to_string()),
        };

        let html = format_alert_as_html_row(&alert);

        // Should contain key elements
        assert!(html.contains("DEF456"));
        assert!(html.contains("Signal"));
        assert!(html.contains("92.0%"));
        assert!(html.contains("badge-danger"));
        assert!(html.contains("signal_outlier"));
    }

    #[test]
    fn test_format_alert_minimal_data() {
        let alert = AlertMessage {
            ts_ms: chrono::Utc::now().timestamp_millis(),
            hex: "GHI789".to_string(),
            anomaly_type: "Unknown".to_string(),
            confidence: 0.75,
            details_json: None,
        };

        let html = format_alert_as_html_row(&alert);

        // Should handle missing data gracefully
        assert!(html.contains("GHI789"));
        assert!(html.contains("Unknown"));
        assert!(html.contains("75.0%"));
        assert!(html.contains("—")); // Default for missing details
        assert!(html.contains("general")); // Default subtype
    }
}
