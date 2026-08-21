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
    use crate::{Domain, Endpoint};

    #[test]
    fn display() {
        let endpoint: Endpoint = Domain::localhost().to_endpoint(80);

        let result: String = endpoint.to_string();
        let expected: &str = "localhost:80";
        assert_eq!(result, expected);
    }
}
