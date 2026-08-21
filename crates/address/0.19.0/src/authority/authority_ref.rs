use crate::{Authority, HostRef};

/// A host reference with an associated port.
#[must_use]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct AuthorityRef<'a> {
    host: HostRef<'a>,
    port: u16,
}

impl<'a> AuthorityRef<'a> {
    //! Construction

    /// Creates a new authority reference.
    pub const fn new(host: HostRef<'a>, port: u16) -> Self {
        Self { host, port }
    }
}

impl<'a, H: Into<HostRef<'a>>> From<(H, u16)> for AuthorityRef<'a> {
    fn from(tuple: (H, u16)) -> Self {
        Self::new(tuple.0.into(), tuple.1)
    }
}

impl<'a> From<AuthorityRef<'a>> for (HostRef<'a>, u16) {
    fn from(authority: AuthorityRef<'a>) -> Self {
        (authority.host, authority.port)
    }
}

impl<'a> From<&'a Authority> for AuthorityRef<'a> {
    fn from(authority: &'a Authority) -> Self {
        authority.to_ref()
    }
}

impl<'a> PartialEq<Authority> for AuthorityRef<'a> {
    fn eq(&self, other: &Authority) -> bool {
        *self == other.to_ref()
    }
}

impl<'a> AuthorityRef<'a> {
    //! Properties

    /// Gets the host reference.
    pub const fn host(self) -> HostRef<'a> {
        self.host
    }

    /// Gets the port.
    #[must_use]
    pub const fn port(self) -> u16 {
        self.port
    }
}

#[cfg(test)]
mod tests {
    use crate::{Authority, AuthorityRef, Domain, DomainRef, Host, HostRef};

    #[test]
    fn construction() {
        let authority: AuthorityRef = AuthorityRef::new(HostRef::Name(DomainRef::LOCALHOST), 80);
        assert_eq!(authority.host, HostRef::Name(DomainRef::LOCALHOST));
        assert_eq!(authority.port, 80);

        let authority: AuthorityRef = (DomainRef::LOCALHOST, 80).into();
        assert_eq!(authority.host, HostRef::Name(DomainRef::LOCALHOST));
        assert_eq!(authority.port, 80);

        let owned: Authority = Authority::new(Host::Name(Domain::localhost()), 80);
        let authority: AuthorityRef = (&owned).into();
        assert_eq!(authority.host, HostRef::Name(DomainRef::LOCALHOST));
        assert_eq!(authority.port, 80);
    }

    #[test]
    fn deconstruction() {
        let authority: AuthorityRef = (DomainRef::LOCALHOST, 80).into();
        let (host, port) = authority.into();
        assert_eq!(host, HostRef::Name(DomainRef::LOCALHOST));
        assert_eq!(port, 80);
    }

    #[test]
    fn equality() {
        let owned: Authority = Authority::new(Host::Name(Domain::localhost()), 80);
        let authority: AuthorityRef = owned.to_ref();
        assert_eq!(authority, owned);

        let other: Authority = Authority::new(Host::Name(Domain::localhost()), 81);
        assert_ne!(authority, other);
    }

    #[test]
    fn properties() {
        let authority: AuthorityRef = AuthorityRef::new(HostRef::Name(DomainRef::LOCALHOST), 80);
        assert_eq!(authority.host(), HostRef::Name(DomainRef::LOCALHOST));
        assert_eq!(authority.port(), 80);
    }
}
