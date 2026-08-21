// ABOUTME: Temporal anomaly detection for message timing and frequency patterns
// ABOUTME: Detects rapid transmission rates and burst-after-silence patterns

use crate::config::AnalysisConfig;
use crate::model::{AircraftObservation, AnomalyCandidate, AnomalyType};
use dashmap::DashMap;
use serde_json::json;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, warn};

/// Ring buffer for storing recent observations per aircraft
#[derive(Debug, Clone)]
struct ObservationRingBuffer {
    observations: VecDeque<AircraftObservation>,
    max_size: usize,
    last_seen_ms: i64,
}

impl ObservationRingBuffer {
    fn new(max_size: usize) -> Self {
        Self {
            observations: VecDeque::with_capacity(max_size),
            max_size,
            last_seen_ms: 0,
        }
    }

    fn push(&mut self, obs: AircraftObservation) {
        self.last_seen_ms = obs.ts_ms;

        // Add to front of deque (most recent first)
        self.observations.push_front(obs);

        // Remove oldest if we exceed max size
        if self.observations.len() > self.max_size {
            self.observations.pop_back();
        }
    }

    fn get_recent_observations(&self) -> Vec<AircraftObservation> {
        self.observations.iter().cloned().collect()
    }

    fn get_last_gap_ms(&self, current_ts_ms: i64) -> Option<u64> {
        if self.last_seen_ms > 0 && current_ts_ms > self.last_seen_ms {
            Some((current_ts_ms - self.last_seen_ms) as u64)
        } else {
            None
        }
    }
}

/// Temporal detection service with ring buffers per aircraft
#[derive(Clone)]
pub struct TemporalDetectionService {
    buffers: Arc<DashMap<String, ObservationRingBuffer>>,
    config: Arc<AnalysisConfig>,
    alert_sender: mpsc::UnboundedSender<AnomalyCandidate>,
    window_size: usize,
    max_aircraft: usize,     // Maximum number of aircraft to track
    stale_threshold_ms: i64, // Time threshold for removing stale aircraft
}

impl TemporalDetectionService {
    /// Create new temporal detection service with memory management
    pub fn new(
        config: Arc<AnalysisConfig>,
        alert_sender: mpsc::UnboundedSender<AnomalyCandidate>,
        window_size: Option<usize>,
    ) -> Self {
        Self::new_with_limits(config, alert_sender, window_size, None, None)
    }

    /// Create new temporal detection service with memory limits
    pub fn new_with_limits(
        config: Arc<AnalysisConfig>,
        alert_sender: mpsc::UnboundedSender<AnomalyCandidate>,
        window_size: Option<usize>,
        max_aircraft: Option<usize>,
        stale_threshold_minutes: Option<u32>,
    ) -> Self {
        Self {
            buffers: Arc::new(DashMap::new()),
            config,
            alert_sender,
            window_size: window_size.unwrap_or(10), // Default 10 observations per aircraft
            max_aircraft: max_aircraft.unwrap_or(5000), // Default max 5000 aircraft
            stale_threshold_ms: stale_threshold_minutes.unwrap_or(30) as i64 * 60 * 1000, // Default 30 minutes
        }
    }

    /// Process a new observation and check for temporal anomalies
    pub fn process_observation(&self, obs: AircraftObservation) {
        let hex = obs.hex.clone();

        // Get or create ring buffer for this aircraft
        let mut entry = self
            .buffers
            .entry(hex.clone())
            .or_insert_with(|| ObservationRingBuffer::new(self.window_size));

        // Calculate gap before adding new observation
        let last_gap_ms = entry.get_last_gap_ms(obs.ts_ms);

        // Add observation to ring buffer
        entry.push(obs.clone());

        // Get current window for analysis
        let obs_window = entry.get_recent_observations();

        // Drop the entry to release the lock
        drop(entry);

        // Run temporal detection
        if let Some(anomaly) = detect_temporal(&obs_window, last_gap_ms, &self.config) {
            debug!(
                "Temporal anomaly detected for {}: {} (confidence: {:.2})",
                anomaly.hex, anomaly.subtype, anomaly.confidence
            );

            // Send anomaly to alert channel
            if let Err(e) = self.alert_sender.send(anomaly) {
                warn!("Failed to send temporal anomaly alert: {}", e);
            }
        }
    }

