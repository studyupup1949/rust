// ABOUTME: PiAware/dump1090-fa aircraft.json client with robust JSON parsing
// ABOUTME: Handles sparse and inconsistent ADS-B data with all-optional fields

use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Snapshot from PiAware aircraft.json endpoint
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AircraftSnapshot {
    /// Server timestamp when snapshot was generated
    pub now: Option<f64>,
    /// Total messages processed by dump1090
    pub messages: Option<u64>,
    /// List of aircraft currently visible
    pub aircraft: Vec<AircraftEntry>,
}

/// Individual aircraft entry from PiAware data
/// All fields are optional since ADS-B data can be sparse
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AircraftEntry {
    /// ICAO hex identifier (always present if aircraft is listed)
    pub hex: Option<String>,
    /// Flight callsign/number
    pub flight: Option<String>,
    /// Latitude in decimal degrees
    pub lat: Option<f64>,
    /// Longitude in decimal degrees
    pub lon: Option<f64>,
    /// Barometric altitude in feet
    pub alt_baro: Option<i32>,
    /// Geometric altitude in feet
    pub alt_geom: Option<i32>,
    /// Ground speed in knots
    pub gs: Option<f64>,
    /// Signal strength in dBFS (not dBm)
    pub rssi: Option<f64>,
    /// Total messages received from this aircraft
    pub messages: Option<i64>,
    /// Seconds since last message
    pub seen: Option<f64>,
    /// Seconds since last position message
    pub seen_pos: Option<f64>,
    /// Track/heading in degrees
    pub track: Option<f64>,
    /// Mode A squawk code
    pub squawk: Option<String>,
    /// Aircraft category
    pub category: Option<String>,
}

/// Fetch aircraft snapshot from PiAware/dump1090-fa
#[allow(dead_code)] // Will be used in ingestion service (later prompts)
pub async fn fetch_snapshot(url: &str) -> Result<AircraftSnapshot> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        return Err(crate::error::Error::Generic(format!(
            "HTTP error: {} - {}",
            response.status().as_u16(),
            response.status().canonical_reason().unwrap_or("Unknown")
        )));
    }

    let snapshot: AircraftSnapshot = response.json().await?;
    Ok(snapshot)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{extract::Path, response::Json, routing::get, Router};
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn test_deserialize_aircraft_snapshot_minimal() {
        let json = r#"
        {
            "aircraft": []
        }
        "#;

        let snapshot: AircraftSnapshot = serde_json::from_str(json).unwrap();
        assert_eq!(snapshot.now, None);
        assert_eq!(snapshot.messages, None);
        assert_eq!(snapshot.aircraft.len(), 0);
    }

    #[tokio::test]
    async fn test_deserialize_aircraft_snapshot_complete() {
        let json = include_str!("../../tests/data/aircraft1.json");

        let snapshot: AircraftSnapshot = serde_json::from_str(json).unwrap();
        assert_eq!(snapshot.now, Some(1641024000.5));
        assert_eq!(snapshot.messages, Some(12345));
        assert_eq!(snapshot.aircraft.len(), 2);

        // Check first aircraft (complete data)
        let aircraft1 = &snapshot.aircraft[0];
        assert_eq!(aircraft1.hex, Some("abc123".to_string()));
        assert_eq!(aircraft1.flight, Some("UAL456".to_string()));
        assert_eq!(aircraft1.lat, Some(40.7128));
        assert_eq!(aircraft1.lon, Some(-74.0060));
        assert_eq!(aircraft1.alt_baro, Some(35000));
        assert_eq!(aircraft1.gs, Some(450.2));
        assert_eq!(aircraft1.rssi, Some(-45.5));
        assert_eq!(aircraft1.messages, Some(1000));
        assert_eq!(aircraft1.seen, Some(0.1));
        assert_eq!(aircraft1.track, Some(90.0));
        assert_eq!(aircraft1.squawk, Some("1200".to_string()));

        // Check second aircraft (sparse data)
        let aircraft2 = &snapshot.aircraft[1];
        assert_eq!(aircraft2.hex, Some("def456".to_string()));
        assert_eq!(aircraft2.flight, None);
        assert_eq!(aircraft2.lat, None);
        assert_eq!(aircraft2.lon, None);
        assert_eq!(aircraft2.alt_geom, Some(28000));
        assert_eq!(aircraft2.gs, Some(380.0));
        assert_eq!(aircraft2.rssi, Some(-52.1));
    }

    #[tokio::test]
    async fn test_aircraft_entry_missing_fields() {
        let json = r#"
        {
            "hex": "abc123"
        }
        "#;

        let entry: AircraftEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.hex, Some("abc123".to_string()));
        assert_eq!(entry.flight, None);
        assert_eq!(entry.lat, None);
        assert_eq!(entry.lon, None);
        assert_eq!(entry.alt_baro, None);
        assert_eq!(entry.gs, None);
        assert_eq!(entry.rssi, None);
        assert_eq!(entry.messages, None);
    }

    async fn test_server_handler(Path(file): Path<String>) -> Json<serde_json::Value> {
        let content = match file.as_str() {
            "aircraft1.json" => include_str!("../../tests/data/aircraft1.json"),
            "aircraft2.json" => include_str!("../../tests/data/aircraft2.json"),
            _ => r#"{"aircraft": []}"#,
        };

        let value: serde_json::Value = serde_json::from_str(content).unwrap();
        Json(value)
    }

    #[tokio::test]
    async fn test_fetch_snapshot_integration() {
        // Start test HTTP server
        let app = Router::new().route("/data/:file", get(test_server_handler));

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        // Give server time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Test fetch
        let url = format!("http://127.0.0.1:{}/data/aircraft1.json", addr.port());
        let snapshot = fetch_snapshot(&url).await.unwrap();

        assert_eq!(snapshot.aircraft.len(), 2);
        assert_eq!(snapshot.messages, Some(12345));
        assert_eq!(snapshot.aircraft[0].hex, Some("abc123".to_string()));
    }

    #[tokio::test]
    async fn test_fetch_snapshot_http_error() {
        // Try to fetch from non-existent server
        let result = fetch_snapshot("http://127.0.0.1:9999/nonexistent").await;
        assert!(result.is_err());
    }
}
