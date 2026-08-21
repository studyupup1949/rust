use std::fmt::{Display, Formatter};

use crate::{SocketAddress, SocketAddressV4, SocketAddressV6};

impl Display for SocketAddressV4 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.ip(), self.port())
    }
}

impl Display for SocketAddressV6 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]:{}", self.ip(), self.port())
    }
}

impl Display for SocketAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::V4(v4) => write!(f, "{}", v4),
            Self::V6(v6) => write!(f, "{}", v6),
        }
    }
}
