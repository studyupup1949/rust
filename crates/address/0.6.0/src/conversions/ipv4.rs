use crate::{Authority, Host, IPAddress, IPv4Address, IPv6Address, SocketAddress, SocketAddressV4};

impl IPv4Address {
    //! Conversions

    /// Converts the address to an IPv6 compatible address. (::a.b.c.d)
    pub const fn to_v6_compatible(&self) -> IPv6Address {
        let (a, b, c, d) = self.bytes();
        IPv6Address::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, a, b, c, d])
    }

    /// Converts the address to an IPv6 mapped address. (::ffff:a.b.c.d)
    pub const fn to_v6_mapped(&self) -> IPv6Address {
        let (a, b, c, d) = self.bytes();
        IPv6Address::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xFF, 0xFF, a, b, c, d])
    }

    /// Converts the address to an IP address.
    pub const fn to_ip(&self) -> IPAddress {
        IPAddress::V4(*self)
    }

    /// Converts the address to a socket address v4 with the port.
    pub const fn to_socket_v4(&self, port: u16) -> SocketAddressV4 {
        SocketAddressV4::new(*self, port)
    }

    /// Converts the address to a socket address with the port.
    pub const fn to_socket(&self, port: u16) -> SocketAddress {
        SocketAddress::new(self.to_ip(), port)
    }

    /// Converts the address to a host.
    pub const fn to_host(&self) -> Host {
        Host::Address(self.to_ip())
    }

    /// Converts the address to an authority with the port.
    pub const fn to_authority(&self, port: u16) -> Authority {
        Authority::new(self.to_host(), port)
    }
}

impl From<[u8; 4]> for IPv4Address {
    fn from(address: [u8; 4]) -> Self {
        Self::new(address)
    }
}

impl From<(u8, u8, u8, u8)> for IPv4Address {
    fn from(t: (u8, u8, u8, u8)) -> Self {
        Self::new([t.0, t.1, t.2, t.3])
    }
}

impl From<u32> for IPv4Address {
    fn from(value: u32) -> Self {
        Self::new(value.to_be_bytes())
    }
}
