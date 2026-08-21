use crate::{IPv4Address, IPv6Address};

/// Either an IPv4 address or an IPv6 address.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum IPAddress {
    /// An IPv4 address.
    V4(IPv4Address),

    /// An IPv6 address.
    V6(IPv6Address),
}

impl From<IPv4Address> for IPAddress {
    fn from(v4: IPv4Address) -> Self {
        IPAddress::V4(v4)
    }
}

impl From<[u8; 4]> for IPAddress {
    fn from(address: [u8; 4]) -> Self {
        Self::from(IPv4Address::from(address))
    }
}

impl From<(u8, u8, u8, u8)> for IPAddress {
    fn from(tuple: (u8, u8, u8, u8)) -> Self {
        Self::from(IPv4Address::from(tuple))
    }
}

impl From<u32> for IPAddress {
    fn from(value: u32) -> Self {
        Self::from(IPv4Address::from(value))
    }
}

impl From<IPv6Address> for IPAddress {
    fn from(v6: IPv6Address) -> Self {
        Self::V6(v6)
    }
}

impl From<[u8; 16]> for IPAddress {
    fn from(address: [u8; 16]) -> Self {
        Self::from(IPv6Address::from(address))
    }
}

impl From<[u16; 8]> for IPAddress {
    fn from(segments: [u16; 8]) -> Self {
        Self::from(IPv6Address::from(segments))
    }
}

impl From<u128> for IPAddress {
    fn from(value: u128) -> Self {
        Self::from(IPv6Address::from(value))
    }
}

impl IPAddress {
    //! Matching

    /// Checks if the address is an IPv4 address.
    pub const fn is_v4(&self) -> bool {
        matches!(self, Self::V4(_))
    }

    /// Checks if the address is an IPv6 address.
    pub const fn is_v6(&self) -> bool {
        matches!(self, Self::V6(_))
    }
}

#[cfg(test)]
mod tests {
    use crate::{IPAddress, IPv4Address, IPv6Address};

    #[test]
    fn construction() {
        let expected: IPAddress = IPAddress::V4(IPv4Address::LOCALHOST);
        let ip: IPAddress = IPv4Address::LOCALHOST.into();
        assert_eq!(ip, expected);
        let ip: IPAddress = [127, 0, 0, 1].into();
        assert_eq!(ip, expected);
        let ip: IPAddress = (127, 0, 0, 1).into();
        assert_eq!(ip, expected);
        let ip: IPAddress = 0x7F000001u32.into();
        assert_eq!(ip, expected);

        let expected: IPAddress = IPAddress::V6(IPv6Address::LOCALHOST);
        let ip: IPAddress = IPv6Address::LOCALHOST.into();
        assert_eq!(ip, expected);
        let ip: IPAddress = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1].into();
        assert_eq!(ip, expected);
        let ip: IPAddress = [0, 0, 0, 0, 0, 0, 0, 1].into();
        assert_eq!(ip, expected);
        let ip: IPAddress = 1u128.into();
        assert_eq!(ip, expected);
    }

    #[test]
    fn matching() {
        let ip: IPAddress = IPAddress::V4(IPv4Address::LOCALHOST);
        assert!(ip.is_v4());
        assert!(!ip.is_v6());

        let ip: IPAddress = IPAddress::V6(IPv6Address::LOCALHOST);
        assert!(!ip.is_v4());
        assert!(ip.is_v6());
    }
}
