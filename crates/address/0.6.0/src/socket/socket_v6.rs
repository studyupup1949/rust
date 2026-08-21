use crate::IPv6Address;

/// Represents a IPv6 address with an associated port.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct SocketAddressV6 {
    ip: IPv6Address,
    port: u16,
}

impl SocketAddressV6 {
    //! Constructors

    /// Creates a new socket address.
    pub const fn new(ip: IPv6Address, port: u16) -> Self {
        Self { ip, port }
    }
}

impl SocketAddressV6 {
    //! Properties

    /// Gets the IPv6 address.
    pub const fn ip(&self) -> IPv6Address {
        self.ip
    }

    /// Gets the port.
    pub const fn port(&self) -> u16 {
        self.port
    }
}
