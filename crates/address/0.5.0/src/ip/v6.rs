use std::fmt::{Debug, Display, Error, Formatter};
use std::net::Ipv6Addr;

use crate::{Authority, Host, IPAddress, IPv4Address, SocketAddress};

/// Represents an IPv6 address. (a:b:c:d:e:f:g:h)
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct IPv6Address {
    /// The address. [a, b, c, d, e, f, g, h]
    address: [u8; 16],
}

impl IPv6Address {
    //! Special Addresses

    /// The unspecified address. (::)
    pub const UNSPECIFIED: Self = Self::new(0, 0, 0, 0, 0, 0, 0, 0);

    /// The localhost address. (::1)
    pub const LOCALHOST: Self = Self::new(0, 0, 0, 0, 0, 0, 0, 1);
}

impl IPv6Address {
    //! Constructors

    /// Creates a new IPv6 address from the segments. (a:b:c:d:e:f:g:h)
    pub const fn new(a: u16, b: u16, c: u16, d: u16, e: u16, f: u16, g: u16, h: u16) -> Self {
        Self {
            address: [
                (a >> 8) as u8,
                a as u8,
                (b >> 8) as u8,
                b as u8,
                (c >> 8) as u8,
                c as u8,
                (d >> 8) as u8,
                d as u8,
                (e >> 8) as u8,
                e as u8,
                (f >> 8) as u8,
                f as u8,
                (g >> 8) as u8,
                g as u8,
                (h >> 8) as u8,
                h as u8,
            ],
        }
    }
}

impl IPv6Address {
    //! Conversions

    /// Converts the address to an IP address.
    pub const fn to_ip(&self) -> IPAddress {
        IPAddress::V6(*self)
    }

    /// Converts the address to an optional IPv4 address. Returns None when the IPv6 address is not
    /// an IPv4 compatible or IPv4 mapped address.
    pub fn to_v4(&self) -> Option<IPv4Address> {
        match self.address {
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, a, b, c, d] => Some(IPv4Address::new(a, b, c, d)),
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xFF, 0xFF, a, b, c, d] => {
                Some(IPv4Address::new(a, b, c, d))
            }
            _ => None,
        }
    }

    /// Converts the address to a host.
    pub const fn to_host(&self) -> Host {
        Host::Address(self.to_ip())
    }

    /// Converts the address to a socket address.
    pub const fn to_socket(&self, port: u16) -> SocketAddress {
        SocketAddress::new(self.to_ip(), port)
    }

    /// Converts the address to an authority with the port.
    pub const fn to_authority(&self, port: u16) -> Authority {
        Authority::Address(self.to_socket(port))
    }
}

impl From<[u8; 16]> for IPv6Address {
    fn from(address: [u8; 16]) -> Self {
        Self { address }
    }
}

impl From<[u16; 8]> for IPv6Address {
    fn from(segments: [u16; 8]) -> Self {
        Self::new(
            segments[0],
            segments[1],
            segments[2],
            segments[3],
            segments[4],
            segments[5],
            segments[6],
            segments[7],
        )
    }
}

impl From<u128> for IPv6Address {
    fn from(value: u128) -> Self {
        Self {
            address: value.to_be_bytes(),
        }
    }
}

impl From<IPv6Address> for u128 {
    fn from(ip: IPv6Address) -> Self {
        u128::from_be_bytes(ip.address)
    }
}

impl IPv6Address {
    //! Standard Library Conversions

    /// Converts the address to a standard library address.
    pub const fn to_std(&self) -> Ipv6Addr {
        let segments: [u16; 8] = self.segments();
        Ipv6Addr::new(
            segments[0],
            segments[1],
            segments[2],
            segments[3],
            segments[4],
            segments[5],
            segments[6],
            segments[7],
        )
    }
}

impl From<Ipv6Addr> for IPv6Address {
    fn from(std: Ipv6Addr) -> Self {
        IPv6Address::from(std.octets())
    }
}

impl From<IPv6Address> for Ipv6Addr {
    fn from(ip: IPv6Address) -> Self {
        Ipv6Addr::from(ip.address)
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
            (self.address[0] as u16) << 8 | (self.address[1] as u16),
            (self.address[2] as u16) << 8 | (self.address[3] as u16),
            (self.address[4] as u16) << 8 | (self.address[5] as u16),
            (self.address[6] as u16) << 8 | (self.address[7] as u16),
            (self.address[8] as u16) << 8 | (self.address[9] as u16),
            (self.address[10] as u16) << 8 | (self.address[11] as u16),
            (self.address[12] as u16) << 8 | (self.address[13] as u16),
            (self.address[14] as u16) << 8 | (self.address[15] as u16),
        ]
    }
}

impl IPv6Address {
    //! Classification

