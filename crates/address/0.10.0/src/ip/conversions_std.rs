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
        IPv4Address::new(std.octets())
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
        IPv6Address::new(std.octets())
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
            IpAddr::V4(ip) => IPv4Address::from(ip).to_ip(),
            IpAddr::V6(ip) => IPv6Address::from(ip).to_ip(),
        }
    }
}

impl From<IPAddress> for IpAddr {
    fn from(ip: IPAddress) -> Self {
        match ip {
            IPAddress::V4(ip) => ip.to_ip().to_std(),
            IPAddress::V6(ip) => ip.to_ip().to_std(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    use crate::{IPAddress, IPv4Address, IPv6Address};

    #[test]
    fn v4() {
        let ip: IPv4Address = IPv4Address::LOCALHOST;
        let std: Ipv4Addr = Ipv4Addr::LOCALHOST;

        let result: Ipv4Addr = ip.to_std();
        assert_eq!(result, std);

        let result: Ipv4Addr = ip.into();
        assert_eq!(result, std);

        let result: IPv4Address = std.into();
        assert_eq!(result, ip);
    }

    #[test]
    fn v6() {
        let ip: IPv6Address = IPv6Address::LOCALHOST;
        let std: Ipv6Addr = Ipv6Addr::LOCALHOST;

        let result: Ipv6Addr = ip.to_std();
        assert_eq!(result, std);

        let result: Ipv6Addr = ip.into();
        assert_eq!(result, std);

        let result: IPv6Address = std.into();
        assert_eq!(result, ip);
    }

    #[test]
    fn ip() {
        let ip: IPAddress = IPv4Address::LOCALHOST.to_ip();
        let std: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);

        let result: IpAddr = ip.to_std();
        assert_eq!(result, std);

        let result: IpAddr = ip.into();
        assert_eq!(result, std);

        let result: IPAddress = std.into();
        assert_eq!(result, ip);

        let ip: IPAddress = IPv6Address::LOCALHOST.to_ip();
        let std: IpAddr = IpAddr::V6(Ipv6Addr::LOCALHOST);

        let result: IpAddr = ip.to_std();
        assert_eq!(result, std);

        let result: IpAddr = ip.into();
        assert_eq!(result, std);

        let result: IPAddress = std.into();
        assert_eq!(result, ip);
    }
}
