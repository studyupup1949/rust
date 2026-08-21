use crate::{Authority, AuthorityRef, EndpointRef, HostRef};
use std::fmt::{Display, Formatter};

impl Display for Authority {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.to_ref().fmt(f)
    }
}

impl<'a> Display for AuthorityRef<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.host() {
            HostRef::Name(domain) => EndpointRef::new(domain, self.port()).fmt(f),
            HostRef::Address(ip) => ip.to_socket(self.port()).fmt(f),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Authority, Domain, IPv4Address, IPv6Address};

    #[test]
    fn display() {
        let test_cases: &[(Authority, &str)] = &[
            (
                Domain::localhost().to_host().to_authority(80),
                "localhost:80",
            ),
            (
                IPv4Address::LOCALHOST.to_host().to_authority(80),
                "127.0.0.1:80",
            ),
            (
                IPv6Address::LOCALHOST.to_host().to_authority(80),
                "[::1]:80",
            ),
        ];

        for (authority, expected) in test_cases {
            let result: String = authority.to_string();
            assert_eq!(result, *expected);
        }
    }
}
