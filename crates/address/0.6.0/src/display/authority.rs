use std::fmt::{Display, Formatter};

use crate::{Authority, Host, IPAddress};

impl Display for Authority {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.host() {
            Host::Name(domain) => write!(f, "{}:{}", domain, self.port()),
            Host::Address(ip) => match ip {
                IPAddress::V4(v4) => write!(f, "{}:{}", v4, self.port()),
                IPAddress::V6(v6) => write!(f, "[{}]:{}", v6, self.port()),
            },
        }
    }
}
