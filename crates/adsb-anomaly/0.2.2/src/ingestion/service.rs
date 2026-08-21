// ABOUTME: Real-time ADS-B data ingestion service with PiAware polling
// ABOUTME: Fetches aircraft.json snapshots, normalizes data, and populates database

use crate::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState};
use crate::detect::behavior::BehaviorDetectionService;
use crate::detect::identity::IdentityDetectionService;
use crate::detect::signal::SignalDetectionService;
use crate::detect::temporal::TemporalDetectionService;
use crate::error::Result;
use crate::ingestion::{normalize, piaware};
use crate::store::{observations, sessions};
use sqlx::SqlitePool;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{sleep, Duration, Instant};
use tracing::{debug, error, info, warn, Instrument, Span};

/// Run continuous ingestion loop fetching from PiAware and updating database
#[allow(dead_code)] // Will be used in main.rs
pub async fn run_ingestion(pool: SqlitePool, url: String, interval_ms: u64) -> Result<()> {
    let circuit_config = CircuitBreakerConfig::default();
    run_ingestion_with_circuit_breaker(pool, url, interval_ms, circuit_config).await
}

/// Run continuous ingestion loop with circuit breaker configuration
#[allow(dead_code)] // Will be used in main.rs
pub async fn run_ingestion_with_circuit_breaker(
    pool: SqlitePool,
    url: String,
    interval_ms: u64,
    circuit_config: CircuitBreakerConfig,
) -> Result<()> {
    run_ingestion_with_detection_and_circuit_breaker(
        pool,
        url,
        interval_ms,
        circuit_config,
        None,
        None,
        None,
        None,
    )
    .await
}

/// Run continuous ingestion loop with temporal, signal, identity, and behavioral anomaly detection
#[allow(dead_code)]
pub async fn run_ingestion_with_detection(
    pool: SqlitePool,
    url: String,
    interval_ms: u64,
    temporal_service: Option<TemporalDetectionService>,
    signal_service: Option<SignalDetectionService>,
    identity_service: Option<IdentityDetectionService>,
    behavior_service: Option<BehaviorDetectionService>,
) -> Result<()> {
    let circuit_config = CircuitBreakerConfig::default();
    run_ingestion_with_detection_and_circuit_breaker(
        pool,
        url,
        interval_ms,
        circuit_config,
        temporal_service,
        signal_service,
        identity_service,
        behavior_service,
    )
    .await
}

