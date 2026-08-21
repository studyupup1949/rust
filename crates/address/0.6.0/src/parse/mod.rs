use std::str::FromStr;

pub use authority::*;
pub use domain::*;
pub use endpoint::*;
pub use host::*;
pub use ip::*;
pub use socket::*;

mod authority;
mod domain;
mod endpoint;
mod host;
mod ip;
mod socket;

#[cfg(test)]
mod authority_tests;
#[cfg(test)]
mod domain_tests;
#[cfg(test)]
mod endpoint_tests;
#[cfg(test)]
mod host_tests;
#[cfg(test)]
mod ip_tests;
#[cfg(test)]
mod socket_tests;

/// Parses the port from the end of the string. ({item}:{port}). Returns the {item} string with the port.
pub(in crate::parse) fn parse_port(s: &str) -> Result<(&str, u16), ()> {
    let colon: usize = s.as_bytes().iter().rposition(|c| *c == b':').ok_or(())?;
    let port: u16 = u16::from_str(&s[colon + 1..]).map_err(|_| ())?;
    Ok((&s[..colon], port))
}
