use crate::{
    Authority, IPAddress, IPv4Address, IPv6Address, SocketAddress, SocketAddressV4, SocketAddressV6,
};

impl SocketAddressV4 {
    //! Conversions

    /// Converts the address to a socket address.
    pub const fn to_socket(&self) -> SocketAddress {
        SocketAddress::new(self.ip().to_ip(), self.port())
    }

    /// Converts the address to an authority.
    pub const fn to_authority(&self) -> Authority {
        Authority::new(self.ip().to_host(), self.port())
    }
}

impl<A: Into<IPv4Address>> From<(A, u16)> for SocketAddressV4 {
    fn from(t: (A, u16)) -> Self {
        Self::new(t.0.into(), t.1)
    }
}

impl SocketAddressV6 {
    //! Conversions

    /// Converts the address to a socket address.
    pub const fn to_socket(&self) -> SocketAddress {
        SocketAddress::new(self.ip().to_ip(), self.port())
    }

    /// Converts the address to an authority.
    pub const fn to_authority(&self) -> Authority {
        Authority::new(self.ip().to_host(), self.port())
    }
}

impl<A: Into<IPv6Address>> From<(A, u16)> for SocketAddressV6 {
    fn from(t: (A, u16)) -> Self {
        Self::new(t.0.into(), t.1)
    }
}

impl SocketAddress {
    //! Conversions

    /// Converts the address to an optional socket address v4.
    pub const fn to_socket_v4(&self) -> Option<SocketAddressV4> {
        match self.ip() {
            IPAddress::V4(v4) => Some(SocketAddressV4::new(v4, self.port())),
            _ => None,
        }
    }

    /// Converts the address to an optional socket address v6.
    pub const fn to_socket_v6(&self) -> Option<SocketAddressV6> {
        match self.ip() {
            IPAddress::V6(v6) => Some(SocketAddressV6::new(v6, self.port())),
            _ => None,
        }
    }

    /// Converts the address to an authority.
    pub const fn to_authority(&self) -> Authority {
        Authority::new(self.ip().to_host(), self.port())
    }
}

impl<A: Into<IPAddress>> From<(A, u16)> for SocketAddress {
    fn from(t: (A, u16)) -> Self {
        Self::new(t.0.into(), t.1)
    }
}

impl From<SocketAddressV4> for SocketAddress {
    fn from(v4: SocketAddressV4) -> Self {
        SocketAddress::new(v4.ip().to_ip(), v4.port())
    }
}

impl From<SocketAddressV6> for SocketAddress {
    fn from(v6: SocketAddressV6) -> Self {
        SocketAddress::new(v6.ip().to_ip(), v6.port())
    }
}
