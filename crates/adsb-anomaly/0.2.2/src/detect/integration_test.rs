// ABOUTME: Integration test for temporal detection within the full ingestion pipeline
// ABOUTME: Tests that temporal anomalies are detected during real ingestion

#[cfg(test)]
mod tests {
    use crate::config::AnalysisConfig;
    use crate::detect::temporal::TemporalDetectionService;
    use crate::ingestion::service;
    use crate::model::AnomalyType;
    use crate::store::connect_and_migrate;
    use axum::{http::StatusCode, response::Json, routing::get, Router};
    use serde_json::json;
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;
    use tokio::net::TcpListener;
    use tokio::sync::mpsc;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_temporal_detection_integration() {
        // Create test database
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = connect_and_migrate(db_path.to_str().unwrap(), true)
            .await
            .unwrap();

        // Create alert processing system
        let (alert_sender, mut alert_receiver) = mpsc::unbounded_channel();

        // Create test config with low threshold for easier testing
        let config = Arc::new(AnalysisConfig {
            max_messages_per_second: 5.0, // Low threshold for testing
            min_message_interval_ms: 50,
            max_session_gap_seconds: 300,
            min_rssi_units: -120.0,
            max_rssi_units: -10.0,
            suspicious_rssi_units: -20.0,
            suspicious_callsigns: vec![],
            invalid_hex_patterns: vec![],
        });

        // Create temporal detection service
        let temporal_service = TemporalDetectionService::new(config, alert_sender, Some(5));

        // Create mock HTTP server with high message rate scenario
        let response_index = Arc::new(Mutex::new(0));
        let response_index_clone = response_index.clone();

        let handler = move |(): ()| {
            let response_index = response_index_clone.clone();
            async move {
                let mut idx = response_index.lock().unwrap();
                let current_idx = *idx;
                *idx += 1;

                if current_idx == 0 {
                    // First response - normal rate
                    Ok::<Json<serde_json::Value>, StatusCode>(Json(json!({
                        "now": 1641024000.0,
                        "messages": 12345,
                        "aircraft": [{
                            "hex": "abc123",
                            "flight": "TEST123",
                            "lat": 40.7128,
                            "lon": -74.0060,
                            "alt_baro": 35000,
                            "gs": 450.0,
                            "rssi": -45.0,
                            "messages": 100  // Starting point
                        }]
                    })))
                } else {
                    // Second response - high message rate (50 messages in ~1 second)
                    Ok(Json(json!({
                        "now": 1641024001.0,
                        "messages": 12395,
                        "aircraft": [{
                            "hex": "abc123",
                            "flight": "TEST123",
                            "lat": 40.7129,
                            "lon": -74.0061,
                            "alt_baro": 35100,
                            "gs": 451.0,
                            "rssi": -44.0,
                            "messages": 150  // Delta of 50 messages in ~1s = ~50 Hz (above 5.0 threshold)
                        }]
                    })))
                }
            }
        };

        let app = Router::new().route("/data/aircraft.json", get(handler));
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        sleep(Duration::from_millis(10)).await;

        let url = format!("http://{}/data/aircraft.json", addr);

        // Run first ingestion tick (normal rate)
        service::run_ingestion_tick_with_detection(
            &pool,
            &url,
            Some(&temporal_service),
            None,
            None,
            None,
        )
        .await
        .unwrap();

        // Wait a bit to simulate real polling interval
        sleep(Duration::from_millis(300)).await;

        // Run second ingestion tick (high rate - should trigger anomaly)
        service::run_ingestion_tick_with_detection(
            &pool,
            &url,
            Some(&temporal_service),
            None,
            None,
            None,
        )
        .await
        .unwrap();

        // Give temporal detector time to process
        sleep(Duration::from_millis(100)).await;

        // Check that we received a temporal anomaly alert
        let alert = alert_receiver
            .try_recv()
            .expect("Should have received a temporal anomaly alert");

        assert_eq!(alert.hex, "ABC123"); // Normalized to uppercase
        assert_eq!(alert.anomaly_type, AnomalyType::Temporal);
        assert_eq!(alert.subtype, "rapid_transmission");
        assert!(alert.confidence > 0.8);

        // Verify details contain the high message rate
        let details = alert.details.expect("Alert should have details");
        assert!(details["msg_rate_hz"].as_f64().unwrap() > 5.0);
        assert_eq!(details["threshold"], 5.0);
    }