    /// Clean up stale aircraft and enforce memory limits
    #[allow(dead_code)] // Used in tests, will be used for production monitoring
    pub fn cleanup_and_manage_memory(&self, current_time_ms: i64) -> (usize, usize) {
        let initial_count = self.buffers.len();

        // First, remove stale aircraft
        let cutoff_time = current_time_ms - self.stale_threshold_ms;
        let mut stale_aircraft = Vec::new();

        for entry in self.buffers.iter() {
            let (hex, buffer) = entry.pair();
            if buffer.last_seen_ms < cutoff_time {
                stale_aircraft.push(hex.clone());
            }
        }

        for hex in &stale_aircraft {
            self.buffers.remove(hex);
        }

        let after_stale_cleanup = self.buffers.len();
        let stale_removed = initial_count - after_stale_cleanup;

        // If still over limit, remove oldest aircraft (LRU eviction)
        let mut lru_removed = 0;
        if after_stale_cleanup > self.max_aircraft {
            let excess_count = after_stale_cleanup - self.max_aircraft;

            // Collect aircraft with their last_seen times for LRU sorting
            let mut aircraft_times: Vec<(String, i64)> = self
                .buffers
                .iter()
                .map(|entry| {
                    let (hex, buffer) = entry.pair();
                    (hex.clone(), buffer.last_seen_ms)
                })
                .collect();

            // Sort by last_seen_ms ascending (oldest first)
            aircraft_times.sort_by_key(|(_, time)| *time);

            // Remove oldest aircraft
            for (hex, _) in aircraft_times.into_iter().take(excess_count) {
                self.buffers.remove(&hex);
                lru_removed += 1;
            }
        }

        if stale_removed > 0 || lru_removed > 0 {
            debug!(
                "Temporal memory cleanup: removed {} stale aircraft, {} by LRU eviction. Buffer count: {} -> {}",
                stale_removed, lru_removed, initial_count, self.buffers.len()
            );
        }

        (stale_removed, lru_removed)
    }

    /// Get current buffer stats for debugging and monitoring
    #[allow(dead_code)] // Will be used for monitoring/debugging
    pub fn get_buffer_stats(&self) -> Vec<(String, usize, i64)> {
        self.buffers
            .iter()
            .map(|entry| {
                let (hex, buffer) = entry.pair();
                (hex.clone(), buffer.observations.len(), buffer.last_seen_ms)
            })
            .collect()
    }

    /// Get memory usage statistics
    #[allow(dead_code)] // Used in tests, will be used for production monitoring
    pub fn get_memory_stats(&self) -> (usize, usize, usize) {
        let aircraft_count = self.buffers.len();
        let total_observations: usize = self
            .buffers
            .iter()
            .map(|entry| entry.value().observations.len())
            .sum();

        // Estimate memory usage (rough calculation)
        // Each observation ~200 bytes, each buffer ~100 bytes overhead
        let estimated_bytes = (total_observations * 200) + (aircraft_count * 100);

        (aircraft_count, total_observations, estimated_bytes)
    }

    /// Check if memory pressure cleanup is needed
    #[allow(dead_code)] // Used in tests, will be used for production monitoring
    pub fn needs_cleanup(&self, current_time_ms: i64) -> bool {
        let aircraft_count = self.buffers.len();

        // Always cleanup if over the limit
        if aircraft_count > self.max_aircraft {
            return true;
        }

        // Check for stale aircraft periodically
        let cutoff_time = current_time_ms - self.stale_threshold_ms;
        self.buffers
            .iter()
            .any(|entry| entry.value().last_seen_ms < cutoff_time)
    }
}

