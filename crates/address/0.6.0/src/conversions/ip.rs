use crate::{Authority, Host, IPAddress, IPv4Address, IPv6Address, SocketAddress};

impl IPAddress {
    //! Conversions

    /// Converts the address to an optional IPv4 address.
    pub const fn to_v4(&self) -> Option<IPv4Address> {
        if let Self::V4(v4) = self {
            Some(*v4)
        } else {
            None
        }
    }

    /// Converts the address to an optional IPv6 address.
    pub const fn to_v6(&self) -> Option<IPv6Address> {
        if let Self::V6(v6) = self {
            Some(*v6)
        } else {
            None
        }
    }

    /// Converts the address to a socket address with the port.
    pub const fn to_socket(&self, port: u16) -> SocketAddress {
        SocketAddress::new(*self, port)
    }

    /// Converts the address to a host.
    pub const fn to_host(&self) -> Host {
        Host::Address(*self)
    }

    /// Converts the address to an authority with the port.
    pub const fn to_authority(&self, port: u16) -> Authority {
        Authority::new(self.to_host(), port)
    }
}

impl From<[u8; 4]> for IPAddress {
    fn from(address: [u8; 4]) -> Self {
        Self::V4(IPv4Address::from(address))
    }
}

impl From<(u8, u8, u8, u8)> for IPAddress {
    fn from(t: (u8, u8, u8, u8)) -> Self {
        Self::V4(IPv4Address::from(t))
    }
}

impl From<u32> for IPAddress {
    fn from(value: u32) -> Self {
        Self::V4(IPv4Address::from(value))
    }
}

impl From<IPv4Address> for IPAddress {
    fn from(v4: IPv4Address) -> Self {
        Self::V4(v4)
    }
}

impl From<[u8; 16]> for IPAddress {
    fn from(address: [u8; 16]) -> Self {
        Self::V6(IPv6Address::from(address))
    }
}

impl From<[u16; 8]> for IPAddress {
    fn from(segments: [u16; 8]) -> Self {
        Self::V6(IPv6Address::from(segments))
    }
}

impl From<u128> for IPAddress {
    fn from(value: u128) -> Self {
        Self::V6(IPv6Address::from(value))
    }
}

impl From<IPv6Address> for IPAddress {
    fn from(v6: IPv6Address) -> Self {
        Self::V6(v6)
    }
}
