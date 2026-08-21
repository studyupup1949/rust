use std::fmt::{Display, Error, Formatter};

use crate::ip::IPAddress;

/// Represents an IPv4 address.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct IPv4Address {
    address: [u8; 4],
}

impl IPv4Address {

    /// The unspecified address.
    pub const UNSPECIFIED: Self = Self::new(0, 0, 0, 0);

    /// The localhost address.
    pub const LOCALHOST: Self = Self::new(127, 0, 0, 1);

    /// The broadcast address.
    pub const BROADCAST: Self = Self::new(255, 255, 255, 255);
}

impl IPv4Address {

    /// Creates a new IPv4Address.
    ///
    /// ```
    /// use address::ip::IPv4Address;
    ///
    /// assert_eq!(IPv4Address::new(127, 0, 0, 1).address(), [127, 0, 0, 1]);
    /// assert_eq!(IPv4Address::new(255, 255, 255, 255).address(), [255, 255, 255, 255]);
    /// ```
    pub const fn new(a: u8, b: u8, c: u8, d: u8) -> IPv4Address {
        IPv4Address{ address: [a, b, c, d] }
    }
}

impl IPv4Address {

    /// Gets the address.
    pub fn address(&self) -> [u8; 4] {
        self.address
    }

    /// Gets the IPAddress.
    pub fn ip(&self) -> IPAddress {
        IPAddress::V4(*self)
    }
}

impl Display for IPv4Address {

    /// ```
    /// use address::ip::IPv4Address;
    ///
    /// assert_eq!(IPv4Address::UNSPECIFIED.to_string(), "0.0.0.0");
    /// assert_eq!(IPv4Address::LOCALHOST.to_string(), "127.0.0.1");
    /// assert_eq!(IPv4Address::BROADCAST.to_string(), "255.255.255.255");
    /// ```
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let a: u8 = self.address[0];
        let b: u8 = self.address[1];
        let c: u8 = self.address[2];
        let d: u8 = self.address[3];
        write!(f, "{}.{}.{}.{}", a, b, c, d)
    }
}
