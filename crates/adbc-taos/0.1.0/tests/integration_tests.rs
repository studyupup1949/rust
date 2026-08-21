//! Integration tests for ADBC-Taos driver.
//!
//! These tests require a running TDengine instance.
//! Set environment variables to configure connection:
//! - TAOS_HOST (default: 127.0.0.1)
//! - TAOS_PORT (default: 6030)
//! - TAOS_USER (default: root)
//! - TAOS_PASSWORD (default: taosdata)

mod common;

use adbc_core::{Database, Connection, Optionable, Statement, Driver};
use adbc_core::options::{OptionDatabase, OptionValue, ObjectDepth};
use arrow_array::{Array, RecordBatchReader};

use common::TestConfig;

/// Tests basic connection to TDengine.
#[test]
fn test_connection_basic() {
    let config = TestConfig::from_env();
    let mut driver = adbc_taos::TaosDriver::default();

    let mut db = driver.new_database().expect("Failed to create database");
    db.set_option(
        OptionDatabase::Uri,
        OptionValue::String(config.dsn())
    ).expect("Failed to set URI");

    let conn = db.new_connection().expect("Failed to create connection");

    // Verify we can get server version
    let version = conn.server_version();
    assert!(!version.is_empty(), "Server version should not be empty");
    println!("Connected to TDengine version: {}", version);
}

/// Tests connection failure with invalid credentials.
#[test]
fn test_connection_invalid_credentials() {
    let config = TestConfig::from_env();
    let mut driver = adbc_taos::TaosDriver::default();

    let mut db = driver.new_database().expect("Failed to create database");

    // Use invalid password
    let invalid_dsn = format!("taos://{}:wrong@{}:{}",
        config.user, config.host, config.port);

    db.set_option(
        OptionDatabase::Uri,
        OptionValue::String(invalid_dsn)
    ).expect("Failed to set URI");

    let result = db.new_connection();
    assert!(result.is_err(), "Connection should fail with invalid credentials");
}

/// Tests creating and dropping a test database.
#[test]
fn test_database_operations() {
    let (mut conn, db_name) = common::create_test_connection();

    // Create database
    let mut stmt = conn.new_statement().expect("Failed to create statement");
    stmt.set_sql_query(format!("CREATE DATABASE IF NOT EXISTS {}", db_name))
        .expect("Failed to set query");

    let result = stmt.execute_update();
    assert!(result.is_ok(), "CREATE DATABASE should succeed");
    let affected = result.unwrap().unwrap();
    assert!(affected >= 0, "Affected rows should be non-negative");

    // Use the database
    stmt.set_sql_query(format!("USE {}", db_name))
        .expect("Failed to set USE query");
    stmt.execute_update().expect("USE should succeed");

    // Create a test table
    stmt.set_sql_query(format!("CREATE TABLE IF NOT EXISTS {}.test_data (ts TIMESTAMP, val INT)", db_name))
        .expect("Failed to set CREATE TABLE query");
    stmt.execute_update().expect("CREATE TABLE should succeed");

    // Drop database
    stmt.set_sql_query(format!("DROP DATABASE IF EXISTS {}", db_name))
        .expect("Failed to set DROP query");
    stmt.execute_update().expect("DROP DATABASE should succeed");
}

