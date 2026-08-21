// ABOUTME: Tier 3 Identity anomaly detection for suspicious callsigns, invalid hex patterns, and hex duplicates
// ABOUTME: Detects spoofed or malicious aircraft identities using regex patterns and network signature analysis

#![allow(dead_code)]

use crate::config::AnalysisConfig;
use crate::model::{AircraftObservation, AnomalyCandidate, AnomalyType};
use regex::Regex;
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// Network signature for detecting duplicate hex codes
#[derive(Debug, Clone)]
pub struct NetworkSignature {
    pub hex: String,
    pub first_rssi: Option<f64>,
    pub first_position: Option<(f64, f64)>, // lat, lon
    pub first_seen_ms: i64,
    pub rssi_samples: Vec<f64>,
    pub position_samples: Vec<(f64, f64)>,
}

impl NetworkSignature {
    pub fn new(obs: &AircraftObservation) -> Self {
        let mut signature = Self {
            hex: obs.hex.clone(),
            first_rssi: obs.rssi,
            first_position: obs.lat.and_then(|lat| obs.lon.map(|lon| (lat, lon))),
            first_seen_ms: obs.ts_ms,
            rssi_samples: Vec::new(),
            position_samples: Vec::new(),
        };
        signature.update(obs);
        signature
    }

    pub fn update(&mut self, obs: &AircraftObservation) {
        if let Some(rssi) = obs.rssi {
            self.rssi_samples.push(rssi);
        }
        if let (Some(lat), Some(lon)) = (obs.lat, obs.lon) {
            self.position_samples.push((lat, lon));
        }
    }

    /// Calculate average RSSI if samples available
    pub fn avg_rssi(&self) -> Option<f64> {
        if self.rssi_samples.is_empty() {
            None
        } else {
            Some(self.rssi_samples.iter().sum::<f64>() / self.rssi_samples.len() as f64)
        }
    }

    /// Calculate centroid of positions if samples available
    pub fn centroid_position(&self) -> Option<(f64, f64)> {
        if self.position_samples.is_empty() {
            None
        } else {
            let sum_lat: f64 = self.position_samples.iter().map(|(lat, _)| lat).sum();
            let sum_lon: f64 = self.position_samples.iter().map(|(_, lon)| lon).sum();
            let count = self.position_samples.len() as f64;
            Some((sum_lat / count, sum_lon / count))
        }
    }

    /// Calculate distance between two positions using Haversine formula (in km)
    pub fn haversine_distance(pos1: (f64, f64), pos2: (f64, f64)) -> f64 {
        let (lat1, lon1) = pos1;
        let (lat2, lon2) = pos2;

        let r = 6371.0; // Earth's radius in km
        let d_lat = (lat2 - lat1).to_radians();
        let d_lon = (lon2 - lon1).to_radians();

        let a = (d_lat / 2.0).sin().powi(2)
            + lat1.to_radians().cos() * lat2.to_radians().cos() * (d_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        r * c
    }

    /// Check if this signature conflicts with another (for duplicate detection)
    pub fn conflicts_with(&self, other: &NetworkSignature) -> Option<String> {
        // Must be same hex
        if self.hex != other.hex {
            return None;
        }

        // Check RSSI divergence (>10dB difference in averages)
        if let (Some(rssi1), Some(rssi2)) = (self.avg_rssi(), other.avg_rssi()) {
            if (rssi1 - rssi2).abs() > 10.0 {
                return Some(format!("RSSI divergence: {:.1} vs {:.1} dB", rssi1, rssi2));
            }
        }

        // Check position divergence (>50km between centroids)
        if let (Some(pos1), Some(pos2)) = (self.centroid_position(), other.centroid_position()) {
            let distance = Self::haversine_distance(pos1, pos2);
            if distance > 50.0 {
                return Some(format!("Position divergence: {:.1} km apart", distance));
            }
        }

        None
    }
}

/// Identity anomaly detector
#[derive(Debug)]
pub struct IdentityDetector {
    config: Arc<AnalysisConfig>,
    callsign_patterns: Vec<Regex>,
    hex_patterns: Vec<Regex>,
    network_signatures: HashMap<String, NetworkSignature>,
    duplicate_detection_window_ms: i64,
}

impl IdentityDetector {
    pub fn new(config: Arc<AnalysisConfig>) -> Self {
        // Compile regex patterns from config
        let callsign_patterns = config
            .suspicious_callsigns
            .iter()
            .filter_map(|pattern| match Regex::new(pattern) {
                Ok(regex) => Some(regex),
                Err(e) => {
                    tracing::warn!("Invalid callsign regex '{}': {}", pattern, e);
                    None
                }
            })
            .collect();

        let hex_patterns = config
            .invalid_hex_patterns
            .iter()
            .filter_map(|pattern| match Regex::new(pattern) {
                Ok(regex) => Some(regex),
                Err(e) => {
                    tracing::warn!("Invalid hex regex '{}': {}", pattern, e);
                    None
                }
            })
            .collect();

        Self {
            config,
            callsign_patterns,
            hex_patterns,
            network_signatures: HashMap::new(),
            duplicate_detection_window_ms: 60 * 1000, // 60 seconds
        }
    }

