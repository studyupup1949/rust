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
        Self::new(std.octets())
    }
}

impl From<IPv4Address> for Ipv4Addr {
    fn from(ip: IPv4Address) -> Self {
        ip.to_std()
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
        Self::new(std.octets())
    }
}

impl From<IPv6Address> for Ipv6Addr {
    fn from(ip: IPv6Address) -> Self {
        ip.to_std()
    }
}

impl IPAddress {
    //! Standard Library Conversions

    /// Converts the address to a standard library address.
    pub const fn to_std(&self) -> IpAddr {
        match self {
            Self::V4(ip) => IpAddr::V4(ip.to_std()),
            Self::V6(ip) => IpAddr::V6(ip.to_std()),
        }
    }
}

impl From<IpAddr> for IPAddress {
    fn from(std: IpAddr) -> Self {
        match std {
            IpAddr::V4(ip) => Self::V4(IPv4Address::from(ip)),
            IpAddr::V6(ip) => Self::V6(IPv6Address::from(ip)),
        }
    }
}

impl From<IPAddress> for IpAddr {
    fn from(ip: IPAddress) -> Self {
        ip.to_std()
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    use crate::{IPAddress, IPv4Address, IPv6Address};

    #[test]
    fn v4() {
        let result: Ipv4Addr = IPv4Address::LOCALHOST.to_std();
        assert_eq!(result, Ipv4Addr::LOCALHOST);

        let result: Ipv4Addr = IPv4Address::LOCALHOST.into();
        assert_eq!(result, Ipv4Addr::LOCALHOST);

        let result: IPv4Address = Ipv4Addr::LOCALHOST.into();
        assert_eq!(result, IPv4Address::LOCALHOST);
    }

    #[test]
    fn v6() {
        let result: Ipv6Addr = IPv6Address::LOCALHOST.to_std();
        assert_eq!(result, Ipv6Addr::LOCALHOST);

        let result: Ipv6Addr = IPv6Address::LOCALHOST.into();
        assert_eq!(result, Ipv6Addr::LOCALHOST);

        let result: IPv6Address = Ipv6Addr::LOCALHOST.into();
        assert_eq!(result, IPv6Address::LOCALHOST);
    }

    #[test]
    fn ip() {
        let result: IpAddr = IPAddress::V4(IPv4Address::LOCALHOST).into();
        assert_eq!(result, IpAddr::V4(Ipv4Addr::LOCALHOST));
        let result: IpAddr = IPAddress::V6(IPv6Address::LOCALHOST).into();
        assert_eq!(result, IpAddr::V6(Ipv6Addr::LOCALHOST));

        let result: IpAddr = IPAddress::V4(IPv4Address::LOCALHOST).into();
        assert_eq!(result, IpAddr::V4(Ipv4Addr::LOCALHOST));
        let result: IpAddr = IPAddress::V6(IPv6Address::LOCALHOST).into();
        assert_eq!(result, IpAddr::V6(Ipv6Addr::LOCALHOST));

        let result: IPAddress = IpAddr::V4(Ipv4Addr::LOCALHOST).into();
        assert_eq!(result, IPAddress::V4(IPv4Address::LOCALHOST));
        let result: IPAddress = IpAddr::V6(Ipv6Addr::LOCALHOST).into();
        assert_eq!(result, IPAddress::V6(IPv6Address::LOCALHOST));
    }
}
