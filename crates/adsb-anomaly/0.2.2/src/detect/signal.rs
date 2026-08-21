// ABOUTME: Tier 2 Signal anomaly detection using RSSI baseline and outlier analysis
// ABOUTME: Maintains per-hex EMA statistics and detects signal strength anomalies

#![allow(dead_code)]

use crate::config::AnalysisConfig;
use crate::model::{AircraftObservation, AnomalyCandidate, AnomalyType};
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// Signal statistics for a single aircraft hex
#[derive(Debug, Clone)]
pub struct SignalStats {
    pub hex: String,
    pub mean: f64,
    pub variance: f64,
    pub sample_count: u64,
    pub last_updated_ms: i64,
}

impl SignalStats {
    pub fn new(hex: String, initial_rssi: f64, timestamp_ms: i64) -> Self {
        Self {
            hex,
            mean: initial_rssi,
            variance: 0.0, // No variance with single sample
            sample_count: 1,
            last_updated_ms: timestamp_ms,
        }
    }

    /// Update stats with new RSSI value using EMA (Exponential Moving Average)
    pub fn update(&mut self, rssi: f64, timestamp_ms: i64, alpha: f64) {
        // EMA for mean: μ_new = α * rssi + (1 - α) * μ_old
        self.mean = alpha * rssi + (1.0 - alpha) * self.mean;

        // EMA for variance: σ²_new = α * (rssi - μ_new)² + (1 - α) * σ²_old
        let deviation = rssi - self.mean;
        self.variance = alpha * deviation * deviation + (1.0 - alpha) * self.variance;

        self.sample_count += 1;
        self.last_updated_ms = timestamp_ms;
    }

    /// Calculate Z-score for given RSSI value
    pub fn calculate_z_score(&self, rssi: f64) -> f64 {
        let std_dev = self.variance.sqrt();
        // For RSSI measurements, use 1e-3 threshold (0.001 dBm is negligible variance)
        if std_dev < 1e-3 || !std_dev.is_finite() {
            // Avoid division by zero/infinity, return 0 if no meaningful variance
            0.0
        } else {
            let z_score = (rssi - self.mean) / std_dev;
            // Clamp extreme Z-scores to prevent numerical issues
            z_score.clamp(-100.0, 100.0)
        }
    }

    /// Check if this aircraft has sufficient data for reliable detection
    pub fn is_mature(&self) -> bool {
        self.sample_count >= 30 // Need at least 30 samples for statistically reliable baseline (3σ rule)
    }
}

/// Signal anomaly detector with per-hex baseline tracking
#[derive(Debug)]
pub struct SignalDetector {
    config: Arc<AnalysisConfig>,
    aircraft_stats: HashMap<String, SignalStats>,
    alpha: f64, // EMA decay factor
    z_score_threshold: f64,
    max_age_ms: i64, // Max age before removing stale aircraft
}

impl SignalDetector {
    pub fn new(config: Arc<AnalysisConfig>) -> Self {
        Self {
            config,
            aircraft_stats: HashMap::new(),
            alpha: 0.1,                 // EMA decay factor - configurable later
            z_score_threshold: 3.0,     // Z-score threshold for outliers (99.7% confidence)
            max_age_ms: 30 * 60 * 1000, // 30 minutes
        }
    }

