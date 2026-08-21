//! Metadata retrieval example for ADBC-Taos driver.
//!
//! This example demonstrates how to retrieve driver and server metadata
//! including version information, table types, and table schemas.

use adbc_core::{Connection, Database, Driver, options::OptionDatabase};

fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let host = std::env::var("TAOS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("TAOS_PORT").unwrap_or_else(|_| "6030".to_string());
    let user = std::env::var("TAOS_USER").unwrap_or_else(|_| "root".to_string());
    let password = std::env::var("TAOS_PASSWORD").unwrap_or_else(|_| "taosdata".to_string());
    let db_name = std::env::var("TAOS_DATABASE").unwrap_or_else(|_| "log".to_string());

    println!("Retrieving metadata from TDengine at {}:{}", host, port);

    let mut driver = adbc_taos::driver::TaosDriver::default();

    let db = driver.new_database_with_opts([
        (OptionDatabase::Uri, format!("taos://{}:{}", host, port).into()),
        (OptionDatabase::Username, user.into()),
        (OptionDatabase::Password, password.into()),
    ])?;
    // db.database = Some(db_name.clone());

    let connection = db.new_connection()?;

    // Get driver and server info
    println!("\n=== Driver and Server Info ===");
    let mut info_reader = connection.get_info(None)?;
    println!("\nInfo Code | Info Value");
    println!("----------|------------");

    while let Some(batch) = info_reader.next() {
        let batch = batch?;
        for row in 0..batch.num_rows() {
            let code_col = batch.column(0);
            let value_col = batch.column(1);

            if let Some(code) = format_column(code_col, row) {
                if let Some(value) = format_column(value_col, row) {
                    println!("{:>9} | {}", code, value);
                }
            }
        }
    }

    // Get table types
    println!("\n=== Supported Table Types ===");
    let mut types_reader = connection.get_table_types()?;
    println!("\nTable Type");
    println!("----------");

    while let Some(batch) = types_reader.next() {
        let batch = batch?;
        for row in 0..batch.num_rows() {
            let col = batch.column(0);
            if let Some(value) = format_column(col, row) {
                println!("{}", value);
            }
        }
    }

    // Get table schema
    println!("\n=== Table Schema Example ===");
    match connection.get_table_schema(None, Some(&db_name), "logs") {
        Ok(schema) => {
            println!("\nTable: {}.{}", db_name, "logs");
            println!("\nColumn Name       | Data Type              | Nullable");
            println!("------------------|------------------------|---------");
            for field in schema.fields() {
                println!("{:>18} | {:22} | {}",
                    field.name(),
                    format!("{:?}", field.data_type()),
                    if field.is_nullable() { "YES" } else { "NO" }
                );
            }
        }
        Err(e) => {
            println!("Could not retrieve table schema: {}", e);
            println!("(Table 'logs' may not exist in database '{}')", db_name);
        }
    }

    // Get statistic names
    println!("\n=== Supported Statistics ===");
    let mut stats_reader = connection.get_statistic_names()?;
    let mut has_stats = false;

    while let Some(batch) = stats_reader.next() {
        let batch = batch?;
        if batch.num_rows() > 0 {
            has_stats = true;
            println!("\nStatistic Name    | Description");
            println!("------------------|------------");
            for row in 0..batch.num_rows() {
                let name_col = batch.column(0);
                let desc_col = batch.column(1);

                if let Some(name) = format_column(name_col, row) {
                    let desc = format_column(desc_col, row).unwrap_or_else(|| "".to_string());
                    println!("{:>18} | {}", name, desc);
                }
            }
        }
    }

    if !has_stats {
        println!("(No named statistics available)");
    }

    Ok(())
}

/// Format Arrow column value as string.
fn format_column(column: &dyn arrow_array::Array, row: usize) -> Option<String> {
    use arrow_array::{StringArray, PrimitiveArray};

    if column.is_null(row) {
        return Some("NULL".to_string());
    }

    let as_any = column.as_any();

    if let Some(arr) = as_any.downcast_ref::<StringArray>() {
        let val: &str = arr.value(row);
        Some(val.to_string())
    } else if let Some(arr) = as_any.downcast_ref::<PrimitiveArray<arrow_array::types::UInt32Type>>() {
        Some(arr.value(row).to_string())
    } else if as_any.is::<arrow_array::BinaryArray>() {
        Some("[binary data]".to_string())
    } else if as_any.is::<arrow_array::BooleanArray>() {
        Some("[boolean]".to_string())
    } else {
        Some("[value]".to_string())
    }
}