    /// Checks if the address is the unspecified address. (::)
    pub fn is_unspecified(&self) -> bool {
        *self == Self::UNSPECIFIED
    }

    /// Checks if the address is the loopback address. (::1)
    pub fn is_loopback(&self) -> bool {
        *self == Self::LOCALHOST
    }
}

impl Display for IPv6Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let std: Ipv6Addr = Ipv6Addr::from(self.address);
        std::fmt::Display::fmt(&std, f)
    }
}

#[cfg(test)]
mod constructor_tests {
    use crate::IPv6Address;

    #[test]
    fn new() {
        let ip: IPv6Address = IPv6Address::new(
            0x0123, 0x4567, 0x89AB, 0xCDEF, 0x0123, 0x4567, 0x89AB, 0xCDEF,
        );
        let address: [u8; 16] = [
            0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB,
            0xCD, 0xEF,
        ];
        assert_eq!(ip.address, address);
    }
}

#[cfg(test)]
mod conversion_tests {
    use crate::{Host, IPAddress, IPv4Address, IPv6Address, SocketAddress};

    #[test]
    fn to_ip() {
        assert_eq!(
            IPv6Address::LOCALHOST.to_ip(),
            IPAddress::V6(IPv6Address::LOCALHOST)
        );
    }

    #[test]
    fn to_v4() {
        let ip: IPv6Address = IPv6Address::new(0, 0, 0, 0, 0, 0, 0x7F00, 0x0001);
        assert_eq!(ip.to_v4(), Some(IPv4Address::LOCALHOST));

        let ip: IPv6Address = IPv6Address::new(0, 0, 0, 0, 0, 0xFFFF, 0x7F00, 0x0001);
        assert_eq!(ip.to_v4(), Some(IPv4Address::LOCALHOST));

        let ip: IPv6Address = IPv6Address::new(0, 0, 0, 0, 0, 1, 0x7F00, 0x0001);
        assert_eq!(ip.to_v4(), None);

        let ip: IPv6Address = IPv6Address::new(1, 0, 0, 0, 0, 0, 0x7F00, 0x0001);
        assert_eq!(ip.to_v4(), None);
    }

    #[test]
    fn to_socket() {
        assert_eq!(
            IPv6Address::LOCALHOST.to_socket(80),
            SocketAddress::new(IPv6Address::LOCALHOST.to_ip(), 80)
        );
    }

    #[test]
    fn to_host() {
        assert_eq!(
            IPv6Address::LOCALHOST.to_host(),
            Host::Address(IPv6Address::LOCALHOST.to_ip())
        );
    }
}

#[cfg(test)]
mod from_tests {
    use crate::IPv6Address;

    #[test]
    fn from_u8_16() {
        let address: [u8; 16] = [
            0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB,
            0xCD, 0xEF,
        ];
        assert_eq!(IPv6Address::from(address).address, address);
    }

    #[test]
    fn from_u16_8() {
        let segments: [u16; 8] = [
            0x0123, 0x4567, 0x89AB, 0xCDEF, 0x0123, 0x4567, 0x89AB, 0xCDEF,
        ];
        let address: [u8; 16] = [
            0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB,
            0xCD, 0xEF,
        ];
        assert_eq!(IPv6Address::from(segments).address, address);
    }

    #[test]
    fn from_u128() {
        let value: u128 = 0x0123456789ABCDEF0123456789ABCDEF;
        let address: [u8; 16] = [
            0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB,
            0xCD, 0xEF,
        ];
        assert_eq!(IPv6Address::from(value).address, address);
    }

    #[test]
    fn u128_from_v6() {
        let address: [u8; 16] = [
            0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB,
            0xCD, 0xEF,
        ];
        let ip: IPv6Address = IPv6Address::from(address);
        let value: u128 = 0x0123456789ABCDEF0123456789ABCDEF;
        assert_eq!(u128::from(ip), value);
    }
}

#[cfg(test)]
mod std_tests {
    use std::net::Ipv6Addr;

    use crate::IPv6Address;

    #[test]
    fn to_std() {
        assert_eq!(IPv6Address::LOCALHOST.to_std(), Ipv6Addr::LOCALHOST);
    }

    #[test]
    fn from_std() {
        assert_eq!(
            IPv6Address::from(Ipv6Addr::LOCALHOST),
            IPv6Address::LOCALHOST
        );
    }

    #[test]
    fn std_from_v6() {
        assert_eq!(Ipv6Addr::from(IPv6Address::LOCALHOST), Ipv6Addr::LOCALHOST);
    }
}

#[cfg(test)]
mod property_tests {
    use crate::IPv6Address;

