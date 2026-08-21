//! Integration tests for adbc-driver-flightsql
//!
//! These tests require a running FlightSQL server on localhost:4214

use adbc_core::{
    options::{AdbcVersion, OptionDatabase, OptionValue},
    Connection, Database, Driver, Statement,
};
use adbc_driver_flightsql::DRIVER_PATH;
use adbc_driver_manager::ManagedDriver;

// https://arrow.apache.org/adbc/current/rust/quickstart.html
#[test]
fn test_load_driver() -> Result<(), Box<dyn std::error::Error>> {
    // Load the FlightSQL driver using the bundled library path
    let mut driver = ManagedDriver::load_dynamic_from_filename(
        DRIVER_PATH, // Just use the const - no need to hardcode paths!
        None,
        AdbcVersion::default(),
    )?;

    println!("Successfully loaded FlightSQL driver from: {}", DRIVER_PATH);
    let database = driver.new_database_with_opts([(
        OptionDatabase::Uri,
        OptionValue::from("grpc://localhost:4214"),
    )])?;
    let mut conn = database.new_connection()?;
    println!("Created connection");
    let mut stmt = conn.new_statement()?;
    stmt.set_sql_query("SELECT 1")?;
    let reader = stmt.execute()?;
    for batch in reader {
        let batch = batch.expect("Failed to read batch");
        println!("{:?}", batch);
    }
    println!("3");
    // NOTE: if we don't exit, process may hang due to gRPC threads
    std::process::exit(0);
}
