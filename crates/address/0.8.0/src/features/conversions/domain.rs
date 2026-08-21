use crate::{Domain, DomainRef, Endpoint, EndpointRef, Host, HostRef};

impl Domain {
    /// Converts the domain to a domain reference.
    pub fn to_ref(&self) -> DomainRef {
        unsafe { DomainRef::new(self.name()) }
    }

    /// Converts the domain to a host.
    pub fn to_host(self) -> Host {
        Host::Name(self)
    }

    /// Converts the domain to an endpoint with the port.
    pub fn to_endpoint(self, port: u16) -> Endpoint {
        Endpoint::new(self, port)
    }
}

impl TryFrom<String> for Domain {
    type Error = ();

    fn try_from(mut name: String) -> Result<Self, Self::Error> {
        name.make_ascii_lowercase();
        if Self::is_valid_name_str(name.as_str(), false) {
            Ok(unsafe { Self::new(name) })
        } else {
            Err(())
        }
    }
}

impl TryFrom<&str> for Domain {
    type Error = ();

    fn try_from(name: &str) -> Result<Self, Self::Error> {
        if Self::is_valid_name_str(name, false) {
            Ok(unsafe { Self::new(name) })
        } else if Self::is_valid_name_str(name, true) {
            let name: String = name.to_ascii_lowercase();
            Ok(unsafe { Self::new(name) })
        } else {
            Err(())
        }
    }
}

impl TryFrom<Vec<u8>> for Domain {
    type Error = ();

    fn try_from(name: Vec<u8>) -> Result<Self, Self::Error> {
        if Self::is_valid_name(name.as_slice(), false) {
            let name: String = unsafe { String::from_utf8_unchecked(name) };
            Ok(unsafe { Self::new(name) })
        } else if Self::is_valid_name(name.as_slice(), true) {
            let mut name: String = unsafe { String::from_utf8_unchecked(name) };
            name.make_ascii_lowercase();
            Ok(unsafe { Self::new(name) })
        } else {
            Err(())
        }
    }
}

impl TryFrom<&[u8]> for Domain {
    type Error = ();

    fn try_from(name: &[u8]) -> Result<Self, Self::Error> {
        if Self::is_valid_name(name, false) {
            let name: String = unsafe { std::str::from_utf8_unchecked(name) }.to_string();
            Ok(unsafe { Self::new(name) })
        } else if Self::is_valid_name(name, true) {
            let mut name: String = unsafe { std::str::from_utf8_unchecked(name) }.to_string();
            name.make_ascii_lowercase();
            Ok(unsafe { Self::new(name) })
        } else {
            Err(())
        }
    }
}

impl<'a> DomainRef<'a> {
    /// Converts the domain reference to a domain.
    pub fn to_domain(&self) -> Domain {
        unsafe { Domain::new(self.name()) }
    }

    /// Converts the domain reference to a host reference.
    pub const fn to_host(&self) -> HostRef {
        HostRef::Name(*self)
    }

    /// Converts the domain reference to an endpoint reference with the port.
    pub fn to_endpoint(&self, port: u16) -> EndpointRef {
        EndpointRef::new(*self, port)
    }
}

impl<'a> TryFrom<&'a str> for DomainRef<'a> {
    type Error = ();

    fn try_from(name: &'a str) -> Result<Self, Self::Error> {
        if Domain::is_valid_name_str(name, false) {
            Ok(unsafe { Self::new(name) })
        } else {
            Err(())
        }
    }
}

