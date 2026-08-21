// ABOUTME: Tier 4 Behavioral anomaly detection for physics violations and position jumps
// ABOUTME: Detects impossible aircraft movements violating physical laws like speed limits and teleportation

#![allow(dead_code)]

use crate::config::AnalysisConfig;
use crate::model::{AircraftObservation, AnomalyCandidate, AnomalyType};
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// Aircraft position and timing information for kinematic calculations
#[derive(Debug, Clone)]
pub struct KinematicState {
    pub hex: String,
    pub lat: f64,
    pub lon: f64,
    pub altitude: Option<i32>,
    pub ts_ms: i64,
    pub ground_speed: Option<f64>, // knots
}

impl KinematicState {
    pub fn from_observation(obs: &AircraftObservation) -> Option<Self> {
        // Require position data for kinematic calculations
        let lat = obs.lat?;
        let lon = obs.lon?;

        Some(Self {
            hex: obs.hex.clone(),
            lat,
            lon,
            altitude: obs.altitude,
            ts_ms: obs.ts_ms,
            ground_speed: obs.gs,
        })
    }

    /// Calculate distance in kilometers using Haversine formula
    pub fn distance_km(&self, other: &KinematicState) -> f64 {
        let r = 6371.0; // Earth's radius in km
        let d_lat = (other.lat - self.lat).to_radians();
        let d_lon = (other.lon - self.lon).to_radians();

        let a = (d_lat / 2.0).sin().powi(2)
            + self.lat.to_radians().cos()
                * other.lat.to_radians().cos()
                * (d_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        r * c
    }

    /// Calculate time difference in seconds
    pub fn time_diff_seconds(&self, other: &KinematicState) -> f64 {
        (other.ts_ms - self.ts_ms) as f64 / 1000.0
    }

    /// Calculate ground speed in knots based on position change
    pub fn computed_ground_speed(&self, other: &KinematicState) -> Option<f64> {
        let distance_km = self.distance_km(other);
        let time_seconds = self.time_diff_seconds(other);

        if time_seconds <= 0.0 || !time_seconds.is_finite() {
            return None;
        }

        // Convert km/s to knots (1 knot = 1.852 km/h)
        let speed_kmh = distance_km / time_seconds * 3600.0;
        Some(speed_kmh / 1.852)
    }

    /// Calculate vertical rate in feet per minute
    pub fn vertical_rate_fpm(&self, other: &KinematicState) -> Option<f64> {
        let alt1 = self.altitude? as f64;
        let alt2 = other.altitude? as f64;
        let time_seconds = self.time_diff_seconds(other);

        if time_seconds <= 0.0 || !time_seconds.is_finite() {
            return None;
        }

        // Convert feet/second to feet/minute
        let alt_rate_fps = (alt2 - alt1) / time_seconds;
        Some(alt_rate_fps * 60.0)
    }
}

/// Behavioral anomaly detector for physics violations
#[derive(Debug)]
pub struct BehaviorDetector {
    config: Arc<AnalysisConfig>,
    aircraft_states: HashMap<String, KinematicState>,
    max_ground_speed_knots: f64,
    max_vertical_rate_fpm: f64,
    max_teleport_distance_km: f64,
    min_teleport_time_seconds: f64,
}

impl BehaviorDetector {
    pub fn new(config: Arc<AnalysisConfig>) -> Self {
        Self {
            config,
            aircraft_states: HashMap::new(),
            max_ground_speed_knots: 800.0, // Realistic civilian aircraft max (Mach 1.2 at altitude)
            max_vertical_rate_fpm: 5000.0, // Conservative limit for civilian aircraft anomaly detection
            max_teleport_distance_km: 5.0, // 5km movement in <1s is impossible
            min_teleport_time_seconds: 1.0,
        }
    }

    /// Detect speed violation anomalies
    pub fn detect_speed_violation(
        &self,
        current: &KinematicState,
        previous: &KinematicState,
    ) -> Option<AnomalyCandidate> {
        let computed_speed = previous.computed_ground_speed(current)?;

        if computed_speed > self.max_ground_speed_knots {
            return Some(
                AnomalyCandidate::new(
                    current.hex.clone(),
                    AnomalyType::Behavioral,
                    "physics_violation".to_string(),
                    0.95,
                )
                .with_details(json!({
                    "violation_type": "excessive_speed",
                    "computed_speed_knots": computed_speed,
                    "max_allowed_knots": self.max_ground_speed_knots,
                    "distance_km": previous.distance_km(current),
                    "time_seconds": previous.time_diff_seconds(current),
                    "reason": format!("Computed speed {:.1} kt exceeds maximum {:.1} kt", computed_speed, self.max_ground_speed_knots)
                }))
            );
        }

        None
    }

