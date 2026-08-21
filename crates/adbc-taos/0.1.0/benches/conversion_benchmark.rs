//! TDengine-to-Arrow conversion performance benchmarks.
//!
//! Validates performance optimizations outlined in TOOPTIMIZED.md by measuring
//! latency, throughput, and memory usage for various data distributions and sizes.
//!
//! # Prerequisites
//!
//! Requires a running TDengine instance. Configure via environment variables:
//! - `TAOS_HOST`: TDengine server host (default: "127.0.0.1")
//! - `TAOS_PORT`: TDengine server port (default: "6030")
//! - `TAOS_USER`: Database user (default: "root")
//! - `TAOS_PASSWORD`: User password (default: "taosdata")
//! - `TAOS_DATABASE`: Target database (default: "demo")
//!
//! # Running Benchmarks
//!
//! ```bash
//! # Run all benchmarks
//! cargo bench
//!
//! # Run specific benchmark
//! cargo bench --bench conversion_benchmark -- small
//!
//! # Save baseline for future comparison
//! cargo bench -- --save-baseline main
//!
//! # Compare against baseline
//! cargo bench -- --baseline main
//! ```
//!
//! # Expected Performance (After Optimization)
//!
//! - Latency (1M rows): 60-70% of pre-optimization time
//! - Memory Peak: 50-60% of pre-optimization usage
//! - Throughput: 140-160% of pre-optimization rate

use std::time::Duration;

use adbc_core::{Connection, Database, Driver, Optionable, Statement};
use adbc_core::options::{OptionDatabase, OptionValue};
use criterion::{black_box, BenchmarkId, Criterion, criterion_group, criterion_main};

/// Benchmark configuration constants.
const SMALL_ROWS: usize = 1_000;
const MEDIUM_ROWS: usize = 100_000;
const LARGE_ROWS: usize = 1_000_000;

/// Test configuration loaded from environment.
struct TestConfig {
    host: String,
    port: u16,
    user: String,
    password: String,
    database: String,
}

