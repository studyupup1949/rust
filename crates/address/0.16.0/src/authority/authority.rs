use crate::{AuthorityRef, Host, HostRef};

/// A host with an associated port.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct Authority {
    host: Host,
    port: u16,
}

impl Authority {
    //! Construction

    /// Creates a new authority.
    pub const fn new(host: Host, port: u16) -> Self {
        Self { host, port }
    }
}

impl<H: Into<Host>> From<(H, u16)> for Authority {
    fn from(tuple: (H, u16)) -> Self {
        Self::new(tuple.0.into(), tuple.1)
    }
}

impl<'a> From<AuthorityRef<'a>> for Authority {
    fn from(authority: AuthorityRef<'a>) -> Self {
        Self {
            host: authority.host().to_host(),
            port: authority.port(),
        }
    }
}

impl From<Authority> for (Host, u16) {
    fn from(authority: Authority) -> Self {
        (authority.host, authority.port)
    }
}

impl Authority {
    //! Properties

    /// Gets the host reference.
    pub fn host(&self) -> HostRef<'_> {
        self.host.to_ref()
    }

    /// Gets the port.
    pub const fn port(&self) -> u16 {
        self.port
    }
}

#[cfg(test)]
mod tests {
    use crate::{Authority, Domain, DomainRef, Host, HostRef};

    #[test]
    fn construction() {
        let authority: Authority = Authority::new(Host::Name(Domain::localhost()), 80);
        assert_eq!(authority.host, Host::Name(Domain::localhost()));
        assert_eq!(authority.port, 80);

        let authority: Authority = (Domain::localhost(), 80).into();
        assert_eq!(authority.host, Host::Name(Domain::localhost()));
        assert_eq!(authority.port, 80);

        let authority: Authority = Authority::new(Host::Name(Domain::localhost()), 80);
        let authority: Authority = authority.to_ref().into();
        assert_eq!(authority.host, Host::Name(Domain::localhost()));
        assert_eq!(authority.port, 80);
    }

    #[test]
    fn deconstruction() {
        let authority: Authority = (Domain::localhost(), 80).into();
        let (host, port) = authority.into();
        assert_eq!(host, Host::Name(Domain::localhost()));
        assert_eq!(port, 80);
    }

    #[test]
    fn properties() {
        let authority: Authority = Authority::new(Host::Name(Domain::localhost()), 80);
        assert_eq!(authority.host(), HostRef::Name(DomainRef::LOCALHOST));
        assert_eq!(authority.port(), 80);
    }
}
