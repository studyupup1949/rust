use std::str::FromStr;

use crate::{parse_port, Domain, Endpoint, ParseError};

impl FromStr for Endpoint {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (name, port) = parse_port(s)?;
        let name: Domain = Domain::try_from(name)?;
        Ok((name, port).into())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::ParseError::{InvalidDomain, InvalidPort};
    use crate::{Domain, Endpoint, ParseError};

    #[test]
    fn test() {
        let test_cases: &[(&str, Result<Endpoint, ParseError>)] = &[
            ("localhost", Err(InvalidPort)),
            ("80", Err(InvalidPort)),
            ("invalid!domain:80", Err(InvalidDomain)),
            ("localhost:invalid-port", Err(InvalidPort)),
            ("localhost:80", Ok(Domain::localhost().to_endpoint(80))),
            ("LocalHost:80", Ok(Domain::localhost().to_endpoint(80))),
        ];
        for (input, expected) in test_cases {
            let result: Result<Endpoint, ParseError> = Endpoint::from_str(*input);
            assert_eq!(result, *expected, "input={}", *input);
        }
    }
}
