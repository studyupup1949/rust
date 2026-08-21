//! Table operations example for ADBC-Taos driver.
//!
//! This example demonstrates comprehensive table operations including
//! creating tables with various data types, querying, and managing data.

use adbc_core::{Connection, Database, Driver, Statement, options::OptionDatabase};
use arrow_array::RecordBatchReader;

fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let host = std::env::var("TAOS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("TAOS_PORT").unwrap_or_else(|_| "6030".to_string());
    let user = std::env::var("TAOS_USER").unwrap_or_else(|_| "root".to_string());
    let password = std::env::var("TAOS_PASSWORD").unwrap_or_else(|_| "taosdata".to_string());
    let db_name = std::env::var("TAOS_DATABASE").unwrap_or_else(|_| "demo".to_string());

    println!("Table operations example - TDengine at {}:{}", host, port);

    let mut driver = adbc_taos::driver::TaosDriver::default();

    let db = driver.new_database_with_opts([
        (OptionDatabase::Uri, format!("taos://{}:{}", host, port).into()),
        (OptionDatabase::Username, user.into()),
        (OptionDatabase::Password, password.into()),
    ])?;

    let mut connection = db.new_connection()?;

    // Create database
    println!("\n=== Step 1: Create Database ===");
    execute_sql(&mut connection, &format!("CREATE DATABASE IF NOT EXISTS {}", db_name))?;
    // db.database = Some(db_name.clone());
    let mut connection = db.new_connection()?;

    // Create a regular table
    println!("\n=== Step 2: Create Regular Table ===");
    let table_sql = format!(
        "CREATE TABLE IF NOT EXISTS {}.weather (\
         ts TIMESTAMP, \
         location NCHAR(50), \
         temperature FLOAT, \
         humidity FLOAT, \
         wind_speed FLOAT, \
         precipitation FLOAT, \
         is_raining BOOL\
         )",
        db_name
    );
    execute_sql(&mut connection, &table_sql)?;

    // Create a super table
    println!("\n=== Step 3: Create Super Table ===");
    let stable_sql = format!(
        "CREATE STABLE IF NOT EXISTS {}.sensors (\
         ts TIMESTAMP, \
         temperature FLOAT, \
         humidity FLOAT\
         ) TAGS (device_id NCHAR(20), location NCHAR(50))",
        db_name
    );
    execute_sql(&mut connection, &stable_sql)?;

    // Create child tables using the super table
    println!("\n=== Step 4: Create Child Tables ===");
    execute_sql(&mut connection, &format!(
        "CREATE TABLE IF NOT EXISTS {}.sensor_1 USING {}.sensors TAGS ('sensor_001', 'Building A')",
        db_name, db_name
    ))?;
    execute_sql(&mut connection, &format!(
        "CREATE TABLE IF NOT EXISTS {}.sensor_2 USING {}.sensors TAGS ('sensor_002', 'Building B')",
        db_name, db_name
    ))?;

    // Insert data into regular table
    println!("\n=== Step 5: Insert Data ===");
    execute_sql(&mut connection, &format!(
        "INSERT INTO {}.weather VALUES \
         (NOW, 'New York', 22.5, 65.0, 12.3, 0.0, false), \
         (NOW + 1h, 'New York', 23.0, 63.0, 15.2, 0.5, true), \
         (NOW + 2h, 'London', 15.5, 78.0, 8.5, 2.0, true)",
        db_name
    ))?;

    // Insert data into child tables
    execute_sql(&mut connection, &format!(
        "INSERT INTO {}.sensor_1 VALUES \
         (NOW, 25.3, 60.5), \
         (NOW + 10s, 25.5, 60.8), \
         (NOW + 20s, 25.7, 61.0)",
        db_name
    ))?;
    execute_sql(&mut connection, &format!(
        "INSERT INTO {}.sensor_2 VALUES \
         (NOW, 24.0, 58.0), \
         (NOW + 10s, 24.2, 58.5), \
         (NOW + 20s, 24.4, 59.0)",
        db_name
    ))?;

    // Query from regular table
    println!("\n=== Step 6: Query Regular Table ===");
    query_and_print(&mut connection, &format!("SELECT * FROM {}.weather", db_name))?;

    // Query from super table
    println!("\n=== Step 7: Query Super Table ===");
    query_and_print(&mut connection, &format!("SELECT * FROM {}.sensors", db_name))?;

    // Aggregate query
    println!("\n=== Step 8: Aggregate Query ===");
    query_and_print(&mut connection, &format!(
        "SELECT device_id, location, AVG(temperature) as avg_temp, AVG(humidity) as avg_humidity \
         FROM {}.sensors GROUP BY device_id, location",
        db_name
    ))?;

    // Get table schema
    println!("\n=== Step 9: Get Table Schema ===");
    match connection.get_table_schema(None, Some(&db_name), "weather") {
        Ok(schema) => {
            println!("\nSchema for 'weather' table:");
            for field in schema.fields() {
                println!("  - {:20} {:30} nullable={}",
                    field.name(),
                    format!("{:?}", field.data_type()),
                    field.is_nullable()
                );
            }
        }
        Err(e) => println!("Error getting schema: {}", e),
    }

    // Drop tables (cleanup)
    println!("\n=== Step 10: Cleanup ===");
    execute_sql(&mut connection, &format!("DROP TABLE IF EXISTS {}.sensor_1", db_name))?;
    execute_sql(&mut connection, &format!("DROP TABLE IF EXISTS {}.sensor_2", db_name))?;
    execute_sql(&mut connection, &format!("DROP STABLE IF EXISTS {}.sensors", db_name))?;
    execute_sql(&mut connection, &format!("DROP TABLE IF EXISTS {}.weather", db_name))?;
    execute_sql(&mut connection, &format!("DROP DATABASE IF EXISTS {}", db_name))?;

    println!("\nExample completed successfully!");

    Ok(())
}

