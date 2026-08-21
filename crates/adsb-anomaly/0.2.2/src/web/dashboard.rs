// ABOUTME: Clean dashboard implementation using Askama templates
// ABOUTME: Provides main dashboard, aircraft details with interactive map

use askama::Template;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Html,
    routing::get,
    Router,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::store::observations::list_observations_by_hex;

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<usize>,
}

/// Template for the main dashboard page
#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardTemplate {
    aircraft: Vec<AircraftView>,
    aircraft_count: usize,
    alerts: Vec<AlertView>,
    alerts_count: usize,
    total_aircraft_tracked: i64,
    aircraft_tracked_24h: i64,
    map_center_lat: f64,
    map_center_lon: f64,
}

/// Template for aircraft detail page
#[derive(Template)]
#[template(path = "aircraft_detail.html")]
struct AircraftDetailTemplate {
    hex: String,
    session: Option<SessionView>,
    observations: Vec<ObservationView>,
    historical_sessions: Vec<HistoricalSessionView>,
    flight_path: Vec<FlightPathPoint>,
    tail_number: Option<String>,
    map_center_lat: f64,
    map_center_lon: f64,
}

/// Template for alerts page
#[derive(Template)]
#[template(path = "alerts.html")]
struct AlertsTemplate {
    alerts: Vec<AlertView>,
}

/// Template for sessions page
#[derive(Template)]
#[template(path = "sessions.html")]
struct SessionsTemplate {
    sessions: Vec<SessionPageView>,
}

/// View model for aircraft in the dashboard
#[derive(Debug, Clone)]
pub struct AircraftView {
    pub hex: String,
    pub flight: Option<String>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub altitude: Option<i32>,
    pub speed: Option<f64>,
    pub has_position: bool,
    #[allow(dead_code)]
    pub message_count: i64,
}

/// View model for alerts
#[derive(Debug)]
pub struct AlertView {
    pub hex: String,
    pub anomaly_type: String,
    pub confidence: f64,
    #[allow(dead_code)]
    pub ts_ms: i64,
    pub time_ago: String,
    pub subtype: String,
    pub details_summary: String,
}

/// View model for sessions
#[derive(Debug, Clone)]
pub struct SessionView {
    #[allow(dead_code)]
    pub hex: String,
    pub flight: Option<String>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub altitude: Option<i32>,
    pub speed: Option<f64>,
    pub speed_mph: Option<i32>,
    pub speed_kmh: Option<i32>,
    pub message_count: i64,
    pub has_position: bool,
    pub has_altitude: bool,
    pub has_callsign: bool,
    #[allow(dead_code)]
    pub last_seen_ms: i64,
}

/// View model for observations
#[derive(Debug)]
pub struct ObservationView {
    #[allow(dead_code)]
    pub ts_ms: i64,
    pub flight: Option<String>,
    #[allow(dead_code)]
    pub lat: Option<f64>,
    #[allow(dead_code)]
    pub lon: Option<f64>,
    #[allow(dead_code)]
    pub altitude: Option<i32>,
    #[allow(dead_code)]
    pub gs: Option<f64>,
    #[allow(dead_code)]
    pub rssi: Option<f64>,
    pub time_ago: String,
}

/// View model for historical sessions
#[derive(Debug)]
pub struct HistoricalSessionView {
    #[allow(dead_code)]
    pub first_seen_ms: i64,
    #[allow(dead_code)]
    pub last_seen_ms: i64,
    pub message_count: i64,
    pub flight: Option<String>,
    pub duration_minutes: i64,
    pub first_seen_ago: String,
    pub last_seen_ago: String,
}

/// View model for sessions page
#[derive(Debug)]
pub struct SessionPageView {
    pub hex: String,
    pub flight: Option<String>,
    pub last_seen_ago: String,
    #[allow(dead_code)]
    pub message_rate: String,
    pub has_position: bool,
    pub has_altitude: bool,
    pub has_callsign: bool,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub altitude: Option<i32>,
    pub speed: Option<f64>,
}

/// View model for flight path points
#[derive(Debug, Clone)]
pub struct FlightPathPoint {
    pub lat: f64,
    pub lon: f64,
    #[allow(dead_code)]
    pub ts_ms: i64,
    #[allow(dead_code)]
    pub altitude: Option<i32>,
}

pub fn router() -> Router<crate::web::AppState> {
    Router::new()
        .route("/", get(dashboard_handler))
        .route("/dashboard", get(dashboard_handler))
        .route("/aircraft/:hex", get(aircraft_detail_handler))
        .route("/alerts", get(alerts_handler))
        .route("/sessions", get(sessions_page_handler))
}