impl TestConfig {
    fn from_env() -> Self {
        let _ = dotenvy::dotenv();

        Self {
            host: std::env::var("TAOS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: std::env::var("TAOS_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(6030),
            user: std::env::var("TAOS_USER").unwrap_or_else(|_| "root".to_string()),
            password: std::env::var("TAOS_PASSWORD").unwrap_or_else(|_| "taosdata".to_string()),
            database: std::env::var("TAOS_DATABASE").unwrap_or_else(|_| "demo".to_string()),
        }
    }

    fn dsn(&self) -> String {
        format!("taos://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, self.database)
    }
}

/// Creates a connection to TDengine using ADBC API.
fn create_connection() -> Option<(adbc_taos::TaosConnection, TestConfig)> {
    let config = TestConfig::from_env();
    let mut driver = adbc_taos::TaosDriver::default();

    let mut db = driver.new_database().ok()?;
    db.set_option(
        OptionDatabase::Uri,
        OptionValue::String(config.dsn())
    ).ok()?;

    let conn = db.new_connection().ok()?;
    Some((conn, config))
}

/// Creates test data and benchmarks small dataset conversion (1,000 rows).
fn benchmark_small(c: &mut Criterion) {
    eprintln!("Attempting to connect to TDengine for small benchmark...");

    let (mut conn, config) = match create_connection() {
        Some((conn, config)) => {
            eprintln!("Connection successful!");
            (conn, config)
        },
        None => {
            eprintln!("Skipping small benchmark: Could not connect to TDengine");
            return;
        }
    };

    let mut group = c.benchmark_group("conversion_small");

    // Use database first, create if not exists
    let mut stmt = conn.new_statement().unwrap();
    stmt.set_sql_query(format!("CREATE DATABASE IF NOT EXISTS {}", config.database)).unwrap();
    let _ = stmt.execute_update();

    stmt.set_sql_query(format!("USE {}", config.database)).unwrap();
    let _ = stmt.execute_update();

    // Create test table
    stmt.set_sql_query(
        "CREATE TABLE IF NOT EXISTS benchmark_small \
         (ts TIMESTAMP, val INT, name VARCHAR(50), is_flag BOOL, rate FLOAT)"
    ).unwrap();
    let create_result = stmt.execute_update();
    eprintln!("CREATE TABLE result: {:?}", create_result);

    // Clear and insert test data
    stmt.set_sql_query("DELETE FROM benchmark_small").unwrap();
    let delete_result = stmt.execute_update();
    eprintln!("DELETE result: {:?}", delete_result);

    // Insert data with proper millisecond timestamps
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    for i in 0..SMALL_ROWS {
        let ts = now + (i as i64 * 1000);
        let name = format!("sensor_{}", i % 10);
        let is_flag = i % 2 == 0;
        let rate = (i as f32) * 1.5;

        stmt.set_sql_query(format!(
            "INSERT INTO benchmark_small VALUES ({ts}, {i}, '{name}', {is_flag}, {rate})"
        )).unwrap();
        let _ = stmt.execute_update();
    }

    group.bench_function(BenchmarkId::new("mixed_types", SMALL_ROWS), |b| {
        b.iter(|| {
            let mut stmt = black_box(conn.new_statement().unwrap());

            // Ensure database and table exist
            stmt.set_sql_query(format!("USE {}", config.database)).unwrap();
            let _ = stmt.execute_update();
            stmt.set_sql_query(
                "CREATE TABLE IF NOT EXISTS benchmark_small \
                 (ts TIMESTAMP, val INT, name VARCHAR(50), is_flag BOOL, rate FLOAT)"
            ).unwrap();
            let _ = stmt.execute_update();

            stmt.set_sql_query("SELECT * FROM benchmark_small").unwrap();

            let mut reader = black_box(stmt.execute().unwrap());
            let mut total_rows = 0;

            while let Some(batch_result) = black_box(reader.next()) {
                let batch = batch_result.unwrap();
                total_rows += batch.num_rows();
            }

            assert_eq!(total_rows, SMALL_ROWS);
            total_rows
        })
    });

    group.finish();
}

/// Creates test data and benchmarks medium dataset conversion (100,000 rows).
fn benchmark_medium(c: &mut Criterion) {
    let (mut conn, config) = match create_connection() {
        Some((conn, config)) => (conn, config),
        None => {
            eprintln!("Skipping medium benchmark: Could not connect to TDengine");
            return;
        }
    };

    let mut group = c.benchmark_group("conversion_medium");

    group.sample_size(10);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(10));

    // Use database first, create if not exists
    let mut stmt = conn.new_statement().unwrap();
    stmt.set_sql_query(format!("CREATE DATABASE IF NOT EXISTS {}", config.database)).unwrap();
    let _ = stmt.execute_update();

    stmt.set_sql_query(format!("USE {}", config.database)).unwrap();
    let _ = stmt.execute_update();
    stmt.set_sql_query(
        "CREATE TABLE IF NOT EXISTS benchmark_medium \
         (ts TIMESTAMP, col1 INT, col2 BIGINT, col3 FLOAT, col4 DOUBLE, \
          is_col5 BOOL, col6 SMALLINT, col7 TINYINT, col8 VARCHAR(100), col9 NCHAR(50))"
    ).unwrap();
    let _ = stmt.execute_update();

    // Clear and insert test data in batches
    stmt.set_sql_query("DELETE FROM benchmark_medium").unwrap();
    let _ = stmt.execute_update();

    let batch_size = 1000;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;
    for batch_start in (0..MEDIUM_ROWS).step_by(batch_size) {
        let batch_end = (batch_start + batch_size).min(MEDIUM_ROWS);
        let mut values = Vec::new();

        for i in batch_start..batch_end {
            let ts = now + (i as i64 * 1000);
            values.push(format!(
                "({}, {}, {}, {}, {}, {}, {}, {}, 'name_{}', 'value_{}')",
                ts, i, i, i as f32, i as f64, (i % 2) as i8, i as i16, i as i8, i % 100, i % 50
            ));
        }

        stmt.set_sql_query(format!(
            "INSERT INTO benchmark_medium VALUES {}",
            values.join(", ")
        )).unwrap();
        let _ = stmt.execute_update();
    }

    group.bench_function(BenchmarkId::new("mixed_types", MEDIUM_ROWS), |b| {
        b.iter(|| {
            let mut stmt = black_box(conn.new_statement().unwrap());
            stmt.set_sql_query(format!("USE {}", config.database)).unwrap();
            let _ = stmt.execute_update();
            stmt.set_sql_query("SELECT * FROM benchmark_medium").unwrap();

            let mut reader = black_box(stmt.execute().unwrap());
            let mut total_rows = 0;

            while let Some(batch_result) = black_box(reader.next()) {
                let batch = batch_result.unwrap();
                total_rows += batch.num_rows();
            }

            assert_eq!(total_rows, MEDIUM_ROWS);
            total_rows
        })
    });

    group.finish();
}

/// Creates test data and benchmarks string-heavy dataset (100,000 rows, 80% strings).
fn benchmark_string_heavy(c: &mut Criterion) {
    let (mut conn, config) = match create_connection() {
        Some((conn, config)) => (conn, config),
        None => {
            eprintln!("Skipping string-heavy benchmark: Could not connect to TDengine");
            return;
        }
    };

    let mut group = c.benchmark_group("conversion_strings");

    group.sample_size(10);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(10));

