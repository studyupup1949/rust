use crate::{Domain, Host, IPv4Address, IPv6Address};
use std::str::FromStr;

#[test]
fn parse_host() {
    let result: Result<Host, ()> = Host::from_str("127.0.0.1");
    assert_eq!(result, Ok(IPv4Address::LOCALHOST.to_host()));

    let result: Result<Host, ()> = Host::from_str("::1");
    assert_eq!(result, Ok(IPv6Address::LOCALHOST.to_host()));

    let result: Result<Host, ()> = Host::from_str("localhost");
    assert_eq!(result, Ok(Domain::localhost().to_host()));
}
