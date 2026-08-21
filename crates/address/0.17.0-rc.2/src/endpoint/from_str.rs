use std::str::FromStr;

use crate::parse_port;
use crate::{Domain, Endpoint, ParseError};

impl FromStr for Endpoint {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (domain, port) = parse_port(s)?;
        let domain: Domain = Domain::from_str(domain)?;
        Ok((domain, port).into())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::ParseError::{InvalidDomain, InvalidPort};
    use crate::{DomainRef, Endpoint, ParseError};

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
}