    /// Check for suspicious callsign patterns
    pub fn detect_suspicious_callsign(
        &self,
        obs: &AircraftObservation,
    ) -> Option<AnomalyCandidate> {
        let flight = obs.flight.as_ref()?;

        for pattern in &self.callsign_patterns {
            if pattern.is_match(flight) {
                return Some(
                    AnomalyCandidate::new(
                        obs.hex.clone(),
                        AnomalyType::Identity,
                        "suspicious_callsign".to_string(),
                        0.9,
                    )
                    .with_details(json!({
                        "callsign": flight,
                        "pattern": pattern.as_str(),
                        "reason": "Callsign matches suspicious pattern"
                    })),
                );
            }
        }

        None
    }

    /// Check for invalid hex patterns
    pub fn detect_invalid_hex(&self, obs: &AircraftObservation) -> Option<AnomalyCandidate> {
        for pattern in &self.hex_patterns {
            if pattern.is_match(&obs.hex) {
                return Some(
                    AnomalyCandidate::new(
                        obs.hex.clone(),
                        AnomalyType::Identity,
                        "invalid_hex".to_string(),
                        0.95,
                    )
                    .with_details(json!({
                        "hex": obs.hex,
                        "pattern": pattern.as_str(),
                        "reason": "Hex code matches invalid pattern"
                    })),
                );
            }
        }

        None
    }

    /// Update network signature and check for hex duplicates
    pub fn update_and_detect_hex_duplicate(
        &mut self,
        obs: &AircraftObservation,
    ) -> Option<AnomalyCandidate> {
        // Clean old signatures first
        self.cleanup_old_signatures(obs.ts_ms);

        // Create new signature for comparison
        let new_sig = NetworkSignature::new(obs);

        // Check if we already have a signature for this exact hex
        if let Some(existing_sig) = self.network_signatures.get(&obs.hex) {
            // Check if this is a conflicting signature for the same hex
            if let Some(conflict_reason) = new_sig.conflicts_with(existing_sig) {
                // Found same hex with conflicting network signature
                let anomaly = AnomalyCandidate::new(
                    obs.hex.clone(),
                    AnomalyType::Identity,
                    "hex_duplicate".to_string(),
                    0.85,
                )
                .with_details(json!({
                    "hex": obs.hex,
                    "conflict_reason": conflict_reason,
                    "new_first_seen": new_sig.first_seen_ms,
                    "existing_first_seen": existing_sig.first_seen_ms,
                    "reason": "Same hex code with conflicting network signatures"
                }));

                // Update the signature with new data
                self.network_signatures
                    .get_mut(&obs.hex)
                    .unwrap()
                    .update(obs);
                return Some(anomaly);
            } else {
                // No conflict, just update the existing signature
                self.network_signatures
                    .get_mut(&obs.hex)
                    .unwrap()
                    .update(obs);
                return None;
            }
        }

        // No existing signature for this hex, store the new one
        self.network_signatures.insert(obs.hex.clone(), new_sig);
        None
    }

