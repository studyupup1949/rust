use std::fmt::{Display, Formatter};

use crate::Endpoint;

impl Display for Endpoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.domain(), self.port())
    }
}
