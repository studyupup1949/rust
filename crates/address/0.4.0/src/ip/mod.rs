use std::fmt::{Display, Error, Formatter};

pub use v4::*;
pub use v6::*;
use crate::host::Host;

mod v4;
mod v6;

/// Represents an IP address.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum IPAddress {

    /// An IPv4 Address
    V4(IPv4Address),

    /// An IPv6 Address
    V6(IPv6Address),
}

impl IPAddress {

    /// Gets the host.
    pub fn host(&self) -> Host {
        Host::Address(*self)
    }
}

impl IPAddress {

    /// Checks if the address is an IPv4 address.
    pub fn is_v4(&self) -> bool {
        match self {
            IPAddress::V4(_) => true,
            IPAddress::V6(_) => false,
        }
    }

    /// Checks if the address is an IPv6 address.
    pub fn is_v6(&self) -> bool {
        match self {
            IPAddress::V4(_) => false,
            IPAddress::V6(_) => true,
        }
    }
}

impl Display for IPAddress {

    /// ```
    /// use address::ip::{IPv4Address, IPv6Address};
    ///
    /// assert_eq!(IPv4Address::LOCALHOST.ip().to_string(), "127.0.0.1");
    /// assert_eq!(IPv6Address::LOCALHOST.ip().to_string(), "::1");
    /// ```
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            IPAddress::V4(ip) => ip.fmt(f),
            IPAddress::V6(ip) => ip.fmt(f),
        }
    }
}
