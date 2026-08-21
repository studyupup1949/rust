use std::str::FromStr;

use crate::{Authority, Domain, IPv4Address, IPv6Address};

#[test]
fn parse_authority() {
    let result: Result<Authority, ()> = Authority::from_str("127.0.0.1:80");
    assert_eq!(result, Ok(IPv4Address::LOCALHOST.to_authority(80)));

    let result: Result<Authority, ()> = Authority::from_str("[::1]:80");
    assert_eq!(result, Ok(IPv6Address::LOCALHOST.to_authority(80)));

    let result: Result<Authority, ()> = Authority::from_str("localhost:80");
    assert_eq!(result, Ok(Domain::localhost().to_authority(80)));
}