    /// Update and detect RSSI anomalies for a single aircraft
    pub fn update_and_detect_rssi(
        &mut self,
        hex: &str,
        rssi_opt: Option<f64>,
        timestamp_ms: i64,
    ) -> Option<AnomalyCandidate> {
        let rssi = rssi_opt?; // Return None if no RSSI data

        // Check global RSSI bounds first
        if rssi < self.config.min_rssi_units || rssi > self.config.max_rssi_units {
            return Some(
                AnomalyCandidate::new(
                    hex.to_string(),
                    AnomalyType::Signal,
                    "rssi_out_of_bounds".to_string(),
                    0.95,
                )
                .with_details(json!({
                    "rssi": rssi,
                    "min_rssi_units": self.config.min_rssi_units,
                    "max_rssi_units": self.config.max_rssi_units,
                    "reason": "RSSI outside configured bounds"
                })),
            );
        }

        // Check suspicious RSSI threshold first (global threshold, always applicable)
        if rssi > self.config.suspicious_rssi_units {
            // Still need to update stats even if we find a suspicious RSSI
            match self.aircraft_stats.get_mut(hex) {
                Some(stats) => {
                    stats.update(rssi, timestamp_ms, self.alpha);
                }
                None => {
                    let stats = SignalStats::new(hex.to_string(), rssi, timestamp_ms);
                    self.aircraft_stats.insert(hex.to_string(), stats);
                }
            }

            return Some(
                AnomalyCandidate::new(
                    hex.to_string(),
                    AnomalyType::Signal,
                    "suspicious_rssi_strength".to_string(),
                    0.8,
                )
                .with_details(json!({
                    "rssi": rssi,
                    "suspicious_threshold": self.config.suspicious_rssi_units,
                    "reason": "RSSI exceeds suspicious threshold"
                })),
            );
        }

        // Get or create aircraft stats for Z-score analysis
        let stats = match self.aircraft_stats.get_mut(hex) {
            Some(stats) => {
                stats.update(rssi, timestamp_ms, self.alpha);
                stats
            }
            None => {
                let stats = SignalStats::new(hex.to_string(), rssi, timestamp_ms);
                self.aircraft_stats.insert(hex.to_string(), stats.clone());
                return None; // No Z-score anomaly on first observation
            }
        };

        // Only detect Z-score anomalies if we have enough data
        if !stats.is_mature() {
            return None;
        }

        // Calculate Z-score and check for outlier
        let z_score = stats.calculate_z_score(rssi);
        if z_score.abs() > self.z_score_threshold {
            return Some(
                AnomalyCandidate::new(
                    hex.to_string(),
                    AnomalyType::Signal,
                    "signal_outlier".to_string(),
                    0.7,
                )
                .with_details(json!({
                    "rssi": rssi,
                    "baseline_mean": stats.mean,
                    "baseline_variance": stats.variance,
                    "z_score": z_score,
                    "threshold": self.z_score_threshold,
                    "sample_count": stats.sample_count,
                    "reason": "RSSI Z-score exceeds threshold"
                })),
            );
        }

        None
    }

    /// Clean up stale aircraft stats
    pub fn cleanup_stale_aircraft(&mut self, current_time_ms: i64) {
        let cutoff_time = current_time_ms - self.max_age_ms;
        let initial_count = self.aircraft_stats.len();

        self.aircraft_stats
            .retain(|_, stats| stats.last_updated_ms >= cutoff_time);

        let removed_count = initial_count - self.aircraft_stats.len();
        if removed_count > 0 {
            tracing::debug!("Cleaned up {} stale aircraft signal stats", removed_count);
        }
    }

    /// Get current statistics for debugging/monitoring
    pub fn get_stats(&self) -> Vec<&SignalStats> {
        self.aircraft_stats.values().collect()
    }

    /// Get statistics for a specific aircraft
    pub fn get_aircraft_stats(&self, hex: &str) -> Option<&SignalStats> {
        self.aircraft_stats.get(hex)
    }
}

/// Service for signal anomaly detection that processes observations and sends alerts
pub struct SignalDetectionService {
    detector: Arc<Mutex<SignalDetector>>,
    alert_sender: mpsc::UnboundedSender<AnomalyCandidate>,
}

impl SignalDetectionService {
    /// Create new signal detection service
    pub fn new(
        config: Arc<AnalysisConfig>,
        alert_sender: mpsc::UnboundedSender<AnomalyCandidate>,
    ) -> Self {
        let detector = SignalDetector::new(config);
        Self {
            detector: Arc::new(Mutex::new(detector)),
            alert_sender,
        }
    }

    /// Process an observation for signal anomalies
    pub fn process_observation(&self, obs: AircraftObservation) {
        // Only process observations with RSSI data
        if obs.rssi.is_none() {
            return;
        }

        let mut detector = self.detector.lock().unwrap();
        if let Some(anomaly) = detector.update_and_detect_rssi(&obs.hex, obs.rssi, obs.ts_ms) {
            // Send alert through channel
            if self.alert_sender.send(anomaly).is_err() {
                tracing::warn!("Failed to send signal anomaly alert: channel closed");
            }
        }
    }

