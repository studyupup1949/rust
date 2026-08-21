use std::fmt::{Display, Error, Formatter};
use std::net::Ipv4Addr;

use crate::{Authority, Host, IPAddress, IPv6Address, SocketAddress};

/// Represents an IPv4 address. (a.b.c.d)
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct IPv4Address {
    /// The address. [a, b, c, d]
    address: [u8; 4],
}

impl IPv4Address {
    //! Special Addresses

    /// The unspecified address. (0.0.0.0)
    pub const UNSPECIFIED: Self = Self::new(0, 0, 0, 0);

    /// The localhost address. (127.0.0.1)
    pub const LOCALHOST: Self = Self::new(127, 0, 0, 1);

    /// The broadcast address. (255.255.255.255)
    pub const BROADCAST: Self = Self::new(255, 255, 255, 255);
}

impl IPv4Address {
    //! Constructors

    /// Creates a new IPv4 address from the bytes. (a.b.c.d)
    pub const fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Self {
            address: [a, b, c, d],
        }
    }
}

impl IPv4Address {
    //! Conversions

    /// Converts the address to an IP address.
    pub const fn to_ip(&self) -> IPAddress {
        IPAddress::V4(*self)
    }

    /// Converts the address to an IPv6 compatible address. (::a.b.c.d)
    pub const fn to_v6_compatible(&self) -> IPv6Address {
        let (a, b, c, d) = self.bytes();
        IPv6Address::new(
            0,
            0,
            0,
            0,
            0,
            0,
            (a as u16) << 8 | b as u16,
            (c as u16) << 8 | d as u16,
        )
    }

    /// Converts the address to an IPv6 mapped address. (::ffff:a.b.c.d)
    pub const fn to_v6_mapped(&self) -> IPv6Address {
        let (a, b, c, d) = self.bytes();
        IPv6Address::new(
            0,
            0,
            0,
            0,
            0,
            0xFFFF,
            (a as u16) << 8 | b as u16,
            (c as u16) << 8 | d as u16,
        )
    }

    /// Converts the address to a host.
    pub const fn to_host(&self) -> Host {
        Host::Address(self.to_ip())
    }

    /// Converts the address to a socket address with the port.
    pub const fn to_socket(&self, port: u16) -> SocketAddress {
        SocketAddress::new(self.to_ip(), port)
    }

    /// Converts the address to an authority with the port.
    pub const fn to_authority(&self, port: u16) -> Authority {
        Authority::Address(self.to_socket(port))
    }
}

impl From<[u8; 4]> for IPv4Address {
    fn from(address: [u8; 4]) -> Self {
        Self { address }
    }
}

impl From<u32> for IPv4Address {
    fn from(value: u32) -> Self {
        Self {
            address: value.to_be_bytes(),
        }
    }
}

impl From<IPv4Address> for u32 {
    fn from(ip: IPv4Address) -> Self {
        u32::from_be_bytes(ip.address)
    }
}

impl IPv4Address {
    //! Standard Library Conversions

    /// Converts the address to a standard library address.
    pub const fn to_std(&self) -> Ipv4Addr {
        let (a, b, c, d) = self.bytes();
        Ipv4Addr::new(a, b, c, d)
    }
}

impl From<Ipv4Addr> for IPv4Address {
    fn from(std: Ipv4Addr) -> Self {
        IPv4Address::from(std.octets())
    }
}

impl From<IPv4Address> for Ipv4Addr {
    fn from(ip: IPv4Address) -> Self {
        Ipv4Addr::from(ip.address)
    }
}

impl IPv4Address {
    //! Properties

    /// Gets the address. [a, b, c, d]
    pub const fn address(&self) -> [u8; 4] {
        self.address
    }

    /// Gets the bytes. (a, b, c, d)
    pub const fn bytes(&self) -> (u8, u8, u8, u8) {
        (
            self.address[0],
            self.address[1],
            self.address[2],
            self.address[3],
        )
    }
}

impl IPv4Address {
    //! Classification

    /// Checks if the address is the unspecified address. (0.0.0.0)
    pub fn is_unspecified(&self) -> bool {
        *self == Self::UNSPECIFIED
    }

    /// Checks if the address is a loopback address. (127.0.0.0/8)
    pub fn is_loopback(&self) -> bool {
        match self.address {
            [127, 0, 0, _] => true,
            _ => false,
        }
    }
}

impl Display for IPv4Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let (a, b, c, d) = self.bytes();
        write!(f, "{}.{}.{}.{}", a, b, c, d)
    }
}

#[cfg(test)]
mod constructor_tests {
    use crate::IPv4Address;

