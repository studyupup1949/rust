After reviewing the codebase, I've identified several issues that need addressing. Here are the top problems:

## Issue #1: Circuit Breaker Vulnerable to Race Conditions
**File**: `src/circuit_breaker.rs`

The circuit breaker implementation uses atomic variables but has race conditions in its state transitions. The `should_allow_request()` and state update operations aren't atomic, potentially allowing multiple requests through when transitioning from Open to Half-Open state.

```rust
// Line 77-96
match current_state {
    // ...
    CircuitState::Open => {
        let last_failure = self.last_failure.load(Ordering::Relaxed);
        let time_since_failure = now_ms - last_failure;
        if time_since_failure > self.config.timeout_ms {
            // RACE CONDITION: Multiple concurrent requests can all enter this block
            // and all transition to half-open simultaneously
            self.state.store(CircuitState::HalfOpen as u8, Ordering::Relaxed);
            self.success_count.store(0, Ordering::Relaxed);
            // ...
            true // All concurrent requests will return true
        } else { /* ... */ }
    }
    // ...
}
```

## Issue #2: Insecure WebSocket Implementation
**File**: `src/web/websocket.rs`

The WebSocket handlers don't authenticate connections and blindly execute scripts from arbitrary sources, potentially allowing XSS attacks:

```rust
// Line 233-258
// Execute any script elements
const scripts = tempDiv.querySelectorAll('script');
console.log('Found script elements:', scripts.length);
scripts.forEach((script, index) => {
    console.log(`Executing script ${index}:`, script.textContent.substring(0, 100));
    eval(script.textContent); // Dangerous eval of arbitrary script content
});
```

## Issue #3: Inefficient Batch Database Operations
**File**: `src/store/sessions.rs`

The `batch_upsert_sessions_from_observations` function fetches all existing sessions at once, but then performs individual updates in a loop instead of a true batch update:

```rust
// Line 410-463
// Step 4: Batch update existing sessions
for (
    hex,
    last_seen_ms,
    last_msg_total,
    message_count,
    has_position,
    has_altitude,
    has_callsign,
    flight,
    lat,
    lon,
    altitude,
    speed,
    has_rssi,
) in session_updates {
    sqlx::query(
        r#"
        UPDATE aircraft_sessions SET
            last_seen_ms = ?,
            // ... many parameters
        WHERE hex = ?
        "#,
    )
    .bind(last_seen_ms)
    // ... many binds
    .bind(&hex)
    .execute(&mut *tx)
    .await?;
}
```

## Issue #4: Missing Error Handling in WebSocket Code
**File**: `src/web/mod.rs`

The WebSocket error handling is inadequate, with errors simply logged but not properly communicated to clients:

```rust
// Line 230-237
if sender
    .send(Message::Text(message.to_string()))
    .await
    .is_err()
{
    warn!("Failed to send alert to WebSocket client");
    break;
}
```

## Issue #5: Problematic ML Detection Memory Management
**File**: `src/detect/ml.rs`

The ML detection service stores unbounded history with no effective cleanup mechanism, potentially causing memory leaks:

```rust
// Line 204-217
pub fn add_observations(&mut self, observations: &[AircraftObservation], current_time_ms: i64) {
    for obs in observations {
        let history = self.feature_history.entry(obs.hex.clone()).or_default();
        history.push(obs.clone()); // Accumulates forever
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
```

## Issue #6: SQL Injection Vulnerability in Sessions Operations
**File**: `src/store/sessions.rs`

The batch insert function builds SQL manually without proper parameterization, opening a SQL injection vulnerability:

```rust
// Line 357-405
let values: Vec<String> = new_sessions
    .iter()
    .map(
        |(
            hex,
            first_seen,
            last_seen,
            last_msg_total,
            has_position,
            has_altitude,
            has_callsign,
            flight,
            lat,
            lon,
            altitude,
            speed,
            has_rssi,
        )| {
            format!(
                "('{}', {}, {}, {}, 1, {}, {}, {}, {}, {}, {}, {}, {}, 1, {}, 1, {})",
                hex.replace("'", "''"), // Insufficient SQL injection protection
                first_seen,
                last_seen,
                // ... more parameters
            )
        },
    )
    .collect();
let insert_sql = format!(
    r#"INSERT INTO aircraft_sessions (
        hex, first_seen_ms, last_seen_ms, last_msg_total, message_count,
        has_position, has_altitude, has_callsign,
        flight, lat, lon, altitude, speed,
        tier_temporal, tier_signal, tier_identity, tier_behavioral
    ) VALUES {}"#,
    values.join(",")
);
sqlx::query(&insert_sql).execute(&mut *tx).await?;
```

## Issue #7: Poor Error Handling in Ingestion Service
**File**: `src/ingestion/service.rs`

The error handling in the ingestion service is inadequate, ignoring proper error propagation and using generic error types:

```rust
// Line 228-254
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
```

## Issue #8: Missing Timestamp Validation in Session Service
**File**: `src/store/sessions.rs`

The session service doesn't validate timestamps before using them, potentially allowing future timestamps:

```rust
// Line 24-125
// Missing validation for obs.ts_ms to ensure it's not in the future
let now_ms = obs.ts_ms;  // This could be arbitrary
```

## Issue #9: Potential Deadlocks in Detection Services
**Files**: Various detector service implementations

The detection services use `Mutex::lock().unwrap()` which can deadlock if a panic occurs while the mutex is held:

```rust
// Example from src/detect/identity.rs, line 284-293
pub fn process_observation(&self, obs: AircraftObservation) {
    let mut detector = self.detector.lock().unwrap(); // Can deadlock if panic occurs while locked
    let anomalies = detector.detect_all(&obs);
    // Send all detected anomalies through the alert channel
    for anomaly in anomalies {
        if self.alert_sender.send(anomaly).is_err() {
            tracing::warn!("Failed to send identity anomaly alert: channel closed");
        }
    }
}
```

## Issue #10: No Rate Limiting on API Endpoints
**File**: `src/web/mod.rs`

The web API endpoints have no rate limiting protection, making them vulnerable to DoS attacks:

```rust
// Line 96-101 (Router configuration without rate limiting)
Router::new()
    .route("/healthz", get(health_handler))
    .route("/metrics", get(metrics_handler))
    // Dashboard pages (HTML) - use full dashboard with enhanced features
    .merge(dashboard::router().with_state(state.clone()))
    // ... more routes without rate limiting
```
