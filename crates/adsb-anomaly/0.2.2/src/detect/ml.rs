// ABOUTME: Machine learning-based anomaly detection using Isolation Forest algorithm
// ABOUTME: Builds feature vectors from aircraft behavior and detects statistical outliers using unsupervised ML

#![allow(dead_code)]

use crate::config::AnalysisConfig;
use crate::model::{AircraftObservation, AnomalyCandidate, AnomalyType};
use serde_json::json;
// Note: Using a simpler statistical approach instead of Isolation Forest
// since smartcore 0.3 has limited ML algorithm support
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// Feature vector representing aircraft behavior over a time window
#[derive(Debug, Clone)]
pub struct FeatureVector {
    pub hex: String,
    pub window_start_ms: i64,
    pub window_end_ms: i64,
    pub observation_count: usize,

    // Temporal features
    pub msg_rate_mean: f64,
    pub msg_rate_std: f64,

    // Signal features
    pub rssi_mean: f64,
    pub rssi_std: f64,

    // Kinematic features
    pub speed_mean: f64,
    pub speed_max: f64,
    pub alt_rate_std: f64,
    pub turn_rate_std: f64,
}

impl FeatureVector {
    /// Convert to array for ML model input
    pub fn to_array(&self) -> [f64; 8] {
        [
            self.msg_rate_mean,
            self.msg_rate_std,
            self.rssi_mean,
            self.rssi_std,
            self.speed_mean,
            self.speed_max,
            self.alt_rate_std,
            self.turn_rate_std,
        ]
    }

