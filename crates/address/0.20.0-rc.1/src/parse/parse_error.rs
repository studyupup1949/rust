use std::fmt::{Display, Formatter};

/// An error parsing an address.
///
/// Errors name the most specific part that can be blamed. When the expected address version is declared by the type
/// (`SocketAddressV4`) or by the syntax (brackets imply IPv6), the specific IP error is returned. When the version is
/// ambiguous (an unbracketed `SocketAddress`), the aggregate error is returned.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[non_exhaustive]
pub enum ParseError {
    /// The domain is invalid.
    InvalidDomain,

    /// The IP address is invalid. (neither IPv4 nor IPv6)
    InvalidIPAddress,

    /// The IPv4 address is invalid.
    InvalidIPv4Address,

    /// The IPv6 address is invalid.
    InvalidIPv6Address,

    /// The socket address is invalid. (neither IPv4 nor bracketed IPv6)
    InvalidSocketAddress,

    /// The IPv6 socket address is invalid. (the IP address must be bracketed)
    InvalidSocketAddressV6,

    /// The port is missing or invalid.
    InvalidPort,

    /// The host is invalid. (neither an IP address nor a domain)
    InvalidHost,

    /// The authority is invalid. (an IPv6 host must be bracketed)
    InvalidAuthority,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s: &str = match self {
            Self::InvalidDomain => "invalid domain",
            Self::InvalidIPAddress => "invalid IP address",
            Self::InvalidIPv4Address => "invalid IPv4 address",
            Self::InvalidIPv6Address => "invalid IPv6 address",
            Self::InvalidSocketAddress => "invalid socket address",
            Self::InvalidSocketAddressV6 => "invalid IPv6 socket address",
            Self::InvalidPort => "invalid port",
            Self::InvalidHost => "invalid host",
            Self::InvalidAuthority => "invalid authority",
        };
        f.pad(s)
    }
}

impl std::error::Error for ParseError {}
