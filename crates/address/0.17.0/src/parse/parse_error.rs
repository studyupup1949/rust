use std::fmt::{Display, Formatter};

/// An error parsing an address.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum ParseError {
    InvalidDomain,
    InvalidIPAddress,
    InvalidIPv4Address,
    InvalidIPv6Address,
    InvalidSocketAddressV6,
    InvalidPort,
    InvalidHost,
    InvalidAuthority,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s: &str = match self {
            Self::InvalidDomain => "invalid domain",
            Self::InvalidIPAddress => "invalid IP address",
            Self::InvalidIPv4Address => "invalid IPv4 address",
            Self::InvalidIPv6Address => "invalid IPv6 address",
            Self::InvalidSocketAddressV6 => "invalid IPv6 socket address",
            Self::InvalidPort => "invalid port",
            Self::InvalidHost => "invalid host",
            Self::InvalidAuthority => "invalid authority",
        };
        write!(f, "{}", s)
    }
}

impl std::error::Error for ParseError {}
