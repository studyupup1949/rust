//! Basic connection example for ADBC-Taos driver.
//!
//! This example demonstrates how to establish a connection to TDengine
//! using the ADBC API.

use adbc_core::{Database, Driver, options::OptionDatabase};

fn main() -> anyhow::Result<()> {
    // Load environment variables from .env file if present
    dotenvy::dotenv().ok();

    // Get connection parameters from environment or use defaults
    let host = std::env::var("TAOS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("TAOS_PORT").unwrap_or_else(|_| "6030".to_string());
    let user = std::env::var("TAOS_USER").unwrap_or_else(|_| "root".to_string());
    let password = std::env::var("TAOS_PASSWORD").unwrap_or_else(|_| "taosdata".to_string());
    // let db_name = std::env::var("TAOS_DATABASE").ok();

    println!("Connecting to TDengine at {}:{}", host, port);

    // Create driver instance
    let mut driver = adbc_taos::driver::TaosDriver::default();

    // Create database with connection options
    let database = driver.new_database_with_opts([
        (OptionDatabase::Uri, format!("taos://{}:{}", host, port).into()),
        (OptionDatabase::Username, user.into()),
        (OptionDatabase::Password, password.into()),
    ])?;

    // Establish connection
    let connection = database.new_connection()?;

    println!("Connected successfully!");
    println!("Server version: {}", connection.server_version());

    Ok(())
}
