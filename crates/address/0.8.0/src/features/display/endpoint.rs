use std::fmt::{Display, Formatter};

use crate::{Endpoint, EndpointRef};

impl Display for Endpoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_ref())
    }
}

impl<'a> Display for EndpointRef<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.domain(), self.port())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Domain, Endpoint, EndpointRef};

    #[test]
    fn endpoint() {
        let endpoint: Endpoint = Domain::localhost().to_endpoint(80);
        assert_eq!(endpoint.to_string(), "localhost:80");
    }

    #[test]
    fn endpoint_ref() {
        let endpoint: Endpoint = Domain::localhost().to_endpoint(80);
        let endpoint: EndpointRef = endpoint.to_ref();
        assert_eq!(endpoint.to_string(), "localhost:80");
    }
}