/// Main dashboard page handler
async fn dashboard_handler(
    State(app_state): State<crate::web::AppState>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Html<String>, StatusCode> {
    let pool = &app_state.pool;
    let limit = pagination.limit.unwrap_or(200).min(500); // Increased default to show all active aircraft

    // Get active aircraft sessions with complete data only
    let sessions = crate::store::sessions::list_active_sessions_with_complete_data(
        pool,
        limit,
        app_state.analysis_config.max_session_gap_seconds,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Convert to view models
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

    // Get recent alerts
    let alerts: Vec<AlertView> = get_recent_alerts(pool, 10)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|a| {
            let time_dt = DateTime::from_timestamp_millis(a.ts_ms).unwrap_or_else(Utc::now);
            let duration = Utc::now().signed_duration_since(time_dt);
            let time_ago = if duration.num_seconds() < 60 {
                format!("{}s ago", duration.num_seconds())
            } else if duration.num_minutes() < 60 {
                format!("{}m ago", duration.num_minutes())
            } else {
                format!("{}h ago", duration.num_hours())
            };

            // Parse details JSON to extract subtype and summary
            let (subtype, details_summary) = if let Some(ref details_json) = a.details_json {
                if let Ok(details) = serde_json::from_str::<serde_json::Value>(details_json) {
                    let subtype = details
                        .get("subtype")
                        .and_then(|s| s.as_str())
                        .unwrap_or("general")
                        .to_string();
                    let summary = details
                        .get("reason")
                        .and_then(|s| s.as_str())
                        .unwrap_or("—")
                        .to_string();
                    (subtype, summary)
                } else {
                    ("general".to_string(), "—".to_string())
                }
            } else {
                ("general".to_string(), "—".to_string())
            };

            AlertView {
                hex: a.hex,
                anomaly_type: a.anomaly_type,
                confidence: a.confidence,
                ts_ms: a.ts_ms,
                time_ago,
                subtype,
                details_summary,
            }
        })
        .collect();

    // Calculate dynamic map center from aircraft with position data
    let positioned_aircraft: Vec<&AircraftView> = aircraft
        .iter()
        .filter(|a| a.has_position && a.lat.is_some() && a.lon.is_some())
        .collect();

    let (map_center_lat, map_center_lon) = if positioned_aircraft.is_empty() {
        // Use configured fallback center if no positioned aircraft
        (
            app_state.web_config.map_center_lat,
            app_state.web_config.map_center_lon,
        )
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

    // Get total aircraft tracked (all-time unique count)
    let total_aircraft_tracked = get_total_aircraft_tracked(pool).await.unwrap_or(0);

    // Get aircraft tracked in last 24 hours
    let aircraft_tracked_24h = get_aircraft_tracked_24h(pool).await.unwrap_or(0);

    let template = DashboardTemplate {
        aircraft_count: aircraft.len(),
        aircraft,
        alerts_count: alerts.len(),
        alerts,
        total_aircraft_tracked,
        aircraft_tracked_24h,
        map_center_lat,
        map_center_lon,
    };

    let html = template
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html))
}

/// Aircraft detail page handler
async fn aircraft_detail_handler(
    Path(hex): Path<String>,
    State(app_state): State<crate::web::AppState>,
) -> Result<Html<String>, StatusCode> {
    let pool = &app_state.pool;
    let hex = hex.to_uppercase();

    // Get session data
    let session = get_session_by_hex(pool, &hex)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(|s| {
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

    // Get recent observations
    let observations: Vec<ObservationView> = list_observations_by_hex(pool, &hex, 50)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .map(|obs| {
            let time_dt = DateTime::from_timestamp_millis(obs.ts_ms).unwrap_or_else(Utc::now);
            let duration = Utc::now().signed_duration_since(time_dt);
            let time_ago = if duration.num_seconds() < 60 {
                format!("{}s ago", duration.num_seconds())
            } else {
                format!("{}m ago", duration.num_minutes())
            };

            ObservationView {
                ts_ms: obs.ts_ms,
                flight: obs.flight,
                lat: obs.lat,
                lon: obs.lon,
                altitude: obs.altitude,
                gs: obs.gs,
                rssi: obs.rssi,
                time_ago,
            }
        })
        .collect();

    // Get historical sessions
    let historical_sessions = get_historical_sessions_by_hex(pool, &hex, 20)
        .await
        .unwrap_or_default();

    // Get flight path data
    let flight_path = get_flight_path_by_hex(pool, &hex, 200)
        .await
        .unwrap_or_default();

    // Extract tail number (simplified)
    let tail_number = extract_tail_number(&hex);

    let template = AircraftDetailTemplate {
        hex: hex.clone(),
        session,
        observations,
        historical_sessions,
        flight_path,
        tail_number,
        map_center_lat: app_state.web_config.map_center_lat,
        map_center_lon: app_state.web_config.map_center_lon,
    };

    let html = template
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html))
}

