//! Parses the JSON returned by the [ADS-B Exchange
//! API](https://www.adsbexchange.com/data/).
//!
//! This crate currently only supports v2 of the API.
//!
//! To parse a v2 JSON response:
//!
//! ```
//! use adsbx_json::v2::Response;
//! use std::str::FromStr;
//!
//! # let json_str = include_str!("../tests/v2-specimen-nearby.json");
//! let response = Response::from_str(&json_str).unwrap();
//! println!("Got {} aircraft", response.aircraft.len());
//! let ac = &response.aircraft[0];
//! println!("ICAO: {}", ac.hex);
//! if let Some(reg) = &ac.registration {
//!     println!("Registration: {}", reg);
//! }
//! if let (Some(lat), Some(lon)) = (ac.lat, ac.lon) {
//!     println!("Aircraft is at {}, {}", lat, lon);
//! }
//! ```

use std::{error::Error, fmt, num::ParseIntError};

pub mod v2;

/// Represents a parsing error.
#[derive(Debug, Clone)]
pub struct ParseError(String);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ParseError {}

impl From<serde_json::Error> for ParseError
where
    serde_json::Error: std::fmt::Debug,
{
    fn from(error: serde_json::Error) -> Self {
        ParseError(format!("{:?}", error))
    }
}

impl From<ParseIntError> for ParseError
where
    serde_json::Error: std::fmt::Debug,
{
    fn from(error: ParseIntError) -> Self {
        ParseError(format!("{:?}", error))
    }
}
