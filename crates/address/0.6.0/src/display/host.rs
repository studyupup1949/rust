use std::fmt::{Display, Formatter};

use crate::Host;

impl Display for Host {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Name(domain) => write!(f, "{}", domain),
            Self::Address(ip) => write!(f, "{}", ip),
        }
    }
}
