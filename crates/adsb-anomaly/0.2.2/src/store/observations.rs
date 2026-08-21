// ABOUTME: Database operations for aircraft observations - batch inserts and queries
// ABOUTME: Handles time-series data efficiently with proper indexing and pagination

use crate::error::Result;
use crate::model::AircraftObservation;
use sqlx::{Row, SqlitePool};

/// Insert multiple observations using high-performance batch operation
/// Uses single SQL statement with VALUES clause for 10-100x performance improvement
#[allow(dead_code)] // Will be used in ingestion service (later prompts)
pub async fn insert_observations(
    pool: &SqlitePool,
    observations: &[AircraftObservation],
) -> Result<usize> {
    if observations.is_empty() {
        return Ok(0);
    }

    // Use batch INSERT with VALUES clause for maximum performance
    // This reduces O(n) individual operations to O(1) batch operation
    let values: Vec<String> = observations
        .iter()
        .map(|obs| {
            format!(
                "({}, '{}', {}, {}, {}, {}, {}, {}, {}, '{}')",
                obs.ts_ms,
                obs.hex.replace("'", "''"), // Escape single quotes
                match &obs.flight {
                    Some(f) => format!("'{}'", f.replace("'", "''")),
                    None => "NULL".to_string(),
                },
                obs.lat.map_or("NULL".to_string(), |v| v.to_string()),
                obs.lon.map_or("NULL".to_string(), |v| v.to_string()),
                obs.altitude.map_or("NULL".to_string(), |v| v.to_string()),
                obs.gs.map_or("NULL".to_string(), |v| v.to_string()),
                obs.rssi.map_or("NULL".to_string(), |v| v.to_string()),
                obs.msg_count_total
                    .map_or("NULL".to_string(), |v| v.to_string()),
                obs.raw_json.replace("'", "''")
            )
        })
        .collect();

    let sql = format!(
        "INSERT INTO aircraft_observations (ts_ms, hex, flight, lat, lon, altitude, gs, rssi, msg_count_total, raw_json) VALUES {}",
        values.join(",")
    );

    let result = sqlx::query(&sql).execute(pool).await?;
    Ok(result.rows_affected() as usize)
}

