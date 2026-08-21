use crate::{IPv4Address, SocketAddressV4};

#[test]
fn new() {
    let socket: SocketAddressV4 = SocketAddressV4::new(IPv4Address::LOCALHOST, 80);
    assert_eq!(socket.ip(), IPv4Address::LOCALHOST);
    assert_eq!(socket.port(), 80);
}

#[test]
fn ip() {
    let socket: SocketAddressV4 = SocketAddressV4::new(IPv4Address::LOCALHOST, 80);
    assert_eq!(socket.ip(), IPv4Address::LOCALHOST);
}

#[test]
fn port() {
    let socket: SocketAddressV4 = SocketAddressV4::new(IPv4Address::LOCALHOST, 80);
    assert_eq!(socket.port(), 80);
}