    /// Detect vertical rate violation anomalies
    pub fn detect_vertical_rate_violation(
        &self,
        current: &KinematicState,
        previous: &KinematicState,
    ) -> Option<AnomalyCandidate> {
        let vertical_rate = previous.vertical_rate_fpm(current)?;

        if vertical_rate.abs() > self.max_vertical_rate_fpm {
            return Some(
                AnomalyCandidate::new(
                    current.hex.clone(),
                    AnomalyType::Behavioral,
                    "physics_violation".to_string(),
                    0.90,
                )
                .with_details(json!({
                    "violation_type": "excessive_vertical_rate",
                    "vertical_rate_fpm": vertical_rate,
                    "max_allowed_fpm": self.max_vertical_rate_fpm,
                    "altitude_change_ft": current.altitude.unwrap_or(0) - previous.altitude.unwrap_or(0),
                    "time_seconds": previous.time_diff_seconds(current),
                    "reason": format!("Vertical rate {:.1} fpm exceeds maximum {:.1} fpm", vertical_rate.abs(), self.max_vertical_rate_fpm)
                }))
            );
        }

        None
    }

    /// Detect position jump (teleportation) anomalies
    pub fn detect_position_jump(
        &self,
        current: &KinematicState,
        previous: &KinematicState,
    ) -> Option<AnomalyCandidate> {
        let distance_km = previous.distance_km(current);
        let time_seconds = previous.time_diff_seconds(current);

        if distance_km > self.max_teleport_distance_km
            && time_seconds < self.min_teleport_time_seconds
        {
            return Some(
                AnomalyCandidate::new(
                    current.hex.clone(),
                    AnomalyType::Behavioral,
                    "position_jump".to_string(),
                    0.98,
                )
                .with_details(json!({
                    "distance_km": distance_km,
                    "time_seconds": time_seconds,
                    "max_allowed_distance_km": self.max_teleport_distance_km,
                    "min_time_seconds": self.min_teleport_time_seconds,
                    "previous_position": [previous.lat, previous.lon],
                    "current_position": [current.lat, current.lon],
                    "reason": format!("Position jump {:.2} km in {:.2} seconds exceeds teleport threshold", distance_km, time_seconds)
                }))
            );
        }

        None
    }

    /// Update aircraft state and detect behavioral anomalies
    pub fn update_and_detect(&mut self, obs: &AircraftObservation) -> Vec<AnomalyCandidate> {
        let current_state = match KinematicState::from_observation(obs) {
            Some(state) => state,
            None => return Vec::new(), // Need position data for behavioral analysis
        };

        let mut anomalies = Vec::new();

        // Check for anomalies against previous state
        if let Some(previous_state) = self.aircraft_states.get(&obs.hex) {
            // Check speed violation
            if let Some(anomaly) = self.detect_speed_violation(&current_state, previous_state) {
                anomalies.push(anomaly);
            }

            // Check vertical rate violation
            if let Some(anomaly) =
                self.detect_vertical_rate_violation(&current_state, previous_state)
            {
                anomalies.push(anomaly);
            }

            // Check position jump
            if let Some(anomaly) = self.detect_position_jump(&current_state, previous_state) {
                anomalies.push(anomaly);
            }
        }

        // Update state for future comparisons
        self.aircraft_states.insert(obs.hex.clone(), current_state);

        anomalies
    }

    /// Clean up old aircraft states to prevent memory growth
    pub fn cleanup_old_states(&mut self, current_time_ms: i64, max_age_ms: i64) {
        let cutoff_time = current_time_ms - max_age_ms;
        let initial_count = self.aircraft_states.len();

        self.aircraft_states
            .retain(|_, state| state.ts_ms >= cutoff_time);

        let removed_count = initial_count - self.aircraft_states.len();
        if removed_count > 0 {
            tracing::debug!("Cleaned up {} old behavioral states", removed_count);
        }
    }

    /// Get current state count for monitoring
    pub fn get_state_count(&self) -> usize {
        self.aircraft_states.len()
    }

