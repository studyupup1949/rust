// ABOUTME: Database operations for aircraft sessions - upserts and session management
// ABOUTME: Tracks aircraft presence, capabilities, and last known state

use crate::error::Result;
use crate::model::{AircraftObservation, AircraftSession};
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;

/// Upsert aircraft session based on observation data with message rate calculation
/// Creates new session or updates existing one with fresh data and computes msg_rate_hz
/// Returns the updated observation with msg_rate_hz populated (if calculable)
#[allow(dead_code)] // Will be used in ingestion service (later prompts)
pub async fn upsert_session_from_observation(
    pool: &SqlitePool,
    obs: &mut AircraftObservation,
) -> Result<()> {
    let mut tx = pool.begin().await?;

    // Check if session already exists
    let existing = sqlx::query(
        "SELECT hex, first_seen_ms, last_seen_ms, last_msg_total, message_count FROM aircraft_sessions WHERE hex = ?",
    )
    .bind(&obs.hex)
    .fetch_optional(&mut *tx)
    .await?;

    let now_ms = obs.ts_ms;
    let mut message_count_delta = 1; // Default increment

    if let Some(existing_row) = existing {
        // Update existing session
        let existing_message_count: i64 = existing_row.get("message_count");
        let last_seen_ms: i64 = existing_row.get("last_seen_ms");
        let last_msg_total: Option<i64> = existing_row.get("last_msg_total");

        // Calculate message rate if we have both previous and current msg_count_total
        if let (Some(prev_total), Some(curr_total)) = (last_msg_total, obs.msg_count_total) {
            let msg_delta = curr_total - prev_total;
            let time_delta_ms = now_ms - last_seen_ms;

            if msg_delta > 0 && time_delta_ms > 0 {
                // Calculate rate in Hz (messages per second)
                let msg_rate_hz = msg_delta as f64 / (time_delta_ms as f64 / 1000.0);
                obs.msg_rate_hz = Some(msg_rate_hz);
                message_count_delta = msg_delta; // Use actual delta instead of +1
            } else {
                // Handle counter reset or negative delta - ignore increment
                message_count_delta = 0;
                obs.msg_rate_hz = None;
            }
        } else {
            obs.msg_rate_hz = None;
        }

        let new_message_count = existing_message_count + message_count_delta;

        sqlx::query(
            r#"
            UPDATE aircraft_sessions SET
                last_seen_ms = ?,
                last_msg_total = ?,
                message_count = ?,
                has_position = has_position OR ?,
                has_altitude = has_altitude OR ?,
                has_callsign = has_callsign OR ?,
                flight = COALESCE(?, flight),
                lat = COALESCE(?, lat),
                lon = COALESCE(?, lon),
                altitude = COALESCE(?, altitude),
                speed = COALESCE(?, speed),
                tier_temporal = 1,
                tier_signal = tier_signal OR ?,
                tier_identity = 1,
                tier_behavioral = tier_behavioral OR ?
            WHERE hex = ?
            "#,
        )
        .bind(now_ms)
        .bind(obs.msg_count_total)
        .bind(new_message_count)
        .bind(if obs.lat.is_some() && obs.lon.is_some() {
            1i32
        } else {
            0i32
        })
        .bind(if obs.altitude.is_some() { 1i32 } else { 0i32 })
        .bind(if obs.flight.is_some() { 1i32 } else { 0i32 })
        .bind(&obs.flight)
        .bind(obs.lat)
        .bind(obs.lon)
        .bind(obs.altitude)
        .bind(obs.gs)
        .bind(if obs.rssi.is_some() { 1i32 } else { 0i32 })
        .bind(if obs.lat.is_some() && obs.lon.is_some() {
            1i32
        } else {
            0i32
        })
        .bind(&obs.hex)
        .execute(&mut *tx)
        .await?;
    } else {
        // Create new session - no rate calculation on first observation
        obs.msg_rate_hz = None;

        sqlx::query(
            r#"
            INSERT INTO aircraft_sessions (
                hex, first_seen_ms, last_seen_ms, last_msg_total, message_count,
                has_position, has_altitude, has_callsign,
                flight, lat, lon, altitude, speed,
                tier_temporal, tier_signal, tier_identity, tier_behavioral
            ) VALUES (?, ?, ?, ?, 1, ?, ?, ?, ?, ?, ?, ?, ?, 1, ?, 1, ?)
            "#,
        )
        .bind(&obs.hex)
        .bind(now_ms)
        .bind(now_ms)
        .bind(obs.msg_count_total)
        .bind(obs.lat.is_some() && obs.lon.is_some())
        .bind(obs.altitude.is_some())
        .bind(obs.flight.is_some())
        .bind(&obs.flight)
        .bind(obs.lat)
        .bind(obs.lon)
        .bind(obs.altitude)
        .bind(obs.gs)
        .bind(obs.rssi.is_some())
        .bind(obs.lat.is_some() && obs.lon.is_some())
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

/// Query aircraft sessions ordered by last_seen_ms descending with pagination
#[allow(dead_code)] // Will be used in API endpoints
pub async fn list_sessions(pool: &SqlitePool, limit: usize) -> Result<Vec<AircraftSession>> {
    let limit = std::cmp::min(limit, 1000); // Cap at 1000 for safety

    let rows = sqlx::query(
        r#"
        SELECT hex, first_seen_ms, last_seen_ms, last_msg_total, message_count,
               has_position, has_altitude, has_callsign,
               flight, lat, lon, altitude, speed,
               tier_temporal, tier_signal, tier_identity, tier_behavioral
        FROM aircraft_sessions
        ORDER BY last_seen_ms DESC
        LIMIT ?
        "#,
    )
    .bind(limit as i64)
    .fetch_all(pool)
    .await?;

    let sessions = rows
        .into_iter()
        .map(|row| {
            let hex: String = row.get("hex");
            let first_seen_ms: i64 = row.get("first_seen_ms");
            let last_seen_ms: i64 = row.get("last_seen_ms");
            let last_msg_total: Option<i64> = row.get("last_msg_total");
            let message_count: i64 = row.get("message_count");
            let has_position: i32 = row.get("has_position");
            let has_altitude: i32 = row.get("has_altitude");
            let has_callsign: i32 = row.get("has_callsign");
            let flight: Option<String> = row.get("flight");
            let lat: Option<f64> = row.get("lat");
            let lon: Option<f64> = row.get("lon");
            let altitude: Option<i32> = row.get("altitude");
            let speed: Option<f64> = row.get("speed");
            let tier_temporal: i32 = row.get("tier_temporal");
            let tier_signal: i32 = row.get("tier_signal");
            let tier_identity: i32 = row.get("tier_identity");
            let tier_behavioral: i32 = row.get("tier_behavioral");

            AircraftSession {
                hex,
                first_seen_ms,
                last_seen_ms,
                last_msg_total,
                message_count,
                has_position: has_position != 0,
                has_altitude: has_altitude != 0,
                has_callsign: has_callsign != 0,
                flight,
                lat,
                lon,
                altitude,
                speed,
                tier_temporal: tier_temporal != 0,
                tier_signal: tier_signal != 0,
                tier_identity: tier_identity != 0,
                tier_behavioral: tier_behavioral != 0,
            }
        })
        .collect();

    Ok(sessions)
}

/// Query only active aircraft sessions with complete data (position and altitude)
/// Active sessions are those seen within the last 30 minutes (configurable timeout)
/// Complete data means has_position and has_altitude flags are true
#[allow(dead_code)] // Will be used in API endpoints
pub async fn list_active_sessions_with_complete_data(
    pool: &SqlitePool,
    limit: usize,
    session_timeout_seconds: u64,
) -> Result<Vec<AircraftSession>> {
    let limit = std::cmp::min(limit, 1000); // Cap at 1000 for safety

    // Calculate cutoff time: current time minus timeout
    let now_ms = chrono::Utc::now().timestamp_millis();
    let cutoff_ms = now_ms - (session_timeout_seconds as i64 * 1000);

    let rows = sqlx::query(
        r#"
        SELECT hex, first_seen_ms, last_seen_ms, last_msg_total, message_count,
               has_position, has_altitude, has_callsign,
               flight, lat, lon, altitude, speed,
               tier_temporal, tier_signal, tier_identity, tier_behavioral
        FROM aircraft_sessions
        WHERE last_seen_ms > ?
          AND has_position = 1
          AND has_altitude = 1
        ORDER BY last_seen_ms DESC
        LIMIT ?
        "#,
    )
    .bind(cutoff_ms)
    .bind(limit as i64)
    .fetch_all(pool)
    .await?;

    let sessions = rows
        .into_iter()
        .map(|row| {
            let hex: String = row.get("hex");
            let first_seen_ms: i64 = row.get("first_seen_ms");
            let last_seen_ms: i64 = row.get("last_seen_ms");
            let last_msg_total: Option<i64> = row.get("last_msg_total");
            let message_count: i64 = row.get("message_count");
            let has_position: i32 = row.get("has_position");
            let has_altitude: i32 = row.get("has_altitude");
            let has_callsign: i32 = row.get("has_callsign");
            let flight: Option<String> = row.get("flight");
            let lat: Option<f64> = row.get("lat");
            let lon: Option<f64> = row.get("lon");
            let altitude: Option<i32> = row.get("altitude");
            let speed: Option<f64> = row.get("speed");
            let tier_temporal: i32 = row.get("tier_temporal");
            let tier_signal: i32 = row.get("tier_signal");
            let tier_identity: i32 = row.get("tier_identity");
            let tier_behavioral: i32 = row.get("tier_behavioral");

            AircraftSession {
                hex,
                first_seen_ms,
                last_seen_ms,
                last_msg_total,
                message_count,
                has_position: has_position != 0,
                has_altitude: has_altitude != 0,
                has_callsign: has_callsign != 0,
                flight,
                lat,
                lon,
                altitude,
                speed,
                tier_temporal: tier_temporal != 0,
                tier_signal: tier_signal != 0,
                tier_identity: tier_identity != 0,
                tier_behavioral: tier_behavioral != 0,
            }
        })
        .collect();

    Ok(sessions)
}

/// Batch upsert sessions from multiple observations for maximum performance
/// Processes all session updates in a single transaction with batched operations
/// Returns updated observations with msg_rate_hz populated where calculable
#[allow(dead_code)]
pub async fn batch_upsert_sessions_from_observations(
    pool: &SqlitePool,
    observations: &mut [AircraftObservation],
) -> Result<()> {
    if observations.is_empty() {
        return Ok(());
    }

    let mut tx = pool.begin().await?;

    // Step 1: Fetch all existing sessions in one query
    let hex_codes: Vec<&String> = observations.iter().map(|obs| &obs.hex).collect();
    let hex_placeholders = hex_codes.iter().map(|_| "?").collect::<Vec<_>>().join(",");

    let query = format!(
        "SELECT hex, first_seen_ms, last_seen_ms, last_msg_total, message_count FROM aircraft_sessions WHERE hex IN ({})",
        hex_placeholders
    );

    let mut query_builder = sqlx::query(&query);
    for hex in &hex_codes {
        query_builder = query_builder.bind(hex);
    }

    let existing_rows = query_builder.fetch_all(&mut *tx).await?;

    // Build lookup map of existing sessions
    let mut existing_sessions: HashMap<String, (i64, i64, Option<i64>, i64)> = HashMap::new();
    for row in existing_rows {
        let hex: String = row.get("hex");
        let first_seen: i64 = row.get("first_seen_ms");
        let last_seen: i64 = row.get("last_seen_ms");
        let last_msg_total: Option<i64> = row.get("last_msg_total");
        let message_count: i64 = row.get("message_count");
        existing_sessions.insert(hex, (first_seen, last_seen, last_msg_total, message_count));
    }

    // Step 2: Process observations and prepare batch operations
    let mut new_sessions = Vec::new();
    let mut session_updates = Vec::new();

    for obs in observations.iter_mut() {
        let now_ms = obs.ts_ms;

        if let Some((_first_seen, last_seen_ms, last_msg_total, existing_message_count)) =
            existing_sessions.get(&obs.hex)
        {
            // Existing session - calculate message rate and prepare update
            let mut message_count_delta = 1i64;

            if let (Some(prev_total), Some(curr_total)) = (*last_msg_total, obs.msg_count_total) {
                let msg_delta = curr_total - prev_total;
                let time_delta_ms = now_ms - last_seen_ms;

                if msg_delta > 0 && time_delta_ms > 0 {
                    let msg_rate_hz = msg_delta as f64 / (time_delta_ms as f64 / 1000.0);
                    obs.msg_rate_hz = Some(msg_rate_hz);
                    message_count_delta = msg_delta;
                } else {
                    message_count_delta = 0;
                    obs.msg_rate_hz = None;
                }
            } else {
                obs.msg_rate_hz = None;
            }

            let new_message_count = existing_message_count + message_count_delta;

            session_updates.push((
                obs.hex.clone(),
                now_ms,
                obs.msg_count_total,
                new_message_count,
                obs.lat.is_some() && obs.lon.is_some(),
                obs.altitude.is_some(),
                obs.flight.is_some(),
                obs.flight.clone(),
                obs.lat,
                obs.lon,
                obs.altitude,
                obs.gs,
                obs.rssi.is_some(),
            ));
        } else {
            // New session
            obs.msg_rate_hz = None;

            new_sessions.push((
                obs.hex.clone(),
                now_ms, // first_seen
                now_ms, // last_seen
                obs.msg_count_total,
                obs.lat.is_some() && obs.lon.is_some(),
                obs.altitude.is_some(),
                obs.flight.is_some(),
                obs.flight.clone(),
                obs.lat,
                obs.lon,
                obs.altitude,
                obs.gs,
                obs.rssi.is_some(),
            ));
        }
    }

    // Step 3: Batch insert new sessions
    if !new_sessions.is_empty() {
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
                        hex.replace("'", "''"),
                        first_seen,
                        last_seen,
                        last_msg_total.map_or("NULL".to_string(), |v| v.to_string()),
                        if *has_position { 1 } else { 0 },
                        if *has_altitude { 1 } else { 0 },
                        if *has_callsign { 1 } else { 0 },
                        match flight {
                            Some(f) => format!("'{}'", f.replace("'", "''")),
                            None => "NULL".to_string(),
                        },
                        lat.map_or("NULL".to_string(), |v| v.to_string()),
                        lon.map_or("NULL".to_string(), |v| v.to_string()),
                        altitude.map_or("NULL".to_string(), |v| v.to_string()),
                        speed.map_or("NULL".to_string(), |v| v.to_string()),
                        if *has_rssi { 1 } else { 0 },
                        if *has_position { 1 } else { 0 }
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
    }

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
    ) in session_updates
    {
        sqlx::query(
            r#"
            UPDATE aircraft_sessions SET
                last_seen_ms = ?,
                last_msg_total = ?,
                message_count = ?,
                has_position = has_position OR ?,
                has_altitude = has_altitude OR ?,
                has_callsign = has_callsign OR ?,
                flight = COALESCE(?, flight),
                lat = COALESCE(?, lat),
                lon = COALESCE(?, lon),
                altitude = COALESCE(?, altitude),
                speed = COALESCE(?, speed),
                tier_temporal = 1,
                tier_signal = tier_signal OR ?,
                tier_identity = 1,
                tier_behavioral = tier_behavioral OR ?
            WHERE hex = ?
            "#,
        )
        .bind(last_seen_ms)
        .bind(last_msg_total)
        .bind(message_count)
        .bind(if has_position { 1i32 } else { 0i32 })
        .bind(if has_altitude { 1i32 } else { 0i32 })
        .bind(if has_callsign { 1i32 } else { 0i32 })
        .bind(&flight)
        .bind(lat)
        .bind(lon)
        .bind(altitude)
        .bind(speed)
        .bind(if has_rssi { 1i32 } else { 0i32 })
        .bind(if has_position { 1i32 } else { 0i32 })
        .bind(&hex)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::connect_and_migrate;
    use tempfile::TempDir;

    async fn setup_test_db() -> (SqlitePool, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = connect_and_migrate(db_path.to_str().unwrap(), true)
            .await
            .unwrap();
        (pool, temp_dir)
    }

    #[tokio::test]
    async fn test_upsert_session_new() {
        let (pool, _temp_dir) = setup_test_db().await;

        let mut obs = AircraftObservation {
            id: None,
            ts_ms: 1641024000000,
            hex: "ABC123".to_string(),
            flight: Some("UAL456".to_string()),
            lat: Some(40.7128),
            lon: Some(-74.0060),
            altitude: Some(35000),
            gs: Some(450.2),
            rssi: Some(-45.5),
            msg_count_total: Some(1000),
            raw_json: "{}".to_string(),
            msg_rate_hz: None,
        };

        upsert_session_from_observation(&pool, &mut obs)
            .await
            .unwrap();

        // First observation should have no rate calculation
        assert_eq!(obs.msg_rate_hz, None);

        // Verify session was created
        let session = sqlx::query(
            r#"
            SELECT hex, first_seen_ms, last_seen_ms, last_msg_total, message_count, has_position,
                   has_altitude, has_callsign, flight, lat, lon, altitude, speed,
                   tier_temporal, tier_signal, tier_identity, tier_behavioral
            FROM aircraft_sessions WHERE hex = ?
            "#,
        )
        .bind("ABC123")
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(session.get::<String, _>("hex"), "ABC123");
        assert_eq!(session.get::<i64, _>("first_seen_ms"), 1641024000000);
        assert_eq!(session.get::<i64, _>("last_seen_ms"), 1641024000000);
        assert_eq!(session.get::<Option<i64>, _>("last_msg_total"), Some(1000));
        assert_eq!(session.get::<i64, _>("message_count"), 1);
        assert_eq!(session.get::<i32, _>("has_position"), 1); // true
        assert_eq!(session.get::<i32, _>("has_altitude"), 1); // true
        assert_eq!(session.get::<i32, _>("has_callsign"), 1); // true
        assert_eq!(
            session.get::<Option<String>, _>("flight"),
            Some("UAL456".to_string())
        );
        assert_eq!(session.get::<Option<f64>, _>("lat"), Some(40.7128));
        assert_eq!(session.get::<Option<i32>, _>("altitude"), Some(35000));
        assert_eq!(session.get::<i32, _>("tier_temporal"), 1);
        assert_eq!(session.get::<i32, _>("tier_signal"), 1); // Has RSSI
        assert_eq!(session.get::<i32, _>("tier_identity"), 1);
        assert_eq!(session.get::<i32, _>("tier_behavioral"), 1); // Has position
    }

    #[tokio::test]
    async fn test_message_rate_calculation() {
        let (pool, _temp_dir) = setup_test_db().await;

        // First observation - establishes baseline
        let mut obs1 = AircraftObservation {
            id: None,
            ts_ms: 1641024000000, // Time: 0s
            hex: "MSG123".to_string(),
            flight: Some("TEST123".to_string()),
            lat: Some(40.7128),
            lon: Some(-74.0060),
            altitude: Some(35000),
            gs: Some(450.0),
            rssi: Some(-45.0),
            msg_count_total: Some(100), // Starting at 100 messages
            raw_json: "{}".to_string(),
            msg_rate_hz: None,
        };

        upsert_session_from_observation(&pool, &mut obs1)
            .await
            .unwrap();
        assert_eq!(obs1.msg_rate_hz, None); // No rate on first observation

        // Second observation - 1 second later with 10 more messages
        let mut obs2 = AircraftObservation {
            id: None,
            ts_ms: 1641024001000, // Time: +1000ms (1 second later)
            hex: "MSG123".to_string(),
            flight: Some("TEST123".to_string()),
            lat: Some(40.7129),
            lon: Some(-74.0061),
            altitude: Some(35100),
            gs: Some(451.0),
            rssi: Some(-44.8),
            msg_count_total: Some(110), // 10 messages in 1 second
            raw_json: "{}".to_string(),
            msg_rate_hz: None,
        };

        upsert_session_from_observation(&pool, &mut obs2)
            .await
            .unwrap();

        // Should calculate rate: 10 messages / 1 second = 10.0 Hz
        assert!(obs2.msg_rate_hz.is_some());
        let rate = obs2.msg_rate_hz.unwrap();
        assert!((rate - 10.0).abs() < 0.1, "Expected ~10.0 Hz, got {}", rate);

        // Verify session was updated with correct message count delta
        let session = sqlx::query(
            "SELECT message_count, last_msg_total FROM aircraft_sessions WHERE hex = ?",
        )
        .bind("MSG123")
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(session.get::<i64, _>("message_count"), 11); // 1 + 10 delta
        assert_eq!(session.get::<Option<i64>, _>("last_msg_total"), Some(110));
    }

    #[tokio::test]
    async fn test_message_counter_reset_handling() {
        let (pool, _temp_dir) = setup_test_db().await;

        // First observation
        let mut obs1 = AircraftObservation {
            id: None,
            ts_ms: 1641024000000,
            hex: "RST123".to_string(),
            flight: Some("RESET123".to_string()),
            lat: Some(40.0),
            lon: Some(-74.0),
            altitude: Some(30000),
            gs: Some(400.0),
            rssi: Some(-50.0),
            msg_count_total: Some(1000),
            raw_json: "{}".to_string(),
            msg_rate_hz: None,
        };

        upsert_session_from_observation(&pool, &mut obs1)
            .await
            .unwrap();

        // Second observation with counter reset (lower value)
        let mut obs2 = AircraftObservation {
            id: None,
            ts_ms: 1641024001000, // 1 second later
            hex: "RST123".to_string(),
            flight: Some("RESET123".to_string()),
            lat: Some(40.001),
            lon: Some(-74.001),
            altitude: Some(30100),
            gs: Some(401.0),
            rssi: Some(-49.8),
            msg_count_total: Some(50), // Counter reset/wrapped
            raw_json: "{}".to_string(),
            msg_rate_hz: None,
        };

        upsert_session_from_observation(&pool, &mut obs2)
            .await
            .unwrap();

        // Should not calculate rate for negative delta (counter reset)
        assert_eq!(obs2.msg_rate_hz, None);

        // Message count should not increment when delta <= 0
        let session = sqlx::query("SELECT message_count FROM aircraft_sessions WHERE hex = ?")
            .bind("RST123")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(session.get::<i64, _>("message_count"), 1); // Still 1 (no increment)
    }

    #[tokio::test]
    async fn test_upsert_session_update_existing() {
        let (pool, _temp_dir) = setup_test_db().await;

        // First observation
        let mut obs1 = AircraftObservation {
            id: None,
            ts_ms: 1641024000000,
            hex: "ABC123".to_string(),
            flight: Some("UAL456".to_string()),
            lat: Some(40.7128),
            lon: Some(-74.0060),
            altitude: Some(35000),
            gs: Some(450.2),
            rssi: Some(-45.5),
            msg_count_total: Some(1000),
            raw_json: "{}".to_string(),
            msg_rate_hz: None,
        };

        upsert_session_from_observation(&pool, &mut obs1)
            .await
            .unwrap();

        // Second observation with updated data
        let mut obs2 = AircraftObservation {
            id: None,
            ts_ms: 1641024001000, // 1 second later
            hex: "ABC123".to_string(),
            flight: Some("UAL456".to_string()),
            lat: Some(40.7129), // Moved slightly
            lon: Some(-74.0061),
            altitude: Some(35100), // Climbed
            gs: Some(451.0),
            rssi: Some(-45.0),
            msg_count_total: Some(1010),
            raw_json: "{}".to_string(),
            msg_rate_hz: None,
        };

        upsert_session_from_observation(&pool, &mut obs2)
            .await
            .unwrap();

        // Verify session was updated, not duplicated
        let sessions = sqlx::query(
            "SELECT hex, first_seen_ms, last_seen_ms, message_count, lat, altitude FROM aircraft_sessions WHERE hex = ?",
        )
        .bind("ABC123")
        .fetch_all(&pool)
        .await
        .unwrap();

        assert_eq!(sessions.len(), 1); // Should only have one session
        let session = &sessions[0];
        assert_eq!(session.get::<i64, _>("first_seen_ms"), 1641024000000); // Unchanged
        assert_eq!(session.get::<i64, _>("last_seen_ms"), 1641024001000); // Updated
        assert_eq!(session.get::<i64, _>("message_count"), 11); // 1 + 10 delta from msg counter
        assert_eq!(session.get::<Option<f64>, _>("lat"), Some(40.7129)); // Updated position
        assert_eq!(session.get::<Option<i32>, _>("altitude"), Some(35100)); // Updated altitude
    }

    #[tokio::test]
    async fn test_upsert_session_minimal_data() {
        let (pool, _temp_dir) = setup_test_db().await;

        let mut obs = AircraftObservation {
            id: None,
            ts_ms: 1641024000000,
            hex: "MIN123".to_string(),
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

        upsert_session_from_observation(&pool, &mut obs)
            .await
            .unwrap();

        let session = sqlx::query(
            r#"
            SELECT hex, has_position, has_altitude, has_callsign, flight,
                   tier_signal, tier_behavioral
            FROM aircraft_sessions WHERE hex = ?
            "#,
        )
        .bind("MIN123")
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(session.get::<String, _>("hex"), "MIN123");
        assert_eq!(session.get::<i32, _>("has_position"), 0); // false - no lat/lon
        assert_eq!(session.get::<i32, _>("has_altitude"), 0); // false
        assert_eq!(session.get::<i32, _>("has_callsign"), 0); // false
        assert_eq!(session.get::<Option<String>, _>("flight"), None);
        assert_eq!(session.get::<i32, _>("tier_signal"), 0); // No RSSI
        assert_eq!(session.get::<i32, _>("tier_behavioral"), 0); // No position
    }

    #[tokio::test]
    async fn test_upsert_session_preserves_existing_data() {
        let (pool, _temp_dir) = setup_test_db().await;

        // First observation with full data
        let mut obs1 = AircraftObservation {
            id: None,
            ts_ms: 1641024000000,
            hex: "ABC123".to_string(),
            flight: Some("UAL456".to_string()),
            lat: Some(40.7128),
            lon: Some(-74.0060),
            altitude: Some(35000),
            gs: Some(450.2),
            rssi: Some(-45.5),
            msg_count_total: Some(1000),
            raw_json: "{}".to_string(),
            msg_rate_hz: None,
        };

        upsert_session_from_observation(&pool, &mut obs1)
            .await
            .unwrap();

        // Second observation with sparse data (should preserve existing flight)
        let mut obs2 = AircraftObservation {
            id: None,
            ts_ms: 1641024001000,
            hex: "ABC123".to_string(),
            flight: None, // Missing - should preserve existing
            lat: Some(40.7129),
            lon: Some(-74.0061),
            altitude: None, // Missing - should preserve existing
            gs: Some(451.0),
            rssi: None, // Missing
            msg_count_total: Some(1010),
            raw_json: "{}".to_string(),
            msg_rate_hz: None,
        };

        upsert_session_from_observation(&pool, &mut obs2)
            .await
            .unwrap();

        let session = sqlx::query("SELECT flight, altitude FROM aircraft_sessions WHERE hex = ?")
            .bind("ABC123")
            .fetch_one(&pool)
            .await
            .unwrap();

        // Should preserve flight and altitude from first observation
        assert_eq!(
            session.get::<Option<String>, _>("flight"),
            Some("UAL456".to_string())
        );
        assert_eq!(session.get::<Option<i32>, _>("altitude"), Some(35000));
    }

    #[tokio::test]
    async fn test_capability_flags_cumulative_behavior() {
        let (pool, _temp_dir) = setup_test_db().await;

        // First observation with full capabilities
        let mut obs1 = AircraftObservation {
            id: None,
            ts_ms: 1641024000000,
            hex: "CUM123".to_string(),
            flight: Some("UAL456".to_string()),
            lat: Some(40.7128),
            lon: Some(-74.0060),
            altitude: Some(35000),
            gs: Some(450.2),
            rssi: Some(-45.5),
            msg_count_total: Some(1000),
            raw_json: "{}".to_string(),
            msg_rate_hz: None,
        };

        upsert_session_from_observation(&pool, &mut obs1)
            .await
            .unwrap();

        // Verify all capabilities are set
        let session = sqlx::query(
            r#"
            SELECT has_position, has_altitude, has_callsign, tier_signal, tier_behavioral
            FROM aircraft_sessions WHERE hex = ?
            "#,
        )
        .bind("CUM123")
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(session.get::<i32, _>("has_position"), 1);
        assert_eq!(session.get::<i32, _>("has_altitude"), 1);
        assert_eq!(session.get::<i32, _>("has_callsign"), 1);
        assert_eq!(session.get::<i32, _>("tier_signal"), 1);
        assert_eq!(session.get::<i32, _>("tier_behavioral"), 1);

        // Second observation with NO capabilities (minimal data)
        let mut obs2 = AircraftObservation {
            id: None,
            ts_ms: 1641024001000,
            hex: "CUM123".to_string(),
            flight: None, // No callsign
            lat: None,    // No position
            lon: None,
            altitude: None, // No altitude
            gs: None,
            rssi: None, // No RSSI
            msg_count_total: Some(1010),
            raw_json: "{}".to_string(),
            msg_rate_hz: None,
        };

        upsert_session_from_observation(&pool, &mut obs2)
            .await
            .unwrap();

        // Verify capabilities are PRESERVED (cumulative), not overwritten
        let session = sqlx::query(
            r#"
            SELECT has_position, has_altitude, has_callsign, tier_signal, tier_behavioral,
                   flight, lat, lon, altitude, speed
            FROM aircraft_sessions WHERE hex = ?
            "#,
        )
        .bind("CUM123")
        .fetch_one(&pool)
        .await
        .unwrap();

        // Capability flags should still be true (cumulative)
        assert_eq!(
            session.get::<i32, _>("has_position"),
            1,
            "has_position should remain true"
        );
        assert_eq!(
            session.get::<i32, _>("has_altitude"),
            1,
            "has_altitude should remain true"
        );
        assert_eq!(
            session.get::<i32, _>("has_callsign"),
            1,
            "has_callsign should remain true"
        );
        assert_eq!(
            session.get::<i32, _>("tier_signal"),
            1,
            "tier_signal should remain true"
        );
        assert_eq!(
            session.get::<i32, _>("tier_behavioral"),
            1,
            "tier_behavioral should remain true"
        );

        // Data should be preserved (COALESCE behavior)
        assert_eq!(
            session.get::<Option<String>, _>("flight"),
            Some("UAL456".to_string())
        );
        assert_eq!(session.get::<Option<f64>, _>("lat"), Some(40.7128));
        assert_eq!(session.get::<Option<f64>, _>("lon"), Some(-74.0060));
        assert_eq!(session.get::<Option<i32>, _>("altitude"), Some(35000));
    }

    #[tokio::test]
    async fn test_upsert_session_capability_flags() {
        let (pool, _temp_dir) = setup_test_db().await;

        // Test various combinations of data availability
        let test_cases = vec![
            // (lat, lon, alt, flight, rssi) -> (has_pos, has_alt, has_call, tier_sig, tier_behav)
            (
                Some(40.0),
                Some(-74.0),
                Some(35000),
                Some("TEST"),
                Some(-45.0),
                (true, true, true, true, true),
            ),
            (
                Some(40.0),
                Some(-74.0),
                None,
                None,
                None,
                (true, false, false, false, true),
            ),
            (
                None,
                None,
                Some(35000),
                Some("TEST"),
                Some(-45.0),
                (false, true, true, true, false),
            ),
            (
                Some(40.0),
                None,
                None,
                None,
                None,
                (false, false, false, false, false),
            ), // Incomplete position
            (
                None,
                None,
                None,
                None,
                None,
                (false, false, false, false, false),
            ),
        ];

        for (i, (lat, lon, alt, flight, rssi, expected)) in test_cases.into_iter().enumerate() {
            let hex = format!("TEST{:02}", i);
            let mut obs = AircraftObservation {
                id: None,
                ts_ms: 1641024000000 + i as i64 * 1000,
                hex: hex.clone(),
                flight: flight.map(|s| s.to_string()),
                lat,
                lon,
                altitude: alt,
                gs: Some(450.0),
                rssi,
                msg_count_total: Some(1000),
                raw_json: "{}".to_string(),
                msg_rate_hz: None,
            };

            upsert_session_from_observation(&pool, &mut obs)
                .await
                .unwrap();

            let session = sqlx::query(
                r#"
                SELECT has_position, has_altitude, has_callsign, tier_signal, tier_behavioral
                FROM aircraft_sessions WHERE hex = ?
                "#,
            )
            .bind(&hex)
            .fetch_one(&pool)
            .await
            .unwrap();

            let (exp_pos, exp_alt, exp_call, exp_sig, exp_behav) = expected;
            assert_eq!(
                session.get::<i32, _>("has_position") != 0,
                exp_pos,
                "Test case {}: has_position",
                i
            );
            assert_eq!(
                session.get::<i32, _>("has_altitude") != 0,
                exp_alt,
                "Test case {}: has_altitude",
                i
            );
            assert_eq!(
                session.get::<i32, _>("has_callsign") != 0,
                exp_call,
                "Test case {}: has_callsign",
                i
            );
            assert_eq!(
                session.get::<i32, _>("tier_signal") != 0,
                exp_sig,
                "Test case {}: tier_signal",
                i
            );
            assert_eq!(
                session.get::<i32, _>("tier_behavioral") != 0,
                exp_behav,
                "Test case {}: tier_behavioral",
                i
            );
        }
    }

    #[tokio::test]
    async fn test_capability_flags_build_incrementally() {
        let (pool, _temp_dir) = setup_test_db().await;

        // First observation with only position
        let mut obs1 = AircraftObservation {
            id: None,
            ts_ms: 1641024000000,
            hex: "INC123".to_string(),
            flight: None,
            lat: Some(40.7128),
            lon: Some(-74.0060),
            altitude: None,
            gs: Some(450.0),
            rssi: None,
            msg_count_total: Some(1000),
            raw_json: "{}".to_string(),
            msg_rate_hz: None,
        };

        upsert_session_from_observation(&pool, &mut obs1)
            .await
            .unwrap();

        // Should only have position capability
        let session = sqlx::query(
            "SELECT has_position, has_altitude, has_callsign, tier_signal, tier_behavioral FROM aircraft_sessions WHERE hex = ?"
        )
        .bind("INC123")
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(session.get::<i32, _>("has_position"), 1);
        assert_eq!(session.get::<i32, _>("has_altitude"), 0);
        assert_eq!(session.get::<i32, _>("has_callsign"), 0);
        assert_eq!(session.get::<i32, _>("tier_signal"), 0);
        assert_eq!(session.get::<i32, _>("tier_behavioral"), 1); // Has position

        // Second observation adds altitude
        let mut obs2 = AircraftObservation {
            id: None,
            ts_ms: 1641024001000,
            hex: "INC123".to_string(),
            flight: None,
            lat: None, // No position this time
            lon: None,
            altitude: Some(35000), // Add altitude
            gs: Some(451.0),
            rssi: None,
            msg_count_total: Some(1010),
            raw_json: "{}".to_string(),
            msg_rate_hz: None,
        };

        upsert_session_from_observation(&pool, &mut obs2)
            .await
            .unwrap();

        // Should now have both position AND altitude capabilities
        let session = sqlx::query(
            "SELECT has_position, has_altitude, has_callsign, tier_signal, tier_behavioral FROM aircraft_sessions WHERE hex = ?"
        )
        .bind("INC123")
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(
            session.get::<i32, _>("has_position"),
            1,
            "should keep position capability"
        );
        assert_eq!(
            session.get::<i32, _>("has_altitude"),
            1,
            "should add altitude capability"
        );
        assert_eq!(session.get::<i32, _>("has_callsign"), 0);
        assert_eq!(session.get::<i32, _>("tier_signal"), 0);
        assert_eq!(session.get::<i32, _>("tier_behavioral"), 1);

        // Third observation adds callsign and signal
        let mut obs3 = AircraftObservation {
            id: None,
            ts_ms: 1641024002000,
            hex: "INC123".to_string(),
            flight: Some("UAL789".to_string()), // Add callsign
            lat: None,
            lon: None,
            altitude: None,
            gs: Some(452.0),
            rssi: Some(-42.0), // Add RSSI
            msg_count_total: Some(1020),
            raw_json: "{}".to_string(),
            msg_rate_hz: None,
        };

        upsert_session_from_observation(&pool, &mut obs3)
            .await
            .unwrap();

        // Should now have all capabilities accumulated
        let session = sqlx::query(
            "SELECT has_position, has_altitude, has_callsign, tier_signal, tier_behavioral FROM aircraft_sessions WHERE hex = ?"
        )
        .bind("INC123")
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(
            session.get::<i32, _>("has_position"),
            1,
            "should keep position capability"
        );
        assert_eq!(
            session.get::<i32, _>("has_altitude"),
            1,
            "should keep altitude capability"
        );
        assert_eq!(
            session.get::<i32, _>("has_callsign"),
            1,
            "should add callsign capability"
        );
        assert_eq!(
            session.get::<i32, _>("tier_signal"),
            1,
            "should add signal tier"
        );
        assert_eq!(
            session.get::<i32, _>("tier_behavioral"),
            1,
            "should keep behavioral tier"
        );
    }

    #[tokio::test]
    async fn test_list_active_sessions_with_complete_data() {
        let (pool, _temp_dir) = setup_test_db().await;

        let now_ms = chrono::Utc::now().timestamp_millis();

        // Create test sessions with different timestamps and capabilities
        let test_cases = vec![
            // Active session with complete data - should be included
            ("ACTIVE1", now_ms - 60_000, true, true, true), // 1 minute ago, complete data
            // Active session without position - should be excluded
            ("ACTIVE2", now_ms - 120_000, false, true, false), // 2 minutes ago, no position
            // Active session without altitude - should be excluded
            ("ACTIVE3", now_ms - 180_000, true, false, false), // 3 minutes ago, no altitude
            // Old session with complete data - should be excluded (too old)
            ("OLD1", now_ms - 2000_000, true, true, true), // 33 minutes ago, complete data but too old
            // Recent session with complete data - should be included
            ("RECENT1", now_ms - 30_000, true, true, true), // 30 seconds ago, complete data
        ];

        for (hex, last_seen_ms, has_position, has_altitude, has_callsign) in test_cases {
            // Insert session directly to control timestamps and capabilities
            sqlx::query(
                r#"
                INSERT INTO aircraft_sessions (
                    hex, first_seen_ms, last_seen_ms, last_msg_total, message_count,
                    has_position, has_altitude, has_callsign,
                    flight, lat, lon, altitude, speed,
                    tier_temporal, tier_signal, tier_identity, tier_behavioral
                ) VALUES (?, ?, ?, ?, 1, ?, ?, ?, ?, ?, ?, ?, ?, 1, 0, 1, ?)
                "#,
            )
            .bind(hex)
            .bind(last_seen_ms)
            .bind(last_seen_ms)
            .bind(Some(1000i64))
            .bind(has_position)
            .bind(has_altitude)
            .bind(has_callsign)
            .bind(if has_callsign {
                Some(format!("TEST{}", hex))
            } else {
                None
            })
            .bind(if has_position { Some(40.0) } else { None })
            .bind(if has_position { Some(-74.0) } else { None })
            .bind(if has_altitude { Some(35000) } else { None })
            .bind(Some(450.0))
            .bind(has_position)
            .execute(&pool)
            .await
            .unwrap();
        }

        // Query active sessions with complete data (30 minute timeout)
        let active_sessions = list_active_sessions_with_complete_data(&pool, 100, 1800)
            .await
            .unwrap();

        // Should only return ACTIVE1 and RECENT1 (both have complete data and are within timeout)
        assert_eq!(active_sessions.len(), 2);

        let hex_codes: Vec<&str> = active_sessions.iter().map(|s| s.hex.as_str()).collect();
        assert!(hex_codes.contains(&"ACTIVE1"));
        assert!(hex_codes.contains(&"RECENT1"));

        // Verify all returned sessions have complete data
        for session in &active_sessions {
            assert!(
                session.has_position,
                "Session {} should have position",
                session.hex
            );
            assert!(
                session.has_altitude,
                "Session {} should have altitude",
                session.hex
            );
        }

        // Verify sessions are ordered by last_seen_ms descending
        if active_sessions.len() > 1 {
            for i in 0..active_sessions.len() - 1 {
                assert!(
                    active_sessions[i].last_seen_ms >= active_sessions[i + 1].last_seen_ms,
                    "Sessions should be ordered by last_seen_ms descending"
                );
            }
        }
    }

    #[tokio::test]
    async fn test_active_sessions_timeout_boundary() {
        let (pool, _temp_dir) = setup_test_db().await;

        let now_ms = chrono::Utc::now().timestamp_millis();
        let timeout_seconds = 300; // 5 minutes
        let cutoff_ms = now_ms - (timeout_seconds * 1000);

        // Session exactly at the boundary (should be excluded)
        sqlx::query(
            r#"
            INSERT INTO aircraft_sessions (
                hex, first_seen_ms, last_seen_ms, last_msg_total, message_count,
                has_position, has_altitude, has_callsign,
                flight, lat, lon, altitude, speed,
                tier_temporal, tier_signal, tier_identity, tier_behavioral
            ) VALUES (?, ?, ?, ?, 1, 1, 1, 1, ?, ?, ?, ?, ?, 1, 0, 1, 1)
            "#,
        )
        .bind("BOUNDARY")
        .bind(cutoff_ms)
        .bind(cutoff_ms) // Exactly at cutoff
        .bind(Some(1000i64))
        .bind(Some("TEST123"))
        .bind(Some(40.0))
        .bind(Some(-74.0))
        .bind(Some(35000))
        .bind(Some(450.0))
        .execute(&pool)
        .await
        .unwrap();

        // Session just after the boundary (should be included)
        sqlx::query(
            r#"
            INSERT INTO aircraft_sessions (
                hex, first_seen_ms, last_seen_ms, last_msg_total, message_count,
                has_position, has_altitude, has_callsign,
                flight, lat, lon, altitude, speed,
                tier_temporal, tier_signal, tier_identity, tier_behavioral
            ) VALUES (?, ?, ?, ?, 1, 1, 1, 1, ?, ?, ?, ?, ?, 1, 0, 1, 1)
            "#,
        )
        .bind("ACTIVE")
        .bind(cutoff_ms + 1000)
        .bind(cutoff_ms + 1000) // 1 second after cutoff
        .bind(Some(1000i64))
        .bind(Some("TEST456"))
        .bind(Some(41.0))
        .bind(Some(-75.0))
        .bind(Some(36000))
        .bind(Some(460.0))
        .execute(&pool)
        .await
        .unwrap();

        let active_sessions =
            list_active_sessions_with_complete_data(&pool, 100, timeout_seconds as u64)
                .await
                .unwrap();

        // Should only include the session after the boundary
        assert_eq!(active_sessions.len(), 1);
        assert_eq!(active_sessions[0].hex, "ACTIVE");
    }

    #[tokio::test]
    async fn test_active_sessions_limit() {
        let (pool, _temp_dir) = setup_test_db().await;

        let now_ms = chrono::Utc::now().timestamp_millis();

        // Create 10 active sessions with complete data
        for i in 0..10 {
            let hex = format!("TEST{:02}", i);
            let last_seen = now_ms - (i * 10_000); // Spread out by 10 seconds each

            sqlx::query(
                r#"
                INSERT INTO aircraft_sessions (
                    hex, first_seen_ms, last_seen_ms, last_msg_total, message_count,
                    has_position, has_altitude, has_callsign,
                    flight, lat, lon, altitude, speed,
                    tier_temporal, tier_signal, tier_identity, tier_behavioral
                ) VALUES (?, ?, ?, ?, 1, 1, 1, 1, ?, ?, ?, ?, ?, 1, 0, 1, 1)
                "#,
            )
            .bind(&hex)
            .bind(last_seen)
            .bind(last_seen)
            .bind(Some(1000i64))
            .bind(Some(format!("FLIGHT{}", i)))
            .bind(Some(40.0 + i as f64 * 0.01))
            .bind(Some(-74.0 + i as f64 * 0.01))
            .bind(Some(35000 + i * 100))
            .bind(Some(450.0))
            .execute(&pool)
            .await
            .unwrap();
        }

        // Query with limit of 5
        let active_sessions = list_active_sessions_with_complete_data(&pool, 5, 1800)
            .await
            .unwrap();

        // Should return exactly 5 sessions
        assert_eq!(active_sessions.len(), 5);

        // Should return the 5 most recent sessions (TEST00, TEST01, TEST02, TEST03, TEST04)
        let expected_hex_codes = vec!["TEST00", "TEST01", "TEST02", "TEST03", "TEST04"];
        let actual_hex_codes: Vec<&str> = active_sessions.iter().map(|s| s.hex.as_str()).collect();

        for expected in &expected_hex_codes {
            assert!(
                actual_hex_codes.contains(expected),
                "Should contain {}",
                expected
            );
        }
    }

    #[tokio::test]
    async fn test_batch_upsert_sessions_performance() {
        let (pool, _temp_dir) = setup_test_db().await;

        // Create a batch of observations to test performance
        let mut observations: Vec<AircraftObservation> = (0..100)
            .map(|i| AircraftObservation {
                id: None,
                ts_ms: 1641024000000 + i * 1000,
                hex: format!("BATCH{:03}", i),
                flight: Some(format!("TEST{:03}", i)),
                lat: Some(40.0 + i as f64 * 0.001),
                lon: Some(-74.0 + i as f64 * 0.001),
                altitude: Some(35000 + (i as i32) * 10),
                gs: Some(450.0 + i as f64),
                rssi: Some(-45.0 - i as f64 * 0.1),
                msg_count_total: Some(1000 + i),
                raw_json: format!("{{\"hex\":\"BATCH{:03}\"}}", i),
                msg_rate_hz: None,
            })
            .collect();

        let start = std::time::Instant::now();
        batch_upsert_sessions_from_observations(&pool, &mut observations)
            .await
            .unwrap();
        let batch_duration = start.elapsed();

        // Verify all sessions were created
        let sessions = list_sessions(&pool, 200).await.unwrap();
        assert_eq!(sessions.len(), 100);

        // Test with second batch (updates)
        let mut observations2: Vec<AircraftObservation> = (0..100)
            .map(|i| AircraftObservation {
                id: None,
                ts_ms: 1641024001000 + i * 1000, // 1 second later
                hex: format!("BATCH{:03}", i),
                flight: Some(format!("TEST{:03}", i)),
                lat: Some(40.0 + i as f64 * 0.001),
                lon: Some(-74.0 + i as f64 * 0.001),
                altitude: Some(35000 + (i as i32) * 10),
                gs: Some(450.0 + i as f64),
                rssi: Some(-45.0 - i as f64 * 0.1),
                msg_count_total: Some(1010 + i), // 10 more messages
                raw_json: format!("{{\"hex\":\"BATCH{:03}\"}}", i),
                msg_rate_hz: None,
            })
            .collect();

        let start2 = std::time::Instant::now();
        batch_upsert_sessions_from_observations(&pool, &mut observations2)
            .await
            .unwrap();
        let batch_update_duration = start2.elapsed();

        println!("Batch insert time: {:?}", batch_duration);
        println!("Batch update time: {:?}", batch_update_duration);

        // Performance should be sub-second for 100 aircraft
        assert!(
            batch_duration.as_millis() < 1000,
            "Batch insert too slow: {:?}",
            batch_duration
        );
        assert!(
            batch_update_duration.as_millis() < 1000,
            "Batch update too slow: {:?}",
            batch_update_duration
        );

        // Verify message rates were calculated for updates
        let observations_with_rates: Vec<_> = observations2
            .iter()
            .filter(|obs| obs.msg_rate_hz.is_some())
            .collect();
        assert_eq!(
            observations_with_rates.len(),
            100,
            "All observations should have message rates calculated"
        );

        // Verify sessions still exist (no duplicates)
        let final_sessions = list_sessions(&pool, 200).await.unwrap();
        assert_eq!(
            final_sessions.len(),
            100,
            "Should still have exactly 100 sessions"
        );
    }

    #[tokio::test]
    async fn test_batch_vs_individual_performance_comparison() {
        let (pool, _temp_dir) = setup_test_db().await;

        // Test individual operations (old method)
        let mut individual_observations: Vec<AircraftObservation> = (0..50)
            .map(|i| AircraftObservation {
                id: None,
                ts_ms: 1641024000000 + i * 1000,
                hex: format!("IND{:02}", i),
                flight: Some(format!("INDIV{:02}", i)),
                lat: Some(40.0 + i as f64 * 0.001),
                lon: Some(-74.0 + i as f64 * 0.001),
                altitude: Some(35000),
                gs: Some(450.0),
                rssi: Some(-45.0),
                msg_count_total: Some(1000),
                raw_json: "{}".to_string(),
                msg_rate_hz: None,
            })
            .collect();

        let start_individual = std::time::Instant::now();
        for mut obs in &mut individual_observations {
            upsert_session_from_observation(&pool, &mut obs)
                .await
                .unwrap();
        }
        let individual_duration = start_individual.elapsed();

        // Test batch operations (new method)
        let mut batch_observations: Vec<AircraftObservation> = (50..100)
            .map(|i| AircraftObservation {
                id: None,
                ts_ms: 1641024000000 + i * 1000,
                hex: format!("BATCH{:02}", i),
                flight: Some(format!("BATCH{:02}", i)),
                lat: Some(40.0 + i as f64 * 0.001),
                lon: Some(-74.0 + i as f64 * 0.001),
                altitude: Some(35000),
                gs: Some(450.0),
                rssi: Some(-45.0),
                msg_count_total: Some(1000),
                raw_json: "{}".to_string(),
                msg_rate_hz: None,
            })
            .collect();

        let start_batch = std::time::Instant::now();
        batch_upsert_sessions_from_observations(&pool, &mut batch_observations)
            .await
            .unwrap();
        let batch_duration = start_batch.elapsed();

        println!(
            "Individual operations (50 aircraft): {:?}",
            individual_duration
        );
        println!("Batch operations (50 aircraft): {:?}", batch_duration);

        // Batch should be significantly faster
        let performance_ratio =
            individual_duration.as_millis() as f64 / batch_duration.as_millis() as f64;
        println!("Performance improvement: {:.1}x", performance_ratio);

        // Batch should be at least 3x faster (conservative expectation)
        assert!(
            performance_ratio > 3.0,
            "Batch operations should be significantly faster. Got {:.1}x improvement",
            performance_ratio
        );

        // Verify both methods created sessions correctly
        let all_sessions = list_sessions(&pool, 200).await.unwrap();
        assert_eq!(all_sessions.len(), 100, "Should have 100 total sessions");
    }
}
