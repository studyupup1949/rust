use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6};

use crate::{IPAddress, SocketAddress, SocketAddressV4, SocketAddressV6};

impl SocketAddressV4 {
    /// Converts the address to a standard library address.
    pub const fn to_std(&self) -> SocketAddrV4 {
        SocketAddrV4::new(self.ip().to_std(), self.port())
    }
}

impl From<SocketAddrV4> for SocketAddressV4 {
    fn from(socket: SocketAddrV4) -> Self {
        Self::new((*socket.ip()).into(), socket.port())
    }
}

impl From<SocketAddressV4> for SocketAddrV4 {
    fn from(socket: SocketAddressV4) -> Self {
        socket.to_std()
    }
}

impl SocketAddressV6 {
    /// Converts the address to a standard library address.
    pub const fn to_std(&self, flow_info: u32, scope_id: u32) -> SocketAddrV6 {
        SocketAddrV6::new(self.ip().to_std(), self.port(), flow_info, scope_id)
    }
}

impl From<SocketAddrV6> for SocketAddressV6 {
    fn from(socket: SocketAddrV6) -> Self {
        Self::new((*socket.ip()).into(), socket.port())
    }
}

impl SocketAddress {
    /// Converts the address to a standard library address.
    pub const fn to_std(&self, flow_info: u32, scope_id: u32) -> SocketAddr {
        match self.ip() {
            IPAddress::V4(v4) => SocketAddr::V4(SocketAddrV4::new(v4.to_std(), self.port())),
            IPAddress::V6(v6) => SocketAddr::V6(SocketAddrV6::new(
                v6.to_std(),
                self.port(),
                flow_info,
                scope_id,
            )),
        }
    }
}

impl From<SocketAddr> for SocketAddress {
    fn from(socket: SocketAddr) -> Self {
        Self::new(socket.ip().into(), socket.port())
    }
}

#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

    use crate::{IPv4Address, IPv6Address, SocketAddress, SocketAddressV4, SocketAddressV6};

    #[test]
    fn v4() {
        let socket: SocketAddressV4 = SocketAddressV4::new(IPv4Address::LOCALHOST, 80);
        let result: SocketAddrV4 = socket.to_std();
        let expected: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 80);
        assert_eq!(result, expected);

        let result: SocketAddressV4 = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 80).into();
        let expected: SocketAddressV4 = SocketAddressV4::new(IPv4Address::LOCALHOST, 80);
        assert_eq!(result, expected);
    }

    #[test]
    fn v6() {
        let socket: SocketAddressV6 = SocketAddressV6::new(IPv6Address::LOCALHOST, 80);
        let result: SocketAddrV6 = socket.to_std(1, 2);
        let expected: SocketAddrV6 = SocketAddrV6::new(Ipv6Addr::LOCALHOST, 80, 1, 2);
        assert_eq!(result, expected);

        let result: SocketAddressV6 = SocketAddrV6::new(Ipv6Addr::LOCALHOST, 80, 1, 2).into();
        let expected: SocketAddressV6 = SocketAddressV6::new(IPv6Address::LOCALHOST, 80);
        assert_eq!(result, expected);
    }

    #[test]
    fn socket() {
        let socket: SocketAddress = SocketAddress::new(IPv4Address::LOCALHOST.to_ip(), 80);
        let result: SocketAddr = socket.to_std(0, 0);
        let expected: SocketAddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 80));
        assert_eq!(result, expected);

        let socket: SocketAddress = SocketAddress::new(IPv6Address::LOCALHOST.to_ip(), 80);
        let result: SocketAddr = socket.to_std(1, 2);
        let expected: SocketAddr = SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::LOCALHOST, 80, 1, 2));
        assert_eq!(result, expected);

        let result: SocketAddress =
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 80)).into();
        let expected: SocketAddress = SocketAddress::new(IPv4Address::LOCALHOST.to_ip(), 80);
        assert_eq!(result, expected);
    }
}
