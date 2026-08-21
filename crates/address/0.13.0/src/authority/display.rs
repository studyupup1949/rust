use std::fmt::{Display, Formatter};

use crate::{Authority, AuthorityRef, HostRef, IPAddress};

impl Display for Authority {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_ref())
    }
}

impl<'a> Display for AuthorityRef<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.host() {
            HostRef::Name(domain) => write!(f, "{}:{}", domain, self.port()),
            HostRef::Address(ip) => match ip {
                IPAddress::V4(ip) => write!(f, "{}:{}", ip, self.port()),
                IPAddress::V6(ip) => write!(f, "[{}]:{}", ip, self.port()),
            },
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

        for (host, expected) in test_cases {
            let result: String = host.to_string();
            assert_eq!(result, *expected);
        }
    }
}
