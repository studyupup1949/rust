/// An IPv6 address. (a:b:c:d:e:f:g:h)
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default)]
pub struct IPv6Address {
    address: [u8; 16],
}

impl IPv6Address {
    //! Special Addresses

    /// The unspecified address. (::)
    pub const UNSPECIFIED: Self = Self::from_segments([0, 0, 0, 0, 0, 0, 0, 0]);

    /// The localhost address. (::1)
    pub const LOCALHOST: Self = Self::from_segments([0, 0, 0, 0, 0, 0, 0, 1]);
}

impl IPv6Address {
    //! Construction

    /// Creates a new IPv6 address. [a-high, a-low, b-high, b-low, ..., h-high, h-low]
    pub const fn new(address: [u8; 16]) -> Self {
        Self { address }
    }

    /// Creates an IPv6 address from the `segments`. [a, b, c, d, e, f, g, h]
    pub const fn from_segments(segments: [u16; 8]) -> Self {
        Self {
            address: [
                (segments[0] >> 8) as u8,
                segments[0] as u8,
                (segments[1] >> 8) as u8,
                segments[1] as u8,
                (segments[2] >> 8) as u8,
                segments[2] as u8,
                (segments[3] >> 8) as u8,
                segments[3] as u8,
                (segments[4] >> 8) as u8,
                segments[4] as u8,
                (segments[5] >> 8) as u8,
                segments[5] as u8,
                (segments[6] >> 8) as u8,
                segments[6] as u8,
                (segments[7] >> 8) as u8,
                segments[7] as u8,
            ],
        }
    }
}

impl From<[u8; 16]> for IPv6Address {
    fn from(address: [u8; 16]) -> Self {
        Self { address }
    }
}

impl From<IPv6Address> for [u8; 16] {
    fn from(ip: IPv6Address) -> Self {
        ip.address
    }
}

impl From<[u16; 8]> for IPv6Address {
    fn from(segments: [u16; 8]) -> Self {
        Self::from_segments(segments)
    }
}

impl From<IPv6Address> for [u16; 8] {
    fn from(ip: IPv6Address) -> Self {
        ip.segments()
    }
}

impl From<u128> for IPv6Address {
    fn from(value: u128) -> Self {
        Self::new(value.to_be_bytes())
    }
}

impl From<IPv6Address> for u128 {
    fn from(ip: IPv6Address) -> Self {
        u128::from_be_bytes(ip.address)
    }
}

impl IPv6Address {
    //! Properties

    /// Gets the address. [a-high, a-low, b-high, b-low, ..., h-high, h-low]
    pub const fn address(&self) -> [u8; 16] {
        self.address
    }

    /// Gets the segments. [a, b, c, d, e, f, g, h]
    pub const fn segments(&self) -> [u16; 8] {
        [
            ((self.address[0] as u16) << 8) | (self.address[1] as u16),
            ((self.address[2] as u16) << 8) | (self.address[3] as u16),
            ((self.address[4] as u16) << 8) | (self.address[5] as u16),
            ((self.address[6] as u16) << 8) | (self.address[7] as u16),
            ((self.address[8] as u16) << 8) | (self.address[9] as u16),
            ((self.address[10] as u16) << 8) | (self.address[11] as u16),
            ((self.address[12] as u16) << 8) | (self.address[13] as u16),
            ((self.address[14] as u16) << 8) | (self.address[15] as u16),
        ]
    }
}

#[cfg(test)]
mod tests {
    use crate::IPv6Address;

    #[test]
    fn default() {
        assert_eq!(IPv6Address::default(), IPv6Address::UNSPECIFIED);
    }

    #[test]
    fn specials() {
        assert_eq!(
            IPv6Address::UNSPECIFIED.address,
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(
            IPv6Address::LOCALHOST.address,
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]
        );
    }

    #[test]
    fn construction() {
        let address: [u8; 16] = [
            0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB,
            0xCD, 0xEF,
        ];
        let segments: [u16; 8] = [
            0x0123, 0x4567, 0x89AB, 0xCDEF, 0x0123, 0x4567, 0x89AB, 0xCDEF,
        ];
        let value: u128 = 0x0123456789ABCDEF0123456789ABCDEFu128;

        let ip: IPv6Address = IPv6Address::new(address);
        assert_eq!(ip.address, address);

        assert_eq!(ip, IPv6Address::from_segments(segments));
        assert_eq!(ip, IPv6Address::from(address));
        assert_eq!(ip, IPv6Address::from(segments));
        assert_eq!(ip, IPv6Address::from(value));

        let result: [u8; 16] = ip.into();
        assert_eq!(result, address);
        let result: [u16; 8] = ip.into();
        assert_eq!(result, segments);
        let result: u128 = ip.into();
        assert_eq!(result, value);
    }

    #[test]
    fn properties() {
        let address: [u8; 16] = [
            0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB,
            0xCD, 0xEF,
        ];
        let segments: [u16; 8] = [
            0x0123, 0x4567, 0x89AB, 0xCDEF, 0x0123, 0x4567, 0x89AB, 0xCDEF,
        ];

        let ip: IPv6Address = IPv6Address::new(address);
        assert_eq!(ip.address(), address);
        assert_eq!(ip.segments(), segments);
    }
}