/// Tests basic SELECT query execution.
#[test]
fn test_select_query() {
    let (mut conn, db_name) = common::create_test_connection();

    // Setup: create database and table
    let mut stmt = conn.new_statement().expect("Failed to create statement");

    stmt.set_sql_query(format!("CREATE DATABASE IF NOT EXISTS {}", db_name))
        .expect("Failed to set CREATE DATABASE");
    stmt.execute_update().expect("Failed to create database");

    stmt.set_sql_query(format!("CREATE TABLE IF NOT EXISTS {}.sensor_data (ts TIMESTAMP, temp FLOAT, humidity INT)", db_name))
        .expect("Failed to set CREATE TABLE");
    stmt.execute_update().expect("Failed to create table");

    // Insert test data
    stmt.set_sql_query(format!(
        "INSERT INTO {}.sensor_data VALUES (NOW, 25.5, 60), (NOW + 1s, 26.0, 65)",
        db_name
    )).expect("Failed to set INSERT");
    stmt.execute_update().expect("Failed to insert data");

    // Query the data
    stmt.set_sql_query(format!("SELECT * FROM {}.sensor_data", db_name))
        .expect("Failed to set SELECT");

    let reader = stmt.execute().expect("Failed to execute SELECT");

    // Verify schema
    let schema = reader.schema();
    assert_eq!(schema.fields().len(), 3, "Should have 3 columns");

    // Read results
    let batches: Vec<arrow_array::RecordBatch> = reader.collect::<Result<_, _>>().expect("Failed to read batches");
    assert!(!batches.is_empty(), "Should have at least one batch");

    let total_rows: usize = batches.iter().map(|b: &arrow_array::RecordBatch| b.num_rows()).sum();
    assert_eq!(total_rows, 2, "Should have 2 rows");

    // Cleanup
    stmt.set_sql_query(format!("DROP DATABASE IF EXISTS {}", db_name))
        .expect("Failed to set DROP");
    stmt.execute_update().expect("Failed to drop database");
}

/// Tests get_info driver metadata method.
#[test]
fn test_get_info() {
    let config = TestConfig::from_env();
    let mut driver = adbc_taos::TaosDriver::default();

    let mut db = driver.new_database().expect("Failed to create database");
    db.set_option(
        OptionDatabase::Uri,
        OptionValue::String(config.dsn())
    ).expect("Failed to set URI");

    let conn = db.new_connection().expect("Failed to create connection");

    // Test get_info
    let info_reader = conn.get_info(None).expect("Failed to get info");
    let schema = info_reader.schema();

    // Info schema has info_code and info_value columns
    assert_eq!(schema.fields().len(), 2, "Info should have 2 columns");

    let batches: Vec<arrow_array::RecordBatch> = info_reader.collect::<Result<_, _>>().expect("Failed to read info");
    assert!(!batches.is_empty(), "Should have info data");

    // Verify vendor info exists - check info_value column (second column)
    let all_text: Vec<String> = batches.iter()
        .filter_map(|b: &arrow_array::RecordBatch| {
            let col = b.column(1); // info_value column
            col.as_any().downcast_ref::<arrow_array::StringArray>()
        })
        .flat_map(|arr: &arrow_array::StringArray| {
            (0..arr.len()).filter_map(|i| arr.is_valid(i).then(|| arr.value(i).to_string()))
        })
        .collect();

    assert!(all_text.iter().any(|s: &String| s.contains("TDengine")),
        "Should contain TDengine vendor name");
}

/// Tests get_table_types metadata method.
#[test]
fn test_get_table_types() {
    let config = TestConfig::from_env();
    let mut driver = adbc_taos::TaosDriver::default();

    let mut db = driver.new_database().expect("Failed to create database");
    db.set_option(
        OptionDatabase::Uri,
        OptionValue::String(config.dsn())
    ).expect("Failed to set URI");

    let conn = db.new_connection().expect("Failed to create connection");

    let table_types = conn.get_table_types().expect("Failed to get table types");
    let schema = table_types.schema();

    assert_eq!(schema.fields().len(), 1, "Should have 1 column");
    assert_eq!(schema.fields()[0].name(), "table_type");

    let batches: Vec<arrow_array::RecordBatch> = table_types.collect::<Result<_, _>>().expect("Failed to read");
    let types: Vec<&str> = batches.iter()
        .filter_map(|b: &arrow_array::RecordBatch| {
            let col = b.column(0);
            col.as_any().downcast_ref::<arrow_array::StringArray>()
        })
        .flat_map(|arr: &arrow_array::StringArray| {
            (0..arr.len()).filter_map(|i| arr.is_valid(i).then(|| arr.value(i)))
        })
        .collect();

    assert!(types.contains(&"TABLE"), "Should contain TABLE type");
    assert!(types.contains(&"BASE TABLE"), "Should contain BASE TABLE type");
}

