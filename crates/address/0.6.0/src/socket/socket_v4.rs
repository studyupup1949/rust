use crate::IPv4Address;

/// Represents a IPv4 address with an associated port.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct SocketAddressV4 {
    ip: IPv4Address,
    port: u16,
}

impl SocketAddressV4 {
    //! Constructors

    /// Creates a new socket address.
    pub const fn new(ip: IPv4Address, port: u16) -> Self {
        Self { ip, port }
    }
}

impl SocketAddressV4 {
    //! Properties

    /// Gets the IPv4 address.
    pub const fn ip(&self) -> IPv4Address {
        self.ip
    }

    /// Gets the port.
    pub const fn port(&self) -> u16 {
        self.port
    }
}
