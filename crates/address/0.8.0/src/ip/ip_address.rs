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
        Self::V4(v4)
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
    //! Properties

    /// Checks if the address is an IPv4 address.
    pub const fn is_v4(&self) -> bool {
        return matches!(self, Self::V4(_));
    }

    /// Checks if the address is an IPv6 address.
    pub const fn is_v6(&self) -> bool {
        return matches!(self, Self::V6(_));
    }
}

#[cfg(test)]
mod tests {
    use crate::{IPAddress, IPv4Address, IPv6Address};

    #[test]
    fn construction_v4() {
        let address: [u8; 4] = [1, 2, 3, 4];
        let tuple: (u8, u8, u8, u8) = (1, 2, 3, 4);
        let value: u32 = 0x01020304u32;
        let v4: IPv4Address = IPv4Address::new(address);
        let ip: IPAddress = IPAddress::V4(IPv4Address::new(address));

        assert_eq!(ip, address.into());
        assert_eq!(ip, tuple.into());
        assert_eq!(ip, value.into());
        assert_eq!(ip, v4.into());
    }

    #[test]
    fn construction_v6() {
        let address: [u8; 16] = [
            0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB,
            0xCD, 0xEF,
        ];
        let segments: [u16; 8] = [
            0x0123, 0x4567, 0x89AB, 0xCDEF, 0x0123, 0x4567, 0x89AB, 0xCDEF,
        ];
        let value: u128 = 0x0123456789ABCDEF0123456789ABCDEFu128;
        let v6: IPv6Address = IPv6Address::new(address);
        let ip: IPAddress = IPAddress::V6(IPv6Address::new(address));

        assert_eq!(ip, address.into());
        assert_eq!(ip, segments.into());
        assert_eq!(ip, value.into());
        assert_eq!(ip, v6.into());
    }

    #[test]
    fn properties() {
        let ip: IPAddress = IPv4Address::LOCALHOST.into();
        assert!(ip.is_v4());
        assert!(!ip.is_v6());

        let ip: IPAddress = IPv6Address::LOCALHOST.into();
        assert!(!ip.is_v4());
        assert!(ip.is_v6());
    }
}
