use std::str::FromStr;

use crate::ParseError;
use crate::ParseError::InvalidPort;

/// Parses the port from address string.
/// - Returns `(string_without_last_colon, port)`.
pub(crate) fn parse_port(s: &str) -> Result<(&str, u16), ParseError> {
    if let Some(colon) = s.as_bytes().iter().rposition(|c| *c == b':') {
        let port: u16 = u16::from_str(&s[colon + 1..]).map_err(|_| InvalidPort)?;
        let s: &str = &s[..colon];
        Ok((s, port))
    } else {
        Err(InvalidPort)
    }
}
