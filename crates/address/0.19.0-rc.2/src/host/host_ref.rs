use crate::ParseError::InvalidHost;
use crate::{DomainRef, IPAddress, ParseError};
use std::str::FromStr;

/// Either a domain reference or an IP address.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum HostRef<'a> {
    /// A domain reference.
    Name(DomainRef<'a>),

    /// An IP address.
    Address(IPAddress),
}

impl<'a> From<DomainRef<'a>> for HostRef<'a> {
    fn from(domain: DomainRef<'a>) -> Self {
        Self::Name(domain)
    }
}

impl<'a, A: Into<IPAddress>> From<A> for HostRef<'a> {
    fn from(ip: A) -> Self {
        Self::Address(ip.into())
    }
}

impl<'a> TryFrom<&'a str> for HostRef<'a> {
    type Error = ParseError;

    /// A domain name must already be lowercase, since a borrowed name cannot be normalized. Use
    /// `Host::from_str` to parse mixed-case domain names.
    fn try_from(host: &'a str) -> Result<Self, Self::Error> {
        if let Ok(ip) = IPAddress::from_str(host) {
            Ok(ip.to_host_ref())
        } else if let Ok(domain) = DomainRef::try_from(host) {
            Ok(domain.to_host_ref())
        } else {
            Err(InvalidHost)
        }
    }
}

impl<'a> HostRef<'a> {
    //! Matching

    /// Checks if the host is a domain.
    pub const fn is_domain(self) -> bool {
        matches!(self, Self::Name(_))
    }

    /// Checks if the host is an IP address.
    pub const fn is_ip(self) -> bool {
        matches!(self, Self::Address(_))
    }
}

#[cfg(test)]
mod tests {
    use crate::{DomainRef, HostRef, IPAddress, IPv4Address};

    #[test]
    fn construction() {
        let result: HostRef = DomainRef::LOCALHOST.into();
        let expected: HostRef = HostRef::Name(DomainRef::LOCALHOST);
        assert_eq!(result, expected);

        let result: HostRef = IPAddress::V4(IPv4Address::LOCALHOST).into();
        let expected: HostRef = HostRef::Address(IPAddress::V4(IPv4Address::LOCALHOST));
        assert_eq!(result, expected);
    }

    #[test]
    fn matching() {
        let host: HostRef = DomainRef::LOCALHOST.into();
        assert!(host.is_domain());
        assert!(!host.is_ip());

        let host: HostRef = IPv4Address::LOCALHOST.into();
        assert!(!host.is_domain());
        assert!(host.is_ip());
    }
}