/// Detect temporal anomalies in aircraft message patterns
pub fn detect_temporal(
    obs_window: &[AircraftObservation],
    last_gap_ms: Option<u64>,
    config: &AnalysisConfig,
) -> Option<AnomalyCandidate> {
    if obs_window.is_empty() {
        return None;
    }

    let latest_obs = &obs_window[0]; // Assume sorted by timestamp descending
    let hex = latest_obs.hex.clone();

    // Check for burst after silence pattern first (higher priority)
    if let Some(gap_ms) = last_gap_ms {
        let max_gap_ms = config.max_session_gap_seconds * 1000;
        if gap_ms > max_gap_ms {
            // We had a long silence, now check if current rate is elevated
            if let Some(msg_rate_hz) = latest_obs.msg_rate_hz {
                // Consider it a burst if rate is above 50% of normal threshold
                let burst_threshold = config.max_messages_per_second * 0.5;
                if msg_rate_hz > burst_threshold {
                    return Some(
                        AnomalyCandidate::new(
                            hex,
                            AnomalyType::Temporal,
                            "burst_after_silence".to_string(),
                            0.8, // Slightly lower confidence as pattern is more complex
                        )
                        .with_details(json!({
                            "silence_duration_ms": gap_ms,
                            "max_gap_ms": max_gap_ms,
                            "burst_rate_hz": msg_rate_hz,
                            "burst_threshold": burst_threshold
                        }))
                        .with_trigger_observation(latest_obs.clone()),
                    );
                }
            }
        }
    }

    // Check for rapid transmission rate (only if no burst after silence detected)
    if let Some(msg_rate_hz) = latest_obs.msg_rate_hz {
        if msg_rate_hz > config.max_messages_per_second {
            return Some(
                AnomalyCandidate::new(
                    hex,
                    AnomalyType::Temporal,
                    "rapid_transmission".to_string(),
                    0.9, // High confidence for clear threshold violation
                )
                .with_details(json!({
                    "msg_rate_hz": msg_rate_hz,
                    "threshold": config.max_messages_per_second,
                    "violation_factor": if config.max_messages_per_second > 0.0 {
                        msg_rate_hz / config.max_messages_per_second
                    } else {
                        f64::INFINITY
                    }
                }))
                .with_trigger_observation(latest_obs.clone()),
            );
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::AircraftObservation;
    use tokio::sync::mpsc;

    fn create_test_config() -> AnalysisConfig {
        AnalysisConfig {
            max_messages_per_second: 10.0,
            min_message_interval_ms: 50,
            max_session_gap_seconds: 600, // 10 minutes
            min_rssi_units: -120.0,
            max_rssi_units: -10.0,
            suspicious_rssi_units: -20.0,
            suspicious_callsigns: vec![],
            invalid_hex_patterns: vec![],
        }
    }

    fn create_test_observation(
        hex: &str,
        ts_ms: i64,
        msg_rate_hz: Option<f64>,
    ) -> AircraftObservation {
        AircraftObservation {
            id: None,
            ts_ms,
            hex: hex.to_string(),
            flight: Some("TEST123".to_string()),
            lat: Some(40.0),
            lon: Some(-74.0),
            altitude: Some(35000),
            gs: Some(450.0),
            rssi: Some(-45.0),
            msg_count_total: Some(1000),
            raw_json: "{}".to_string(),
            msg_rate_hz,
        }
    }

    #[test]
    fn test_detect_rapid_transmission() {
        let config = create_test_config();
        let obs = create_test_observation("ABC123", 1641024000000, Some(15.0)); // Above 10.0 threshold
        let obs_window = vec![obs];

        let result = detect_temporal(&obs_window, None, &config);

        assert!(result.is_some());
        let anomaly = result.unwrap();
        assert_eq!(anomaly.anomaly_type, AnomalyType::Temporal);
        assert_eq!(anomaly.subtype, "rapid_transmission");
        assert_eq!(anomaly.confidence, 0.9);
        assert_eq!(anomaly.hex, "ABC123");

        // Check details
        let details = anomaly.details.unwrap();
        assert_eq!(details["msg_rate_hz"], 15.0);
        assert_eq!(details["threshold"], 10.0);
        assert_eq!(details["violation_factor"], 1.5);
    }

    #[test]
    fn test_detect_burst_after_silence() {
        let config = create_test_config();
        let obs = create_test_observation("ABC123", 1641024000000, Some(6.0)); // Above 5.0 burst threshold
        let obs_window = vec![obs];

        // Simulate a gap of 15 minutes (900 seconds = 900000 ms)
        let gap_ms = 900000u64; // Exceeds 600 second threshold

        let result = detect_temporal(&obs_window, Some(gap_ms), &config);

        assert!(result.is_some());
        let anomaly = result.unwrap();
        assert_eq!(anomaly.anomaly_type, AnomalyType::Temporal);
        assert_eq!(anomaly.subtype, "burst_after_silence");
        assert_eq!(anomaly.confidence, 0.8);
        assert_eq!(anomaly.hex, "ABC123");

        // Check details
        let details = anomaly.details.unwrap();
        assert_eq!(details["silence_duration_ms"], 900000);
        assert_eq!(details["max_gap_ms"], 600000);
        assert_eq!(details["burst_rate_hz"], 6.0);
        assert_eq!(details["burst_threshold"], 5.0);
    }

    #[test]
    fn test_no_anomaly_normal_rate() {
        let config = create_test_config();
        let obs = create_test_observation("ABC123", 1641024000000, Some(2.0)); // Well below threshold
        let obs_window = vec![obs];

        let result = detect_temporal(&obs_window, None, &config);
        assert!(result.is_none());
    }

    #[test]
    fn test_no_anomaly_steady_rate() {
        let config = create_test_config();
        let obs = create_test_observation("ABC123", 1641024000000, Some(1.5)); // Steady low rate
        let obs_window = vec![obs];

        let result = detect_temporal(&obs_window, None, &config);
        assert!(result.is_none());
    }

    #[test]
    fn test_no_anomaly_short_gap_high_rate() {
        let config = create_test_config();
        let obs = create_test_observation("ABC123", 1641024000000, Some(6.0)); // Above burst threshold
        let obs_window = vec![obs];

        // Short gap (5 minutes = 300000 ms) - within acceptable range
        let gap_ms = 300000u64;

        let result = detect_temporal(&obs_window, Some(gap_ms), &config);
        assert!(result.is_none());
    }

    #[test]
    fn test_no_anomaly_long_gap_low_rate() {
        let config = create_test_config();
        let obs = create_test_observation("ABC123", 1641024000000, Some(2.0)); // Below burst threshold
        let obs_window = vec![obs];

        // Long gap but low rate after - not a burst
        let gap_ms = 900000u64;

        let result = detect_temporal(&obs_window, Some(gap_ms), &config);
        assert!(result.is_none());
    }

    #[test]
    fn test_no_anomaly_missing_msg_rate() {
        let config = create_test_config();
        let obs = create_test_observation("ABC123", 1641024000000, None); // No message rate available
        let obs_window = vec![obs];

        let result = detect_temporal(&obs_window, None, &config);
        assert!(result.is_none());
    }

    #[test]
    fn test_empty_window() {
        let config = create_test_config();
        let obs_window = vec![];

        let result = detect_temporal(&obs_window, None, &config);
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_temporal_detection_service() {
        let config = Arc::new(create_test_config());
        let (alert_sender, mut alert_receiver) = mpsc::unbounded_channel();
        let service = TemporalDetectionService::new(config, alert_sender, Some(5));

        // Send normal observation - should not trigger anomaly
        let obs1 = create_test_observation("ABC123", 1641024000000, Some(2.0));
        service.process_observation(obs1);

        // Give a moment for processing
        tokio::task::yield_now().await;

        // Should be no alerts
        assert!(alert_receiver.try_recv().is_err());

        // Send high rate observation - should trigger anomaly
        let obs2 = create_test_observation("ABC123", 1641024001000, Some(15.0));
        service.process_observation(obs2);

        // Give a moment for processing
        tokio::task::yield_now().await;

        // Should have an alert
        let alert = alert_receiver
            .try_recv()
            .expect("Should have received an alert");
        assert_eq!(alert.anomaly_type, AnomalyType::Temporal);
        assert_eq!(alert.subtype, "rapid_transmission");
        assert_eq!(alert.hex, "ABC123");
    }

    #[tokio::test]
    async fn test_burst_after_silence_detection() {
        let config = Arc::new(create_test_config());
        let (alert_sender, mut alert_receiver) = mpsc::unbounded_channel();
        let service = TemporalDetectionService::new(config, alert_sender, Some(5));

        // Send initial observation
        let obs1 = create_test_observation("DEF456", 1641024000000, Some(2.0));
        service.process_observation(obs1);

        // Wait a moment
        tokio::task::yield_now().await;
        assert!(alert_receiver.try_recv().is_err());

        // Send observation after long silence with elevated rate
        let obs2 = create_test_observation("DEF456", 1641024000000 + 900000, Some(6.0)); // 15 min gap
        service.process_observation(obs2);

        // Should have a burst after silence alert
        let alert = alert_receiver
            .try_recv()
            .expect("Should have received an alert");
        assert_eq!(alert.anomaly_type, AnomalyType::Temporal);
        assert_eq!(alert.subtype, "burst_after_silence");
        assert_eq!(alert.hex, "DEF456");
    }

    #[test]
    fn test_ring_buffer() {
        let mut buffer = ObservationRingBuffer::new(3);

        let obs1 = create_test_observation("ABC123", 1000, Some(2.0));
        let obs2 = create_test_observation("ABC123", 2000, Some(3.0));
        let obs3 = create_test_observation("ABC123", 3000, Some(4.0));
        let obs4 = create_test_observation("ABC123", 4000, Some(5.0));

        buffer.push(obs1);
        buffer.push(obs2);
        buffer.push(obs3);
        assert_eq!(buffer.observations.len(), 3);

        // Adding 4th should remove oldest
        buffer.push(obs4);
        assert_eq!(buffer.observations.len(), 3);

        // Most recent should be first
        let recent = buffer.get_recent_observations();
        assert_eq!(recent[0].ts_ms, 4000);
        assert_eq!(recent[1].ts_ms, 3000);
        assert_eq!(recent[2].ts_ms, 2000);
    }

    #[test]
    fn test_ring_buffer_gap_calculation() {
        let mut buffer = ObservationRingBuffer::new(5);

        let obs1 = create_test_observation("ABC123", 1000, Some(2.0));
        buffer.push(obs1);

        // Gap should be 500ms
        let gap = buffer.get_last_gap_ms(1500);
        assert_eq!(gap, Some(500));

        // No gap for earlier timestamp
        let gap2 = buffer.get_last_gap_ms(800);
        assert_eq!(gap2, None);
    }

    #[tokio::test]
    async fn test_temporal_service_memory_management() {
        let config = Arc::new(create_test_config());
        let (alert_sender, _alert_receiver) = mpsc::unbounded_channel();

        // Create service with small limits for testing
        let service = TemporalDetectionService::new_with_limits(
            config,
            alert_sender,
            Some(5),
            Some(3), // Max 3 aircraft
            Some(1), // 1 minute timeout
        );

        let base_time = 1641024000000;

        // Add observations for 5 aircraft (exceeds limit of 3)
        for i in 0..5 {
            let hex = format!("AIRCRAFT{:02}", i);
            let obs = create_test_observation(&hex, base_time + (i * 1000), Some(2.0));
            service.process_observation(obs);
        }

        let (aircraft_count, _total_obs, _estimated_bytes) = service.get_memory_stats();
        assert_eq!(aircraft_count, 5); // All should be present initially

        // Trigger memory cleanup - should remove 2 oldest aircraft (LRU)
        let (stale_removed, lru_removed) = service.cleanup_and_manage_memory(base_time + 5000);
        assert_eq!(stale_removed, 0); // None are stale yet
        assert_eq!(lru_removed, 2); // Should remove 2 to get to limit of 3

        let (aircraft_count_after, _, _) = service.get_memory_stats();
        assert_eq!(aircraft_count_after, 3); // Should be at limit

        // Verify the oldest aircraft were removed (AIRCRAFT00 and AIRCRAFT01)
        let stats = service.get_buffer_stats();
        let remaining_aircraft: Vec<String> = stats.into_iter().map(|(hex, _, _)| hex).collect();
        assert!(!remaining_aircraft.contains(&"AIRCRAFT00".to_string()));
        assert!(!remaining_aircraft.contains(&"AIRCRAFT01".to_string()));
        assert!(remaining_aircraft.contains(&"AIRCRAFT02".to_string()));
        assert!(remaining_aircraft.contains(&"AIRCRAFT03".to_string()));
        assert!(remaining_aircraft.contains(&"AIRCRAFT04".to_string()));
    }

    #[tokio::test]
    async fn test_temporal_service_stale_cleanup() {
        let config = Arc::new(create_test_config());
        let (alert_sender, _alert_receiver) = mpsc::unbounded_channel();

        // Create service with 1 minute timeout
        let service = TemporalDetectionService::new_with_limits(
            config,
            alert_sender,
            Some(5),
            Some(10), // High limit
            Some(1),  // 1 minute timeout
        );

        let base_time = 1641024000000;

        // Add old observations (2 minutes ago)
        let old_obs = create_test_observation("OLD_AIRCRAFT", base_time - 120000, Some(2.0));
        service.process_observation(old_obs);

        // Add recent observation
        let recent_obs = create_test_observation("NEW_AIRCRAFT", base_time, Some(2.0));
        service.process_observation(recent_obs);

        let (aircraft_count, _, _) = service.get_memory_stats();
        assert_eq!(aircraft_count, 2);

        // Trigger cleanup - should remove stale aircraft
        let (stale_removed, lru_removed) = service.cleanup_and_manage_memory(base_time);
        assert_eq!(stale_removed, 1); // Should remove 1 stale aircraft
        assert_eq!(lru_removed, 0);

        let (aircraft_count_after, _, _) = service.get_memory_stats();
        assert_eq!(aircraft_count_after, 1);

        // Verify only the recent aircraft remains
        let stats = service.get_buffer_stats();
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].0, "NEW_AIRCRAFT");
    }

    #[tokio::test]
    async fn test_temporal_service_needs_cleanup() {
        let config = Arc::new(create_test_config());
        let (alert_sender, _alert_receiver) = mpsc::unbounded_channel();

        let service = TemporalDetectionService::new_with_limits(
            config,
            alert_sender,
            Some(5),
            Some(2), // Max 2 aircraft
            Some(1), // 1 minute timeout
        );

        let base_time = 1641024000000;

        // Should not need cleanup initially
        assert!(!service.needs_cleanup(base_time));

        // Add aircraft beyond limit
        for i in 0..3 {
            let hex = format!("AIRCRAFT{:02}", i);
            let obs = create_test_observation(&hex, base_time + (i * 1000), Some(2.0));
            service.process_observation(obs);
        }

        // Should need cleanup due to exceeding limit
        assert!(service.needs_cleanup(base_time));

        // Add stale aircraft
        let stale_obs = create_test_observation("STALE", base_time - 120000, Some(2.0));
        service.process_observation(stale_obs);

        // Should need cleanup due to stale aircraft
        assert!(service.needs_cleanup(base_time));
    }

    #[tokio::test]
    async fn test_memory_stats_calculation() {
        let config = Arc::new(create_test_config());
        let (alert_sender, _alert_receiver) = mpsc::unbounded_channel();

        let service = TemporalDetectionService::new_with_limits(
            config,
            alert_sender,
            Some(3), // 3 observations per aircraft
            Some(10),
            Some(30),
        );

        let base_time = 1641024000000;

        // Add multiple observations for multiple aircraft
        for aircraft in 0..2 {
            for obs in 0..3 {
                let hex = format!("PLANE{}", aircraft);
                let observation =
                    create_test_observation(&hex, base_time + (obs * 1000), Some(2.0));
                service.process_observation(observation);
            }
        }

        let (aircraft_count, total_observations, estimated_bytes) = service.get_memory_stats();
        assert_eq!(aircraft_count, 2);
        assert_eq!(total_observations, 6); // 2 aircraft * 3 observations each
        assert!(estimated_bytes > 0);

        // Memory should scale with observations
        assert!(estimated_bytes >= total_observations * 200); // Rough minimum
    }
}