    #[test]
    fn new() {
        assert_eq!(IPv4Address::new(127, 0, 0, 1).address, [127, 0, 0, 1]);
    }
}

#[cfg(test)]
mod conversion_tests {
    use crate::{Authority, Host, IPAddress, IPv4Address, SocketAddress};

    #[test]
    fn to_ip() {
        assert_eq!(
            IPv4Address::LOCALHOST.to_ip(),
            IPAddress::V4(IPv4Address::LOCALHOST)
        );
    }

    #[test]
    fn to_v6_compatible() {
        assert_eq!(
            IPv4Address::LOCALHOST.to_v6_compatible().segments(),
            [0, 0, 0, 0, 0, 0, 0x7F00, 0x0001]
        );
    }

    #[test]
    fn to_v6_mapped() {
        assert_eq!(
            IPv4Address::LOCALHOST.to_v6_mapped().segments(),
            [0, 0, 0, 0, 0, 0xFFFF, 0x7F00, 0x0001]
        );
    }

    #[test]
    fn to_host() {
        assert_eq!(
            IPv4Address::LOCALHOST.to_host(),
            Host::Address(IPv4Address::LOCALHOST.to_ip())
        );
    }

    #[test]
    fn to_socket() {
        assert_eq!(
            IPv4Address::LOCALHOST.to_socket(80),
            SocketAddress::new(IPv4Address::LOCALHOST.to_ip(), 80)
        );
    }

    #[test]
    fn to_authority() {
        assert_eq!(
            IPv4Address::LOCALHOST.to_authority(80),
            Authority::Address(SocketAddress::new(IPv4Address::LOCALHOST.to_ip(), 80))
        );
    }
}

#[cfg(test)]
mod from_tests {
    use crate::IPv4Address;

    #[test]
    fn from_u8_4() {
        assert_eq!(IPv4Address::from([127, 0, 0, 1]), IPv4Address::LOCALHOST);
    }

    #[test]
    fn from_u32() {
        assert_eq!(IPv4Address::from(0x7F000001), IPv4Address::LOCALHOST);
    }

    #[test]
    fn u32_from_v4() {
        assert_eq!(u32::from(IPv4Address::LOCALHOST), 0x7F000001);
    }
}

#[cfg(test)]
mod std_tests {
    use std::net::Ipv4Addr;

    use crate::IPv4Address;

    #[test]
    fn to_std() {
        assert_eq!(IPv4Address::LOCALHOST.to_std(), Ipv4Addr::LOCALHOST);
    }

    #[test]
    fn from_std() {
        assert_eq!(
            IPv4Address::from(Ipv4Addr::LOCALHOST),
            IPv4Address::LOCALHOST
        );
    }

    #[test]
    fn std_from_v4() {
        assert_eq!(Ipv4Addr::from(IPv4Address::LOCALHOST), Ipv4Addr::LOCALHOST);
    }
}

#[cfg(test)]
mod property_tests {
    use crate::IPv4Address;

    #[test]
    fn address() {
        assert_eq!(IPv4Address::LOCALHOST.address(), [127, 0, 0, 1]);
    }

    #[test]
    fn bytes() {
        assert_eq!(IPv4Address::LOCALHOST.bytes(), (127, 0, 0, 1));
    }
}

#[cfg(test)]
mod classification_tests {
    use crate::IPv4Address;

    #[test]
    fn is_unspecified() {
        assert_eq!(IPv4Address::UNSPECIFIED.is_unspecified(), true);
        assert_eq!(IPv4Address::new(0, 0, 0, 1).is_unspecified(), false);
        assert_eq!(IPv4Address::new(1, 0, 0, 0).is_unspecified(), false);
    }

    #[test]
    fn is_loopback() {
        assert_eq!(IPv4Address::LOCALHOST.is_loopback(), true);
        assert_eq!(IPv4Address::new(127, 0, 0, 0).is_loopback(), true);
        assert_eq!(IPv4Address::new(127, 0, 0, 255).is_loopback(), true);
        assert_eq!(IPv4Address::new(128, 0, 0, 8).is_loopback(), false);
        assert_eq!(IPv4Address::new(126, 0, 0, 8).is_loopback(), false);
    }
}

#[cfg(test)]
mod display_tests {
    use crate::IPv4Address;

    #[test]
    fn display() {
        assert_eq!(IPv4Address::UNSPECIFIED.to_string(), "0.0.0.0");
        assert_eq!(IPv4Address::LOCALHOST.to_string(), "127.0.0.1");
        assert_eq!(IPv4Address::BROADCAST.to_string(), "255.255.255.255");
    }
}