/// Query observations for a specific aircraft hex code, ordered by timestamp descending
#[allow(dead_code)] // Will be used in API endpoints (later prompts)
pub async fn list_observations_by_hex(
    pool: &SqlitePool,
    hex: &str,
    limit: usize,
) -> Result<Vec<AircraftObservation>> {
    let limit = std::cmp::min(limit, 1000); // Cap at 1000 for safety

    let rows = sqlx::query(
        r#"
        SELECT id, ts_ms, hex, flight, lat, lon, altitude, gs, rssi, msg_count_total, raw_json
        FROM aircraft_observations
        WHERE hex = ?
        ORDER BY ts_ms DESC
        LIMIT ?
        "#,
    )
    .bind(hex)
    .bind(limit as i64)
    .fetch_all(pool)
    .await?;

    let observations = rows
        .into_iter()
        .map(|row| {
            let id: i64 = row.get("id");
            let ts_ms: i64 = row.get("ts_ms");
            let hex: String = row.get("hex");
            let flight: Option<String> = row.get("flight");
            let lat: Option<f64> = row.get("lat");
            let lon: Option<f64> = row.get("lon");
            let altitude: Option<i32> = row.get("altitude");
            let gs: Option<f64> = row.get("gs");
            let rssi: Option<f64> = row.get("rssi");
            let msg_count_total: Option<i64> = row.get("msg_count_total");
            let raw_json: String = row.get("raw_json");

            AircraftObservation {
                id: Some(id),
                ts_ms,
                hex,
                flight,
                lat,
                lon,
                altitude,
                gs,
                rssi,
                msg_count_total,
                raw_json,
                msg_rate_hz: None, // Computed field, not stored
            }
        })
        .collect();

    Ok(observations)
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
    async fn test_insert_observations_single() {
        let (pool, _temp_dir) = setup_test_db().await;

        let obs = AircraftObservation {
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
            raw_json: r#"{"hex":"ABC123","flight":"UAL456"}"#.to_string(),
            msg_rate_hz: Some(2.5), // Should not be stored
        };

        let inserted = insert_observations(&pool, &[obs]).await.unwrap();
        assert_eq!(inserted, 1);

        // Verify insertion
        let observations = list_observations_by_hex(&pool, "ABC123", 10).await.unwrap();
        assert_eq!(observations.len(), 1);
        let stored_obs = &observations[0];
        assert_eq!(stored_obs.hex, "ABC123");
        assert_eq!(stored_obs.flight, Some("UAL456".to_string()));
        assert_eq!(stored_obs.lat, Some(40.7128));
        assert_eq!(stored_obs.altitude, Some(35000));
        assert!(stored_obs.id.is_some()); // Should have been assigned
        assert_eq!(stored_obs.msg_rate_hz, None); // Should not be stored
    }

    #[tokio::test]
    async fn test_insert_observations_batch() {
        let (pool, _temp_dir) = setup_test_db().await;

        let observations = vec![
            AircraftObservation {
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
                raw_json: r#"{"hex":"ABC123"}"#.to_string(),
                msg_rate_hz: None,
            },
            AircraftObservation {
                id: None,
                ts_ms: 1641024001000,
                hex: "ABC123".to_string(),
                flight: Some("UAL456".to_string()),
                lat: Some(40.7129),
                lon: Some(-74.0061),
                altitude: Some(35100),
                gs: Some(451.0),
                rssi: Some(-45.0),
                msg_count_total: Some(1010),
                raw_json: r#"{"hex":"ABC123"}"#.to_string(),
                msg_rate_hz: None,
            },
            AircraftObservation {
                id: None,
                ts_ms: 1641024002000,
                hex: "DEF456".to_string(),
                flight: Some("DAL789".to_string()),
                lat: Some(34.0522),
                lon: Some(-118.2437),
                altitude: Some(28000),
                gs: Some(380.0),
                rssi: Some(-52.1),
                msg_count_total: Some(750),
                raw_json: r#"{"hex":"DEF456"}"#.to_string(),
                msg_rate_hz: None,
            },
        ];

        let inserted = insert_observations(&pool, &observations).await.unwrap();
        assert_eq!(inserted, 3);

        // Test querying by hex
        let abc_observations = list_observations_by_hex(&pool, "ABC123", 10).await.unwrap();
        assert_eq!(abc_observations.len(), 2);

        // Should be ordered by timestamp descending
        assert!(abc_observations[0].ts_ms > abc_observations[1].ts_ms);
        assert_eq!(abc_observations[0].altitude, Some(35100));
        assert_eq!(abc_observations[1].altitude, Some(35000));

        let def_observations = list_observations_by_hex(&pool, "DEF456", 10).await.unwrap();
        assert_eq!(def_observations.len(), 1);
        assert_eq!(def_observations[0].flight, Some("DAL789".to_string()));
    }

    #[tokio::test]
    async fn test_insert_observations_empty() {
        let (pool, _temp_dir) = setup_test_db().await;

        let inserted = insert_observations(&pool, &[]).await.unwrap();
        assert_eq!(inserted, 0);
    }

    #[tokio::test]
    async fn test_list_observations_limit() {
        let (pool, _temp_dir) = setup_test_db().await;

        // Insert 5 observations
        let observations: Vec<AircraftObservation> = (0..5)
            .map(|i| AircraftObservation {
                id: None,
                ts_ms: 1641024000000 + i * 1000,
                hex: "TEST123".to_string(),
                flight: Some(format!("TEST{:03}", i)),
                lat: Some(40.0 + i as f64 * 0.001),
                lon: Some(-74.0 + i as f64 * 0.001),
                altitude: Some(35000 + (i as i32) * 100),
                gs: Some(450.0 + i as f64),
                rssi: Some(-45.0 - i as f64),
                msg_count_total: Some(1000 + i),
                raw_json: format!(r#"{{"hex":"TEST123","i":{}}}"#, i),
                msg_rate_hz: None,
            })
            .collect();

        insert_observations(&pool, &observations).await.unwrap();

        // Test limit functionality
        let limited = list_observations_by_hex(&pool, "TEST123", 3).await.unwrap();
        assert_eq!(limited.len(), 3);

        // Should be in descending timestamp order
        assert!(limited[0].ts_ms > limited[1].ts_ms);
        assert!(limited[1].ts_ms > limited[2].ts_ms);

        // First result should be the most recent (i=4)
        assert_eq!(limited[0].flight, Some("TEST004".to_string()));
    }

    #[tokio::test]
    async fn test_list_observations_nonexistent_hex() {
        let (pool, _temp_dir) = setup_test_db().await;

        let observations = list_observations_by_hex(&pool, "NONEXISTENT", 10)
            .await
            .unwrap();
        assert_eq!(observations.len(), 0);
    }

    #[tokio::test]
    async fn test_insert_observations_minimal_data() {
        let (pool, _temp_dir) = setup_test_db().await;

        let obs = AircraftObservation {
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

        let inserted = insert_observations(&pool, &[obs]).await.unwrap();
        assert_eq!(inserted, 1);

        let observations = list_observations_by_hex(&pool, "MIN123", 10).await.unwrap();
        assert_eq!(observations.len(), 1);
        let stored_obs = &observations[0];
        assert_eq!(stored_obs.hex, "MIN123");
        assert_eq!(stored_obs.flight, None);
        assert_eq!(stored_obs.lat, None);
        assert_eq!(stored_obs.altitude, None);
    }

    #[tokio::test]
    async fn test_batch_insert_performance_1000_aircraft() {
        let (pool, _temp_dir) = setup_test_db().await;

        // Create a realistic test with 1000 aircraft observations
        let observations: Vec<AircraftObservation> = (0..1000)
            .map(|i| AircraftObservation {
                id: None,
                ts_ms: 1641024000000 + i * 100, // 100ms intervals
                hex: format!("{:06X}", 0x100000 + i), // Realistic hex codes
                flight: if i % 3 == 0 {
                    Some(format!("UAL{:04}", i % 9999))
                } else {
                    None
                },
                lat: Some(40.0 + (i as f64) * 0.001),
                lon: Some(-74.0 + (i as f64) * 0.001),
                altitude: if i % 2 == 0 {
                    Some(35000 + (i as i32) * 10)
                } else {
                    None
                },
                gs: Some(450.0 + (i as f64) * 0.5),
                rssi: Some(-45.0 - (i as f64) * 0.01),
                msg_count_total: Some(1000 + i),
                raw_json: format!("{{\"hex\":\"{:06X}\"}}", 0x100000 + i),
                msg_rate_hz: None,
            })
            .collect();

        let start = std::time::Instant::now();
        let inserted = insert_observations(&pool, &observations).await.unwrap();
        let duration = start.elapsed();

        println!(
            "Batch inserted {} observations in {:?} ({:.0} ops/sec)",
            inserted,
            duration,
            inserted as f64 / duration.as_secs_f64()
        );

        assert_eq!(inserted, 1000);

        // Performance target: Should complete 1000 aircraft in under 100ms
        assert!(
            duration.as_millis() < 100,
            "Batch insert too slow: {:?} for 1000 aircraft",
            duration
        );

        // Verify all observations were inserted correctly
        let sample_obs = list_observations_by_hex(&pool, "100064", 1).await.unwrap();
        assert_eq!(sample_obs.len(), 1);
        assert_eq!(sample_obs[0].hex, "100064");
    }
}