/// Run continuous ingestion loop with circuit breaker and anomaly detection
#[allow(clippy::too_many_arguments)]
pub async fn run_ingestion_with_detection_and_circuit_breaker(
    pool: SqlitePool,
    url: String,
    interval_ms: u64,
    circuit_config: CircuitBreakerConfig,
    temporal_service: Option<TemporalDetectionService>,
    signal_service: Option<SignalDetectionService>,
    identity_service: Option<IdentityDetectionService>,
    behavior_service: Option<BehaviorDetectionService>,
) -> Result<()> {
    info!("Starting ADS-B ingestion service");
    info!("PiAware URL: {}", url);
    info!("Poll interval: {}ms", interval_ms);
    info!(
        "Circuit breaker config: failure_threshold={}, timeout_ms={}, success_threshold={}",
        circuit_config.failure_threshold,
        circuit_config.timeout_ms,
        circuit_config.success_threshold
    );

    // Create circuit breaker for PiAware requests
    let circuit_breaker = Arc::new(CircuitBreaker::new("PiAware".to_string(), circuit_config));

    let mut backoff_delay = 1000u64; // Start with 1 second backoff
    let max_backoff = 60000u64; // Max 60 second backoff

    loop {
        let loop_start = Instant::now();
        let span = Span::current();

        match run_ingestion_tick_with_circuit_breaker(
            &pool,
            &url,
            circuit_breaker.clone(),
            temporal_service.as_ref(),
            signal_service.as_ref(),
            identity_service.as_ref(),
            behavior_service.as_ref(),
        )
        .instrument(span)
        .await
        {
            Ok(count) => {
                let cycle_time = loop_start.elapsed();
                debug!(
                    "Processed {} aircraft observations in {:?} ({:.2} aircraft/sec)",
                    count,
                    cycle_time,
                    count as f64 / cycle_time.as_secs_f64()
                );
                backoff_delay = 1000; // Reset backoff on success

                // Log circuit breaker state on successful cycles
                let cb_state = circuit_breaker.get_state();
                if cb_state != CircuitState::Closed {
                    info!(
                        "Circuit breaker state: {:?}, failures: {}, successes: {}",
                        cb_state,
                        circuit_breaker.get_failure_count(),
                        circuit_breaker.get_success_count()
                    );
                }

                // Performance warning if cycle exceeds poll interval
                if cycle_time.as_millis() > interval_ms.into() {
                    warn!(
                        "Performance bottleneck: Cycle time {}ms exceeds poll interval {}ms",
                        cycle_time.as_millis(),
                        interval_ms
                    );
                }
            }
            Err(e) => {
                let cb_state = circuit_breaker.get_state();
                let cb_failures = circuit_breaker.get_failure_count();

                match cb_state {
                    CircuitState::Open => {
                        warn!(
                            "Circuit breaker is OPEN (failures: {}). Ingestion paused - error: {}",
                            cb_failures, e
                        );
                        info!("Waiting for circuit breaker timeout before retry");
                    }
                    _ => {
                        warn!(
                            "Ingestion tick failed (CB state: {:?}, failures: {}): {}",
                            cb_state, cb_failures, e
                        );
                        error!("Backing off for {}ms before retry", backoff_delay);
                        backoff_delay = std::cmp::min(backoff_delay * 2, max_backoff);
                    }
                }

                sleep(Duration::from_millis(backoff_delay)).await;
                continue;
            }
        }

        // Sleep for remaining interval time
        let elapsed = loop_start.elapsed();
        let interval_duration = Duration::from_millis(interval_ms);
        if elapsed < interval_duration {
            sleep(interval_duration - elapsed).await;
        }
    }
}

/// Run a single ingestion tick: fetch, normalize, store, and update sessions
#[allow(dead_code)] // Used in tests
async fn run_ingestion_tick(pool: &SqlitePool, url: &str) -> Result<usize> {
    run_ingestion_tick_with_detection(pool, url, None, None, None, None).await
}

/// Run a single ingestion tick with circuit breaker and anomaly detection
#[allow(dead_code)] // Used in tests and integration tests
pub async fn run_ingestion_tick_with_circuit_breaker(
    pool: &SqlitePool,
    url: &str,
    circuit_breaker: Arc<CircuitBreaker>,
    temporal_service: Option<&TemporalDetectionService>,
    signal_service: Option<&SignalDetectionService>,
    identity_service: Option<&IdentityDetectionService>,
    behavior_service: Option<&BehaviorDetectionService>,
) -> Result<usize> {
    // Check circuit breaker before attempting request
    if !circuit_breaker.should_allow_request() {
        return Err(crate::error::Error::Generic(
            "Circuit breaker is open - requests are blocked".to_string(),
        ));
    }

    let snapshot = piaware_fetch_with_circuit_breaker(url, circuit_breaker).await?;

    process_snapshot_data(
        pool,
        snapshot,
        temporal_service,
        signal_service,
        identity_service,
        behavior_service,
    )
    .await
}