    /// Clean up stale aircraft statistics
    pub fn cleanup_stale_aircraft(&self, current_time_ms: i64) {
        let mut detector = self.detector.lock().unwrap();
        detector.cleanup_stale_aircraft(current_time_ms);
    }

    /// Get current statistics count for monitoring
    pub fn get_stats_count(&self) -> usize {
        let detector = self.detector.lock().unwrap();
        detector.aircraft_stats.len()
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
            suspicious_callsigns: vec!["TEST.*".to_string()],
            invalid_hex_patterns: vec!["000000".to_string()],
        })
    }

    #[test]
    fn test_signal_stats_creation() {
        let stats = SignalStats::new("ABC123".to_string(), -50.0, 1641024000000);
        assert_eq!(stats.hex, "ABC123");
        assert_eq!(stats.mean, -50.0);
        assert_eq!(stats.variance, 0.0);
        assert_eq!(stats.sample_count, 1);
        assert!(!stats.is_mature()); // Not mature with only 1 sample
    }

    #[test]
    fn test_signal_stats_update_ema() {
        let mut stats = SignalStats::new("ABC123".to_string(), -50.0, 1641024000000);

        // Add second sample
        stats.update(-52.0, 1641024001000, 0.1);
        assert_eq!(stats.sample_count, 2);

        // EMA calculation: new_mean = 0.1 * (-52.0) + 0.9 * (-50.0) = -50.2
        assert!((stats.mean - (-50.2)).abs() < 1e-10);

        // Add more samples to see EMA evolution
        stats.update(-48.0, 1641024002000, 0.1);
        assert_eq!(stats.sample_count, 3);

        // Mean should trend toward recent values
        assert!(stats.mean > -50.2); // Moving toward -48.0
    }

    #[test]
    fn test_z_score_calculation() {
        let mut stats = SignalStats::new("ABC123".to_string(), -50.0, 1641024000000);

        // Add enough samples to build variance (need 30+ for maturity)
        for i in 1..=30 {
            let rssi = -50.0 + (i as f64 - 15.0) / 3.0; // Values spread around -50
            stats.update(rssi, 1641024000000 + i * 1000, 0.1);
        }

        assert!(stats.is_mature());

        // Test Z-score for normal value (close to mean)
        let z_normal = stats.calculate_z_score(stats.mean);
        assert!(z_normal.abs() < 0.1); // Should be very close to baseline

        // Test Z-score for outlier
        let z_outlier = stats.calculate_z_score(-30.0); // Very high RSSI
        assert!(z_outlier.abs() > 2.0); // Should be significant outlier
    }

    #[test]
    fn test_rssi_out_of_bounds_detection() {
        let config = create_test_config();
        let mut detector = SignalDetector::new(config);

        // Test RSSI below minimum
        let anomaly = detector.update_and_detect_rssi("ABC123", Some(-150.0), 1641024000000);
        assert!(anomaly.is_some());
        let anomaly = anomaly.unwrap();
        assert_eq!(anomaly.anomaly_type, AnomalyType::Signal);
        assert_eq!(anomaly.subtype, "rssi_out_of_bounds");
        assert_eq!(anomaly.confidence, 0.95);

        // Test RSSI above maximum
        let anomaly = detector.update_and_detect_rssi("DEF456", Some(-5.0), 1641024000000);
        assert!(anomaly.is_some());
        let anomaly = anomaly.unwrap();
        assert_eq!(anomaly.subtype, "rssi_out_of_bounds");
    }

    #[test]
    fn test_suspicious_rssi_detection() {
        let config = create_test_config();
        let mut detector = SignalDetector::new(config);

        // Test RSSI above suspicious threshold but within bounds
        let anomaly = detector.update_and_detect_rssi("ABC123", Some(-15.0), 1641024000000);
        assert!(anomaly.is_some());
        let anomaly = anomaly.unwrap();
        assert_eq!(anomaly.subtype, "suspicious_rssi_strength");
        assert_eq!(anomaly.confidence, 0.8);
    }

    #[test]
    fn test_no_anomaly_normal_rssi() {
        let config = create_test_config();
        let mut detector = SignalDetector::new(config);

        // Test normal RSSI values
        for i in 0..5 {
            let anomaly =
                detector.update_and_detect_rssi("ABC123", Some(-45.0), 1641024000000 + i * 1000);
            assert!(anomaly.is_none()); // No anomaly for normal values
        }
    }

    #[test]
    fn test_signal_outlier_detection() {
        let config = create_test_config();
        let mut detector = SignalDetector::new(config);
        let hex = "ABC123";

        // Build baseline with consistent values (need 30+ samples for maturity)
        for i in 0..35 {
            let rssi = -45.0 + ((i % 3) as f64 - 1.0); // Values -46, -45, -44
            let anomaly =
                detector.update_and_detect_rssi(hex, Some(rssi), 1641024000000 + i * 1000);
            // Should not trigger anomaly during baseline building
            if i < 30 {
                assert!(anomaly.is_none());
            }
        }

        // Now inject a significant outlier (much weaker signal that won't trigger suspicious check)
        let stats = detector.get_aircraft_stats(hex).unwrap();
        println!(
            "Before outlier: mean={:.3}, variance={:.3}, samples={}",
            stats.mean, stats.variance, stats.sample_count
        );

        let anomaly = detector.update_and_detect_rssi(hex, Some(-80.0), 1641024015000); // Much weaker signal

        let stats_after = detector.get_aircraft_stats(hex).unwrap();
        let z_score = stats_after.calculate_z_score(-80.0);
        println!(
            "After outlier: mean={:.3}, variance={:.3}, z_score={:.3}",
            stats_after.mean, stats_after.variance, z_score
        );

        assert!(
            anomaly.is_some(),
            "Expected anomaly for RSSI -80.0, z_score={:.3}",
            z_score
        );
        let anomaly = anomaly.unwrap();
        assert_eq!(anomaly.subtype, "signal_outlier");
        assert_eq!(anomaly.confidence, 0.7);

        // Verify details contain expected fields
        let details = anomaly.details.unwrap();
        assert!(details.get("rssi").is_some());
        assert!(details.get("z_score").is_some());
        assert!(details.get("baseline_mean").is_some());
    }

    #[test]
    fn test_no_anomaly_for_none_rssi() {
        let config = create_test_config();
        let mut detector = SignalDetector::new(config);

        let anomaly = detector.update_and_detect_rssi("ABC123", None, 1641024000000);
        assert!(anomaly.is_none());
    }

    #[test]
    fn test_cleanup_stale_aircraft() {
        let config = create_test_config();
        let mut detector = SignalDetector::new(config);

        // Add aircraft at different times
        detector.update_and_detect_rssi("OLD123", Some(-45.0), 1000);
        detector.update_and_detect_rssi("NEW456", Some(-45.0), 2000000); // Much later

        assert_eq!(detector.aircraft_stats.len(), 2);

        // Cleanup with cutoff that should remove old aircraft
        detector.cleanup_stale_aircraft(2000000);

        assert_eq!(detector.aircraft_stats.len(), 1);
        assert!(detector.aircraft_stats.contains_key("NEW456"));
        assert!(!detector.aircraft_stats.contains_key("OLD123"));
    }

    #[test]
    fn test_get_aircraft_stats() {
        let config = create_test_config();
        let mut detector = SignalDetector::new(config);

        detector.update_and_detect_rssi("ABC123", Some(-45.0), 1641024000000);

        let stats = detector.get_aircraft_stats("ABC123");
        assert!(stats.is_some());
        assert_eq!(stats.unwrap().hex, "ABC123");

        let stats = detector.get_aircraft_stats("NONEXISTENT");
        assert!(stats.is_none());
    }

    #[tokio::test]
    async fn test_signal_detection_service() {
        let config = create_test_config();
        let (alert_sender, mut alert_receiver) = mpsc::unbounded_channel();

        let service = SignalDetectionService::new(config, alert_sender);

        // Create test observation with suspicious RSSI
        let obs = AircraftObservation {
            id: None,
            ts_ms: 1641024000000,
            hex: "ABC123".to_string(),
            flight: Some("TEST123".to_string()),
            lat: Some(40.7128),
            lon: Some(-74.0060),
            altitude: Some(35000),
            gs: Some(450.0),
            rssi: Some(-15.0), // Above suspicious threshold (-20.0)
            msg_count_total: Some(1000),
            raw_json: r#"{"hex":"ABC123"}"#.to_string(),
            msg_rate_hz: Some(5.0),
        };

        // Process observation
        service.process_observation(obs);

        // Should receive alert for suspicious RSSI
        let alert = alert_receiver.try_recv().expect("Should receive alert");
        assert_eq!(alert.hex, "ABC123");
        assert_eq!(alert.anomaly_type, AnomalyType::Signal);
        assert_eq!(alert.subtype, "suspicious_rssi_strength");
    }

    #[tokio::test]
    async fn test_signal_detection_service_no_rssi() {
        let config = create_test_config();
        let (alert_sender, mut alert_receiver) = mpsc::unbounded_channel();

        let service = SignalDetectionService::new(config, alert_sender);

        // Create test observation without RSSI
        let obs = AircraftObservation {
            id: None,
            ts_ms: 1641024000000,
            hex: "ABC123".to_string(),
            flight: Some("TEST123".to_string()),
            lat: Some(40.7128),
            lon: Some(-74.0060),
            altitude: Some(35000),
            gs: Some(450.0),
            rssi: None, // No RSSI data
            msg_count_total: Some(1000),
            raw_json: r#"{"hex":"ABC123"}"#.to_string(),
            msg_rate_hz: Some(5.0),
        };

        // Process observation
        service.process_observation(obs);

        // Should not receive any alert
        assert!(alert_receiver.try_recv().is_err());
    }

    #[test]
    fn test_steady_rssi_no_false_positives() {
        let config = create_test_config();
        let mut detector = SignalDetector::new(config);
        let hex = "ABC123";

        // Feed steady RSSI values around -45 dBFS
        for i in 0..50 {
            let rssi = -45.0 + (i as f64 % 3.0 - 1.0) * 0.5; // Small variations: -45.5 to -44.5
            let anomaly =
                detector.update_and_detect_rssi(hex, Some(rssi), 1641024000000 + i * 1000);
            assert!(
                anomaly.is_none(),
                "False positive at sample {}: RSSI {}",
                i,
                rssi
            );
        }
    }

    // Property-based test using simple random generation
    #[test]
    fn test_rssi_random_walk_stability() {
        let config = create_test_config();
        let mut detector = SignalDetector::new(config);
        let hex = "ABC123";

        // Simulate RSSI random walk (aircraft moving but no anomalies)
        let mut rssi = -45.0;
        let mut false_positive_count = 0;

        for i in 0..100 {
            // Random walk: small changes each time
            let change = ((i * 17 + 23) % 7) as f64 - 3.0; // Pseudo-random -3 to +3
            rssi += change * 0.1; // Small steps
            rssi = rssi.clamp(-60.0, -30.0); // Keep in reasonable range

            let anomaly =
                detector.update_and_detect_rssi(hex, Some(rssi), 1641024000000 + i * 1000);
            if anomaly.is_some() {
                false_positive_count += 1;
            }
        }

        // Should have very few false positives for random walk
        assert!(
            false_positive_count < 5,
            "Too many false positives: {}",
            false_positive_count
        );
    }

    #[test]
    fn test_big_rssi_jump_triggers_anomaly() {
        let config = create_test_config();
        let mut detector = SignalDetector::new(config);
        let hex = "ABC123";

        // Build stable baseline (need 30+ samples for maturity)
        for i in 0..35 {
            detector.update_and_detect_rssi(hex, Some(-45.0), 1641024000000 + i * 1000);
        }

        // Inject big jump
        let anomaly = detector.update_and_detect_rssi(hex, Some(-25.0), 1641024020000);
        assert!(anomaly.is_some());

        let anomaly = anomaly.unwrap();
        assert_eq!(anomaly.subtype, "signal_outlier");

        // Check that Z-score is recorded
        let details = anomaly.details.unwrap();
        let z_score: f64 = details.get("z_score").unwrap().as_f64().unwrap();
        assert!(z_score.abs() > 3.0);
    }
}
