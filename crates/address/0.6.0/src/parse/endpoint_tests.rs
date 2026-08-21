use std::str::FromStr;

use crate::{Domain, Endpoint};

#[test]
fn parse_endpoint() {
    let test_cases: &[(&str, Result<Endpoint, ()>, &str)] = &[
        ("localhost", Err(()), "no port"),
        ("localhost:", Err(()), "empty port"),
        ("localhost:invalid", Err(()), "invalid port"),
        ("80", Err(()), "no domain"),
        (":80", Err(()), "empty domain"),
        ("INVALID:80", Err(()), "invalid domain"),
        (
            "localhost:80",
            Ok(Domain::localhost().to_endpoint(80)),
            "valid",
        ),
    ];
    for (s, expected, message) in test_cases {
        let result: Result<Endpoint, ()> = Endpoint::from_str(s);
        assert_eq!(result, *expected, "{} :: {}", s, message);
    }
}
