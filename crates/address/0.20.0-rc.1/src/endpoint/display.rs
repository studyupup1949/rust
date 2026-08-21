use crate::{Endpoint, EndpointRef};
use std::fmt::{Debug, Display, Formatter};

impl Debug for Endpoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for Endpoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.to_ref(), f)
    }
}

impl<'a> Debug for EndpointRef<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl<'a> Display for EndpointRef<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if f.width().is_none() && f.precision().is_none() {
            write!(f, "{}:{}", self.domain(), self.port())
        } else {
            f.pad(&format!("{}:{}", self.domain(), self.port()))
        }
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
