use std::fmt::{Display, Formatter};

/// An error parsing an address.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum ParseError {
    InvalidDomain,
    InvalidIPv4Address,
    InvalidIPv6Address,
    InvalidIPAddress,
    InvalidSocketAddressV4,
    InvalidSocketAddressV6,
    InvalidSocketAddress,
    InvalidPort,
    InvalidHost,
    InvalidAuthority,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ParseError {}
