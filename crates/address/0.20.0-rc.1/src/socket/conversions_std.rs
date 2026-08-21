use crate::{IPAddress, SocketAddress, SocketAddressV4, SocketAddressV6};
use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6};

impl SocketAddressV4 {
    //! Standard Library Conversions

    /// Converts the address to a standard library address.
    #[must_use]
    pub const fn to_std(self) -> SocketAddrV4 {
        SocketAddrV4::new(self.ip().to_std(), self.port())
    }
}

impl From<SocketAddrV4> for SocketAddressV4 {
    fn from(std: SocketAddrV4) -> Self {
        Self::new((*std.ip()).into(), std.port())
    }
}

impl From<SocketAddressV4> for SocketAddrV4 {
    fn from(socket: SocketAddressV4) -> Self {
        socket.to_std()
    }
}

impl SocketAddressV6 {
    //! Standard Library Conversions

    /// Converts the address to a standard library address with a zero `flow_info` and `scope_id`.
    #[must_use]
    pub const fn to_std(self) -> SocketAddrV6 {
        self.to_std_with(0, 0)
    }

    /// Converts the address to a standard library address with the `flow_info` and `scope_id`.
    #[must_use]
    pub const fn to_std_with(self, flow_info: u32, scope_id: u32) -> SocketAddrV6 {
        SocketAddrV6::new(self.ip().to_std(), self.port(), flow_info, scope_id)
    }
}

impl From<SocketAddrV6> for SocketAddressV6 {
    /// Converts the standard library address, discarding its `flow_info` & `scope_id`.
    fn from(std: SocketAddrV6) -> Self {
        Self::new((*std.ip()).into(), std.port())
    }
}

impl From<SocketAddressV6> for SocketAddrV6 {
    fn from(socket: SocketAddressV6) -> Self {
        socket.to_std()
    }
}

impl SocketAddress {
    //! Standard Library Conversions

    /// Converts the address to a standard library address with a zero `flow_info` and `scope_id` for IPv6 addresses.
    #[must_use]
    pub const fn to_std(self) -> SocketAddr {
        self.to_std_with(0, 0)
    }

    /// Converts the address to a standard library address with the `flow_info` and `scope_id` for IPv6 addresses.
    #[must_use]
    pub const fn to_std_with(self, flow_info: u32, scope_id: u32) -> SocketAddr {
        match self.ip() {
            IPAddress::V4(ip) => SocketAddr::V4(SocketAddrV4::new(ip.to_std(), self.port())),
            IPAddress::V6(ip) => SocketAddr::V6(SocketAddrV6::new(ip.to_std(), self.port(), flow_info, scope_id)),
        }
    }
}

impl From<SocketAddr> for SocketAddress {
    /// Converts the standard library address, discarding the `flow_info` & `scope_id` of IPv6 addresses.
    fn from(std: SocketAddr) -> Self {
        Self::new(std.ip().into(), std.port())
    }
}

impl From<SocketAddress> for SocketAddr {
    fn from(socket: SocketAddress) -> Self {
        socket.to_std()
    }
}

#[cfg(test)]
mod tests {
    use crate::{IPv4Address, IPv6Address, SocketAddress, SocketAddressV4, SocketAddressV6};
    use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

    #[test]
    fn v4() {
        let test_cases: &[(SocketAddressV4, SocketAddrV4)] = &[(
            SocketAddressV4::new(IPv4Address::LOCALHOST, 80),
            SocketAddrV4::new(Ipv4Addr::LOCALHOST, 80),
        )];

        for (socket, std) in test_cases {
            let result: SocketAddrV4 = socket.to_std();
            assert_eq!(result, *std, "socket={:?}", socket);

            let result: SocketAddrV4 = (*socket).into();
            assert_eq!(result, *std, "socket={:?}", socket);

            let result: SocketAddressV4 = (*std).into();
            assert_eq!(result, *socket, "std={:?}", std);
        }
    }

    #[test]
    fn v6() {
        let test_cases: &[(SocketAddressV6, SocketAddrV6)] = &[(
            SocketAddressV6::new(IPv6Address::LOCALHOST, 80),
            SocketAddrV6::new(Ipv6Addr::LOCALHOST, 80, 0, 0),
        )];

        for (socket, std) in test_cases {
            let result: SocketAddrV6 = socket.to_std();
            assert_eq!(result, *std, "socket={:?}", socket);

            let result: SocketAddrV6 = (*socket).into();
            assert_eq!(result, *std, "socket={:?}", socket);

            let result: SocketAddressV6 = (*std).into();
            assert_eq!(result, *socket, "std={:?}", std);
        }

        let socket: SocketAddressV6 = SocketAddressV6::new(IPv6Address::LOCALHOST, 80);
        let result: SocketAddrV6 = socket.to_std_with(123, 456);
        let expected: SocketAddrV6 = SocketAddrV6::new(Ipv6Addr::LOCALHOST, 80, 123, 456);
        assert_eq!(result, expected);

        let std: SocketAddrV6 = SocketAddrV6::new(Ipv6Addr::LOCALHOST, 80, 123, 456);
        let result: SocketAddressV6 = std.into();
        let expected: SocketAddressV6 = SocketAddressV6::new(IPv6Address::LOCALHOST, 80);
        assert_eq!(result, expected);
    }

    #[test]
    fn socket() {
        let test_cases: &[(SocketAddress, SocketAddr)] = &[
            (
                SocketAddress::new(IPv4Address::LOCALHOST.to_ip(), 80),
                SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 80)),
            ),
            (
                SocketAddress::new(IPv6Address::LOCALHOST.to_ip(), 80),
                SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::LOCALHOST, 80, 0, 0)),
            ),
        ];

        for (socket, std) in test_cases {
            let result: SocketAddr = socket.to_std();
            assert_eq!(result, *std, "socket={:?}", socket);

            let result: SocketAddr = (*socket).into();
            assert_eq!(result, *std, "socket={:?}", socket);

            let result: SocketAddress = (*std).into();
            assert_eq!(result, *socket, "std={:?}", std);
        }

        let socket: SocketAddress = SocketAddress::new(IPv4Address::LOCALHOST.to_ip(), 80);
        let result: SocketAddr = socket.to_std_with(123, 456);
        let expected: SocketAddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 80));
        assert_eq!(result, expected);

        let socket: SocketAddress = SocketAddress::new(IPv6Address::LOCALHOST.to_ip(), 80);
        let result: SocketAddr = socket.to_std_with(123, 456);
        let expected: SocketAddr = SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::LOCALHOST, 80, 123, 456));
        assert_eq!(result, expected);

        let std: SocketAddr = SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::LOCALHOST, 80, 123, 456));
        let result: SocketAddress = std.into();
        let expected: SocketAddress = SocketAddress::new(IPv6Address::LOCALHOST.to_ip(), 80);
        assert_eq!(result, expected);
    }
}
