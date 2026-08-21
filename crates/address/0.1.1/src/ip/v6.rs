use crate::ip::IPAddress;

/// Represents an IPv6 address.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct IPv6Address {
    chunks: [u16; 8],
}

impl IPv6Address {

    /// The unspecified address.
    pub const UNSPECIFIED: Self = Self::new(0, 0, 0, 0, 0, 0, 0, 0);

    /// The localhost address.
    pub const LOCALHOST: Self = Self::new(0, 0, 0, 0, 0,0 , 0, 1);
}

impl IPv6Address {

    /// Creates a new IPv6Address.
    ///
    /// ```
    /// use address::ip::IPv6Address;
    ///
    /// let ip: IPv6Address = IPv6Address::new(1, 2, 3, 4, 5, 6, 7, 8);
    /// assert_eq!(ip.chunks(), [1, 2, 3, 4, 5, 6, 7, 8]);
    /// ```
    pub const fn new(
            a: u16, b: u16, c: u16, d: u16,
            e: u16, f: u16, g: u16, h: u16)
            -> IPv6Address {
        IPv6Address{ chunks: [a, b, c, d, e, f, g, h] }
    }
}

impl IPv6Address {

    /// Gets the address chunks.
    ///
    /// ```
    /// use address::ip::IPv6Address;
    ///
    /// assert_eq!(IPv6Address::LOCALHOST.chunks(), [0, 0, 0, 0, 0, 0, 0, 1]);
    /// ```
    pub fn chunks(&self) -> [u16; 8] {
        self.chunks
    }

    /// Gets the IP address.
    ///
    /// ```
    /// use address::ip::{IPv6Address, IPAddress};
    ///
    /// assert_eq!(IPv6Address::LOCALHOST.ip(), IPAddress::V6(IPv6Address::LOCALHOST));
    /// ```
    pub const fn ip(&self) -> IPAddress {
        IPAddress::V6(*self)
    }

    /// Checks if the address is the unspecified address. (::)
    ///
    /// ```
    /// use address::ip::IPv6Address;
    ///
    /// assert!(IPv6Address::UNSPECIFIED.is_unspecified());
    /// assert!(!IPv6Address::LOCALHOST.is_unspecified());
    /// ```
    pub fn is_unspecified(&self) -> bool {
        self == &Self::UNSPECIFIED
    }

    /// Checks if the address is the loopback address. (::1)
    ///
    /// ```
    /// use address::ip::IPv6Address;
    ///
    /// assert!(!IPv6Address::UNSPECIFIED.is_loopback());
    /// assert!(IPv6Address::LOCALHOST.is_loopback());
    /// ```
    pub fn is_loopback(&self) -> bool {
        self == &Self::LOCALHOST
    }
}

impl ToString for IPv6Address {

    /// ```
    /// use address::ip::IPv6Address;
    ///
    /// assert_eq!(IPv6Address::UNSPECIFIED.to_string(), "::");
    /// assert_eq!(IPv6Address::LOCALHOST.to_string(), "::1");
    /// ```
    fn to_string(&self) -> String {
        use std::net::Ipv6Addr;
        Ipv6Addr::from(self.chunks).to_string()
    }
}
