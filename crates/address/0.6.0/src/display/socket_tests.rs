use crate::{IPv4Address, IPv6Address};

#[test]
fn display_socket_v4() {
    let result: String = IPv4Address::LOCALHOST.to_socket_v4(80).to_string();
    assert_eq!(result, "127.0.0.1:80");
}

#[test]
fn display_socket_v6() {
    let result: String = IPv6Address::LOCALHOST.to_socket_v6(80).to_string();
    assert_eq!(result, "[::1]:80");
}

#[test]
fn display_socket() {
    let result: String = IPv4Address::LOCALHOST.to_socket(80).to_string();
    assert_eq!(result, "127.0.0.1:80");

    let result: String = IPv6Address::LOCALHOST.to_socket(80).to_string();
    assert_eq!(result, "[::1]:80");
}
