use std::fmt::{Display, Formatter};

use crate::{Authority, AuthorityRef, HostRef, IPAddress};

impl Display for Authority {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_authority_ref())
    }
}

impl<'a> Display for AuthorityRef<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.host() {
            HostRef::Name(domain) => write!(f, "{}", domain),
            HostRef::Address(ip) => match ip {
                IPAddress::V4(ip) => write!(f, "{}", ip),
                IPAddress::V6(ip) => write!(f, "[{}]", ip),
            },
        }?;
        write!(f, ":{}", self.port())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Authority, Domain, IPv4Address, IPv6Address};

    #[test]
    fn display() {
        let authority: Authority = (Domain::localhost(), 80).into();
        assert_eq!(authority.to_string(), "localhost:80");

        let authority: Authority = (IPv4Address::LOCALHOST, 80).into();
        assert_eq!(authority.to_string(), "127.0.0.1:80");

        let authority: Authority = (IPv6Address::LOCALHOST, 80).into();
        assert_eq!(authority.to_string(), "[::1]:80");
    }
}
