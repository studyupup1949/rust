//! Serde support for the address types.
//!
//! # Wire Contract
//!
//! - Types that can contain a domain name (`Domain`, `Host`, `Authority`, `Endpoint`, and their reference types)
//!   serialize as their `Display` string in every format.
//! - The purely numeric types serialize as their `Display` string in human-readable formats and as compact binary
//!   values in other formats: byte arrays for `IPv4Address` and `IPv6Address`, a byte string of 4 or 16 bytes for
//!   `IPAddress`, and an `(ip, port)` tuple for the socket address types.
//! - The reference types deserialize by borrowing from the input, so the input must outlive the value, domain names
//!   must already be lowercase, and escaped input is an error. Use the owned types to deserialize mixed-case or
//!   escaped input.

pub(crate) use from_str_visitor::*;
pub(crate) use try_from_str_visitor::*;

mod from_str_visitor;
mod try_from_str_visitor;

mod impl_serde_string;
mod impl_serde_string_or_binary;
mod ip_address;