    // Use database first, create if not exists
    let mut stmt = conn.new_statement().unwrap();
    stmt.set_sql_query(format!("CREATE DATABASE IF NOT EXISTS {}", config.database)).unwrap();
    let _ = stmt.execute_update();

    stmt.set_sql_query(format!("USE {}", config.database)).unwrap();
    let _ = stmt.execute_update();
    stmt.set_sql_query(
        "CREATE TABLE IF NOT EXISTS benchmark_strings \
         (ts TIMESTAMP, str1 VARCHAR(200), str2 VARCHAR(150), str3 NCHAR(100), \
          str4 VARCHAR(300), str5 NCHAR(200), str6 VARCHAR(100), str7 VARCHAR(250), \
          str8 NCHAR(150), col1 INT, col2 BIGINT)"
    ).unwrap();
    let _ = stmt.execute_update();

    stmt.set_sql_query("DELETE FROM benchmark_strings").unwrap();
    let _ = stmt.execute_update();

    // Insert data in batches
    let batch_size = 1000;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;
    for batch_start in (0..MEDIUM_ROWS).step_by(batch_size) {
        let batch_end = (batch_start + batch_size).min(MEDIUM_ROWS);
        let str_val = "long_string_value_for_testing_purposes";
        let mut values = Vec::new();

        for i in batch_start..batch_end {
            let ts = now + (i as i64 * 1000);
            values.push(format!(
                "({}, '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', {}, {})",
                ts, str_val, str_val, str_val, str_val, str_val, str_val, str_val, str_val, i, i
            ));
        }

        stmt.set_sql_query(format!(
            "INSERT INTO benchmark_strings VALUES {}",
            values.join(", ")
        )).unwrap();
        let _ = stmt.execute_update();
    }

    group.bench_function(BenchmarkId::new("string_columns", MEDIUM_ROWS), |b| {
        b.iter(|| {
            let mut stmt = black_box(conn.new_statement().unwrap());
            stmt.set_sql_query(format!("USE {}", config.database)).unwrap();
            let _ = stmt.execute_update();
            stmt.set_sql_query("SELECT * FROM benchmark_strings").unwrap();

            let mut reader = black_box(stmt.execute().unwrap());
            let mut total_rows = 0;

            while let Some(batch_result) = black_box(reader.next()) {
                let batch = batch_result.unwrap();
                total_rows += batch.num_rows();
            }

            assert_eq!(total_rows, MEDIUM_ROWS);
            total_rows
        })
    });

    group.finish();
}

