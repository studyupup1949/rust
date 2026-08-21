use crate::{Domain, DomainRef, Endpoint, EndpointRef, Host, HostRef};
use std::borrow::Borrow;

impl Domain {
    //! Conversions

    /// Converts the domain to a domain reference.
    pub fn to_ref(&self) -> DomainRef<'_> {
        unsafe { DomainRef::new_unchecked(self.name()) }
    }

    /// Converts the domain to an endpoint with the `port`.
    pub const fn to_endpoint(self, port: u16) -> Endpoint {
        Endpoint::new(self, port)
    }

    /// Converts the domain to a host.
    pub const fn to_host(self) -> Host {
        Host::Name(self)
    }
}

impl<'a> DomainRef<'a> {
    //! Conversions

    /// Converts the domain reference to a domain.
    pub fn to_domain(self) -> Domain {
        unsafe { Domain::new_unchecked(self.name()) }
    }

    /// Converts the domain reference to an endpoint reference with the `port`.
    pub const fn to_endpoint_ref(self, port: u16) -> EndpointRef<'a> {
        EndpointRef::new(self, port)
    }

    /// Converts the domain reference to a host reference.
    pub const fn to_host_ref(self) -> HostRef<'a> {
        HostRef::Name(self)
    }
}

impl AsRef<str> for Domain {
    fn as_ref(&self) -> &str {
        self.name()
    }
}

impl Borrow<str> for Domain {
    fn borrow(&self) -> &str {
        self.name()
    }
}

impl<'a> AsRef<str> for DomainRef<'a> {
    fn as_ref(&self) -> &str {
        self.name()
    }
}

impl<'a> Borrow<str> for DomainRef<'a> {
    fn borrow(&self) -> &str {
        self.name()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Domain, DomainRef, Endpoint, EndpointRef, Host, HostRef};

    #[test]
    fn domain_to_ref() {
        let domain: Domain = Domain::localhost();

        let result: DomainRef = domain.to_ref();
        let expected: DomainRef = DomainRef::LOCALHOST;
        assert_eq!(result, expected);
    }

    #[test]
    fn domain_to_endpoint() {
        let domain: Domain = Domain::localhost();

        let result: Endpoint = domain.to_endpoint(80);
        let expected: Endpoint = Endpoint::new(Domain::localhost(), 80);
        assert_eq!(result, expected);
    }

    #[test]
    fn domain_to_host() {
        let domain: Domain = Domain::localhost();

        let result: Host = domain.to_host();
        let expected: Host = Host::Name(Domain::localhost());
        assert_eq!(result, expected);
    }

    #[test]
    fn ref_to_domain() {
        let domain: DomainRef = DomainRef::LOCALHOST;

        let result: Domain = domain.to_domain();
        let expected: Domain = Domain::localhost();
        assert_eq!(result, expected);
    }

    #[test]
    fn ref_to_endpoint() {
        let domain: DomainRef = DomainRef::LOCALHOST;

        let result: EndpointRef = domain.to_endpoint_ref(80);
        let expected: EndpointRef = EndpointRef::new(DomainRef::LOCALHOST, 80);
        assert_eq!(result, expected);
    }

    #[test]
    fn ref_to_host() {
        let domain: DomainRef = DomainRef::LOCALHOST;

        let result: HostRef = domain.to_host_ref();
        let expected: HostRef = HostRef::Name(DomainRef::LOCALHOST);
        assert_eq!(result, expected);
    }

    #[test]
    fn as_str() {
        let owned: Domain = Domain::localhost();
        let result: &str = owned.as_ref();
        assert_eq!(result, "localhost");

        let domain: DomainRef = DomainRef::LOCALHOST;
        let result: &str = domain.as_ref();
        assert_eq!(result, "localhost");
    }
}
