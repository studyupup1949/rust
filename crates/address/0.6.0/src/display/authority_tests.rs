use crate::{Domain, IPv4Address, IPv6Address};

#[test]
fn display_authority() {
    let result: String = Domain::localhost().to_authority(80).to_string();
    assert_eq!(result, "localhost:80");

    let result: String = IPv4Address::LOCALHOST.to_authority(80).to_string();
    assert_eq!(result, "127.0.0.1:80");

    let result: String = IPv6Address::LOCALHOST.to_authority(80).to_string();
    assert_eq!(result, "[::1]:80");
}
