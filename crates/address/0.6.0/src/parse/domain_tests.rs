use std::str::FromStr;

use crate::Domain;

#[test]
fn parse_domain() {
    let result: Result<Domain, ()> = Domain::from_str("localhost");
    assert_eq!(result, Ok(Domain::localhost()));

    let result: Result<Domain, ()> = Domain::from_str("LOCALHOST");
    assert_eq!(result, Err(()));
}
