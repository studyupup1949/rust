use std::fmt::{Display, Formatter};

use crate::{Host, HostRef};

impl Display for Host {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_ref())
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
    use crate::{Domain, Host, HostRef, IPv4Address};

    #[test]
    fn host() {
        let host: Host = IPv4Address::LOCALHOST.to_host();
        assert_eq!(host.to_string(), "127.0.0.1");

        let host: Host = Domain::localhost().to_host();
        assert_eq!(host.to_string(), "localhost");
    }

    #[test]
    fn host_ref() {
        let host: Host = IPv4Address::LOCALHOST.to_host();
        let host: HostRef = host.to_ref();
        assert_eq!(host.to_string(), "127.0.0.1");

        let host: Host = Domain::localhost().to_host();
        let host: HostRef = host.to_ref();
        assert_eq!(host.to_string(), "localhost");
    }
}
