use std::fmt::{Display, Error, Formatter};

use crate::ip::IPAddress;

/// Represents an IPv6 address.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct IPv6Address {
    segments: [u16; 8],
}

impl IPv6Address {

    /// The unspecified address.
    pub const UNSPECIFIED: Self = Self::new(0, 0, 0, 0, 0, 0, 0, 0);

    /// The localhost address.
    pub const LOCALHOST: Self = Self::new(0, 0, 0, 0, 0, 0, 0, 1);
}

impl IPv6Address {

    /// Creates a new IPv6Address.
    ///
    /// ```
    /// use address::ip::IPv6Address;
    ///
    /// assert_eq!(IPv6Address::new(1, 2, 3, 4, 5, 6, 7, 8).segments(), [1, 2, 3, 4, 5, 6, 7, 8]);
    /// ```
    pub const fn new(a: u16, b: u16, c: u16, d: u16, e: u16, f: u16, g: u16, h: u16) -> IPv6Address {
        IPv6Address{ segments: [a, b, c, d, e, f, g, h] }
    }
}

impl IPv6Address {

    /// Gets the segments.
    pub fn segments(&self) -> [u16; 8] {
        self.segments
    }

    /// Gets the IPAddress.
    pub fn ip(&self) -> IPAddress {
        IPAddress::V6(*self)
    }
}

impl Display for IPv6Address {

    /// ```
    /// use address::ip::IPv6Address;
    ///
    /// assert_eq!(IPv6Address::UNSPECIFIED.to_string(), "::");
    /// assert_eq!(IPv6Address::LOCALHOST.to_string(), "::1");
    /// ```
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", std::net::Ipv6Addr::from(self.segments))
    }
}
