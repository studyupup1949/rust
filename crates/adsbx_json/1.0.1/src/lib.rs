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
//! # let json_str = include_str!("../tests/v2-specimen-01.json");
//! let response = Response::from_str(&json_str).unwrap();
//! println!("Got {} aircraft", response.aircraft.len());
//! println!("1st aircraft:\n{:#?}", response.aircraft[0]);
//! ```

pub mod v2;