    #[tokio::test]
    async fn test_burst_after_silence_integration() {
        // Create test database
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = connect_and_migrate(db_path.to_str().unwrap(), true)
            .await
            .unwrap();

        // Create alert processing system
        let (alert_sender, mut alert_receiver) = mpsc::unbounded_channel();

        // Create test config
        let config = Arc::new(AnalysisConfig {
            max_messages_per_second: 20.0, // Higher threshold so 6Hz doesn't trigger rapid_transmission
            min_message_interval_ms: 50,
            max_session_gap_seconds: 1, // Very short gap for testing (1 second)
            min_rssi_units: -120.0,
            max_rssi_units: -10.0,
            suspicious_rssi_units: -20.0,
            suspicious_callsigns: vec![],
            invalid_hex_patterns: vec![],
        });

        // Create temporal detection service
        let temporal_service = TemporalDetectionService::new(config, alert_sender, Some(5));

        // Create mock HTTP server
        let response_index = Arc::new(Mutex::new(0));
        let response_index_clone = response_index.clone();

        let handler = move |(): ()| {
            let response_index = response_index_clone.clone();
            async move {
                let mut idx = response_index.lock().unwrap();
                let current_idx = *idx;
                *idx += 1;

                if current_idx == 0 {
                    // First response
                    Ok::<Json<serde_json::Value>, StatusCode>(Json(json!({
                        "now": 1641024000.0,
                        "messages": 12345,
                        "aircraft": [{
                            "hex": "def456",
                            "flight": "BURST01",
                            "messages": 100
                        }]
                    })))
                } else {
                    // Second response after gap with burst rate
                    Ok(Json(json!({
                        "now": 1641024010.0, // 10 seconds later (exceeds 5s gap threshold)
                        "messages": 12365,
                        "aircraft": [{
                            "hex": "def456",
                            "flight": "BURST01",
                            "messages": 160  // 60 messages in 10s = 6.0 Hz, above burst threshold of 5.0 Hz
                        }]
                    })))
                }
            }
        };

        let app = Router::new().route("/data/aircraft.json", get(handler));
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        sleep(Duration::from_millis(10)).await;
        let url = format!("http://{}/data/aircraft.json", addr);

        // Run first ingestion tick
        service::run_ingestion_tick_with_detection(
            &pool,
            &url,
            Some(&temporal_service),
            None,
            None,
            None,
        )
        .await
        .unwrap();

        // Wait for gap (simulate time passing - must be > 1 second)
        sleep(Duration::from_millis(1100)).await;

        // Run second ingestion tick (after gap with burst)
        service::run_ingestion_tick_with_detection(
            &pool,
            &url,
            Some(&temporal_service),
            None,
            None,
            None,
        )
        .await
        .unwrap();

        // Give temporal detector time to process
        sleep(Duration::from_millis(100)).await;

        // Check that we received a burst after silence alert
        let alert = alert_receiver
            .try_recv()
            .expect("Should have received a burst after silence alert");

        assert_eq!(alert.hex, "DEF456");
        assert_eq!(alert.anomaly_type, AnomalyType::Temporal);
        assert_eq!(alert.subtype, "burst_after_silence");
        assert!(alert.confidence > 0.7);

        // Verify details
        let details = alert.details.expect("Alert should have details");
        assert!(details["silence_duration_ms"].as_u64().unwrap() >= 1000);
    }
}