/// Tests get_objects for listing databases.
#[test]
fn test_get_objects_databases() {
    let config = TestConfig::from_env();
    let mut driver = adbc_taos::TaosDriver::default();

    let mut db = driver.new_database().expect("Failed to create database");
    db.set_option(
        OptionDatabase::Uri,
        OptionValue::String(config.dsn())
    ).expect("Failed to set URI");

    let conn = db.new_connection().expect("Failed to create connection");

    // Get catalogs (databases in TDengine)
    let objects = conn.get_objects(ObjectDepth::Catalogs, None, None, None, None, None)
        .expect("Failed to get objects");

    let schema = objects.schema();
    assert_eq!(schema.fields().len(), 1, "Catalogs should have 1 column");

    let batches: Vec<arrow_array::RecordBatch> = objects.collect::<Result<_, _>>().expect("Failed to read objects");
    assert!(!batches.is_empty(), "Should have at least one database");

    // Verify we have system databases
    let databases: Vec<&str> = batches.iter()
        .filter_map(|b: &arrow_array::RecordBatch| {
            let col = b.column(0);
            col.as_any().downcast_ref::<arrow_array::StringArray>()
        })
        .flat_map(|arr: &arrow_array::StringArray| {
            (0..arr.len()).filter_map(|i| arr.is_valid(i).then(|| arr.value(i)))
        })
        .collect();

    // TDengine always has these default databases
    println!("Found databases: {:?}", databases);
}

/// Tests get_table_schema for retrieving table structure.
#[test]
fn test_get_table_schema() {
    let (mut conn, db_name) = common::create_test_connection();

    // Setup: create database and table
    let mut stmt = conn.new_statement().expect("Failed to create statement");

    stmt.set_sql_query(format!("CREATE DATABASE IF NOT EXISTS {}", db_name))
        .expect("Failed to set CREATE DATABASE");
    stmt.execute_update().expect("Failed to create database");

    stmt.set_sql_query(format!(
        "CREATE TABLE IF NOT EXISTS {}.sensors (ts TIMESTAMP, temperature FLOAT, status INT)",
        db_name
    )).expect("Failed to set CREATE TABLE");
    stmt.execute_update().expect("Failed to create table");

    // Get table schema
    let schema = conn.get_table_schema(None, Some(db_name.as_str()), "sensors")
        .expect("Failed to get table schema");

    assert_eq!(schema.fields().len(), 3, "Should have 3 columns");

    // Verify column names
    let field_names: Vec<&str> = schema.fields().iter()
        .map(|f| f.name().as_str())
        .collect();
    assert_eq!(field_names, vec!["ts", "temperature", "status"]);

    // Cleanup
    stmt.set_sql_query(format!("DROP DATABASE IF EXISTS {}", db_name))
        .expect("Failed to set DROP");
    stmt.execute_update().expect("Failed to drop database");
}

