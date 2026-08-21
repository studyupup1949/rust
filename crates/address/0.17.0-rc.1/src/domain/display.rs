use std::fmt::{Display, Formatter};

use crate::{Domain, DomainRef};

impl Display for Domain {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_ref())
    }
}

impl<'a> Display for DomainRef<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use crate::Domain;

    #[test]
    fn display() {
        let domain: Domain = Domain::localhost();

        let result: String = domain.to_string();
        let expected: &str = "localhost";
        assert_eq!(result, expected);
    }
}
