// ABOUTME: Core domain types for aircraft observations, sessions, and anomaly detection
// ABOUTME: Defines the data structures that mirror the database schema with additional computed fields

use serde::{Deserialize, Serialize};
use std::fmt;

/// Aircraft observation from a single ADS-B poll
/// Includes both raw data and computed fields like message rate
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AircraftObservation {
    pub id: Option<i64>,
    pub ts_ms: i64,
    pub hex: String,
    pub flight: Option<String>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub altitude: Option<i32>,
    pub gs: Option<f64>,              // Ground speed in knots
    pub rssi: Option<f64>,            // Signal strength
    pub msg_count_total: Option<i64>, // Total message counter from aircraft
    pub raw_json: String,             // Original JSON for debugging

    // Computed field - not stored in DB
    pub msg_rate_hz: Option<f64>, // Messages per second derived from counter delta
}

impl AircraftObservation {
    /// Validate and normalize hex code
    #[allow(dead_code)] // Will be used in ingestion module
    pub fn normalize_hex(&mut self) -> Result<(), crate::error::Error> {
        self.hex = self.hex.trim().to_uppercase();

        if self.hex.len() != 6 {
            return Err(crate::error::Error::InvalidHex {
                hex: self.hex.clone(),
            });
        }

        // Check for invalid hex patterns
        if matches!(self.hex.as_str(), "000000" | "FFFFFF" | "AAAAAA") {
            return Err(crate::error::Error::InvalidHex {
                hex: self.hex.clone(),
            });
        }

        Ok(())
    }

    /// Normalize flight callsign
    #[allow(dead_code)] // Will be used in ingestion module
    pub fn normalize_flight(&mut self) {
        if let Some(ref mut flight) = self.flight {
            *flight = flight.trim().to_uppercase();
            // Limit to 8 characters as per aviation standards
            flight.truncate(8);
        }
    }
}

/// Aircraft session representing the ongoing presence of an aircraft
/// Tracks capabilities, last known state, and tier support flags
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AircraftSession {
    pub hex: String,
    pub first_seen_ms: i64,
    pub last_seen_ms: i64,
    pub last_msg_total: Option<i64>,
    pub message_count: i64,

    // Data availability flags
    pub has_position: bool,
    pub has_altitude: bool,
    pub has_callsign: bool,

    // Last known values
    pub flight: Option<String>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub altitude: Option<i32>,
    pub speed: Option<f64>,

    // Analysis tier support flags
    pub tier_temporal: bool,   // Can do temporal analysis (message timing)
    pub tier_signal: bool,     // Can do signal analysis (RSSI)
    pub tier_identity: bool,   // Can do identity analysis (hex/callsign)
    pub tier_behavioral: bool, // Can do behavioral analysis (position/movement)
}

impl AircraftSession {
    /// Check if session is complete enough for anomaly detection
    #[allow(dead_code)] // Will be used in analysis module
    pub fn is_complete_for_anomaly_detection(&self) -> bool {
        // At minimum need temporal analysis capability
        self.tier_temporal && self.message_count >= 3
    }

    /// Get session duration in seconds
    #[allow(dead_code)] // Will be used in analysis module
    pub fn duration_seconds(&self) -> f64 {
        (self.last_seen_ms - self.first_seen_ms) as f64 / 1000.0
    }
}

/// Types of anomalies that can be detected
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT")]
#[sqlx(rename_all = "snake_case")]
pub enum AnomalyType {
    /// Temporal anomalies: message timing and frequency
    Temporal,
    /// Signal anomalies: RSSI and RF characteristics
    Signal,
    /// Identity anomalies: hex codes and callsign patterns
    Identity,
    /// Behavioral anomalies: flight patterns and physics violations
    Behavioral,
}

impl fmt::Display for AnomalyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnomalyType::Temporal => write!(f, "temporal"),
            AnomalyType::Signal => write!(f, "signal"),
            AnomalyType::Identity => write!(f, "identity"),
            AnomalyType::Behavioral => write!(f, "behavioral"),
        }
    }
}

impl std::str::FromStr for AnomalyType {
    type Err = crate::error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "temporal" => Ok(AnomalyType::Temporal),
            "signal" => Ok(AnomalyType::Signal),
            "identity" => Ok(AnomalyType::Identity),
            "behavioral" => Ok(AnomalyType::Behavioral),
            _ => Err(crate::error::Error::InvalidAircraftData {
                reason: format!("Invalid anomaly type: {}", s),
            }),
        }
    }
}

/// Candidate anomaly before being committed to database
#[derive(Debug, Clone, PartialEq)]
pub struct AnomalyCandidate {
    pub hex: String,
    pub anomaly_type: AnomalyType,
    pub subtype: String,
    pub confidence: f64,
    pub details: Option<serde_json::Value>,
    pub trigger_observation: Option<AircraftObservation>,
}

impl AnomalyCandidate {
    #[allow(dead_code)] // Will be used in detection modules
    pub fn new(hex: String, anomaly_type: AnomalyType, subtype: String, confidence: f64) -> Self {
        Self {
            hex,
            anomaly_type,
            subtype,
            confidence,
            details: None,
            trigger_observation: None,
        }
    }

    #[allow(dead_code)] // Will be used in detection modules
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    #[allow(dead_code)] // Will be used in detection modules
    pub fn with_trigger_observation(mut self, obs: AircraftObservation) -> Self {
        self.trigger_observation = Some(obs);
        self
    }
}

/// Anomaly detection result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnomalyDetection {
    pub id: Option<i64>,
    pub ts_ms: i64,
    pub hex: String,
    pub anomaly_type: AnomalyType,
    pub confidence: f64,
    pub details_json: Option<String>,
    pub reviewed: bool,
}

