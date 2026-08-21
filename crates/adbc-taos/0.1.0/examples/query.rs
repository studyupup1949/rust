//! Query execution example for ADBC-Taos driver.
//!
//! This example demonstrates how to execute SELECT queries and
//! retrieve results as Arrow RecordBatches.

use adbc_core::{Connection, Database, Driver, Statement, options::OptionDatabase};

fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let host = std::env::var("TAOS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("TAOS_PORT").unwrap_or_else(|_| "6030".to_string());
    let user = std::env::var("TAOS_USER").unwrap_or_else(|_| "root".to_string());
    let password = std::env::var("TAOS_PASSWORD").unwrap_or_else(|_| "taosdata".to_string());
    let db_name = std::env::var("TAOS_DATABASE").unwrap_or_else(|_| "test_db".to_string());

    println!("Querying TDengine at {}:{}", host, port);

    let mut driver = adbc_taos::driver::TaosDriver::default();

    let db = driver.new_database_with_opts([
        (OptionDatabase::Uri, format!("taos://{}:{}", host, port).into()),
        (OptionDatabase::Username, user.into()),
        (OptionDatabase::Password, password.into()),
    ])?;
    // db.database = Some(db_name.clone());

    let mut connection = db.new_connection()?;

    // Create a statement
    let mut stmt = connection.new_statement()?;

    // Set SQL query
    let query = format!(
        "SELECT * FROM {}.{} LIMIT 10",
        db_name, "sensor_data"
    );
    stmt.set_sql_query(&query)?;

    // Execute query and get results
    let mut reader = stmt.execute()?;
    let schema = reader.schema();

    println!("\nQuery: {}", query);
    println!("\nSchema:");
    for field in schema.fields() {
        println!("  - {}: {:?}", field.name(), field.data_type());
    }

    println!("\nResults:");
    let mut row_count = 0;

    loop {
        match reader.next() {
            Some(Ok(batch)) => {
                row_count += batch.num_rows();

                println!("  Batch: {} rows", batch.num_rows());

                // Print first 5 rows of each batch
                for _row in 0..batch.num_rows().min(5) {
                    print!("    [");
                    for (col_idx, column) in batch.columns().iter().enumerate() {
                        if col_idx > 0 {
                            print!(", ");
                        }
                        // Simple string representation for display
                        print!("{:?}", column);
                    }
                    println!("]");
                }

                if batch.num_rows() > 5 {
                    println!("    ...");
                }
            }
            Some(Err(e)) => {
                println!("  Error: {}", e);
                break;
            }
            None => break,
        }
    }

    println!("\nTotal rows: {}", row_count);

    Ok(())
}