    /// Run all identity detections on an observation
    pub fn detect_all(&mut self, obs: &AircraftObservation) -> Vec<AnomalyCandidate> {
        let mut anomalies = Vec::new();

        // Check suspicious callsign
        if let Some(anomaly) = self.detect_suspicious_callsign(obs) {
            anomalies.push(anomaly);
        }

        // Check invalid hex
        if let Some(anomaly) = self.detect_invalid_hex(obs) {
            anomalies.push(anomaly);
        }

        // Check hex duplicate (updates internal state)
        if let Some(anomaly) = self.update_and_detect_hex_duplicate(obs) {
            anomalies.push(anomaly);
        }

        anomalies
    }

    /// Clean up old network signatures outside the detection window
    fn cleanup_old_signatures(&mut self, current_time_ms: i64) {
        let cutoff_time = current_time_ms - self.duplicate_detection_window_ms;
        let initial_count = self.network_signatures.len();

        self.network_signatures
            .retain(|_, sig| sig.first_seen_ms >= cutoff_time);

        let removed_count = initial_count - self.network_signatures.len();
        if removed_count > 0 {
            tracing::debug!("Cleaned up {} old network signatures", removed_count);
        }
    }

    /// Get current signature count for monitoring
    pub fn get_signature_count(&self) -> usize {
        self.network_signatures.len()
    }

    /// Get signature for specific hex (for testing)
    pub fn get_signature(&self, hex: &str) -> Option<&NetworkSignature> {
        self.network_signatures.get(hex)
    }
}

/// Service for identity anomaly detection that processes observations and sends alerts
pub struct IdentityDetectionService {
    detector: Arc<Mutex<IdentityDetector>>,
    alert_sender: mpsc::UnboundedSender<AnomalyCandidate>,
}

impl IdentityDetectionService {
    /// Create new identity detection service
    pub fn new(
        config: Arc<AnalysisConfig>,
        alert_sender: mpsc::UnboundedSender<AnomalyCandidate>,
    ) -> Self {
        let detector = IdentityDetector::new(config);
        Self {
            detector: Arc::new(Mutex::new(detector)),
            alert_sender,
        }
    }

    /// Process an observation for identity anomalies
    pub fn process_observation(&self, obs: AircraftObservation) {
        let mut detector = self.detector.lock().unwrap();
        let anomalies = detector.detect_all(&obs);

        // Send all detected anomalies through the alert channel
        for anomaly in anomalies {
            if self.alert_sender.send(anomaly).is_err() {
                tracing::warn!("Failed to send identity anomaly alert: channel closed");
            }
        }
    }

