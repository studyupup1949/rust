use crate::ip::IPAddress;

/// Represents an IPv4 address.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct IPv4Address {
    bytes: [u8; 4],
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
    /// assert_eq!(IPv4Address::new(127, 0, 0, 1).bytes(), [127, 0, 0, 1]);
    /// ```
    pub const fn new(a: u8, b: u8, c: u8, d: u8) -> IPv4Address {
        Self{ bytes: [a, b, c, d] }
    }
}

impl IPv4Address {

    /// Gets the address bytes.
    ///
    /// ```
    /// use address::ip::IPv4Address;
    ///
    /// assert_eq!(IPv4Address::LOCALHOST.bytes(), [127, 0, 0, 1]);
    /// ```
    pub const fn bytes(&self) -> [u8; 4] {
        self.bytes
    }

    /// Gets the IP address.
    ///
    /// ```
    /// use address::ip::{IPv4Address, IPAddress};
    ///
    /// assert_eq!(IPv4Address::LOCALHOST.ip(), IPAddress::V4(IPv4Address::LOCALHOST));
    /// ```
    pub const fn ip(&self) -> IPAddress {
        IPAddress::V4(*self)
    }

    /// Checks if the address is the unspecified address. (0.0.0.0)
    ///
    /// ```
    /// use address::ip::IPv4Address;
    ///
    /// assert!(IPv4Address::UNSPECIFIED.is_unspecified());
    /// assert!(!IPv4Address::LOCALHOST.is_unspecified());
    /// ```
    pub fn is_unspecified(&self) -> bool {
        self == &Self::UNSPECIFIED
    }

    /// Checks if the address is a loopback address. (127.0.0.0/8)
    ///
    /// ```
    /// use address::ip::IPv4Address;
    ///
    /// assert!(!IPv4Address::UNSPECIFIED.is_loopback());
    /// assert!(IPv4Address::LOCALHOST.is_loopback());
    /// ```
    pub fn is_loopback(&self) -> bool {
        self.bytes[0] == 127
    }
}

impl ToString for IPv4Address {

    /// ```
    /// use address::ip::IPv4Address;
    ///
    /// assert_eq!(IPv4Address::LOCALHOST.to_string(), "127.0.0.1");
    /// ```
    fn to_string(&self) -> String {
        let a: u8 = self.bytes[0];
        let b: u8 = self.bytes[1];
        let c: u8 = self.bytes[2];
        let d: u8 = self.bytes[3];
        format!("{}.{}.{}.{}", a, b, c, d)
    }
}
