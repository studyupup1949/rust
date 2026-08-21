use crate::{Authority, AuthorityRef, EndpointRef, HostRef};
use std::fmt::{Debug, Display, Formatter};

impl Debug for Authority {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for Authority {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.to_ref(), f)
    }
}

impl<'a> Debug for AuthorityRef<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl<'a> Display for AuthorityRef<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.host() {
            HostRef::Name(domain) => Display::fmt(&EndpointRef::new(domain, self.port()), f),
            HostRef::Address(ip) => Display::fmt(&ip.to_socket(self.port()), f),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Authority, Domain, IPv4Address, IPv6Address};

    #[test]
    fn display() {
        let test_cases: &[(Authority, &str)] = &[
            (Domain::localhost().to_host().to_authority(80), "localhost:80"),
            (IPv4Address::LOCALHOST.to_host().to_authority(80), "127.0.0.1:80"),
            (IPv6Address::LOCALHOST.to_host().to_authority(80), "[::1]:80"),
        ];

        for (authority, expected) in test_cases {
            let result: String = authority.to_string();
            assert_eq!(result, *expected, "authority={:?}", authority);
        }
    }
}
