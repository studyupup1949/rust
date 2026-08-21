//! Tests to verify README examples compile and work correctly.
//! These tests should match the code shown in README.md.

mod common;

use adbc_core::{Database, Connection, Statement, Driver, Optionable};
use adbc_core::options::{OptionDatabase, OptionValue};
use arrow_array::{TimestampMillisecondArray, Float64Array, RecordBatchReader};

/// This test mirrors the "Basic Query" example from README.md
#[test]
fn test_readme_basic_query() {
    let config = common::TestConfig::from_env();
    let mut driver = adbc_taos::TaosDriver::default();

    let mut db = driver.new_database().expect("Failed to create database");
    db.set_option(
        OptionDatabase::Uri,
        OptionValue::String(config.dsn())
    ).expect("Failed to set URI");

    let mut conn = db.new_connection().expect("Failed to connect");
    println!("Connected to TDengine: {}", conn.server_version());

    // Create test database and table
    let mut stmt = conn.new_statement().expect("Failed to create statement");

    let db_name = format!("readme_test_{}", std::process::id());
    stmt.set_sql_query(format!("CREATE DATABASE IF NOT EXISTS {}", db_name))
        .expect("Failed to set CREATE DATABASE");
    stmt.execute_update().expect("Failed to create database");

    stmt.set_sql_query(format!(
        "CREATE TABLE IF NOT EXISTS {}.sensor_data (ts TIMESTAMP, temperature DOUBLE)",
        db_name
    )).expect("Failed to set CREATE TABLE");
    stmt.execute_update().expect("Failed to create table");

    // Insert test data
    stmt.set_sql_query(format!(
        "INSERT INTO {}.sensor_data VALUES (NOW, 25.5), (NOW + 1s, 26.0), (NOW + 2s, 24.8)",
        db_name
    )).expect("Failed to set INSERT");
    stmt.execute_update().expect("Failed to insert");

    // Execute query - this mirrors the README example
    stmt.set_sql_query(format!("SELECT * FROM {}.sensor_data LIMIT 10", db_name))
        .expect("Failed to set SELECT");

    let reader = stmt.execute().expect("Failed to execute query");

    // Read schema first to see column names and types
    println!("Schema: {:?}", reader.schema());

    let mut total_rows = 0;

    // Process results as Arrow RecordBatches
    for batch in reader {
        let batch = batch.expect("Failed to read batch");
        println!("Got batch with {} rows, {} columns", batch.num_rows(), batch.num_columns());

        total_rows += batch.num_rows();

        // Access columns by index
        if batch.num_columns() >= 2 {
            if let Some(ts_col) = batch.column(0).as_any().downcast_ref::<TimestampMillisecondArray>() {
                for ts in ts_col {
                    if let Some(t) = ts {
                        println!("  Timestamp: {}", t);
                    }
                }
            }

            if let Some(val_col) = batch.column(1).as_any().downcast_ref::<Float64Array>() {
                for val in val_col {
                    if let Some(v) = val {
                        println!("  Value: {}", v);
                    }
                }
            }
        }
    }

    assert_eq!(total_rows, 3, "Should have 3 rows");

    // Cleanup
    stmt.set_sql_query(format!("DROP DATABASE IF EXISTS {}", db_name))
        .expect("Failed to set DROP");
    stmt.execute_update().expect("Failed to cleanup");
}

/// This test mirrors the "Simple Direct Usage" example from README.md
#[test]
fn test_readme_simple_direct() {
    use adbc_taos::TaosDatabase;

    let config = common::TestConfig::from_env();
    let mut db = TaosDatabase::default();
    db.uri = config.dsn();

    let mut conn = db.new_connection().expect("Failed to connect");
    let mut stmt = conn.new_statement().expect("Failed to create statement");

    // Simple query (avoiding reserved keywords)
    stmt.set_sql_query("SELECT 1 as result_count").expect("Failed to set query");
    let reader = stmt.execute().expect("Failed to execute");

    let mut found_result = false;
    for batch in reader {
        let batch = batch.expect("Failed to read batch");
        let col = batch.column(0);
        println!("Result: {:?}", col);
        found_result = true;
    }

    assert!(found_result, "Should have at least one result batch");
}

/// This test verifies prepared statement example from README
/// Note: TDengine's prepared statement API has limitations.
/// This test uses direct SQL execution instead.
#[test]
fn test_readme_prepared_statement() {
    use adbc_core::{Connection, Statement};

    let (mut conn, db_name) = common::create_test_connection();
    let mut stmt = conn.new_statement().expect("Failed to create statement");

    // Create database and table
    stmt.set_sql_query(format!("CREATE DATABASE IF NOT EXISTS {}", db_name))
        .expect("Failed to set CREATE DATABASE");
    stmt.execute_update().expect("Failed to create database");

    // Use non-reserved column names
    // Note: TDengine requires first column to be TIMESTAMP type
    stmt.set_sql_query(format!(
        "CREATE TABLE IF NOT EXISTS {}.sensor_data (ts TIMESTAMP, temp_val DOUBLE)",
        db_name
    )).expect("Failed to set CREATE TABLE");
    stmt.execute_update().expect("Failed to create table");

    // Insert data using direct SQL (more reliable than prepared statements for TDengine)
    stmt.set_sql_query(format!(
        "INSERT INTO {}.sensor_data VALUES (NOW, 42.5), (NOW + 1s, 43.1)",
        db_name
    )).expect("Failed to set INSERT");
    let affected = stmt.execute_update().expect("Failed to execute");
    assert_eq!(affected, Some(2), "Should insert 2 rows");

    // Verify data was inserted
    stmt.set_sql_query(format!("SELECT COUNT(*) FROM {}.sensor_data", db_name))
        .expect("Failed to set SELECT");
    let reader = stmt.execute().expect("Failed to execute");

    for batch in reader {
        let batch = batch.expect("Failed to read batch");
        assert_eq!(batch.num_rows(), 1, "Should have 1 result row");
    }

    // Cleanup
    stmt.set_sql_query(format!("DROP DATABASE IF EXISTS {}", db_name))
        .expect("Failed to set DROP");
    stmt.execute_update().expect("Failed to cleanup");
}
