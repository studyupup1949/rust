use std::fmt::{Display, Formatter};

use crate::{Host, HostRef};

impl Display for Host {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_host_ref())
    }
}

impl<'a> Display for HostRef<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Name(domain) => write!(f, "{}", domain),
            Self::Address(ip) => write!(f, "{}", ip),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Domain, DomainRef, Host, HostRef, IPv4Address};

    #[test]
    fn host() {
        assert_eq!(Host::Name(Domain::localhost()).to_string(), "localhost");
        assert_eq!(
            Host::Address(IPv4Address::LOCALHOST.to_ip()).to_string(),
            "127.0.0.1"
        );
    }

    #[test]
    fn host_ref() {
        assert_eq!(HostRef::Name(DomainRef::LOCALHOST).to_string(), "localhost");
        assert_eq!(
            HostRef::Address(IPv4Address::LOCALHOST.to_ip()).to_string(),
            "127.0.0.1"
        );
    }
}
