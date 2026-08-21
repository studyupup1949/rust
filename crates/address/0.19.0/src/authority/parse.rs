use crate::ParseError::InvalidAuthority;
use crate::{Authority, AuthorityRef, HostRef, IPv6Address, ParseError};
use crate::{parse_port, strip_brackets};
use std::str::FromStr;

impl FromStr for Authority {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.bytes().any(|b| b.is_ascii_uppercase()) {
            let lower: String = s.to_ascii_lowercase();
            Ok(AuthorityRef::try_from(lower.as_str())?.to_authority())
        } else {
            Ok(AuthorityRef::try_from(s)?.to_authority())
        }
    }
}

impl<'a> TryFrom<&'a str> for AuthorityRef<'a> {
    type Error = ParseError;

    /// A domain name must already be lowercase, since a borrowed name cannot be normalized. Use
    /// `Authority::from_str` to parse mixed-case domain names.
    fn try_from(authority: &'a str) -> Result<Self, Self::Error> {
        Self::try_from(authority.as_bytes())
    }
}

impl<'a> TryFrom<&'a [u8]> for AuthorityRef<'a> {
    type Error = ParseError;

    /// A domain name must already be lowercase, since a borrowed name cannot be normalized. Use
    /// `Authority::from_str` to parse mixed-case domain names.
    fn try_from(authority: &'a [u8]) -> Result<Self, Self::Error> {
        let (s, port) = parse_port(authority)?;
        if let Some(s) = strip_brackets(s) {
            let host: HostRef = IPv6Address::parse(s)?.to_host_ref();
            Ok(host.to_authority_ref(port))
        } else {
            let host: HostRef = HostRef::try_from(s)?;
            if let HostRef::Address(ip) = host
                && ip.is_v6()
            {
                return Err(InvalidAuthority);
            }
            Ok(host.to_authority_ref(port))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ParseError::{InvalidAuthority, InvalidHost, InvalidPort};
    use crate::{
        Authority, AuthorityRef, Domain, DomainRef, HostRef, IPv4Address, IPv6Address, ParseError,
    };
    use std::str::FromStr;

    #[test]
    fn from_str() {
        let test_cases: &[(&str, Result<Authority, ParseError>)] = &[
            ("", Err(InvalidPort)),
            ("localhost:", Err(InvalidPort)),
            ("localhost:xx", Err(InvalidPort)),
            (":80", Err(InvalidHost)),
            (
                "127.0.0.1:80",
                Ok(IPv4Address::LOCALHOST.to_host().to_authority(80)),
            ),
            ("::1:80", Err(InvalidAuthority)),
            (
                "[::1]:80",
                Ok(IPv6Address::LOCALHOST.to_host().to_authority(80)),
            ),
            (
                "localhost:80",
                Ok(Domain::localhost().to_host().to_authority(80)),
            ),
            (
                "LocalHost:80",
                Ok(Domain::localhost().to_host().to_authority(80)),
            ),
        ];

        for (input, expected) in test_cases {
            let result: Result<Authority, ParseError> = Authority::from_str(input);
            assert_eq!(result, *expected, "input={}", *input);
        }
    }

    #[test]
    fn try_from_slice() {
        let result: Result<AuthorityRef, ParseError> =
            AuthorityRef::try_from("localhost:80".as_bytes());
        let expected: AuthorityRef = AuthorityRef::new(HostRef::Name(DomainRef::LOCALHOST), 80);
        assert_eq!(result, Ok(expected));

        let result: Result<AuthorityRef, ParseError> =
            AuthorityRef::try_from("LocalHost:80".as_bytes());
        assert_eq!(result, Err(InvalidHost));

        let result: Result<AuthorityRef, ParseError> =
            AuthorityRef::try_from(b"\xFF:80".as_slice());
        assert_eq!(result, Err(InvalidHost));

        let result: Result<AuthorityRef, ParseError> = AuthorityRef::try_from("ü:80".as_bytes());
        assert_eq!(result, Err(InvalidHost));
    }
}
