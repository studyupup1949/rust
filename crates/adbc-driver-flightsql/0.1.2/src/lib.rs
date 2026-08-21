//! ADBC FlightSQL Driver for Rust
//!
//! This crate provides the native ADBC FlightSQL driver library path for use with
//! `adbc_driver_manager::ManagedDriver`.
//!
//! # Example
//!
//! ```no_run
//! use adbc_driver_flightsql::DRIVER_PATH;
//! use adbc_driver_manager::ManagedDriver;
//! use adbc_core::options::AdbcVersion;
//!
//! let mut driver = ManagedDriver::load_dynamic_from_filename(
//!     DRIVER_PATH,
//!     None,
//!     AdbcVersion::default(),
//! ).expect("Failed to load driver");
//! ```

/// Path to the native ADBC FlightSQL driver library.
///
/// This path is determined at build time and points to the downloaded
/// FlightSQL driver shared library for your platform.
pub const DRIVER_PATH: &str = env!("ADBC_FLIGHTSQL_LIB_PATH");

/// Version of the native FlightSQL driver that was bundled at build time.
pub const DRIVER_VERSION: &str = env!("ADBC_FLIGHTSQL_LIB_VERSION");
