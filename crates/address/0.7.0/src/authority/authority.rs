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
        self.host.to_host_ref()
    }

    /// Gets the port.
    pub const fn port(&self) -> u16 {
        self.port
    }
}

impl Authority {
    //! Conversions

    /// Converts the authority to an authority reference.
    pub fn to_authority_ref(&self) -> AuthorityRef {
        AuthorityRef::new(self.host(), self.port)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Authority, Domain, DomainRef, HostRef};

    #[test]
    fn properties() {
        let authority: Authority = (Domain::localhost(), 80).into();
        assert_eq!(authority.host(), HostRef::Name(DomainRef::LOCALHOST));
        assert_eq!(authority.port(), 80);
    }
}
