pub use v4::*;
pub use v6::*;

mod v4;
mod v6;

/// Represents either an IPv4 address or an IPv6 address.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum IPAddress {

    /// An IPv4 Address
    V4(IPv4Address),

    /// An IPv6 Address
    V6(IPv6Address),
}

impl IPAddress {

    /// Checks if the address is an unspecified address.
    ///
    /// ```
    /// use address::ip::{IPv4Address, IPv6Address};
    ///
    /// assert!(IPv4Address::UNSPECIFIED.ip().is_unspecified());
    /// assert!(IPv6Address::UNSPECIFIED.ip().is_unspecified());
    ///
    /// assert!(!IPv4Address::LOCALHOST.ip().is_unspecified());
    /// assert!(!IPv6Address::LOCALHOST.ip().is_unspecified());
    /// ```
    pub fn is_unspecified(&self) -> bool {
        match self {
            IPAddress::V4(ip) => ip.is_unspecified(),
            IPAddress::V6(ip) => ip.is_unspecified(),
        }
    }

    /// Checks if the address is a loopback address.
    ///
    /// ```
    /// use address::ip::{IPv4Address, IPv6Address};
    ///
    /// assert!(!IPv4Address::UNSPECIFIED.ip().is_loopback());
    /// assert!(!IPv6Address::UNSPECIFIED.ip().is_loopback());
    ///
    /// assert!(IPv4Address::LOCALHOST.ip().is_loopback());
    /// assert!(IPv6Address::LOCALHOST.ip().is_loopback());
    /// ```
    pub fn is_loopback(&self) -> bool {
        match self {
            IPAddress::V4(ip) => ip.is_loopback(),
            IPAddress::V6(ip) => ip.is_loopback(),
        }
    }
}

impl ToString for IPAddress {

    /// ```
    /// use address::ip::{IPv4Address, IPv6Address};
    ///
    /// assert_eq!(IPv4Address::LOCALHOST.ip().to_string(), "127.0.0.1");
    /// assert_eq!(IPv6Address::LOCALHOST.ip().to_string(), "::1");
    /// ```
    fn to_string(&self) -> String {
        match self {
            IPAddress::V4(ip) => ip.to_string(),
            IPAddress::V6(ip) => ip.to_string(),
        }
    }
}