    /// Get current signature count for monitoring
    pub fn get_signature_count(&self) -> usize {
        let detector = self.detector.lock().unwrap();
        detector.get_signature_count()
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
            suspicious_callsigns: vec!["TEST.*".to_string(), "FAKE.*".to_string()],
            invalid_hex_patterns: vec!["000000".to_string(), "FFFFFF".to_string()],
        })
    }

    fn create_test_observation(
        hex: &str,
        flight: Option<&str>,
        rssi: Option<f64>,
        pos: Option<(f64, f64)>,
    ) -> AircraftObservation {
        AircraftObservation {
            id: None,
            ts_ms: 1641024000000, // Fixed timestamp for tests
            hex: hex.to_string(),
            flight: flight.map(|s| s.to_string()),
            lat: pos.map(|(lat, _)| lat),
            lon: pos.map(|(_, lon)| lon),
            altitude: Some(35000),
            gs: Some(450.0),
            rssi,
            msg_count_total: Some(1000),
            raw_json: format!(r#"{{"hex":"{}"}}"#, hex),
            msg_rate_hz: Some(5.0),
        }
    }

    #[test]
    fn test_detect_suspicious_callsign() {
        let config = create_test_config();
        let detector = IdentityDetector::new(config);

        // Test matching suspicious callsign
        let obs = create_test_observation("ABC123", Some("TEST123"), Some(-45.0), None);
        let anomaly = detector.detect_suspicious_callsign(&obs);
        assert!(anomaly.is_some());
        let anomaly = anomaly.unwrap();
        assert_eq!(anomaly.subtype, "suspicious_callsign");
        assert_eq!(anomaly.confidence, 0.9);
        assert!(anomaly.details.unwrap().get("callsign").is_some());

        // Test non-matching callsign
        let obs = create_test_observation("ABC123", Some("UAL123"), Some(-45.0), None);
        let anomaly = detector.detect_suspicious_callsign(&obs);
        assert!(anomaly.is_none());

        // Test missing callsign
        let obs = create_test_observation("ABC123", None, Some(-45.0), None);
        let anomaly = detector.detect_suspicious_callsign(&obs);
        assert!(anomaly.is_none());
    }

    #[test]
    fn test_detect_invalid_hex() {
        let config = create_test_config();
        let detector = IdentityDetector::new(config);

        // Test invalid hex pattern
        let obs = create_test_observation("000000", Some("TEST123"), Some(-45.0), None);
        let anomaly = detector.detect_invalid_hex(&obs);
        assert!(anomaly.is_some());
        let anomaly = anomaly.unwrap();
        assert_eq!(anomaly.subtype, "invalid_hex");
        assert_eq!(anomaly.confidence, 0.95);

        // Test valid hex
        let obs = create_test_observation("ABC123", Some("TEST123"), Some(-45.0), None);
        let anomaly = detector.detect_invalid_hex(&obs);
        assert!(anomaly.is_none());
    }

    #[test]
    fn test_network_signature_creation() {
        let obs =
            create_test_observation("ABC123", Some("UAL123"), Some(-45.0), Some((40.7, -74.0)));
        let signature = NetworkSignature::new(&obs);

        assert_eq!(signature.hex, "ABC123");
        assert_eq!(signature.first_rssi, Some(-45.0));
        assert_eq!(signature.first_position, Some((40.7, -74.0)));
        assert_eq!(signature.rssi_samples.len(), 1);
        assert_eq!(signature.position_samples.len(), 1);
    }

    #[test]
    fn test_network_signature_update() {
        let obs1 =
            create_test_observation("ABC123", Some("UAL123"), Some(-45.0), Some((40.7, -74.0)));
        let mut signature = NetworkSignature::new(&obs1);

        let obs2 =
            create_test_observation("ABC123", Some("UAL123"), Some(-46.0), Some((40.8, -74.1)));
        signature.update(&obs2);

        assert_eq!(signature.rssi_samples.len(), 2);
        assert_eq!(signature.position_samples.len(), 2);
        assert_eq!(signature.avg_rssi(), Some(-45.5));

        let centroid = signature.centroid_position().unwrap();
        assert!((centroid.0 - 40.75).abs() < 0.01); // Average lat
        assert!((centroid.1 - (-74.05)).abs() < 0.01); // Average lon
    }

    #[test]
    fn test_haversine_distance() {
        // New York to Los Angeles (approximately 3,944 km)
        let nyc = (40.7128, -74.0060);
        let lax = (34.0522, -118.2437);
        let distance = NetworkSignature::haversine_distance(nyc, lax);

        // Should be approximately 3944 km
        assert!(distance > 3900.0 && distance < 4000.0);
    }

    #[test]
    fn test_signature_conflicts_rssi_divergence() {
        let obs1 = create_test_observation("ABC123", Some("UAL123"), Some(-30.0), None);
        let sig1 = NetworkSignature::new(&obs1);

        let obs2 = create_test_observation("ABC123", Some("DAL456"), Some(-50.0), None);
        let sig2 = NetworkSignature::new(&obs2);

        let conflict = sig1.conflicts_with(&sig2);
        assert!(conflict.is_some());
        assert!(conflict.unwrap().contains("RSSI divergence"));
    }

    #[test]
    fn test_signature_conflicts_position_divergence() {
        let obs1 =
            create_test_observation("ABC123", Some("UAL123"), Some(-45.0), Some((40.7, -74.0))); // NYC
        let sig1 = NetworkSignature::new(&obs1);

        let obs2 =
            create_test_observation("ABC123", Some("DAL456"), Some(-45.0), Some((34.0, -118.2))); // LAX
        let sig2 = NetworkSignature::new(&obs2);

        let conflict = sig1.conflicts_with(&sig2);
        assert!(conflict.is_some());
        assert!(conflict.unwrap().contains("Position divergence"));
    }

    #[test]
    fn test_signature_no_conflict() {
        let obs1 =
            create_test_observation("ABC123", Some("UAL123"), Some(-45.0), Some((40.7, -74.0)));
        let sig1 = NetworkSignature::new(&obs1);

        let obs2 =
            create_test_observation("ABC123", Some("UAL123"), Some(-46.0), Some((40.8, -74.1))); // Similar
        let sig2 = NetworkSignature::new(&obs2);

        let conflict = sig1.conflicts_with(&sig2);
        assert!(conflict.is_none());
    }

    #[test]
    fn test_hex_duplicate_detection() {
        let config = create_test_config();
        let mut detector = IdentityDetector::new(config);

        // First observation for ABC123 in NYC
        let obs1 =
            create_test_observation("ABC123", Some("UAL123"), Some(-30.0), Some((40.7, -74.0)));
        let anomaly1 = detector.update_and_detect_hex_duplicate(&obs1);
        assert!(anomaly1.is_none()); // First occurrence, no anomaly

        // Second observation for same hex but in LAX with different RSSI
        let obs2 =
            create_test_observation("ABC123", Some("DAL456"), Some(-50.0), Some((34.0, -118.2)));
        let anomaly2 = detector.update_and_detect_hex_duplicate(&obs2);
        assert!(anomaly2.is_some());
        let anomaly = anomaly2.unwrap();
        assert_eq!(anomaly.subtype, "hex_duplicate");
        assert_eq!(anomaly.confidence, 0.85);
    }

    #[test]
    fn test_cleanup_old_signatures() {
        let config = create_test_config();
        let mut detector = IdentityDetector::new(config);

        let old_time = 1000;
        let current_time = old_time + 120_000; // 2 minutes later

        // Add old signature
        let obs_old = AircraftObservation {
            id: None,
            ts_ms: old_time,
            hex: "OLD123".to_string(),
            flight: Some("OLD123".to_string()),
            lat: Some(40.0),
            lon: Some(-74.0),
            altitude: Some(35000),
            gs: Some(450.0),
            rssi: Some(-45.0),
            msg_count_total: Some(1000),
            raw_json: r#"{"hex":"OLD123"}"#.to_string(),
            msg_rate_hz: Some(5.0),
        };
        detector.update_and_detect_hex_duplicate(&obs_old);
        assert_eq!(detector.get_signature_count(), 1);

        // Add current signature
        let obs_current = AircraftObservation {
            id: None,
            ts_ms: current_time,
            hex: "NEW456".to_string(),
            flight: Some("NEW456".to_string()),
            lat: Some(40.0),
            lon: Some(-74.0),
            altitude: Some(35000),
            gs: Some(450.0),
            rssi: Some(-45.0),
            msg_count_total: Some(1000),
            raw_json: r#"{"hex":"NEW456"}"#.to_string(),
            msg_rate_hz: Some(5.0),
        };
        detector.update_and_detect_hex_duplicate(&obs_current);

        // Should have cleaned up old signature, keeping only new one
        assert_eq!(detector.get_signature_count(), 1);
        assert!(detector.get_signature("NEW456").is_some());
        assert!(detector.get_signature("OLD123").is_none());
    }

    #[tokio::test]
    async fn test_identity_detection_service() {
        let config = create_test_config();
        let (alert_sender, mut alert_receiver) = mpsc::unbounded_channel();

        let service = IdentityDetectionService::new(config, alert_sender);

        // Test suspicious callsign detection
        let obs = create_test_observation("ABC123", Some("TEST123"), Some(-45.0), None);
        service.process_observation(obs);

        // Should receive alert
        let alert = alert_receiver.try_recv().expect("Should receive alert");
        assert_eq!(alert.subtype, "suspicious_callsign");
    }

    #[test]
    fn test_detect_all_multiple_anomalies() {
        let config = create_test_config();
        let mut detector = IdentityDetector::new(config);

        // Observation with both suspicious callsign and invalid hex
        let obs = create_test_observation("000000", Some("FAKE123"), Some(-45.0), None);
        let anomalies = detector.detect_all(&obs);

        assert_eq!(anomalies.len(), 2);

        // Should have both anomaly types
        let subtypes: Vec<&str> = anomalies.iter().map(|a| a.subtype.as_str()).collect();
        assert!(subtypes.contains(&"suspicious_callsign"));
        assert!(subtypes.contains(&"invalid_hex"));
    }
}