/// Run a single ingestion tick with temporal, signal, identity, and behavioral detection
#[allow(dead_code)] // Used in tests and integration tests
pub async fn run_ingestion_tick_with_detection(
    pool: &SqlitePool,
    url: &str,
    temporal_service: Option<&TemporalDetectionService>,
    signal_service: Option<&SignalDetectionService>,
    identity_service: Option<&IdentityDetectionService>,
    behavior_service: Option<&BehaviorDetectionService>,
) -> Result<usize> {
    // Create a default circuit breaker for backwards compatibility
    let circuit_breaker = Arc::new(CircuitBreaker::new(
        "PiAware-Legacy".to_string(),
        CircuitBreakerConfig::default(),
    ));

    run_ingestion_tick_with_circuit_breaker(
        pool,
        url,
        circuit_breaker,
        temporal_service,
        signal_service,
        identity_service,
        behavior_service,
    )
    .await
}

/// Fetch PiAware data with circuit breaker protection
async fn piaware_fetch_with_circuit_breaker(
    url: &str,
    circuit_breaker: Arc<CircuitBreaker>,
) -> Result<piaware::AircraftSnapshot> {
    debug!("Attempting PiAware fetch through circuit breaker");

    // Use a closure that returns the required error type
    let fetch_operation = || async {
        match piaware::fetch_snapshot(url).await {
            Ok(snapshot) => Ok(snapshot),
            Err(e) => Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
        }
    };

    // Execute through circuit breaker
    match circuit_breaker.execute(fetch_operation).await {
        Ok(snapshot) => Ok(snapshot),
        Err(crate::circuit_breaker::CircuitBreakerError::CircuitOpen) => {
            error!("PiAware circuit breaker is open");
            Err(crate::error::Error::Generic(
                "Circuit breaker is open - PiAware requests are blocked".to_string(),
            ))
        }
        Err(crate::circuit_breaker::CircuitBreakerError::OperationFailed(inner)) => {
            error!("PiAware fetch failed: {}", inner);
            Err(crate::error::Error::Generic(format!(
                "PiAware fetch failed: {}",
                inner
            )))
        }
    }
}

