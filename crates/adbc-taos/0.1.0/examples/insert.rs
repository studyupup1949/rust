//! Data insertion example for ADBC-Taos driver.
//!
//! This example demonstrates how to create tables and insert data
//! using SQL statements.

use adbc_core::{Connection, Database, Driver, Statement, options::OptionDatabase};

fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let host = std::env::var("TAOS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("TAOS_PORT").unwrap_or_else(|_| "6030".to_string());
    let user = std::env::var("TAOS_USER").unwrap_or_else(|_| "root".to_string());
    let password = std::env::var("TAOS_PASSWORD").unwrap_or_else(|_| "taosdata".to_string());
    let db_name = std::env::var("TAOS_DATABASE").unwrap_or_else(|_| "test_db".to_string());

    println!("Connecting to TDengine at {}:{}", host, port);

    let mut driver = adbc_taos::driver::TaosDriver::default();

    let db = driver.new_database_with_opts([
        (OptionDatabase::Uri, format!("taos://{}:{}", host, port).into()),
        (OptionDatabase::Username, user.into()),
        (OptionDatabase::Password, password.into()),
    ])?;

    let mut connection = db.new_connection()?;

    // Create database
    println!("\n1. Creating database '{}'", db_name);
    {
        let mut stmt = connection.new_statement()?;
        stmt.set_sql_query(&format!("CREATE DATABASE IF NOT EXISTS {}", db_name))?;
        stmt.prepare()?;
        let result = stmt.execute_update()?;
        println!("   Done. Affected rows: {:?}", result);
    }

    // // Use the database
    // db.database = Some(db_name.clone());
    let mut connection = db.new_connection()?;

    // Create table
    println!("\n2. Creating table 'sensor_data'");
    let create_table_sql = format!(
        "CREATE TABLE IF NOT EXISTS {}.sensor_data (\
         ts TIMESTAMP, \
         device_id NCHAR(20), \
         temperature FLOAT, \
         humidity FLOAT, \
         pressure INT\
         )",
        db_name
    );
    {
        let mut stmt = connection.new_statement()?;
        stmt.set_sql_query(&create_table_sql)?;
        stmt.prepare()?;
        let result = stmt.execute_update()?;
        println!("   Done. Affected rows: {:?}", result);
    }

    // Insert single row
    println!("\n3. Inserting single row");
    let insert_sql = format!(
        "INSERT INTO {}.sensor_data VALUES (NOW, 'device_001', 25.5, 60.2, 1013)",
        db_name
    );
    {
        let mut stmt = connection.new_statement()?;
        stmt.set_sql_query(&insert_sql)?;
        stmt.prepare()?;
        let result = stmt.execute_update()?;
        println!("   Done. Affected rows: {:?}", result);
    }

    // Insert multiple rows
    println!("\n4. Inserting multiple rows");
    let multi_insert_sql = format!(
        "INSERT INTO {}.sensor_data VALUES \
         (NOW + 1s, 'device_001', 25.8, 61.0, 1014), \
         (NOW + 2s, 'device_002', 24.5, 58.5, 1012), \
         (NOW + 3s, 'device_003', 26.2, 62.1, 1015)",
        db_name
    );
    {
        let mut stmt = connection.new_statement()?;
        stmt.set_sql_query(&multi_insert_sql)?;
        stmt.prepare()?;
        let result = stmt.execute_update()?;
        println!("   Done. Affected rows: {:?}", result);
    }

    // Query back the data
    println!("\n5. Querying inserted data");
    {
        let mut stmt = connection.new_statement()?;
        stmt.set_sql_query(&format!("SELECT * FROM {}.sensor_data", db_name))?;
        let mut reader = stmt.execute()?;

        println!("   Results:");
        loop {
            match reader.next() {
                Some(Ok(batch)) => {
                    println!("   Batch: {} rows", batch.num_rows());
                }
                Some(Err(e)) => {
                    println!("   Error: {}", e);
                    break;
                }
                None => break,
            }
        }
    }

    println!("\nExample completed successfully!");

    Ok(())
}