/// Execute a SQL statement that doesn't return results.
fn execute_sql(connection: &mut impl Connection, sql: &str) -> anyhow::Result<i64> {
    let mut stmt = connection.new_statement()?;
    stmt.set_sql_query(sql)?;
    stmt.prepare()?;
    let result = stmt.execute_update()?;
    println!("  SQL: {}", sql);
    println!("  Result: {:?} rows affected", result);
    Ok(result.unwrap_or(0))
}

/// Execute a query and print results.
fn query_and_print(connection: &mut impl Connection, sql: &str) -> anyhow::Result<()> {
    let mut stmt = connection.new_statement()?;
    stmt.set_sql_query(sql)?;
    let mut reader = stmt.execute()?;

    println!("  SQL: {}", sql);
    println!("  Results:");

    let schema = reader.schema();
    let headers: Vec<String> = schema.fields().iter().map(|f| f.name().as_str().to_string()).collect();
    println!("    | {}", headers.join(" | "));

    let mut total_rows = 0;

    while let Some(batch) = reader.next() {
        let batch = batch?;
        total_rows += batch.num_rows();

        for row in 0..batch.num_rows() {
            let values: Vec<String> = batch
                .columns()
                .iter()
                .map(|col| format_column(col, row))
                .collect();
            println!("    | {}", values.join(" | "));
        }
    }

    println!("  Total rows: {}", total_rows);
    Ok(())
}

/// Format a column value for display.
fn format_column(column: &dyn arrow_array::Array, row: usize) -> String {
    use arrow_array::StringArray;

    if column.is_null(row) {
        return "NULL".to_string();
    }

    let as_any = column.as_any();

    if let Some(arr) = as_any.downcast_ref::<StringArray>() {
        arr.value(row).to_string()
    } else if as_any.is::<arrow_array::Float64Array>() {
        "[float64]".to_string()
    } else if as_any.is::<arrow_array::Float32Array>() {
        "[float32]".to_string()
    } else if as_any.is::<arrow_array::Int64Array>() {
        "[int64]".to_string()
    } else if as_any.is::<arrow_array::Int32Array>() {
        "[int32]".to_string()
    } else if as_any.is::<arrow_array::BooleanArray>() {
        "[bool]".to_string()
    } else {
        format!("{:?}", column.data_type())
    }
}
