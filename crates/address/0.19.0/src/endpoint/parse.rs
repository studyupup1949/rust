use crate::parse_port;
use crate::{DomainRef, Endpoint, EndpointRef, ParseError};
use std::str::FromStr;

impl FromStr for Endpoint {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.bytes().any(|b| b.is_ascii_uppercase()) {
            let lower: String = s.to_ascii_lowercase();
            Ok(EndpointRef::try_from(lower.as_str())?.to_endpoint())
        } else {
            Ok(EndpointRef::try_from(s)?.to_endpoint())
        }
    }
}

impl<'a> TryFrom<&'a str> for EndpointRef<'a> {
    type Error = ParseError;

    /// The domain name must already be lowercase, since a borrowed name cannot be normalized. Use
    /// `Endpoint::from_str` to parse mixed-case domain names.
    fn try_from(endpoint: &'a str) -> Result<Self, Self::Error> {
        Self::try_from(endpoint.as_bytes())
    }
}

impl<'a> TryFrom<&'a [u8]> for EndpointRef<'a> {
    type Error = ParseError;

    /// The domain name must already be lowercase, since a borrowed name cannot be normalized. Use
    /// `Endpoint::from_str` to parse mixed-case domain names.
    fn try_from(endpoint: &'a [u8]) -> Result<Self, Self::Error> {
        let (domain, port) = parse_port(endpoint)?;
        let domain: DomainRef = DomainRef::try_from(domain)?;
        Ok(EndpointRef::new(domain, port))
    }
}

#[cfg(test)]
mod tests {
    use crate::ParseError::{InvalidDomain, InvalidPort};
    use crate::{DomainRef, Endpoint, EndpointRef, ParseError};
    use std::str::FromStr;

    #[test]
    fn from_str() {
        let test_cases: &[(&str, Result<Endpoint, ParseError>)] = &[
            ("", Err(InvalidPort)),
            ("localhost:", Err(InvalidPort)),
            ("localhost:xx", Err(InvalidPort)),
            (":80", Err(InvalidDomain)),
            ("[localhost]:80", Err(InvalidDomain)),
            (
                "LocalHost:80",
                Ok(DomainRef::LOCALHOST.to_domain().to_endpoint(80)),
            ),
            (
                "localhost:80",
                Ok(DomainRef::LOCALHOST.to_domain().to_endpoint(80)),
            ),
        ];

        for (input, expected) in test_cases {
            let result: Result<Endpoint, ParseError> = Endpoint::from_str(input);
            assert_eq!(result, *expected, "input={}", input);
        }
    }

    #[test]
    fn try_from_slice() {
        let result: Result<EndpointRef, ParseError> =
            EndpointRef::try_from("localhost:80".as_bytes());
        assert_eq!(result, Ok(EndpointRef::new(DomainRef::LOCALHOST, 80)));

        let result: Result<EndpointRef, ParseError> =
            EndpointRef::try_from("LocalHost:80".as_bytes());
        assert_eq!(result, Err(InvalidDomain));
    }
}
