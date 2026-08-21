use std::str::FromStr;

use crate::features::parse::util::extract_port;
use crate::{Domain, Endpoint};

impl FromStr for Endpoint {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (s, port) = extract_port(s)?;
        let domain: Domain = Domain::from_str(s)?;
        Ok(Endpoint::new(domain, port))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{Domain, Endpoint};

    #[test]
    fn endpoint() {
        let test_cases: &[(&str, Result<Endpoint, ()>)] = &[
            ("", Err(())),
            ("localhost", Err(())),
            ("localhost:", Err(())),
            ("localhost:x", Err(())),
            ("80", Err(())),
            (":80", Err(())),
            ("$@#:80", Err(())),
            ("127.0.0.1:80", Err(())),
            ("localhost:80", Ok(Domain::localhost().to_endpoint(80))),
            ("LocalHost:80", Ok(Domain::localhost().to_endpoint(80))),
        ];
        for (s, expected) in test_cases {
            let result = Endpoint::from_str(*s);
            assert_eq!(result, *expected, "{}", *s);
        }
    }
}
