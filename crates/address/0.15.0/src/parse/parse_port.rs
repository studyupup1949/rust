use std::str::FromStr;

use crate::ParseError;
use crate::ParseError::InvalidPort;

/// Parses the port from the `address` string.
///
/// Returns `(address_without_last_colon, port)`.
///
/// # Examples
/// localhost:80    -> `Ok("localhost", 80)`
/// :80             -> `Ok("", 80)`
/// :8x             -> `Err(InvalidPort)`
/// 80              -> `Err(InvalidPort)`
pub(crate) fn parse_port(address: &str) -> Result<(&str, u16), ParseError> {
    if let Some(colon) = address.as_bytes().iter().rposition(|c| *c == b':') {
        let port: u16 = u16::from_str(&address[colon + 1..]).map_err(|_| InvalidPort)?;
        let s: &str = &address[..colon];
        Ok((s, port))
    } else {
        Err(InvalidPort)
    }
}