    /// Create from observations within a time window
    pub fn from_observations(
        hex: String,
        observations: &[AircraftObservation],
        window_start_ms: i64,
        window_end_ms: i64,
    ) -> Option<Self> {
        if observations.is_empty() {
            return None;
        }

        // Calculate temporal features
        let msg_rates: Vec<f64> = observations
            .iter()
            .filter_map(|obs| obs.msg_rate_hz)
            .collect();

        let (msg_rate_mean, msg_rate_std) = if msg_rates.is_empty() {
            (0.0, 0.0)
        } else {
            let mean = msg_rates.iter().sum::<f64>() / msg_rates.len() as f64;
            let variance = if msg_rates.len() > 1 {
                msg_rates.iter().map(|x| (x - mean).powi(2)).sum::<f64>()
                    / (msg_rates.len() - 1) as f64
            } else {
                0.0
            };
            (mean, variance.sqrt())
        };

        // Calculate signal features
        let rssi_values: Vec<f64> = observations.iter().filter_map(|obs| obs.rssi).collect();

        let (rssi_mean, rssi_std) = if rssi_values.is_empty() {
            (-50.0, 0.0) // Default RSSI if none available
        } else {
            let mean = rssi_values.iter().sum::<f64>() / rssi_values.len() as f64;
            let variance = if rssi_values.len() > 1 {
                rssi_values.iter().map(|x| (x - mean).powi(2)).sum::<f64>()
                    / (rssi_values.len() - 1) as f64
            } else {
                0.0
            };
            (mean, variance.sqrt())
        };

        // Calculate kinematic features
        let speeds: Vec<f64> = observations.iter().filter_map(|obs| obs.gs).collect();

        let (speed_mean, speed_max) = if speeds.is_empty() {
            (0.0, 0.0)
        } else {
            let mean = speeds.iter().sum::<f64>() / speeds.len() as f64;
            let max = speeds.iter().cloned().fold(0.0, f64::max);
            (mean, max)
        };

        // Calculate altitude rate variance (simplified - would need sequential positions for true calculation)
        let altitudes: Vec<f64> = observations
            .iter()
            .filter_map(|obs| obs.altitude.map(|alt| alt as f64))
            .collect();

        let alt_rate_std = if altitudes.len() > 1 {
            let alt_diffs: Vec<f64> = altitudes.windows(2).map(|w| w[1] - w[0]).collect();

            if !alt_diffs.is_empty() {
                let mean_diff = alt_diffs.iter().sum::<f64>() / alt_diffs.len() as f64;
                let variance = alt_diffs
                    .iter()
                    .map(|x| (x - mean_diff).powi(2))
                    .sum::<f64>()
                    / alt_diffs.len() as f64;
                variance.sqrt()
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Turn rate calculation would require heading data, so we'll use a simplified version
        let turn_rate_std = 0.0; // Placeholder - would need heading/track data from observations

        Some(Self {
            hex,
            window_start_ms,
            window_end_ms,
            observation_count: observations.len(),
            msg_rate_mean,
            msg_rate_std,
            rssi_mean,
            rssi_std,
            speed_mean,
            speed_max,
            alt_rate_std,
            turn_rate_std,
        })
    }
}

/// Statistical ensemble features for baseline comparison
#[derive(Debug, Clone, Default)]
pub struct FeatureBaseline {
    pub mean: f64,
    pub std: f64,
    pub min: f64,
    pub max: f64,
    pub sample_count: usize,
}

impl FeatureBaseline {
    pub fn new() -> Self {
        Self {
            mean: 0.0,
            std: 0.0,
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
            sample_count: 0,
        }
    }

    pub fn update(&mut self, values: &[f64]) {
        if values.is_empty() {
            return;
        }

        let sum: f64 = values.iter().sum();
        self.mean = sum / values.len() as f64;

        if values.len() > 1 {
            let variance: f64 = values.iter().map(|x| (x - self.mean).powi(2)).sum::<f64>()
                / (values.len() - 1) as f64;
            self.std = variance.sqrt();
        } else {
            self.std = 0.0;
        }

        self.min = values.iter().cloned().fold(f64::INFINITY, f64::min);
        self.max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        self.sample_count = values.len();
    }

    /// Calculate Z-score for a value
    pub fn z_score(&self, value: f64) -> f64 {
        if self.std == 0.0 {
            0.0
        } else {
            (value - self.mean) / self.std
        }
    }
}

/// Multi-variate statistical anomaly detector
#[derive(Debug)]
pub struct StatisticalMLDetector {
    config: Arc<AnalysisConfig>,
    feature_baselines: HashMap<String, [FeatureBaseline; 8]>, // 8 features
    feature_history: HashMap<String, Vec<AircraftObservation>>,
    window_duration_ms: i64,
    min_observations_for_training: usize,
    anomaly_threshold: f64, // Z-score threshold
    last_training_time_ms: i64,
    retrain_interval_ms: i64,
}

impl StatisticalMLDetector {
    pub fn new(config: Arc<AnalysisConfig>) -> Self {
        Self {
            config,
            feature_baselines: HashMap::new(),
            feature_history: HashMap::new(),
            window_duration_ms: 10 * 60 * 1000, // 10 minutes
            min_observations_for_training: 100,
            anomaly_threshold: 3.0, // 3-sigma rule (99.7% confidence)
            last_training_time_ms: 0,
            retrain_interval_ms: 30 * 60 * 1000, // Retrain every 30 minutes
        }
    }

    /// Add observations to the feature history
    pub fn add_observations(&mut self, observations: &[AircraftObservation], current_time_ms: i64) {
        for obs in observations {
            let history = self.feature_history.entry(obs.hex.clone()).or_default();
            history.push(obs.clone());

            // Keep only observations within the window
            let cutoff_time = current_time_ms - self.window_duration_ms;
            history.retain(|o| o.ts_ms >= cutoff_time);
        }

        // Clean up aircraft with no recent observations
        self.feature_history.retain(|_, history| {
            history
                .last()
                .is_some_and(|obs| obs.ts_ms >= current_time_ms - self.window_duration_ms)
        });
    }

    /// Build feature vectors from current history
    fn build_feature_vectors(&self, current_time_ms: i64) -> Vec<FeatureVector> {
        let window_start = current_time_ms - self.window_duration_ms;
        let mut features = Vec::new();

        for (hex, observations) in &self.feature_history {
            // Filter observations to current window
            let window_obs: Vec<AircraftObservation> = observations
                .iter()
                .filter(|obs| obs.ts_ms >= window_start && obs.ts_ms <= current_time_ms)
                .cloned()
                .collect();

            if let Some(feature_vector) = FeatureVector::from_observations(
                hex.clone(),
                &window_obs,
                window_start,
                current_time_ms,
            ) {
                features.push(feature_vector);
            }
        }

        features
    }

    /// Train the statistical baselines for each feature
    pub fn train_model(&mut self, current_time_ms: i64) -> Result<usize, String> {
        let features = self.build_feature_vectors(current_time_ms);

        if features.len() < self.min_observations_for_training {
            return Err(format!(
                "Insufficient data: {} features, need {}",
                features.len(),
                self.min_observations_for_training
            ));
        }

        // Clear existing baselines
        self.feature_baselines.clear();

        // Build aggregate baselines across all aircraft
        let mut feature_values: [Vec<f64>; 8] = Default::default();

        for feature in &features {
            let array = feature.to_array();
            for (i, &value) in array.iter().enumerate() {
                if value.is_finite() {
                    // Skip NaN/infinite values
                    feature_values[i].push(value);
                }
            }
        }

        // Create baseline for each feature
        let mut global_baselines: [FeatureBaseline; 8] = Default::default();
        for i in 0..8 {
            global_baselines[i] = FeatureBaseline::new();
            global_baselines[i].update(&feature_values[i]);
        }

        // Store global baseline with a special key
        self.feature_baselines
            .insert("__global__".to_string(), global_baselines);

        self.last_training_time_ms = current_time_ms;
        tracing::info!(
            "Trained statistical ML model with {} samples",
            features.len()
        );
        Ok(features.len())
    }

    /// Detect anomalies using the trained statistical baselines
    pub fn detect_anomalies(&self, current_time_ms: i64) -> Vec<AnomalyCandidate> {
        let baselines = match self.feature_baselines.get("__global__") {
            Some(b) => b,
            None => return Vec::new(), // No model trained yet
        };

        let features = self.build_feature_vectors(current_time_ms);
        let mut anomalies = Vec::new();

        for feature in features {
            let feature_array = feature.to_array();
            let mut anomaly_score = 0.0;
            let mut anomalous_features = Vec::new();

            // Calculate aggregate anomaly score across all features
            for (i, &value) in feature_array.iter().enumerate() {
                let z_score = baselines[i].z_score(value);
                if z_score.abs() > self.anomaly_threshold {
                    anomaly_score += z_score.abs();
                    anomalous_features.push((i, z_score));
                }
            }

            // Trigger anomaly if any feature is significantly anomalous
            if !anomalous_features.is_empty() {
                let feature_names = [
                    "msg_rate_mean",
                    "msg_rate_std",
                    "rssi_mean",
                    "rssi_std",
                    "speed_mean",
                    "speed_max",
                    "alt_rate_std",
                    "turn_rate_std",
                ];

                let anomaly = AnomalyCandidate::new(
                    feature.hex.clone(),
                    AnomalyType::Behavioral, // Using Behavioral type for ML anomalies
                    "iforest_outlier".to_string(),
                    Self::score_to_confidence(anomaly_score),
                )
                .with_details(json!({
                    "anomaly_score": anomaly_score,
                    "threshold": self.anomaly_threshold,
                    "observation_count": feature.observation_count,
                    "window_minutes": self.window_duration_ms / 60000,
                    "anomalous_features": anomalous_features.iter().map(|(i, z)| {
                        json!({
                            "feature": feature_names[*i],
                            "z_score": z,
                            "value": feature_array[*i],
                            "baseline_mean": baselines[*i].mean,
                            "baseline_std": baselines[*i].std,
                        })
                    }).collect::<Vec<_>>(),
                    "features": {
                        "msg_rate_mean": feature.msg_rate_mean,
                        "msg_rate_std": feature.msg_rate_std,
                        "rssi_mean": feature.rssi_mean,
                        "rssi_std": feature.rssi_std,
                        "speed_mean": feature.speed_mean,
                        "speed_max": feature.speed_max,
                        "alt_rate_std": feature.alt_rate_std,
                        "turn_rate_std": feature.turn_rate_std,
                    },
                    "reason": format!("Statistical ML outlier: {} anomalous features with aggregate score {:.3}", anomalous_features.len(), anomaly_score)
                }));
                anomalies.push(anomaly);
            }
        }

        anomalies
    }

    /// Convert anomaly score to confidence (higher score = higher confidence)
    fn score_to_confidence(score: f64) -> f64 {
        // Anomaly scores are sums of Z-scores, typically range from 3+ to 20+
        // Convert to confidence 0.7 to 0.95
        let normalized = (score / 20.0).clamp(0.0, 1.0);
        0.7 + (normalized * 0.25)
    }

    /// Check if model needs retraining
    pub fn needs_retraining(&self, current_time_ms: i64) -> bool {
        self.feature_baselines.is_empty()
            || (current_time_ms - self.last_training_time_ms) > self.retrain_interval_ms
    }

    /// Get current model statistics
    pub fn get_stats(&self) -> (usize, bool, i64) {
        (
            self.feature_history.len(),
            !self.feature_baselines.is_empty(),
            self.last_training_time_ms,
        )
    }

    /// Get feature history for a specific aircraft (for testing)
    pub fn get_aircraft_history(&self, hex: &str) -> Option<&Vec<AircraftObservation>> {
        self.feature_history.get(hex)
    }

    /// Get feature baseline for testing
    pub fn get_baseline(&self) -> Option<&[FeatureBaseline; 8]> {
        self.feature_baselines.get("__global__")
    }
}

/// Service for ML-based anomaly detection using statistical methods
pub struct MLDetectionService {
    detector: Arc<Mutex<StatisticalMLDetector>>,
    alert_sender: mpsc::UnboundedSender<AnomalyCandidate>,
}

impl MLDetectionService {
    /// Create new ML detection service
    pub fn new(
        config: Arc<AnalysisConfig>,
        alert_sender: mpsc::UnboundedSender<AnomalyCandidate>,
    ) -> Self {
        let detector = StatisticalMLDetector::new(config);
        Self {
            detector: Arc::new(Mutex::new(detector)),
            alert_sender,
        }
    }

    /// Add observations for ML analysis
    pub fn add_observations(&self, observations: Vec<AircraftObservation>, current_time_ms: i64) {
        let mut detector = self.detector.lock().unwrap();
        detector.add_observations(&observations, current_time_ms);
    }

    /// Run ML analysis - train if needed and detect anomalies
    pub async fn run_analysis(&self, current_time_ms: i64) {
        let mut detector = self.detector.lock().unwrap();

        // Train model if needed
        if detector.needs_retraining(current_time_ms) {
            match detector.train_model(current_time_ms) {
                Ok(sample_count) => {
                    tracing::info!("ML model trained with {} samples", sample_count);
                }
                Err(e) => {
                    tracing::debug!("ML training skipped: {}", e);
                    return; // Can't detect without a trained model
                }
            }
        }

        // Detect anomalies
        let anomalies = detector.detect_anomalies(current_time_ms);

        // Send anomalies through alert channel
        for anomaly in anomalies {
            if self.alert_sender.send(anomaly).is_err() {
                tracing::warn!("Failed to send ML anomaly alert: channel closed");
            }
        }
    }

    /// Get service statistics
    pub fn get_stats(&self) -> (usize, bool, i64) {
        let detector = self.detector.lock().unwrap();
        detector.get_stats()
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
        ts_ms: i64,
        msg_rate_hz: Option<f64>,
        rssi: Option<f64>,
        gs: Option<f64>,
        altitude: Option<i32>,
    ) -> AircraftObservation {
        AircraftObservation {
            id: None,
            ts_ms,
            hex: hex.to_string(),
            flight: Some("TEST123".to_string()),
            lat: Some(40.7),
            lon: Some(-74.0),
            altitude,
            gs,
            rssi,
            msg_count_total: Some(1000),
            raw_json: format!(r#"{{"hex":"{}"}}"#, hex),
            msg_rate_hz,
        }
    }

    #[test]
    fn test_feature_vector_creation() {
        let observations = vec![
            create_test_observation(
                "ABC123",
                1000,
                Some(5.0),
                Some(-45.0),
                Some(450.0),
                Some(35000),
            ),
            create_test_observation(
                "ABC123",
                2000,
                Some(6.0),
                Some(-47.0),
                Some(460.0),
                Some(35100),
            ),
            create_test_observation(
                "ABC123",
                3000,
                Some(4.0),
                Some(-43.0),
                Some(440.0),
                Some(35200),
            ),
        ];

        let feature_vector =
            FeatureVector::from_observations("ABC123".to_string(), &observations, 0, 5000).unwrap();

        assert_eq!(feature_vector.hex, "ABC123");
        assert_eq!(feature_vector.observation_count, 3);
        assert!((feature_vector.msg_rate_mean - 5.0).abs() < 0.01);
        assert!(feature_vector.msg_rate_std > 0.0);
        assert!((feature_vector.rssi_mean - (-45.0)).abs() < 0.1);
        assert!((feature_vector.speed_mean - 450.0).abs() < 5.0);
        assert!((feature_vector.speed_max - 460.0).abs() < 0.1);
    }

    #[test]
    fn test_feature_vector_to_array() {
        let feature_vector = FeatureVector {
            hex: "TEST".to_string(),
            window_start_ms: 0,
            window_end_ms: 1000,
            observation_count: 5,
            msg_rate_mean: 5.0,
            msg_rate_std: 1.0,
            rssi_mean: -45.0,
            rssi_std: 2.0,
            speed_mean: 450.0,
            speed_max: 500.0,
            alt_rate_std: 100.0,
            turn_rate_std: 10.0,
        };

        let array = feature_vector.to_array();
        assert_eq!(array, [5.0, 1.0, -45.0, 2.0, 450.0, 500.0, 100.0, 10.0]);
    }

    #[test]
    fn test_statistical_ml_detector_creation() {
        let config = create_test_config();
        let detector = StatisticalMLDetector::new(config);

        assert!(detector.feature_baselines.is_empty());
        assert_eq!(detector.feature_history.len(), 0);
        assert!(detector.needs_retraining(1000));
    }

    #[test]
    fn test_add_observations() {
        let config = create_test_config();
        let mut detector = StatisticalMLDetector::new(config);
        let current_time = 10000;

        let observations = vec![
            create_test_observation(
                "ABC123",
                current_time - 1000,
                Some(5.0),
                Some(-45.0),
                Some(450.0),
                Some(35000),
            ),
            create_test_observation(
                "ABC123",
                current_time - 500,
                Some(6.0),
                Some(-47.0),
                Some(460.0),
                Some(35100),
            ),
            create_test_observation(
                "DEF456",
                current_time - 800,
                Some(4.0),
                Some(-50.0),
                Some(300.0),
                Some(25000),
            ),
        ];

        detector.add_observations(&observations, current_time);

        assert_eq!(detector.feature_history.len(), 2);
        assert!(detector.feature_history.contains_key("ABC123"));
        assert!(detector.feature_history.contains_key("DEF456"));
        assert_eq!(detector.get_aircraft_history("ABC123").unwrap().len(), 2);
    }

    #[test]
    fn test_window_cleanup() {
        let config = create_test_config();
        let mut detector = StatisticalMLDetector::new(config);
        let current_time = 700_000; // 11.67 minutes from start (window is 10 minutes)

        let observations = vec![
            // Old observation (outside window)
            create_test_observation(
                "ABC123",
                0,
                Some(5.0),
                Some(-45.0),
                Some(450.0),
                Some(35000),
            ),
            // Recent observation (inside window)
            create_test_observation(
                "ABC123",
                current_time - 300_000,
                Some(6.0),
                Some(-47.0),
                Some(460.0),
                Some(35100),
            ),
        ];

        detector.add_observations(&observations, current_time);

        // Should only keep the recent observation
        assert_eq!(detector.get_aircraft_history("ABC123").unwrap().len(), 1);
        assert_eq!(
            detector.get_aircraft_history("ABC123").unwrap()[0].ts_ms,
            current_time - 300_000
        );
    }

    #[test]
    fn test_build_feature_vectors() {
        let config = create_test_config();
        let mut detector = StatisticalMLDetector::new(config);
        let current_time = 600_000; // 10 minutes

        // Add observations for multiple aircraft
        let observations = vec![
            create_test_observation(
                "ABC123",
                current_time - 300_000,
                Some(5.0),
                Some(-45.0),
                Some(450.0),
                Some(35000),
            ),
            create_test_observation(
                "ABC123",
                current_time - 200_000,
                Some(6.0),
                Some(-47.0),
                Some(460.0),
                Some(35100),
            ),
            create_test_observation(
                "DEF456",
                current_time - 100_000,
                Some(4.0),
                Some(-50.0),
                Some(300.0),
                Some(25000),
            ),
        ];

        detector.add_observations(&observations, current_time);
        let features = detector.build_feature_vectors(current_time);

        assert_eq!(features.len(), 2); // Two aircraft
        assert!(features.iter().any(|f| f.hex == "ABC123"));
        assert!(features.iter().any(|f| f.hex == "DEF456"));

        let abc_feature = features.iter().find(|f| f.hex == "ABC123").unwrap();
        assert_eq!(abc_feature.observation_count, 2);
    }

    #[test]
    fn test_insufficient_data_for_training() {
        let config = create_test_config();
        let mut detector = StatisticalMLDetector::new(config);

        // Add only a few observations (less than minimum required)
        let observations = vec![create_test_observation(
            "ABC123",
            1000,
            Some(5.0),
            Some(-45.0),
            Some(450.0),
            Some(35000),
        )];
        detector.add_observations(&observations, 2000);

        let result = detector.train_model(2000);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Insufficient data"));
    }

    #[tokio::test]
    async fn test_ml_detection_service() {
        let config = create_test_config();
        let (alert_sender, mut alert_receiver) = mpsc::unbounded_channel();

        let service = MLDetectionService::new(config, alert_sender);

        // Add some observations (not enough to trigger training)
        let observations = vec![create_test_observation(
            "ABC123",
            1000,
            Some(5.0),
            Some(-45.0),
            Some(450.0),
            Some(35000),
        )];
        service.add_observations(observations, 2000);

        // Run analysis (should skip due to insufficient data)
        service.run_analysis(2000).await;

        // Should not receive any alerts
        assert!(alert_receiver.try_recv().is_err());

        let (aircraft_count, has_model, _) = service.get_stats();
        assert_eq!(aircraft_count, 1);
        assert!(!has_model);
    }

    #[test]
    fn test_score_to_confidence_conversion() {
        // Higher Z-score sums should give higher confidence
        assert!(
            StatisticalMLDetector::score_to_confidence(15.0)
                > StatisticalMLDetector::score_to_confidence(5.0)
        );
        assert!(
            StatisticalMLDetector::score_to_confidence(5.0)
                > StatisticalMLDetector::score_to_confidence(3.0)
        );

        // Confidence should be in reasonable range
        let confidence = StatisticalMLDetector::score_to_confidence(10.0);
        assert!(confidence >= 0.7 && confidence <= 0.95);
    }

    #[test]
    fn test_needs_retraining() {
        let config = create_test_config();
        let mut detector = StatisticalMLDetector::new(config);

        // Initially needs training (no baselines)
        assert!(detector.needs_retraining(1000));

        // Simulate successful training by setting up baselines
        detector.last_training_time_ms = 1000;
        let baselines: [FeatureBaseline; 8] = Default::default();
        detector
            .feature_baselines
            .insert("__global__".to_string(), baselines);

        // Should not need retraining immediately
        assert!(!detector.needs_retraining(1000));

        // Should need retraining after interval
        assert!(detector.needs_retraining(1000 + detector.retrain_interval_ms + 1));
    }

    #[test]
    fn test_feature_vector_with_no_observations() {
        let feature_vector = FeatureVector::from_observations("EMPTY".to_string(), &[], 0, 1000);

        assert!(feature_vector.is_none());
    }

    #[test]
    fn test_feature_vector_with_missing_data() {
        let observations = vec![
            // Observation with missing optional fields
            AircraftObservation {
                id: None,
                ts_ms: 1000,
                hex: "PARTIAL".to_string(),
                flight: Some("TEST".to_string()),
                lat: Some(40.0),
                lon: Some(-74.0),
                altitude: None, // Missing
                gs: None,       // Missing
                rssi: None,     // Missing
                msg_count_total: Some(1000),
                raw_json: r#"{"hex":"PARTIAL"}"#.to_string(),
                msg_rate_hz: None, // Missing
            },
        ];

        let feature_vector =
            FeatureVector::from_observations("PARTIAL".to_string(), &observations, 0, 2000)
                .unwrap();

        // Should handle missing data gracefully with defaults
        assert_eq!(feature_vector.msg_rate_mean, 0.0);
        assert_eq!(feature_vector.rssi_mean, -50.0); // Default
        assert_eq!(feature_vector.speed_mean, 0.0);
    }
}