/// Process snapshot data through the ingestion pipeline
async fn process_snapshot_data(
    pool: &SqlitePool,
    snapshot: piaware::AircraftSnapshot,
    temporal_service: Option<&TemporalDetectionService>,
    signal_service: Option<&SignalDetectionService>,
    identity_service: Option<&IdentityDetectionService>,
    behavior_service: Option<&BehaviorDetectionService>,
) -> Result<usize> {
    let aircraft_count = snapshot.aircraft.len();
    if aircraft_count == 0 {
        debug!("No aircraft in snapshot");
        return Ok(0);
    }

    debug!("Processing {} aircraft from PiAware", aircraft_count);

    // Get current timestamp
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    // Normalize entries to observations
    let normalize_span = tracing::span!(tracing::Level::DEBUG, "normalize_data");
    let _guard = normalize_span.enter();

    let mut observations = Vec::with_capacity(aircraft_count);
    for entry in &snapshot.aircraft {
        let obs = normalize::normalize_entry(now_ms, entry);
        observations.push(obs);
    }

    debug!("Normalized {} observations", observations.len());
    drop(_guard);

    // Insert observations using high-performance batch operation
    let insert_span = tracing::span!(tracing::Level::DEBUG, "batch_insert_observations");
    let _guard = insert_span.enter();

    let inserted = observations::insert_observations(pool, &observations).await?;
    debug!("Batch inserted {} observations", inserted);
    drop(_guard);

    // Batch upsert sessions for maximum performance and run anomaly detection
    let session_span = tracing::span!(tracing::Level::DEBUG, "batch_upsert_sessions");
    let _guard = session_span.enter();

    // Use high-performance batch session processing
    let mut obs_clone = observations.to_vec();
    sessions::batch_upsert_sessions_from_observations(pool, &mut obs_clone).await?;
    debug!("Batch processed {} sessions", obs_clone.len());

    // Update original observations with calculated message rates
    for (original, updated) in observations.iter_mut().zip(obs_clone.iter()) {
        original.msg_rate_hz = updated.msg_rate_hz;
    }
    drop(_guard);

    // Run anomaly detection on all processed observations
    let detection_span = tracing::span!(tracing::Level::DEBUG, "anomaly_detection");
    let _guard = detection_span.enter();
    let observation_count = observations.len();

    for obs in observations.iter().cloned() {
        // Run temporal detection if service is available
        if let Some(temporal) = temporal_service {
            temporal.process_observation(obs.clone());
        }

        // Run signal detection if service is available
        if let Some(signal) = signal_service {
            signal.process_observation(obs.clone());
        }

        // Run identity detection if service is available
        if let Some(identity) = identity_service {
            identity.process_observation(obs.clone());
        }

        // Run behavioral detection if service is available
        if let Some(behavior) = behavior_service {
            behavior.process_observation(obs);
        }
    }

    debug!(
        "Completed anomaly detection for {} observations",
        observation_count
    );
    drop(_guard);

    info!(
        "Ingestion tick completed: {} aircraft processed",
        aircraft_count
    );

    Ok(aircraft_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::connect_and_migrate;
    use axum::{http::StatusCode, response::Json, routing::get, Router};
    use serde_json::json;
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;
    use tokio::net::TcpListener;
    use tokio::time::{sleep, Duration};

    async fn create_test_pool() -> (SqlitePool, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = connect_and_migrate(db_path.to_str().unwrap(), true)
            .await
            .unwrap();
        (pool, temp_dir)
    }

    #[tokio::test]
    async fn test_run_ingestion_tick_with_data() {
        let (pool, _temp_dir) = create_test_pool().await;

        // Create test HTTP server that serves aircraft JSON
        let _response_data = json!({
            "now": 1641024000.5,
            "messages": 12345,
            "aircraft": [
                {
                    "hex": "abc123",
                    "flight": "UAL456",
                    "lat": 40.7128,
                    "lon": -74.0060,
                    "alt_baro": 35000,
                    "gs": 450.2,
                    "rssi": -45.5,
                    "messages": 1000
                },
                {
                    "hex": "def456",
                    "flight": "DAL789",
                    "lat": 34.0522,
                    "lon": -118.2437,
                    "alt_baro": 28000,
                    "gs": 380.0,
                    "rssi": -52.1,
                    "messages": 750
                }
            ]
        });

        async fn handler() -> std::result::Result<Json<serde_json::Value>, StatusCode> {
            Ok(Json(json!({
                "now": 1641024000.5,
                "messages": 12345,
                "aircraft": [
                    {
                        "hex": "abc123",
                        "flight": "UAL456",
                        "lat": 40.7128,
                        "lon": -74.0060,
                        "alt_baro": 35000,
                        "gs": 450.2,
                        "rssi": -45.5,
                        "messages": 1000
                    },
                    {
                        "hex": "def456",
                        "flight": "DAL789",
                        "lat": 34.0522,
                        "lon": -118.2437,
                        "alt_baro": 28000,
                        "gs": 380.0,
                        "rssi": -52.1,
                        "messages": 750
                    }
                ]
            })))
        }

        let app = Router::new().route("/data/aircraft.json", get(handler));

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        // Give server time to start
        sleep(Duration::from_millis(10)).await;

        let url = format!("http://{}/data/aircraft.json", addr);

        // Run ingestion tick
        let count = run_ingestion_tick(&pool, &url).await.unwrap();
        assert_eq!(count, 2);

        // Verify observations were inserted (normalize converts hex to uppercase)
        let observations = observations::list_observations_by_hex(&pool, "ABC123", 10)
            .await
            .unwrap();
        assert_eq!(observations.len(), 1);
        assert_eq!(observations[0].flight, Some("UAL456".to_string()));

        // Verify sessions were created
        let sessions = sessions::list_sessions(&pool, 10).await.unwrap();
        assert_eq!(
            sessions.len(),
            2,
            "Expected 2 sessions, found {}: {:?}",
            sessions.len(),
            sessions.iter().map(|s| &s.hex).collect::<Vec<_>>()
        );

        // Find ABC123 session
        let abc_session = sessions
            .iter()
            .find(|s| s.hex == "ABC123")
            .expect("ABC123 session should exist");
        assert_eq!(abc_session.flight, Some("UAL456".to_string()));
        assert_eq!(abc_session.message_count, 1);
    }

    #[tokio::test]
    async fn test_circuit_breaker_functionality() {
        let (pool, _temp_dir) = create_test_pool().await;

        // Create a circuit breaker that opens quickly
        let circuit_config = CircuitBreakerConfig {
            failure_threshold: 2,
            timeout_ms: 100, // Short timeout for test
            success_threshold: 1,
        };
        let circuit_breaker = Arc::new(CircuitBreaker::new("test".to_string(), circuit_config));

        // Test error server
        async fn error_handler() -> std::result::Result<Json<serde_json::Value>, StatusCode> {
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }

        let app = Router::new().route("/data/aircraft.json", get(error_handler));
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        sleep(Duration::from_millis(10)).await;
        let url = format!("http://{}/data/aircraft.json", addr);

        // First failure
        let result = run_ingestion_tick_with_circuit_breaker(
            &pool,
            &url,
            circuit_breaker.clone(),
            None,
            None,
            None,
            None,
        )
        .await;
        assert!(result.is_err());
        assert_eq!(circuit_breaker.get_state(), CircuitState::Closed);

        // Second failure should open circuit
        let result = run_ingestion_tick_with_circuit_breaker(
            &pool,
            &url,
            circuit_breaker.clone(),
            None,
            None,
            None,
            None,
        )
        .await;
        assert!(result.is_err());
        assert_eq!(circuit_breaker.get_state(), CircuitState::Open);

        // Third request should be blocked immediately
        let result = run_ingestion_tick_with_circuit_breaker(
            &pool,
            &url,
            circuit_breaker.clone(),
            None,
            None,
            None,
            None,
        )
        .await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Circuit breaker is open"));
    }

    #[tokio::test]
    async fn test_circuit_breaker_recovery() {
        let (pool, _temp_dir) = create_test_pool().await;

        // Create circuit breaker with very short timeout
        let circuit_config = CircuitBreakerConfig {
            failure_threshold: 1,
            timeout_ms: 50,
            success_threshold: 1,
        };
        let circuit_breaker = Arc::new(CircuitBreaker::new(
            "recovery_test".to_string(),
            circuit_config,
        ));

        // Force circuit open
        circuit_breaker.force_open();
        assert_eq!(circuit_breaker.get_state(), CircuitState::Open);

        // Wait for timeout
        sleep(Duration::from_millis(60)).await;

        // Create success server
        async fn success_handler() -> std::result::Result<Json<serde_json::Value>, StatusCode> {
            Ok(Json(json!({
                "now": 1641024000.0,
                "messages": 0,
                "aircraft": []
            })))
        }

        let app = Router::new().route("/data/aircraft.json", get(success_handler));
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        sleep(Duration::from_millis(10)).await;
        let url = format!("http://{}/data/aircraft.json", addr);

        // Should transition to half-open and succeed
        let result = run_ingestion_tick_with_circuit_breaker(
            &pool,
            &url,
            circuit_breaker.clone(),
            None,
            None,
            None,
            None,
        )
        .await;
        assert!(result.is_ok());
        assert_eq!(circuit_breaker.get_state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_integration_two_ticks() {
        let (pool, _temp_dir) = create_test_pool().await;

        // State to track which response to serve
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
                        "aircraft": [
                            {
                                "hex": "abc123",
                                "flight": "UAL456",
                                "lat": 40.7128,
                                "lon": -74.0060,
                                "alt_baro": 35000,
                                "gs": 450.0,
                                "rssi": -45.5,
                                "messages": 100
                            }
                        ]
                    })))
                } else {
                    // Second response - same aircraft, more messages
                    Ok(Json(json!({
                        "now": 1641024001.0,
                        "messages": 12355,
                        "aircraft": [
                            {
                                "hex": "abc123",
                                "flight": "UAL456",
                                "lat": 40.7129,
                                "lon": -74.0061,
                                "alt_baro": 35100,
                                "gs": 451.0,
                                "rssi": -45.0,
                                "messages": 110
                            }
                        ]
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

        // First tick
        let count1 = run_ingestion_tick(&pool, &url).await.unwrap();
        assert_eq!(count1, 1);

        // Wait a bit to simulate real polling interval
        sleep(Duration::from_millis(300)).await;

        // Second tick
        let count2 = run_ingestion_tick(&pool, &url).await.unwrap();
        assert_eq!(count2, 1);

        // Verify we have 2 observations for the same aircraft
        let observations = observations::list_observations_by_hex(&pool, "ABC123", 10)
            .await
            .unwrap();
        assert_eq!(observations.len(), 2);

        // Verify session has been updated (should still be only 1 session)
        let sessions = sessions::list_sessions(&pool, 10).await.unwrap();
        assert_eq!(sessions.len(), 1);

        let session = &sessions[0];
        assert_eq!(session.hex, "ABC123");
        // Message count should be 1 + 10 (delta from 100->110)
        assert_eq!(session.message_count, 11);

        // Check that the second observation has msg_rate_hz calculated
        // (This would be populated during session upsert)
        let _latest_obs = &observations[0]; // Should be ordered by ts_ms DESC
                                            // Note: msg_rate_hz would be calculated during upsert but isn't persisted to DB
    }

    #[tokio::test]
    async fn test_run_ingestion_tick_empty_response() {
        let (pool, _temp_dir) = create_test_pool().await;

        async fn empty_handler() -> std::result::Result<Json<serde_json::Value>, StatusCode> {
            Ok(Json(json!({
                "now": 1641024000.0,
                "messages": 0,
                "aircraft": []
            })))
        }

        let app = Router::new().route("/data/aircraft.json", get(empty_handler));

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        sleep(Duration::from_millis(10)).await;

        let url = format!("http://{}/data/aircraft.json", addr);

        // Run ingestion tick with empty data
        let count = run_ingestion_tick(&pool, &url).await.unwrap();
        assert_eq!(count, 0);

        // Verify no data was inserted
        let sessions = sessions::list_sessions(&pool, 10).await.unwrap();
        assert_eq!(sessions.len(), 0);
    }

    #[tokio::test]
    async fn test_run_ingestion_tick_server_error() {
        let (pool, _temp_dir) = create_test_pool().await;

        async fn error_handler() -> std::result::Result<Json<serde_json::Value>, StatusCode> {
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }

        let app = Router::new().route("/data/aircraft.json", get(error_handler));

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        sleep(Duration::from_millis(10)).await;

        let url = format!("http://{}/data/aircraft.json", addr);

        // Run ingestion tick - should fail
        let result = run_ingestion_tick(&pool, &url).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_circuit_breaker_stress_test() {
        let (pool, _temp_dir) = create_test_pool().await;

        // Create circuit breaker with aggressive settings for stress testing
        let circuit_config = CircuitBreakerConfig {
            failure_threshold: 3,
            timeout_ms: 100,
            success_threshold: 2,
        };
        let circuit_breaker = Arc::new(CircuitBreaker::new(
            "stress_test".to_string(),
            circuit_config,
        ));

        // Counter to simulate intermittent failures
        let request_count = Arc::new(std::sync::atomic::AtomicU32::new(0));

        let request_count_clone = request_count.clone();
        let handler = move |(): ()| {
            let request_count = request_count_clone.clone();
            async move {
                let count = request_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

                // Fail first 3 requests, then succeed twice, then fail again
                if count < 3 || (count >= 5 && count < 8) {
                    Err::<Json<serde_json::Value>, StatusCode>(StatusCode::INTERNAL_SERVER_ERROR)
                } else {
                    Ok(Json(json!({
                        "now": 1641024000.0,
                        "messages": 0,
                        "aircraft": []
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

        // First 3 requests should fail and open the circuit
        for i in 1..=3 {
            let result = run_ingestion_tick_with_circuit_breaker(
                &pool,
                &url,
                circuit_breaker.clone(),
                None,
                None,
                None,
                None,
            )
            .await;
            assert!(result.is_err(), "Request {} should fail", i);
        }

        // Circuit should be open now
        assert_eq!(circuit_breaker.get_state(), CircuitState::Open);
        assert_eq!(circuit_breaker.get_failure_count(), 3);

        // Request should be blocked immediately without hitting server
        let result = run_ingestion_tick_with_circuit_breaker(
            &pool,
            &url,
            circuit_breaker.clone(),
            None,
            None,
            None,
            None,
        )
        .await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Circuit breaker is open"));

        // Wait for timeout to transition to half-open
        sleep(Duration::from_millis(150)).await;

        // Next requests should succeed and close circuit
        for i in 1..=2 {
            let result = run_ingestion_tick_with_circuit_breaker(
                &pool,
                &url,
                circuit_breaker.clone(),
                None,
                None,
                None,
                None,
            )
            .await;
            assert!(result.is_ok(), "Recovery request {} should succeed", i);
        }

        // Circuit should be closed now
        assert_eq!(circuit_breaker.get_state(), CircuitState::Closed);
        assert_eq!(circuit_breaker.get_failure_count(), 0);

        // Test that it opens again on subsequent failures
        for i in 1..=3 {
            let result = run_ingestion_tick_with_circuit_breaker(
                &pool,
                &url,
                circuit_breaker.clone(),
                None,
                None,
                None,
                None,
            )
            .await;
            assert!(result.is_err(), "Second round failure {} should fail", i);
        }

        // Should be open again
        assert_eq!(circuit_breaker.get_state(), CircuitState::Open);
    }

    #[tokio::test]
    async fn test_circuit_breaker_prevents_resource_exhaustion() {
        let (pool, _temp_dir) = create_test_pool().await;

        // Create circuit breaker with very low threshold
        let circuit_config = CircuitBreakerConfig {
            failure_threshold: 2,
            timeout_ms: 5000, // Long timeout to test blocking
            success_threshold: 1,
        };
        let circuit_breaker = Arc::new(CircuitBreaker::new(
            "exhaustion_test".to_string(),
            circuit_config,
        ));

        // Always failing server
        async fn always_fail() -> std::result::Result<Json<serde_json::Value>, StatusCode> {
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }

        let app = Router::new().route("/data/aircraft.json", get(always_fail));
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        sleep(Duration::from_millis(10)).await;
        let url = format!("http://{}/data/aircraft.json", addr);

        // Make requests that will open the circuit
        for _ in 0..2 {
            let result = run_ingestion_tick_with_circuit_breaker(
                &pool,
                &url,
                circuit_breaker.clone(),
                None,
                None,
                None,
                None,
            )
            .await;
            assert!(result.is_err());
        }

        // Circuit should be open
        assert_eq!(circuit_breaker.get_state(), CircuitState::Open);

        // Time many blocked requests to ensure they return immediately
        let start = std::time::Instant::now();
        for _ in 0..10 {
            let result = run_ingestion_tick_with_circuit_breaker(
                &pool,
                &url,
                circuit_breaker.clone(),
                None,
                None,
                None,
                None,
            )
            .await;
            assert!(result.is_err());
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("Circuit breaker is open"));
        }
        let elapsed = start.elapsed();

        // All 10 requests should complete very quickly (much less than timeout period)
        // since they're blocked by the circuit breaker, not waiting for network timeouts
        assert!(
            elapsed < Duration::from_millis(500),
            "Circuit breaker should prevent slow requests, took {:?}",
            elapsed
        );
    }
}
