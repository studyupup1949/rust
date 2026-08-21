use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use crate::{IPAddress, IPv4Address, IPv6Address};

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
        Self::from(std.octets())
    }
}

impl From<IPv4Address> for Ipv4Addr {
    fn from(v4: IPv4Address) -> Self {
        Self::from(v4.address())
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
        Self::from(std.octets())
    }
}

impl From<IPv6Address> for Ipv6Addr {
    fn from(v6: IPv6Address) -> Self {
        Self::from(*v6.address())
    }
}

impl IPAddress {
    //! Standard Library Conversions

    /// Converts the address to a standard library address.
    pub const fn to_std(&self) -> IpAddr {
        match self {
            Self::V4(v4) => IpAddr::V4(v4.to_std()),
            Self::V6(v6) => IpAddr::V6(v6.to_std()),
        }
    }
}

impl From<IpAddr> for IPAddress {
    fn from(std: IpAddr) -> Self {
        match std {
            IpAddr::V4(v4) => Self::V4(IPv4Address::from(v4)),
            IpAddr::V6(v6) => Self::V6(IPv6Address::from(v6)),
        }
    }
}

impl From<IPAddress> for IpAddr {
    fn from(ip: IPAddress) -> Self {
        match ip {
            IPAddress::V4(v4) => IpAddr::V4(v4.to_std()),
            IPAddress::V6(v6) => IpAddr::V6(v6.to_std()),
        }
    }
}
