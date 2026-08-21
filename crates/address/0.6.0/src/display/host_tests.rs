use crate::{Domain, IPv4Address, IPv6Address};

#[test]
fn display_host() {
    let result: String = IPv4Address::LOCALHOST.to_host().to_string();
    assert_eq!(result, "127.0.0.1");

    let result: String = IPv6Address::LOCALHOST.to_host().to_string();
    assert_eq!(result, "::1");

    let result: String = Domain::localhost().to_host().to_string();
    assert_eq!(result, "localhost");
}