/// Alerts page handler
async fn alerts_handler(
    State(app_state): State<crate::web::AppState>,
) -> Result<Html<String>, StatusCode> {
    let pool = &app_state.pool;
    // Get recent alerts for display
    let alerts: Vec<AlertView> = get_recent_alerts(pool, 100)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|a| {
            let time_dt = DateTime::from_timestamp_millis(a.ts_ms).unwrap_or_else(Utc::now);
            let duration = Utc::now().signed_duration_since(time_dt);
            let time_ago = if duration.num_seconds() < 60 {
                format!("{}s ago", duration.num_seconds())
            } else if duration.num_minutes() < 60 {
                format!("{}m ago", duration.num_minutes())
            } else {
                format!("{}h ago", duration.num_hours())
            };

            // Parse details JSON to extract subtype and summary
            let (subtype, details_summary) = if let Some(ref details_json) = a.details_json {
                if let Ok(details) = serde_json::from_str::<serde_json::Value>(details_json) {
                    let subtype = details
                        .get("subtype")
                        .and_then(|s| s.as_str())
                        .unwrap_or("general")
                        .to_string();
                    let summary = details
                        .get("reason")
                        .and_then(|s| s.as_str())
                        .unwrap_or("—")
                        .to_string();
                    (subtype, summary)
                } else {
                    ("general".to_string(), "—".to_string())
                }
            } else {
                ("general".to_string(), "—".to_string())
            };

            AlertView {
                hex: a.hex,
                anomaly_type: a.anomaly_type,
                confidence: a.confidence,
                ts_ms: a.ts_ms,
                time_ago,
                subtype,
                details_summary,
            }
        })
        .collect();

    let template = AlertsTemplate { alerts };

    let html = template
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html))
}

