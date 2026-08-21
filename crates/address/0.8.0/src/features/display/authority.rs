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
                IPAddress::V4(v4) => write!(f, "{}:{}", v4, self.port()),
                IPAddress::V6(v6) => write!(f, "[{}]:{}", v6, self.port()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Authority, AuthorityRef, Domain, IPv4Address, IPv6Address};

    #[test]
    fn authority() {
        let authority: Authority = Domain::localhost().to_endpoint(80).to_authority();
        assert_eq!(authority.to_string(), "localhost:80");

        let authority: Authority = IPv4Address::LOCALHOST.to_host().to_authority(80);
        assert_eq!(authority.to_string(), "127.0.0.1:80");

        let authority: Authority = IPv6Address::LOCALHOST.to_host().to_authority(80);
        assert_eq!(authority.to_string(), "[::1]:80");
    }

    #[test]
    fn authority_ref() {
        let authority: Authority = Domain::localhost().to_endpoint(80).to_authority();
        let authority: AuthorityRef = authority.to_ref();
        assert_eq!(authority.to_string(), "localhost:80");

        let authority: Authority = IPv4Address::LOCALHOST.to_host().to_authority(80);
        let authority: AuthorityRef = authority.to_ref();
        assert_eq!(authority.to_string(), "127.0.0.1:80");

        let authority: Authority = IPv6Address::LOCALHOST.to_host().to_authority(80);
        let authority: AuthorityRef = authority.to_ref();
        assert_eq!(authority.to_string(), "[::1]:80");
    }
}