    /// Get state for specific aircraft (for testing)
    pub fn get_aircraft_state(&self, hex: &str) -> Option<&KinematicState> {
        self.aircraft_states.get(hex)
    }
}

/// Service for behavioral anomaly detection that processes observations and sends alerts
pub struct BehaviorDetectionService {
    detector: Arc<Mutex<BehaviorDetector>>,
    alert_sender: mpsc::UnboundedSender<AnomalyCandidate>,
}

impl BehaviorDetectionService {
    /// Create new behavioral detection service
    pub fn new(
        config: Arc<AnalysisConfig>,
        alert_sender: mpsc::UnboundedSender<AnomalyCandidate>,
    ) -> Self {
        let detector = BehaviorDetector::new(config);
        Self {
            detector: Arc::new(Mutex::new(detector)),
            alert_sender,
        }
    }

    /// Process an observation for behavioral anomalies
    pub fn process_observation(&self, obs: AircraftObservation) {
        let mut detector = self.detector.lock().unwrap();
        let anomalies = detector.update_and_detect(&obs);

        // Send all detected anomalies through the alert channel
        for anomaly in anomalies {
            if self.alert_sender.send(anomaly).is_err() {
                tracing::warn!("Failed to send behavioral anomaly alert: channel closed");
            }
        }
    }

    /// Clean up old states
    pub fn cleanup_old_states(&self, current_time_ms: i64, max_age_ms: i64) {
        let mut detector = self.detector.lock().unwrap();
        detector.cleanup_old_states(current_time_ms, max_age_ms);
    }

