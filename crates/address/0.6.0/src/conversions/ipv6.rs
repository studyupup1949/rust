use crate::{Authority, Host, IPAddress, IPv4Address, IPv6Address, SocketAddress, SocketAddressV6};

impl IPv6Address {
    //! Conversions

    /// Converts the address to an optional IPv4 address. Returns None when the address is not an IPv4 compatible
    /// address (::a.b.c.d) or an IPv4 mapped address (::ffff:.a.b.c.d).
    pub const fn to_v4(&self) -> Option<IPv4Address> {
        if self.is_v4_convertable() {
            let address: &[u8; 16] = self.address();
            Some(IPv4Address::new([
                address[12],
                address[13],
                address[14],
                address[15],
            ]))
        } else {
            None
        }
    }

    /// Converts the address to an IP address.
    pub const fn to_ip(&self) -> IPAddress {
        IPAddress::V6(*self)
    }

    /// Converts the address to a socket address v6 with the port.
    pub const fn to_socket_v6(&self, port: u16) -> SocketAddressV6 {
        SocketAddressV6::new(*self, port)
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

impl From<[u8; 16]> for IPv6Address {
    fn from(address: [u8; 16]) -> Self {
        Self::new(address)
    }
}

impl From<[u16; 8]> for IPv6Address {
    fn from(segments: [u16; 8]) -> Self {
        let a: u16 = segments[0];
        let b: u16 = segments[1];
        let c: u16 = segments[2];
        let d: u16 = segments[3];
        let e: u16 = segments[4];
        let f: u16 = segments[5];
        let g: u16 = segments[6];
        let h: u16 = segments[7];
        Self::new([
            (a >> 8) as u8,
            a as u8,
            (b >> 8) as u8,
            b as u8,
            (c >> 8) as u8,
            c as u8,
            (d >> 8) as u8,
            d as u8,
            (e >> 8) as u8,
            e as u8,
            (f >> 8) as u8,
            f as u8,
            (g >> 8) as u8,
            g as u8,
            (h >> 8) as u8,
            h as u8,
        ])
    }
}

impl From<u128> for IPv6Address {
    fn from(value: u128) -> Self {
        Self::new(value.to_be_bytes())
    }
}