impl<'a> TryFrom<&'a [u8]> for DomainRef<'a> {
    type Error = ();

    fn try_from(name: &'a [u8]) -> Result<Self, Self::Error> {
        if Domain::is_valid_name(name, false) {
            let name: &str = unsafe { std::str::from_utf8_unchecked(name) };
            Ok(unsafe { Self::new(name) })
        } else {
            Err(())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Domain, DomainRef, Endpoint, EndpointRef, Host, HostRef};

    #[test]
    fn domain_to_ref() {
        let domain: Domain = Domain::localhost();
        let result: DomainRef = domain.to_ref();
        assert_eq!(result, DomainRef::LOCALHOST);
    }

    #[test]
    fn domain_to_host() {
        let domain: Domain = Domain::localhost();
        let result: Host = domain.to_host();
        let expected: Host = Host::Name(Domain::localhost());
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
    fn try_from_string() {
        let result: Result<Domain, ()> = Domain::try_from("localhost".to_string());
        assert_eq!(result, Ok(Domain::localhost()));

        let result: Result<Domain, ()> = Domain::try_from("LocalHost".to_string());
        assert_eq!(result, Ok(Domain::localhost()));

        let result: Result<Domain, ()> = Domain::try_from("local!host".to_string());
        assert_eq!(result, Err(()));
    }

    #[test]
    fn try_from_str() {
        let result: Result<Domain, ()> = Domain::try_from("localhost");
        assert_eq!(result, Ok(Domain::localhost()));

        let result: Result<Domain, ()> = Domain::try_from("LocalHost");
        assert_eq!(result, Ok(Domain::localhost()));

        let result: Result<Domain, ()> = Domain::try_from("local!host");
        assert_eq!(result, Err(()));
    }

    #[test]
    fn try_from_vec() {
        let result: Result<Domain, ()> = Domain::try_from("localhost".as_bytes().to_vec());
        assert_eq!(result, Ok(Domain::localhost()));

        let result: Result<Domain, ()> = Domain::try_from("LocalHost".as_bytes().to_vec());
        assert_eq!(result, Ok(Domain::localhost()));

        let result: Result<Domain, ()> = Domain::try_from("local!host".as_bytes().to_vec());
        assert_eq!(result, Err(()));
    }

    #[test]
    fn try_from_slice() {
        let result: Result<Domain, ()> = Domain::try_from("localhost".as_bytes());
        assert_eq!(result, Ok(Domain::localhost()));

        let result: Result<Domain, ()> = Domain::try_from("LocalHost".as_bytes());
        assert_eq!(result, Ok(Domain::localhost()));

        let result: Result<Domain, ()> = Domain::try_from("local!host".as_bytes());
        assert_eq!(result, Err(()));
    }

    #[test]
    fn ref_to_domain() {
        let domain: DomainRef = DomainRef::LOCALHOST;
        let result: Domain = domain.to_domain();
        assert_eq!(result, Domain::localhost());
    }

    #[test]
    fn ref_to_host() {
        let domain: DomainRef = DomainRef::LOCALHOST;
        let result: HostRef = domain.to_host();
        let expected: HostRef = HostRef::Name(DomainRef::LOCALHOST);
        assert_eq!(result, expected);
    }

    #[test]
    fn ref_to_endpoint() {
        let domain: DomainRef = DomainRef::LOCALHOST;
        let result: EndpointRef = domain.to_endpoint(80);
        let expected: EndpointRef = EndpointRef::new(DomainRef::LOCALHOST, 80);
        assert_eq!(result, expected);
    }

    #[test]
    fn ref_try_from_str() {
        let result: Result<DomainRef, ()> = DomainRef::try_from("localhost");
        assert_eq!(result, Ok(DomainRef::LOCALHOST));

        let result: Result<DomainRef, ()> = DomainRef::try_from("LocalHost");
        assert_eq!(result, Err(()));

        let result: Result<DomainRef, ()> = DomainRef::try_from("local!host");
        assert_eq!(result, Err(()));
    }

    #[test]
    fn ref_try_from_slice() {
        let result: Result<DomainRef, ()> = DomainRef::try_from("localhost".as_bytes());
        assert_eq!(result, Ok(DomainRef::LOCALHOST));

        let result: Result<DomainRef, ()> = DomainRef::try_from("LocalHost".as_bytes());
        assert_eq!(result, Err(()));

        let result: Result<DomainRef, ()> = DomainRef::try_from("local!host".as_bytes());
        assert_eq!(result, Err(()));
    }
}
