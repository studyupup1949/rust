// ABOUTME: Simple working dashboard implementation to replace complex one
// ABOUTME: Basic HTML generation with aircraft list, map placeholder, and alerts

#![allow(dead_code)]

use crate::store::observations::list_observations_by_hex;
use crate::store::sessions::list_sessions;
use axum::{
    extract::{Path, Query, State},
    routing::get,
    Router,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::SqlitePool;

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<usize>,
}

pub fn router() -> Router<SqlitePool> {
    Router::new()
        .route("/", get(main_dashboard_page))
        .route("/dashboard", get(main_dashboard_page))
        .route("/aircraft/:hex", get(aircraft_detail_page))
}

/// Handler for the main dashboard page
async fn main_dashboard_page(
    State(pool): State<SqlitePool>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<axum::response::Html<String>, axum::http::StatusCode> {
    let limit = pagination.limit.unwrap_or(50).min(200);

    // Get active aircraft sessions
    let aircraft_sessions = match list_sessions(&pool, limit).await {
        Ok(sessions) => sessions,
        Err(_) => return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
    };

    // Get recent alerts
    let recent_alerts = get_recent_alerts(&pool, 50).await.unwrap_or_default();

    // Build aircraft list HTML
    let mut aircraft_list = String::new();
    for session in aircraft_sessions.iter().take(20) {
        let flight = session.flight.as_deref().unwrap_or("Unknown");
        let position = if session.has_position {
            format!(
                "{:.4}, {:.4}",
                session.lat.unwrap_or(0.0),
                session.lon.unwrap_or(0.0)
            )
        } else {
            "No position".to_string()
        };

        aircraft_list.push_str(&format!(
            r#"<div class="aircraft-item" onclick="window.location.href='/aircraft/{}'">
                <div>
                    <div class="aircraft-hex">{}</div>
                    <div class="aircraft-flight">{}</div>
                </div>
                <div class="aircraft-position">{}</div>
            </div>"#,
            session.hex, session.hex, flight, position
        ));
    }

    // Build alerts list HTML
    let mut alerts_list = String::new();
    for alert in recent_alerts.iter().take(10) {
        alerts_list.push_str(&format!(
            r#"<div class="alert-item">
                <div class="alert-hex">{}</div>
                <div class="alert-type">{}</div>
            </div>"#,
            alert.hex, alert.anomaly_type
        ));
    }

    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>ADS-B Anomaly Detection Dashboard</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 0; background: #f5f5f5; }}
        .header {{ background: #2c3e50; color: white; padding: 1rem 2rem; }}
        .dashboard {{ display: grid; grid-template-columns: 2fr 1fr; gap: 1rem; padding: 1rem 2rem; }}
        .panel {{ background: white; padding: 1rem; border-radius: 8px; }}
        .aircraft-item {{
            display: flex; justify-content: space-between; align-items: center;
            padding: 0.5rem; margin: 0.25rem 0; background: #f8f9fa;
            border-radius: 4px; cursor: pointer;
        }}
        .aircraft-item:hover {{ background: #e9ecef; }}
        .aircraft-hex {{ font-weight: bold; color: #2c3e50; }}
        .aircraft-flight {{ color: #6c757d; font-size: 0.9rem; }}
        .aircraft-position {{ font-size: 0.8rem; color: #27ae60; font-family: monospace; }}
        .alert-item {{ padding: 0.5rem; margin: 0.25rem 0; background: #fff3cd; border-radius: 4px; }}
        .alert-hex {{ font-weight: bold; color: #856404; }}
        .alert-type {{ font-size: 0.8rem; color: #6c757d; }}
        #map {{ height: 400px; background: #e8f4f8; display: flex; align-items: center; justify-content: center; color: #6c757d; }}
        h2 {{ margin-top: 0; color: #2c3e50; border-bottom: 2px solid #3498db; padding-bottom: 0.5rem; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>ADS-B Anomaly Detection Dashboard</h1>
    </div>

    <div class="dashboard">
        <div>
            <div class="panel">
                <div id="map">Interactive Map (Coming Soon)</div>
            </div>
        </div>

        <div>
            <div class="panel">
                <h2>Active Aircraft ({})</h2>
                <div>{}</div>
            </div>

            <div class="panel">
                <h2>Recent Alerts ({})</h2>
                <div>{}</div>
            </div>
        </div>
    </div>

    <script>
        // Auto-refresh every 30 seconds
        setTimeout(function() {{ location.reload(); }}, 30000);
    </script>
</body>
</html>"#,
        aircraft_sessions.len(),
        aircraft_list,
        recent_alerts.len(),
        alerts_list
    );

    Ok(axum::response::Html(html))
}

/// Handler for aircraft detail page
async fn aircraft_detail_page(
    Path(hex): Path<String>,
    State(pool): State<SqlitePool>,
) -> Result<axum::response::Html<String>, axum::http::StatusCode> {
    let hex = hex.to_uppercase();

    // Get session data
    let session = match get_session_by_hex(&pool, &hex).await {
        Ok(Some(s)) => Some(s),
        _ => None,
    };

    // Get recent observations
    let observations: std::vec::Vec<crate::model::AircraftObservation> =
        (list_observations_by_hex(&pool, &hex, 50).await).unwrap_or_default();

    // Simple aircraft detail page
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Aircraft {} Details</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 2rem; }}
        .header {{ background: #2c3e50; color: white; padding: 1rem; margin-bottom: 1rem; }}
        .info {{ background: #f8f9fa; padding: 1rem; margin-bottom: 1rem; border-radius: 5px; }}
        table {{ width: 100%; border-collapse: collapse; }}
        th, td {{ padding: 0.5rem; border: 1px solid #ddd; text-align: left; }}
        th {{ background: #f8f9fa; }}
        a {{ color: #3498db; text-decoration: none; }}
        a:hover {{ text-decoration: underline; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>Aircraft {} Details</h1>
        <a href="/">← Back to Dashboard</a>
    </div>

    <div class="info">
        <h2>Session Information</h2>
        {}
    </div>

    <div class="info">
        <h2>Recent Messages ({} observations)</h2>
        <table>
            <thead>
                <tr>
                    <th>Time</th>
                    <th>Flight</th>
                    <th>Position</th>
                    <th>Altitude</th>
                    <th>Speed</th>
                    <th>RSSI</th>
                </tr>
            </thead>
            <tbody>
                {}
            </tbody>
        </table>
    </div>

    <script>
        // Auto-refresh every 30 seconds
        setTimeout(function() {{ location.reload(); }}, 30000);
    </script>
</body>
</html>"#,
        hex,
        hex,
        if let Some(s) = &session {
            format!(
                "<p><strong>Flight:</strong> {}</p>
                 <p><strong>Position:</strong> {}, {}</p>
                 <p><strong>Messages:</strong> {}</p>",
                s.flight.as_ref().unwrap_or(&"Unknown".to_string()),
                s.lat.map(|l| l.to_string()).unwrap_or("—".to_string()),
                s.lon.map(|l| l.to_string()).unwrap_or("—".to_string()),
                s.message_count
            )
        } else {
            "<p>No session data found</p>".to_string()
        },
        observations.len(),
        observations
            .iter()
            .take(20)
            .map(|obs| {
                let time_dt = DateTime::from_timestamp_millis(obs.ts_ms).unwrap_or_else(Utc::now);
                let duration = Utc::now().signed_duration_since(time_dt);
                let time_ago = if duration.num_seconds() < 60 {
                    format!("{}s ago", duration.num_seconds())
                } else {
                    format!("{}m ago", duration.num_minutes())
                };

                format!(
                "<tr><td>{}</td><td>{}</td><td>{}, {}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                time_ago,
                obs.flight.as_ref().unwrap_or(&"—".to_string()),
                obs.lat.map(|l| format!("{:.4}", l)).unwrap_or("—".to_string()),
                obs.lon.map(|l| format!("{:.4}", l)).unwrap_or("—".to_string()),
                obs.altitude.map(|a| format!("{} ft", a)).unwrap_or("—".to_string()),
                obs.gs.map(|g| format!("{:.0} kt", g)).unwrap_or("—".to_string()),
                obs.rssi.map(|r| format!("{:.1}", r)).unwrap_or("—".to_string())
            )
            })
            .collect::<Vec<_>>()
            .join("\n")
    );

    Ok(axum::response::Html(html))
}

/// Helper to get recent alerts from database
async fn get_recent_alerts(
    pool: &SqlitePool,
    limit: usize,
) -> Result<Vec<AlertRecord>, sqlx::Error> {
    let alerts = sqlx::query_as::<_, AlertRecord>(
        r#"
        SELECT ts_ms, hex, anomaly_type, confidence, details_json
        FROM anomaly_detections
        WHERE reviewed = 0
        ORDER BY ts_ms DESC
        LIMIT ?
        "#,
    )
    .bind(limit as i64)
    .fetch_all(pool)
    .await?;

    Ok(alerts)
}

/// Helper to get session by hex
async fn get_session_by_hex(
    pool: &SqlitePool,
    hex: &str,
) -> Result<Option<SessionRecord>, sqlx::Error> {
    let session = sqlx::query_as::<_, SessionRecord>(
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

    Ok(session)
}

/// Database record for alerts
#[derive(Debug, sqlx::FromRow)]
struct AlertRecord {
    ts_ms: i64,
    hex: String,
    anomaly_type: String,
    confidence: f64,
    details_json: Option<String>,
}

/// Database record for sessions
#[derive(Debug, sqlx::FromRow)]
struct SessionRecord {
    hex: String,
    first_seen_ms: i64,
    last_seen_ms: i64,
    message_count: i64,
    has_position: bool,
    has_altitude: bool,
    has_callsign: bool,
    flight: Option<String>,
    lat: Option<f64>,
    lon: Option<f64>,
    altitude: Option<i32>,
    speed: Option<f64>,
}
