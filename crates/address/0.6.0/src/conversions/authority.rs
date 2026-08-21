use crate::{Authority, Endpoint, Host, SocketAddress, SocketAddressV4, SocketAddressV6};

impl Authority {
    //! Conversions

    /// Converts the address to an optional endpoint.
    pub fn to_endpoint(self) -> Option<Endpoint> {
        let (host, port) = self.into();
        match host {
            Host::Name(domain) => Some(Endpoint::new(domain, port)),
            _ => None,
        }
    }

    /// Converts the address to an optional socket address.
    pub fn to_socket(&self) -> Option<SocketAddress> {
        match self.host() {
            Host::Address(ip) => Some(SocketAddress::new(*ip, self.port())),
            _ => None,
        }
    }
}

impl From<Endpoint> for Authority {
    fn from(endpoint: Endpoint) -> Self {
        endpoint.to_authority()
    }
}

impl From<SocketAddressV4> for Authority {
    fn from(v4: SocketAddressV4) -> Self {
        v4.to_authority()
    }
}

impl From<SocketAddressV6> for Authority {
    fn from(v6: SocketAddressV6) -> Self {
        v6.to_authority()
    }
}

impl From<SocketAddress> for Authority {
    fn from(socket: SocketAddress) -> Self {
        socket.to_authority()
    }
}

impl<H: Into<Host>> From<(H, u16)> for Authority {
    fn from(t: (H, u16)) -> Self {
        Self::new(t.0.into(), t.1)
    }
}

impl From<Authority> for (Host, u16) {
    fn from(authority: Authority) -> Self {
        authority.export()
    }
}