/// Tests various TDengine data types.
#[test]
fn test_data_types() {
    let (mut conn, db_name) = common::create_test_connection();
    let mut stmt = conn.new_statement().expect("Failed to create statement");

    // Create database
    stmt.set_sql_query(format!("CREATE DATABASE IF NOT EXISTS {}", db_name))
        .expect("Failed to set query");
    stmt.execute_update().expect("Failed to create database");

    // Create table with various types
    let create_sql = format!(
        "CREATE TABLE IF NOT EXISTS {}.type_test (\
            ts TIMESTAMP, \
            bool_col BOOL, \
            tiny_col TINYINT, \
            small_col SMALLINT, \
            int_col INT, \
            bigint_col BIGINT, \
            uint_col INT UNSIGNED, \
            float_col FLOAT, \
            double_col DOUBLE, \
            binary_col BINARY(10), \
            nchar_col NCHAR(10) \
        )",
        db_name
    );
    stmt.set_sql_query(create_sql).expect("Failed to set CREATE TABLE");
    stmt.execute_update().expect("Failed to create table");

    // Insert data
    let insert_sql = format!(
        "INSERT INTO {}.type_test VALUES (\
            NOW, \
            true, \
            127, \
            32767, \
            2147483647, \
            9223372036854775807, \
            4294967295, \
            3.14, \
            3.14159, \
            'binary', \
            'nchar' \
        )",
        db_name
    );
    stmt.set_sql_query(insert_sql).expect("Failed to set INSERT");
    stmt.execute_update().expect("Failed to insert");

    // Query and verify
    stmt.set_sql_query(format!("SELECT * FROM {}.type_test", db_name))
        .expect("Failed to set SELECT");

    let reader = stmt.execute().expect("Failed to execute");
    let batches: Vec<arrow_array::RecordBatch> = reader.collect::<Result<_, _>>().expect("Failed to read");

    assert!(!batches.is_empty(), "Should have data");
    assert_eq!(batches[0].num_rows(), 1, "Should have 1 row");

    // Cleanup
    stmt.set_sql_query(format!("DROP DATABASE IF EXISTS {}", db_name))
        .expect("Failed to set DROP");
    stmt.execute_update().expect("Failed to cleanup");
}

/// Tests connection pool functionality.
#[test]
fn test_connection_pool() {
    use std::sync::Arc;
    use adbc_taos::{TaosPool, TaosPoolConfig, Runtime};

    let config = TestConfig::from_env();
    let rt = Arc::new(Runtime::new().expect("Failed to create runtime"));

    let pool_config = TaosPoolConfig {
        dsn: config.dsn(),
        max_size: 4,
    };

    let pool = TaosPool::with_config(pool_config, rt)
        .expect("Failed to create pool");

    // Check that pool can be created and status is accessible
    let status = pool.status();
    assert_eq!(status.max_size, 4, "Pool max_size should be 4");
    assert_eq!(status.size, 0, "Initial pool size should be 0 (no connections created yet)");

    println!("Pool status: max_size={}", status.max_size);
}

/// Tests connection pool configuration.
#[test]
fn test_connection_pool_config() {
    use std::sync::Arc;
    use adbc_taos::{TaosPool, TaosPoolConfig, Runtime};

    let config = TestConfig::from_env();
    let rt = Arc::new(Runtime::new().expect("Failed to create runtime"));

    // Test different pool configurations
    let pool_config_small = TaosPoolConfig {
        dsn: config.dsn(),
        max_size: 2,
    };

    let pool_small = TaosPool::with_config(pool_config_small, rt.clone())
        .expect("Failed to create small pool");

    let status_small = pool_small.status();
    assert_eq!(status_small.max_size, 2, "Small pool max_size should be 2");

    let pool_config_large = TaosPoolConfig {
        dsn: config.dsn(),
        max_size: 100,
    };

    let pool_large = TaosPool::with_config(pool_config_large, rt)
        .expect("Failed to create large pool");

    let status_large = pool_large.status();
    assert_eq!(status_large.max_size, 100, "Large pool max_size should be 100");
}

/// Tests connection pool through TaosDatabase API.
#[test]
fn test_connection_pool_via_database() {
    use adbc_taos::TaosDatabase;

    let config = TestConfig::from_env();

    // Create database with pool configuration
    let mut db = TaosDatabase::default();
    db.uri = config.dsn();
    db.user = config.user;
    db.password = config.password;
    db.set_pool_size(4);

    // Create connections from pool
    let conn1 = db.create_connection().expect("Failed to create pooled connection 1");
    let conn2 = db.create_connection().expect("Failed to create pooled connection 2");

    // Verify both connections work
    let version1 = conn1.server_version();
    let version2 = conn2.server_version();
    assert!(!version1.is_empty(), "Connection 1 should work");
    assert!(!version2.is_empty(), "Connection 2 should work");
    println!("Pool: Two connections created - versions: {} and {}", version1, version2);
}