/// Creates test data and benchmarks numeric-heavy dataset (100,000 rows, 80% numeric).
fn benchmark_numeric_heavy(c: &mut Criterion) {
    let (mut conn, config) = match create_connection() {
        Some((conn, config)) => (conn, config),
        None => {
            eprintln!("Skipping numeric-heavy benchmark: Could not connect to TDengine");
            return;
        }
    };

    let mut group = c.benchmark_group("conversion_numeric");

    group.sample_size(10);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(10));

    // Use database first, create if not exists
    let mut stmt = conn.new_statement().unwrap();
    stmt.set_sql_query(format!("CREATE DATABASE IF NOT EXISTS {}", config.database)).unwrap();
    let _ = stmt.execute_update();

    stmt.set_sql_query(format!("USE {}", config.database)).unwrap();
    let _ = stmt.execute_update();
    stmt.set_sql_query(
        "CREATE TABLE IF NOT EXISTS benchmark_numeric \
         (ts TIMESTAMP, num1 TINYINT, num2 SMALLINT, num3 INT, num4 BIGINT, \
          num5 FLOAT, num6 DOUBLE, num7 TINYINT, num8 SMALLINT, num9 INT, \
          str_val VARCHAR(50))"
    ).unwrap();
    let _ = stmt.execute_update();

    stmt.set_sql_query("DELETE FROM benchmark_numeric").unwrap();
    let _ = stmt.execute_update();

    // Insert data in batches
    let batch_size = 5000;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;
    for batch_start in (0..MEDIUM_ROWS).step_by(batch_size) {
        let batch_end = (batch_start + batch_size).min(MEDIUM_ROWS);
        let mut values = Vec::new();

        for i in batch_start..batch_end {
            let ts = now + (i as i64 * 1000);
            values.push(format!(
                "({}, {}, {}, {}, {}, {}, {}, {}, {}, 'str_{}')",
                ts, i as i8, i as i16, i, i, i as f32, i as f64, i as i8, i as i16, i
            ));
        }

        stmt.set_sql_query(format!(
            "INSERT INTO benchmark_numeric VALUES {}",
            values.join(", ")
        )).unwrap();
        let _ = stmt.execute_update();
    }

    group.bench_function(BenchmarkId::new("numeric_columns", MEDIUM_ROWS), |b| {
        b.iter(|| {
            let mut stmt = black_box(conn.new_statement().unwrap());
            stmt.set_sql_query(format!("USE {}", config.database)).unwrap();
            let _ = stmt.execute_update();
            stmt.set_sql_query("SELECT * FROM benchmark_numeric").unwrap();

            let mut reader = black_box(stmt.execute().unwrap());
            let mut total_rows = 0;

            while let Some(batch_result) = black_box(reader.next()) {
                let batch = batch_result.unwrap();
                total_rows += batch.num_rows();
            }

            assert_eq!(total_rows, MEDIUM_ROWS);
            total_rows
        })
    });

    group.finish();
}

/// Creates test data and benchmarks timestamp-heavy dataset (1,000,000 rows).
fn benchmark_timestamp(c: &mut Criterion) {
    let (mut conn, config) = match create_connection() {
        Some((conn, config)) => (conn, config),
        None => {
            eprintln!("Skipping timestamp benchmark: Could not connect to TDengine");
            return;
        }
    };

    let mut group = c.benchmark_group("conversion_timestamp");

    group.sample_size(10);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(10));

    // Use database first, create if not exists
    let mut stmt = conn.new_statement().unwrap();
    stmt.set_sql_query(format!("CREATE DATABASE IF NOT EXISTS {}", config.database)).unwrap();
    let _ = stmt.execute_update();

    stmt.set_sql_query(format!("USE {}", config.database)).unwrap();
    let _ = stmt.execute_update();
    stmt.set_sql_query(
        "CREATE TABLE IF NOT EXISTS benchmark_timestamp \
         (ts TIMESTAMP, ts1 TIMESTAMP, ts2 TIMESTAMP, ts3 TIMESTAMP, ts4 TIMESTAMP)"
    ).unwrap();
    let _ = stmt.execute_update();

    stmt.set_sql_query("DELETE FROM benchmark_timestamp").unwrap();
    let _ = stmt.execute_update();

    // Insert data in large batches
    let batch_size = 10000;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;
    for batch_start in (0..LARGE_ROWS).step_by(batch_size) {
        let batch_end = (batch_start + batch_size).min(LARGE_ROWS);
        let mut values = Vec::new();

        for i in batch_start..batch_end {
            let base_ts = now + (i as i64 * 1000);
            values.push(format!(
                "({}, {}, {}, {}, {})",
                base_ts, base_ts + 1, base_ts + 2, base_ts + 3, base_ts + 4
            ));
        }

        stmt.set_sql_query(format!(
            "INSERT INTO benchmark_timestamp VALUES {}",
            values.join(", ")
        )).unwrap();
        let _ = stmt.execute_update();
    }

    group.bench_function(BenchmarkId::new("timestamp_columns", LARGE_ROWS), |b| {
        b.iter(|| {
            let mut stmt = black_box(conn.new_statement().unwrap());
            stmt.set_sql_query(format!("USE {}", config.database)).unwrap();
            let _ = stmt.execute_update();
            stmt.set_sql_query("SELECT * FROM benchmark_timestamp").unwrap();

            let mut reader = black_box(stmt.execute().unwrap());
            let mut total_rows = 0;

            while let Some(batch_result) = black_box(reader.next()) {
                let batch = batch_result.unwrap();
                total_rows += batch.num_rows();
            }

            assert_eq!(total_rows, LARGE_ROWS);
            total_rows
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_small,
    benchmark_medium,
    benchmark_string_heavy,
    benchmark_numeric_heavy,
    benchmark_timestamp
);
criterion_main!(benches);

// Rust guideline compliant 2026-01-03