    #[test]
    fn address() {
        let ip: IPv6Address = IPv6Address::new(
            0x0123, 0x4567, 0x89AB, 0xCDEF, 0x0123, 0x4567, 0x89AB, 0xCDEF,
        );
        let address: [u8; 16] = [
            0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB,
            0xCD, 0xEF,
        ];
        assert_eq!(ip.address(), address);
    }

    #[test]
    fn segments() {
        let ip: IPv6Address = IPv6Address::new(
            0x0123, 0x4567, 0x89AB, 0xCDEF, 0x0123, 0x4567, 0x89AB, 0xCDEF,
        );
        let segments: [u16; 8] = [
            0x0123, 0x4567, 0x89AB, 0xCDEF, 0x0123, 0x4567, 0x89AB, 0xCDEF,
        ];
        assert_eq!(ip.segments(), segments);
    }
}

#[cfg(test)]
mod classification_tests {
    use crate::IPv6Address;

    #[test]
    fn is_unspecified() {
        assert_eq!(IPv6Address::UNSPECIFIED.is_unspecified(), true);
        assert_eq!(IPv6Address::LOCALHOST.is_unspecified(), false);
        assert_eq!(
            IPv6Address::new(1, 0, 0, 0, 0, 0, 0, 0).is_unspecified(),
            false
        );
    }

    #[test]
    fn is_loopback() {
        assert_eq!(IPv6Address::UNSPECIFIED.is_loopback(), false);
        assert_eq!(IPv6Address::LOCALHOST.is_loopback(), true);
        assert_eq!(
            IPv6Address::new(0, 0, 0, 0, 0, 0, 0, 2).is_loopback(),
            false
        );
    }
}

#[cfg(test)]
mod display_tests {
    use crate::IPv6Address;

    #[test]
    fn display_specials() {
        assert_eq!(IPv6Address::UNSPECIFIED.to_string(), "::");
        assert_eq!(IPv6Address::LOCALHOST.to_string(), "::1");
    }

    #[test]
    fn display_longest() {
        let ip: IPv6Address = IPv6Address::new(
            0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
        );
        assert_eq!(ip.to_string(), "ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff");
    }

    #[test]
    fn display_chunks() {
        let ip: IPv6Address = IPv6Address::new(0xABCD, 0, 0, 0, 0, 0, 0, 0);
        assert_eq!(ip.to_string(), "abcd::");

        let ip: IPv6Address = IPv6Address::new(0xABC, 0, 0, 0, 0, 0, 0, 0);
        assert_eq!(ip.to_string(), "abc::");

        let ip: IPv6Address = IPv6Address::new(0xAB, 0, 0, 0, 0, 0, 0, 0);
        assert_eq!(ip.to_string(), "ab::");

        let ip: IPv6Address = IPv6Address::new(0xA, 0, 0, 0, 0, 0, 0, 0);
        assert_eq!(ip.to_string(), "a::");
    }

    #[test]
    fn display_zero_ends() {
        let ip: IPv6Address = IPv6Address::new(0, 1, 1, 1, 1, 1, 1, 1);
        assert_eq!(ip.to_string(), "0:1:1:1:1:1:1:1");

        let ip: IPv6Address = IPv6Address::new(0, 0, 1, 1, 1, 1, 1, 1);
        assert_eq!(ip.to_string(), "::1:1:1:1:1:1");

        let ip: IPv6Address = IPv6Address::new(1, 1, 1, 1, 1, 1, 1, 0);
        assert_eq!(ip.to_string(), "1:1:1:1:1:1:1:0");

        let ip: IPv6Address = IPv6Address::new(1, 1, 1, 1, 1, 1, 0, 0);
        assert_eq!(ip.to_string(), "1:1:1:1:1:1::");
    }

    #[test]
    fn display_zero_middles() {
        let ip: IPv6Address = IPv6Address::new(1, 0, 1, 1, 1, 1, 1, 1);
        assert_eq!(ip.to_string(), "1:0:1:1:1:1:1:1");

        let ip: IPv6Address = IPv6Address::new(1, 0, 0, 1, 1, 1, 1, 1);
        assert_eq!(ip.to_string(), "1::1:1:1:1:1");
    }

    #[test]
    fn display_first_zeros() {
        let ip: IPv6Address = IPv6Address::new(1, 0, 0, 1, 1, 0, 0, 1);
        assert_eq!(ip.to_string(), "1::1:1:0:0:1");
    }

    #[test]
    fn display_largest_zeros() {
        let ip: IPv6Address = IPv6Address::new(1, 0, 0, 1, 0, 0, 0, 1);
        assert_eq!(ip.to_string(), "1:0:0:1::1");
    }

    #[test]
    fn display_v4() {
        let ip: IPv6Address = IPv6Address::new(0, 0, 0, 0, 0, 0, 0x7F00, 0x0001);
        assert_eq!(ip.to_string(), "::127.0.0.1");
    }
}