    /// Get current state count for monitoring
    pub fn get_state_count(&self) -> usize {
        let detector = self.detector.lock().unwrap();
        detector.get_state_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AnalysisConfig;

    fn create_test_config() -> Arc<AnalysisConfig> {
        Arc::new(AnalysisConfig {
            max_messages_per_second: 10.0,
            min_message_interval_ms: 50,
            max_session_gap_seconds: 600,
            min_rssi_units: -120.0,
            max_rssi_units: -10.0,
            suspicious_rssi_units: -20.0,
            suspicious_callsigns: vec![],
            invalid_hex_patterns: vec![],
        })
    }

    fn create_test_observation(
        hex: &str,
        lat: f64,
        lon: f64,
        altitude: Option<i32>,
        ts_ms: i64,
        gs: Option<f64>,
    ) -> AircraftObservation {
        AircraftObservation {
            id: None,
            ts_ms,
            hex: hex.to_string(),
            flight: Some("TEST123".to_string()),
            lat: Some(lat),
            lon: Some(lon),
            altitude,
            gs,
            rssi: Some(-45.0),
            msg_count_total: Some(1000),
            raw_json: format!(r#"{{"hex":"{}"}}"#, hex),
            msg_rate_hz: Some(5.0),
        }
    }

    #[test]
    fn test_kinematic_state_creation() {
        let obs = create_test_observation(
            "ABC123",
            40.7,
            -74.0,
            Some(35000),
            1641024000000,
            Some(450.0),
        );
        let state = KinematicState::from_observation(&obs).unwrap();

        assert_eq!(state.hex, "ABC123");
        assert_eq!(state.lat, 40.7);
        assert_eq!(state.lon, -74.0);
        assert_eq!(state.altitude, Some(35000));
        assert_eq!(state.ts_ms, 1641024000000);
        assert_eq!(state.ground_speed, Some(450.0));
    }

    #[test]
    fn test_kinematic_state_requires_position() {
        // Missing lat/lon should return None
        let obs = AircraftObservation {
            id: None,
            ts_ms: 1641024000000,
            hex: "ABC123".to_string(),
            flight: Some("TEST123".to_string()),
            lat: None, // Missing
            lon: None, // Missing
            altitude: Some(35000),
            gs: Some(450.0),
            rssi: Some(-45.0),
            msg_count_total: Some(1000),
            raw_json: r#"{"hex":"ABC123"}"#.to_string(),
            msg_rate_hz: Some(5.0),
        };

        let state = KinematicState::from_observation(&obs);
        assert!(state.is_none());
    }

    #[test]
    fn test_haversine_distance_calculation() {
        let state1 = KinematicState {
            hex: "TEST".to_string(),
            lat: 40.7128, // NYC
            lon: -74.0060,
            altitude: Some(35000),
            ts_ms: 1641024000000,
            ground_speed: Some(450.0),
        };

        let state2 = KinematicState {
            hex: "TEST".to_string(),
            lat: 34.0522, // LAX
            lon: -118.2437,
            altitude: Some(35000),
            ts_ms: 1641024000000,
            ground_speed: Some(450.0),
        };

        let distance = state1.distance_km(&state2);
        // NYC to LAX is approximately 3944 km
        assert!(distance > 3900.0 && distance < 4000.0);
    }

    #[test]
    fn test_computed_ground_speed() {
        let state1 = KinematicState {
            hex: "TEST".to_string(),
            lat: 40.0,
            lon: -74.0,
            altitude: Some(35000),
            ts_ms: 1641024000000, // 1 second
            ground_speed: Some(450.0),
        };

        let state2 = KinematicState {
            hex: "TEST".to_string(),
            lat: 40.01, // About 1.1 km north
            lon: -74.0,
            altitude: Some(35000),
            ts_ms: 1641024001000, // 1 second later
            ground_speed: Some(450.0),
        };

        let speed = state1.computed_ground_speed(&state2).unwrap();
        // ~1.1 km in 1 second = ~3960 km/h = ~2138 knots (way too fast!)
        assert!(speed > 2000.0);
    }

    #[test]
    fn test_vertical_rate_calculation() {
        let state1 = KinematicState {
            hex: "TEST".to_string(),
            lat: 40.0,
            lon: -74.0,
            altitude: Some(35000),
            ts_ms: 1641024000000,
            ground_speed: Some(450.0),
        };

        let state2 = KinematicState {
            hex: "TEST".to_string(),
            lat: 40.0,
            lon: -74.0,
            altitude: Some(36000), // 1000 ft climb
            ts_ms: 1641024060000,  // 60 seconds later
            ground_speed: Some(450.0),
        };

        let vertical_rate = state1.vertical_rate_fpm(&state2).unwrap();
        // 1000 ft in 60 seconds = 1000 fpm
        assert!((vertical_rate - 1000.0).abs() < 0.1);
    }

    #[test]
    fn test_detect_speed_violation() {
        let config = create_test_config();
        let detector = BehaviorDetector::new(config);

        let state1 = KinematicState {
            hex: "FAST123".to_string(),
            lat: 40.0,
            lon: -74.0,
            altitude: Some(35000),
            ts_ms: 1641024000000,
            ground_speed: Some(450.0),
        };

        let state2 = KinematicState {
            hex: "FAST123".to_string(),
            lat: 40.1, // About 11 km north - way too fast for 1 second
            lon: -74.0,
            altitude: Some(35000),
            ts_ms: 1641024001000, // 1 second later
            ground_speed: Some(450.0),
        };

        let anomaly = detector.detect_speed_violation(&state2, &state1);
        assert!(anomaly.is_some());
        let anomaly = anomaly.unwrap();
        assert_eq!(anomaly.subtype, "physics_violation");
        assert_eq!(anomaly.confidence, 0.95);
        assert!(anomaly
            .details
            .unwrap()
            .get("computed_speed_knots")
            .is_some());
    }

    #[test]
    fn test_detect_vertical_rate_violation() {
        let config = create_test_config();
        let detector = BehaviorDetector::new(config);

        let state1 = KinematicState {
            hex: "CLIMB123".to_string(),
            lat: 40.0,
            lon: -74.0,
            altitude: Some(35000),
            ts_ms: 1641024000000,
            ground_speed: Some(450.0),
        };

        let state2 = KinematicState {
            hex: "CLIMB123".to_string(),
            lat: 40.0,
            lon: -74.0,
            altitude: Some(45000), // 10,000 ft climb in 60 seconds = 10,000 fpm (too fast!)
            ts_ms: 1641024060000,  // 60 seconds later
            ground_speed: Some(450.0),
        };

        let anomaly = detector.detect_vertical_rate_violation(&state2, &state1);
        assert!(anomaly.is_some());
        let anomaly = anomaly.unwrap();
        assert_eq!(anomaly.subtype, "physics_violation");
        assert_eq!(anomaly.confidence, 0.90);
        assert!(anomaly.details.unwrap().get("vertical_rate_fpm").is_some());
    }

    #[test]
    fn test_detect_position_jump() {
        let config = create_test_config();
        let detector = BehaviorDetector::new(config);

        let state1 = KinematicState {
            hex: "TELEPORT".to_string(),
            lat: 40.7, // NYC
            lon: -74.0,
            altitude: Some(35000),
            ts_ms: 1641024000000,
            ground_speed: Some(450.0),
        };

        let state2 = KinematicState {
            hex: "TELEPORT".to_string(),
            lat: 40.75, // 5+ km away
            lon: -74.1,
            altitude: Some(35000),
            ts_ms: 1641024000500, // 0.5 seconds later (teleport!)
            ground_speed: Some(450.0),
        };

        let anomaly = detector.detect_position_jump(&state2, &state1);
        assert!(anomaly.is_some());
        let anomaly = anomaly.unwrap();
        assert_eq!(anomaly.subtype, "position_jump");
        assert_eq!(anomaly.confidence, 0.98);
        assert!(anomaly.details.unwrap().get("distance_km").is_some());
    }

    #[test]
    fn test_normal_flight_no_violations() {
        let config = create_test_config();
        let mut detector = BehaviorDetector::new(config);

        // Normal commercial flight at 500 knots, 2000 fpm climb
        let obs1 = create_test_observation(
            "NORMAL",
            40.0,
            -74.0,
            Some(35000),
            1641024000000,
            Some(500.0),
        );
        let obs2 = create_test_observation(
            "NORMAL",
            40.002,
            -74.002,
            Some(35200),
            1641024060000,
            Some(500.0),
        ); // 60 seconds later, small movement

        // Process first observation (establishes state)
        let anomalies1 = detector.update_and_detect(&obs1);
        assert!(anomalies1.is_empty());

        // Process second observation (should be normal)
        let anomalies2 = detector.update_and_detect(&obs2);
        assert!(anomalies2.is_empty());
    }

    #[test]
    fn test_multiple_violations_same_observation() {
        let config = create_test_config();
        let mut detector = BehaviorDetector::new(config);

        // Establish initial state
        let obs1 = create_test_observation(
            "MULTI",
            40.0,
            -74.0,
            Some(35000),
            1641024000000,
            Some(500.0),
        );
        detector.update_and_detect(&obs1);

        // Violation: teleport + excessive speed + excessive climb
        let obs2 = create_test_observation(
            "MULTI",
            40.1,
            -74.1,
            Some(45000),
            1641024000500,
            Some(500.0),
        ); // 0.5s later
        let anomalies = detector.update_and_detect(&obs2);

        // Should detect multiple violations
        assert!(anomalies.len() >= 2); // At minimum speed and position violations

        let subtypes: Vec<&str> = anomalies.iter().map(|a| a.subtype.as_str()).collect();
        assert!(subtypes.contains(&"physics_violation") || subtypes.contains(&"position_jump"));
    }

    #[test]
    fn test_cleanup_old_states() {
        let config = create_test_config();
        let mut detector = BehaviorDetector::new(config);

        // Add old state
        let old_obs =
            create_test_observation("OLD123", 40.0, -74.0, Some(35000), 1000, Some(500.0));
        detector.update_and_detect(&old_obs);

        // Add current state
        let current_obs =
            create_test_observation("NEW456", 40.1, -74.1, Some(36000), 120000, Some(500.0));
        detector.update_and_detect(&current_obs);

        assert_eq!(detector.get_state_count(), 2);

        // Clean up states older than 60 seconds from current time
        detector.cleanup_old_states(120000, 60000);

        // Should have removed the old state, kept the new one
        assert_eq!(detector.get_state_count(), 1);
        assert!(detector.get_aircraft_state("NEW456").is_some());
        assert!(detector.get_aircraft_state("OLD123").is_none());
    }

    #[tokio::test]
    async fn test_behavior_detection_service() {
        let config = create_test_config();
        let (alert_sender, mut alert_receiver) = mpsc::unbounded_channel();

        let service = BehaviorDetectionService::new(config, alert_sender);

        // Establish initial state
        let obs1 = create_test_observation(
            "SERVICE",
            40.0,
            -74.0,
            Some(35000),
            1641024000000,
            Some(500.0),
        );
        service.process_observation(obs1);

        // Trigger violation
        let obs2 = create_test_observation(
            "SERVICE",
            40.1,
            -74.1,
            Some(35000),
            1641024000500,
            Some(500.0),
        ); // Teleport
        service.process_observation(obs2);

        // Should receive alert
        let alert = alert_receiver
            .try_recv()
            .expect("Should receive behavioral alert");
        assert_eq!(alert.hex, "SERVICE");
        assert_eq!(alert.anomaly_type, AnomalyType::Behavioral);
    }

    #[test]
    fn test_no_violation_without_position_data() {
        let config = create_test_config();
        let mut detector = BehaviorDetector::new(config);

        // Observation without position data
        let obs = AircraftObservation {
            id: None,
            ts_ms: 1641024000000,
            hex: "NOPOS".to_string(),
            flight: Some("NOPOS123".to_string()),
            lat: None,
            lon: None,
            altitude: Some(35000),
            gs: Some(450.0),
            rssi: Some(-45.0),
            msg_count_total: Some(1000),
            raw_json: r#"{"hex":"NOPOS"}"#.to_string(),
            msg_rate_hz: Some(5.0),
        };

        let anomalies = detector.update_and_detect(&obs);
        assert!(anomalies.is_empty()); // Can't detect behavioral anomalies without position
    }
}
