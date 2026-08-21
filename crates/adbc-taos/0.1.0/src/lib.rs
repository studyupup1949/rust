//! ADBC-Taos: Arrow Database Connectivity driver for TDengine.
//!
//! This crate provides an ADBC (Arrow Database Connectivity) driver for TDengine,
//! a high-performance time-series database. ADBC provides a standard interface
//! for database access using Apache Arrow format, eliminating unnecessary data
//! copies.
//!
//! # Features
//!
//! - Full ADBC Core API implementation
//! - TDengine 3.3.0+ support
//! - Connection pooling via `deadpool`
//! - Prepared statements with parameter binding
//! - Supertable metadata introspection
//! - All TDengine data types (JSON, GEOMETRY, DECIMAL, BLOB)
//!
//! # Quick Start
//!
//! ```ignore
//! use adbc_core::{Database, Connection, Statement, Driver};
//! use adbc_core::options::{OptionDatabase, OptionValue};
//!
//! let mut driver = adbc_taos::TaosDriver::default();
//! let mut db = driver.new_database()?;
//! db.set_option(
//!     OptionDatabase::Uri,
//!     OptionValue::String("taos://root:taosdata@127.0.0.1:6030".to_string())
//! )?;
//!
//! let conn = db.new_connection()?;
//! let mut stmt = conn.new_statement()?;
//! stmt.set_sql_query("SELECT * FROM my_table")?;
//! let reader = stmt.execute()?;
//!
//! for batch in reader {
//!     let batch = batch?;
//!     println!("Got {} rows", batch.num_rows());
//! }
//! ```
//!
//! # Connection Pooling
//!
//! ```ignore
//! use adbc_taos::TaosDatabase;
//!
//! let mut db = TaosDatabase::default();
//! db.uri = "taos://root:taosdata@127.0.0.1:6030".to_string();
//! db.set_pool_size(10);
//!
//! let conn1 = db.create_connection()?;
//! let conn2 = db.create_connection()?;
//! ```
//!
//! # Module Structure
//!
//! - [`driver`]: Entry point for creating database instances
//! - [`database`]: Connection configuration and database creation
//! - [`connection`]: Active database connection for queries
//! - [`statement`]: SQL statement execution with binding
//! - [`reader`]: Arrow RecordBatch streaming
//! - [`pool`]: Connection pooling utilities
//! - [`error`]: Error types

pub mod connection;
pub mod database;
pub mod driver;
pub mod error;
pub mod pool;
pub mod reader;
pub mod statement;
pub mod types;
pub mod utils;

// Re-export main types for convenience
pub use connection::TaosConnection;
pub use database::TaosDatabase;
pub use driver::TaosDriver;
pub use error::TaosError;
pub use pool::{TaosPool, TaosPoolConfig, PooledTaosConnection};
pub use utils::{Runtime, block_on_on_runtime};

