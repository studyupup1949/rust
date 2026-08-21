use std::str::FromStr;

use crate::{Domain, Endpoint};

impl FromStr for Endpoint {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (s, port) = crate::parse::parse_port(s)?;
        Ok(Endpoint::new(Domain::from_str(s)?, port))
    }
}
