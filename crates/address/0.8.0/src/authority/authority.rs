use crate::{DomainRef, Host, HostRef};

/// A host with an associated port.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct Authority {
    host: Host,
    port: u16,
}

impl Authority {
    //! Construction

    /// Creates a new authority.
    pub fn new<H>(host: H, port: u16) -> Self
    where
        H: Into<Host>,
    {
        Self {
            host: host.into(),
            port,
        }
    }
}

impl<H: Into<Host>> From<(H, u16)> for Authority {
    fn from(tuple: (H, u16)) -> Self {
        Self::new(tuple.0.into(), tuple.1.into())
    }
}

impl Authority {
    //! Properties

    /// Gets the host.
    pub fn host(&self) -> HostRef {
        match &self.host {
            Host::Name(name) => HostRef::Name(unsafe { DomainRef::new(name.name()) }),
            Host::Address(ip) => HostRef::Address(*ip),
        }
    }

    /// Gets the port.
    pub fn port(&self) -> u16 {
        self.port
    }
}

impl From<Authority> for (Host, u16) {
    fn from(authority: Authority) -> Self {
        (authority.host, authority.port)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Authority, Domain, DomainRef, Host, HostRef};

    #[test]
    fn properties() {
        let authority: Authority = (Domain::localhost(), 80).into();
        assert_eq!(authority.host(), HostRef::Name(DomainRef::LOCALHOST));
        assert_eq!(authority.port(), 80);
    }

    #[test]
    pub fn export() {
        let authority: Authority = (Domain::localhost(), 80).into();
        let (host, port) = authority.into();
        assert_eq!(host, Host::Name(Domain::localhost()));
        assert_eq!(port, 80);
    }
}
