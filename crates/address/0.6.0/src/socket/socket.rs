use crate::{IPAddress, SocketAddressV4, SocketAddressV6};

/// Represents a IP address with an associated port.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum SocketAddress {
    /// An IPv4 Socket Address
    V4(SocketAddressV4),

    /// An IPv6 Socket Address
    V6(SocketAddressV6),
}

impl SocketAddress {
    //! Constructors

    /// Creates a new socket address.
    pub const fn new(ip: IPAddress, port: u16) -> Self {
        match ip {
            IPAddress::V4(v4) => SocketAddress::V4(SocketAddressV4::new(v4, port)),
            IPAddress::V6(v6) => SocketAddress::V6(SocketAddressV6::new(v6, port)),
        }
    }
}

impl SocketAddress {
    //! Matching

    /// Checks if the address is an IPv4 socket address.
    pub const fn is_v4(&self) -> bool {
        matches!(self, SocketAddress::V4(_))
    }

    /// Checks if the address is an IPv6 socket address.
    pub const fn is_v6(&self) -> bool {
        matches!(self, SocketAddress::V6(_))
    }
}

impl SocketAddress {
    //! Properties

    /// Gets the IP address.
    pub const fn ip(&self) -> IPAddress {
        match self {
            Self::V4(v4) => v4.ip().to_ip(),
            Self::V6(v6) => v6.ip().to_ip(),
        }
    }

    /// Gets the port.
    pub const fn port(&self) -> u16 {
        match self {
            Self::V4(v4) => v4.port(),
            Self::V6(v6) => v6.port(),
        }
    }
}
