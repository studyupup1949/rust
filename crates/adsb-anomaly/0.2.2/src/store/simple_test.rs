// ABOUTME: Simplified database tests to verify basic functionality works
// ABOUTME: Minimal test coverage to get B3 working without SQLx macro complexity

use crate::model::AircraftObservation;
use crate::store::{connect_and_migrate, observations, sessions};
use tempfile::TempDir;

#[tokio::test]
async fn test_simple_operations() {
    // Setup test database
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let pool = connect_and_migrate(db_path.to_str().unwrap(), true)
        .await
        .unwrap();

    // Create a test observation
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
        raw_json: r#"{"hex":"ABC123","flight":"UAL456"}"#.to_string(),
        msg_rate_hz: None,
    };

    // Test observation insertion
    let inserted = observations::insert_observations(&pool, &[obs.clone()])
        .await
        .unwrap();
    assert_eq!(inserted, 1);

    // Test observation retrieval
    let retrieved = observations::list_observations_by_hex(&pool, "ABC123", 10)
        .await
        .unwrap();
    assert_eq!(retrieved.len(), 1);
    assert_eq!(retrieved[0].hex, "ABC123");
    assert_eq!(retrieved[0].flight, Some("UAL456".to_string()));

    // Test session upsert
    sessions::upsert_session_from_observation(&pool, &mut obs)
        .await
        .unwrap();

    // Verify we can perform basic database operations without errors
    println!("✅ All basic database operations completed successfully");
}