/// Sessions page handler
async fn sessions_page_handler(
    State(app_state): State<crate::web::AppState>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Html<String>, StatusCode> {
    let pool = &app_state.pool;
    let limit = pagination.limit.unwrap_or(200).min(500);

    // Get active aircraft sessions
    let sessions = crate::store::sessions::list_active_sessions_with_complete_data(
        pool,
        limit,
        app_state.analysis_config.max_session_gap_seconds,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Convert to session page view models
    let sessions: Vec<SessionPageView> = sessions
        .into_iter()
        .map(|s| {
            let time_dt = DateTime::from_timestamp_millis(s.last_seen_ms).unwrap_or_else(Utc::now);
            let duration = Utc::now().signed_duration_since(time_dt);
            let last_seen_ago = if duration.num_seconds() < 60 {
                format!("{}s ago", duration.num_seconds())
            } else if duration.num_minutes() < 60 {
                format!("{}m ago", duration.num_minutes())
            } else {
                format!("{}h ago", duration.num_hours())
            };

            SessionPageView {
                hex: s.hex,
                flight: s.flight,
                last_seen_ago,
                message_rate: "—".to_string(), // We don't have rate data in sessions
                has_position: s.has_position,
                has_altitude: s.has_altitude,
                has_callsign: s.has_callsign,
                lat: s.lat,
                lon: s.lon,
                altitude: s.altitude,
                speed: s.speed,
            }
        })
        .collect();

    let template = SessionsTemplate { sessions };

    let html = template
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html))
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

/// Helper to get all historical sessions for an aircraft
async fn get_historical_sessions_by_hex(
    pool: &SqlitePool,
    hex: &str,
    limit: usize,
) -> Result<Vec<HistoricalSessionView>, sqlx::Error> {
    let limit = std::cmp::min(limit, 100); // Cap at 100 for safety

    let sessions = sqlx::query_as::<_, HistoricalSessionRecord>(
        r#"
        SELECT hex, first_seen_ms, last_seen_ms, message_count, flight
        FROM aircraft_sessions
        WHERE hex = ?
        ORDER BY last_seen_ms DESC
        LIMIT ?
        "#,
    )
    .bind(hex)
    .bind(limit as i64)
    .fetch_all(pool)
    .await?;

    let now = chrono::Utc::now();
    let historical_sessions = sessions
        .into_iter()
        .map(|s| {
            let first_seen_dt = DateTime::from_timestamp_millis(s.first_seen_ms).unwrap_or(now);
            let last_seen_dt = DateTime::from_timestamp_millis(s.last_seen_ms).unwrap_or(now);

            let duration_minutes = (s.last_seen_ms - s.first_seen_ms) / (60 * 1000);

            let first_seen_duration = now.signed_duration_since(first_seen_dt);
            let last_seen_duration = now.signed_duration_since(last_seen_dt);

            let first_seen_ago = if first_seen_duration.num_days() > 0 {
                format!("{}d ago", first_seen_duration.num_days())
            } else if first_seen_duration.num_hours() > 0 {
                format!("{}h ago", first_seen_duration.num_hours())
            } else if first_seen_duration.num_minutes() > 0 {
                format!("{}m ago", first_seen_duration.num_minutes())
            } else {
                format!("{}s ago", first_seen_duration.num_seconds())
            };

            let last_seen_ago = if last_seen_duration.num_days() > 0 {
                format!("{}d ago", last_seen_duration.num_days())
            } else if last_seen_duration.num_hours() > 0 {
                format!("{}h ago", last_seen_duration.num_hours())
            } else if last_seen_duration.num_minutes() > 0 {
                format!("{}m ago", last_seen_duration.num_minutes())
            } else {
                format!("{}s ago", last_seen_duration.num_seconds())
            };

            HistoricalSessionView {
                first_seen_ms: s.first_seen_ms,
                last_seen_ms: s.last_seen_ms,
                message_count: s.message_count,
                flight: s.flight,
                duration_minutes,
                first_seen_ago,
                last_seen_ago,
            }
        })
        .collect();

    Ok(historical_sessions)
}

/// Helper to get position observations for flight path visualization
async fn get_flight_path_by_hex(
    pool: &SqlitePool,
    hex: &str,
    limit: usize,
) -> Result<Vec<FlightPathPoint>, sqlx::Error> {
    let limit = std::cmp::min(limit, 500); // Cap at 500 points for performance

    let observations = sqlx::query_as::<_, FlightPathRecord>(
        r#"
        SELECT ts_ms, lat, lon, altitude
        FROM aircraft_observations
        WHERE hex = ? AND lat IS NOT NULL AND lon IS NOT NULL
        ORDER BY ts_ms ASC
        LIMIT ?
        "#,
    )
    .bind(hex)
    .bind(limit as i64)
    .fetch_all(pool)
    .await?;

    let flight_path = observations
        .into_iter()
        .map(|obs| FlightPathPoint {
            lat: obs.lat,
            lon: obs.lon,
            ts_ms: obs.ts_ms,
            altitude: obs.altitude,
        })
        .collect();

    Ok(flight_path)
}

/// Extract tail number from ICAO hex code (simple mapping for common patterns)
fn extract_tail_number(hex: &str) -> Option<String> {
    // This is a simplified approach - proper tail number extraction
    // would require a comprehensive database lookup
    if hex.len() == 6 {
        // US aircraft typically start with 'A' followed by the hex
        // This is just a placeholder - real implementation would need proper mapping
        hex.strip_prefix('A')
            .map(|stripped| format!("N{}", stripped))
    } else {
        None
    }
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
pub struct SessionRecord {
    pub hex: String,
    #[allow(dead_code)]
    pub first_seen_ms: i64,
    pub last_seen_ms: i64,
    pub message_count: i64,
    pub has_position: bool,
    pub has_altitude: bool,
    pub has_callsign: bool,
    pub flight: Option<String>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub altitude: Option<i32>,
    pub speed: Option<f64>,
}

/// Database record for historical sessions
#[derive(Debug, sqlx::FromRow)]
struct HistoricalSessionRecord {
    #[allow(dead_code)]
    hex: String,
    first_seen_ms: i64,
    last_seen_ms: i64,
    message_count: i64,
    flight: Option<String>,
}

/// Database record for flight path points
#[derive(Debug, sqlx::FromRow)]
pub struct FlightPathRecord {
    pub ts_ms: i64,
    pub lat: f64,
    pub lon: f64,
    pub altitude: Option<i32>,
}

/// Get total number of unique aircraft ever tracked
async fn get_total_aircraft_tracked(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
    let result = sqlx::query_scalar::<_, i64>("SELECT COUNT(DISTINCT hex) FROM aircraft_sessions")
        .fetch_one(pool)
        .await?;

    Ok(result)
}

/// Get number of unique aircraft tracked in the last 24 hours
async fn get_aircraft_tracked_24h(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
    let twenty_four_hours_ago = chrono::Utc::now().timestamp_millis() - (24 * 60 * 60 * 1000);

    let result = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(DISTINCT hex) FROM aircraft_sessions WHERE last_seen_ms > ?",
    )
    .bind(twenty_four_hours_ago)
    .fetch_one(pool)
    .await?;

    Ok(result)
}
