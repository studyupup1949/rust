use crate::ParseError::InvalidHost;
use crate::parse_lowercase;
use crate::{DomainRef, Host, HostRef, IPAddress, ParseError};
use std::str::FromStr;

impl FromStr for Host {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_lowercase(s, |s| Ok(HostRef::try_from(s)?.to_host()))
    }
}

impl<'a> TryFrom<&'a str> for HostRef<'a> {
    type Error = ParseError;

    /// A domain name must already be lowercase, since a borrowed name cannot be normalized. Use `Host::from_str` to
    /// parse mixed-case domain names.
    fn try_from(host: &'a str) -> Result<Self, Self::Error> {
        Self::try_from(host.as_bytes())
    }
}

impl<'a> TryFrom<&'a [u8]> for HostRef<'a> {
    type Error = ParseError;

    /// A domain name must already be lowercase, since a borrowed name cannot be normalized. Use `Host::from_str` to
    /// parse mixed-case domain names.
    fn try_from(host: &'a [u8]) -> Result<Self, Self::Error> {
        if let Ok(ip) = IPAddress::parse(host) {
            Ok(ip.to_host_ref())
        } else if let Ok(domain) = DomainRef::try_from(host) {
            Ok(domain.to_host_ref())
        } else {
            Err(InvalidHost)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ParseError::InvalidHost;
    use crate::{Domain, DomainRef, Host, HostRef, IPv4Address, IPv6Address, ParseError};
    use std::str::FromStr;

    #[test]
    fn from_str() {
        let test_cases: &[(&str, Result<Host, ParseError>)] = &[
            ("", Err(InvalidHost)),
            ("localhost", Ok(Domain::localhost().to_host())),
            ("LocalHost", Ok(Domain::localhost().to_host())),
            ("127.0.0.1", Ok(IPv4Address::LOCALHOST.to_host())),
            ("::1", Ok(IPv6Address::LOCALHOST.to_host())),
            ("[::1]", Err(InvalidHost)),
        ];

        for (input, expected) in test_cases {
            let result: Result<Host, ParseError> = Host::from_str(input);
            assert_eq!(result, *expected, "input={}", input);
        }
    }

    #[test]
    fn try_from_slice() {
        let test_cases: &[(&str, Result<HostRef, ParseError>)] = &[
            ("localhost", Ok(HostRef::Name(DomainRef::LOCALHOST))),
            ("127.0.0.1", Ok(IPv4Address::LOCALHOST.to_host_ref())),
            ("LocalHost", Err(InvalidHost)),
        ];

        for (input, expected) in test_cases {
            let result: Result<HostRef, ParseError> = HostRef::try_from(input.as_bytes());
            assert_eq!(result, *expected, "input={}", input);
        }
    }
}
