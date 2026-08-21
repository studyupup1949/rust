use std::str::FromStr;

use crate::{Domain, Host, IPAddress};

impl FromStr for Host {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(ip) = IPAddress::from_str(s) {
            Ok(ip.to_host())
        } else if let Ok(domain) = Domain::from_str(s) {
            Ok(domain.to_host())
        } else {
            Err(())
        }
    }
}
