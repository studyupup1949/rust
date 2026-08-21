use std::fmt::{Display, Formatter};

use crate::{Endpoint, EndpointRef};

impl Display for Endpoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.domain(), self.port())
    }
}

impl<'a> Display for EndpointRef<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.domain(), self.port())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Domain, Endpoint};

    #[test]
    fn display() {
        let endpoint: Endpoint = (Domain::localhost(), 80).into();
        assert_eq!(endpoint.to_string(), "localhost:80");
    }
}
