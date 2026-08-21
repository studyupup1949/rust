use std::str::FromStr;

/// Extracts the port from the string.
pub fn extract_port(s: &str) -> Result<(&str, u16), ()> {
    if let Some(colon) = s.as_bytes().iter().rposition(|c| *c == b':') {
        let port: u16 = u16::from_str(&s[colon + 1..]).map_err(|_| ())?;
        Ok((&s[..colon], port))
    } else {
        Err(())
    }
}