impl AnomalyDetection {
    #[allow(dead_code)] // Will be used in detection modules
    pub fn new(
        hex: String,
        anomaly_type: AnomalyType,
        confidence: f64,
        details: Option<serde_json::Value>,
    ) -> Self {
        Self {
            id: None,
            ts_ms: chrono::Utc::now().timestamp_millis(),
            hex,
            anomaly_type,
            confidence,
            details_json: details.map(|d| d.to_string()),
            reviewed: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aircraft_observation_serde_roundtrip() {
        let obs = AircraftObservation {
            id: Some(123),
            ts_ms: 1641024000000,
            hex: "ABC123".to_string(),
            flight: Some("UAL456".to_string()),
            lat: Some(40.7128),
            lon: Some(-74.0060),
            altitude: Some(35000),
            gs: Some(450.0),
            rssi: Some(-45.5),
            msg_count_total: Some(1000),
            raw_json: r#"{"hex":"abc123","flight":"UAL456"}"#.to_string(),
            msg_rate_hz: Some(2.5),
        };

        let json = serde_json::to_string(&obs).unwrap();
        let deserialized: AircraftObservation = serde_json::from_str(&json).unwrap();

        assert_eq!(obs, deserialized);
    }

    #[test]
    fn test_hex_normalization() {
        let mut obs = AircraftObservation {
            id: None,
            ts_ms: 1641024000000,
            hex: " abc123 ".to_string(),
            flight: None,
            lat: None,
            lon: None,
            altitude: None,
            gs: None,
            rssi: None,
            msg_count_total: None,
            raw_json: "{}".to_string(),
            msg_rate_hz: None,
        };

        obs.normalize_hex().unwrap();
        assert_eq!(obs.hex, "ABC123");
    }

    #[test]
    fn test_invalid_hex_patterns() {
        let mut obs = AircraftObservation {
            id: None,
            ts_ms: 1641024000000,
            hex: "000000".to_string(),
            flight: None,
            lat: None,
            lon: None,
            altitude: None,
            gs: None,
            rssi: None,
            msg_count_total: None,
            raw_json: "{}".to_string(),
            msg_rate_hz: None,
        };

        let result = obs.normalize_hex();
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(crate::error::Error::InvalidHex { .. })
        ));
    }

    #[test]
    fn test_flight_normalization() {
        let mut obs = AircraftObservation {
            id: None,
            ts_ms: 1641024000000,
            hex: "ABC123".to_string(),
            flight: Some(" ual456 ".to_string()),
            lat: None,
            lon: None,
            altitude: None,
            gs: None,
            rssi: None,
            msg_count_total: None,
            raw_json: "{}".to_string(),
            msg_rate_hz: None,
        };

        obs.normalize_flight();
        assert_eq!(obs.flight, Some("UAL456".to_string()));
    }

    #[test]
    fn test_anomaly_type_display() {
        assert_eq!(AnomalyType::Temporal.to_string(), "temporal");
        assert_eq!(AnomalyType::Signal.to_string(), "signal");
        assert_eq!(AnomalyType::Identity.to_string(), "identity");
        assert_eq!(AnomalyType::Behavioral.to_string(), "behavioral");
    }

    #[test]
    fn test_anomaly_type_from_str() {
        assert_eq!(
            "temporal".parse::<AnomalyType>().unwrap(),
            AnomalyType::Temporal
        );
        assert_eq!(
            "SIGNAL".parse::<AnomalyType>().unwrap(),
            AnomalyType::Signal
        );
        assert!("invalid".parse::<AnomalyType>().is_err());
    }

    #[test]
    fn test_anomaly_type_serde_roundtrip() {
        let anomaly_type = AnomalyType::Behavioral;
        let json = serde_json::to_string(&anomaly_type).unwrap();
        let deserialized: AnomalyType = serde_json::from_str(&json).unwrap();
        assert_eq!(anomaly_type, deserialized);
        assert_eq!(json, "\"behavioral\"");
    }

    #[test]
    fn test_aircraft_session_completeness() {
        let mut session = AircraftSession {
            hex: "ABC123".to_string(),
            first_seen_ms: 1641024000000,
            last_seen_ms: 1641024010000,
            last_msg_total: Some(100),
            message_count: 5,
            has_position: true,
            has_altitude: true,
            has_callsign: true,
            flight: Some("UAL456".to_string()),
            lat: Some(40.0),
            lon: Some(-74.0),
            altitude: Some(35000),
            speed: Some(450.0),
            tier_temporal: true,
            tier_signal: false,
            tier_identity: false,
            tier_behavioral: false,
        };

        assert!(session.is_complete_for_anomaly_detection());

        session.message_count = 2;
        assert!(!session.is_complete_for_anomaly_detection());

        session.message_count = 5;
        session.tier_temporal = false;
        assert!(!session.is_complete_for_anomaly_detection());
    }

    #[test]
    fn test_session_duration() {
        let session = AircraftSession {
            hex: "ABC123".to_string(),
            first_seen_ms: 1641024000000,
            last_seen_ms: 1641024010000, // 10 seconds later
            last_msg_total: None,
            message_count: 1,
            has_position: false,
            has_altitude: false,
            has_callsign: false,
            flight: None,
            lat: None,
            lon: None,
            altitude: None,
            speed: None,
            tier_temporal: false,
            tier_signal: false,
            tier_identity: false,
            tier_behavioral: false,
        };

        assert_eq!(session.duration_seconds(), 10.0);
    }
}
