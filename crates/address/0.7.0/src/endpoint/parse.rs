use std::str::FromStr;

use crate::{util, Domain, Endpoint};

impl FromStr for Endpoint {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (s, port) = util::extract_port(s)?;
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
            (
                "127.0.0.1:80",
                Ok(Endpoint::new(unsafe { Domain::new("127.0.0.1") }, 80)),
            ),
            ("[::1]:80", Err(())),
            ("invalid!:80", Err(())),
            ("localhost:80", Ok(Endpoint::new(Domain::localhost(), 80))),
        ];
        for (s, expected) in test_cases {
            let result: Result<Endpoint, ()> = Endpoint::from_str(*s);
            assert_eq!(result, *expected);
        }
    }
}
