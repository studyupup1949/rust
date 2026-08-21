use crate::{IPv6Address, SocketAddressV6};

#[test]
fn new() {
    let socket: SocketAddressV6 = SocketAddressV6::new(IPv6Address::LOCALHOST, 80);
    assert_eq!(socket.ip(), IPv6Address::LOCALHOST);
    assert_eq!(socket.port(), 80);
}

#[test]
fn ip() {
    let socket: SocketAddressV6 = SocketAddressV6::new(IPv6Address::LOCALHOST, 80);
    assert_eq!(socket.ip(), IPv6Address::LOCALHOST);
}

#[test]
fn port() {
    let socket: SocketAddressV6 = SocketAddressV6::new(IPv6Address::LOCALHOST, 80);
    assert_eq!(socket.port(), 80);
}
