use std::str::FromStr;

use crate::Domain;

impl FromStr for Domain {
    type Err = ();

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        if Domain::is_valid_name_str(name, false) {
            Ok(unsafe { Domain::new(name) })
        } else if Domain::is_valid_name_str(name, true) {
            let name: String = name.to_ascii_lowercase();
            Ok(unsafe { Domain::new(name) })
        } else {
            Err(())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::Domain;

    #[test]
    fn domain() {
        let test_cases: &[(&str, Result<Domain, ()>)] = &[
            ("", Err(())),
            ("invalid!", Err(())),
            ("localhost", Ok(Domain::localhost())),
            ("LocalHost", Ok(Domain::localhost())),
        ];
        for (s, expected) in test_cases {
            let result: Result<Domain, ()> = Domain::from_str(*s);
            assert_eq!(result, *expected);
        }
    }
}
