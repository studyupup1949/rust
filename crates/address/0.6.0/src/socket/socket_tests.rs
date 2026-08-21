use crate::{IPAddress, IPv4Address, IPv6Address, SocketAddress};

#[test]
fn new() {
    let socket: SocketAddress = SocketAddress::new(IPAddress::V4(IPv4Address::LOCALHOST), 80);
    assert_eq!(socket.ip(), IPAddress::V4(IPv4Address::LOCALHOST));
    assert_eq!(socket.port(), 80);

    let socket: SocketAddress = SocketAddress::new(IPAddress::V6(IPv6Address::LOCALHOST), 80);
    assert_eq!(socket.ip(), IPAddress::V6(IPv6Address::LOCALHOST));
    assert_eq!(socket.port(), 80);
}

#[test]
fn is_v4() {
    let socket: SocketAddress = SocketAddress::new(IPAddress::V4(IPv4Address::LOCALHOST), 80);
    assert_eq!(socket.is_v4(), true);

    let socket: SocketAddress = SocketAddress::new(IPAddress::V6(IPv6Address::LOCALHOST), 80);
    assert_eq!(socket.is_v4(), false);
}

#[test]
fn is_v6() {
    let socket: SocketAddress = SocketAddress::new(IPAddress::V4(IPv4Address::LOCALHOST), 80);
    assert_eq!(socket.is_v6(), false);

    let socket: SocketAddress = SocketAddress::new(IPAddress::V6(IPv6Address::LOCALHOST), 80);
    assert_eq!(socket.is_v6(), true);
}

#[test]
fn ip() {
    let socket: SocketAddress = SocketAddress::new(IPAddress::V4(IPv4Address::LOCALHOST), 80);
    assert_eq!(socket.ip(), IPAddress::V4(IPv4Address::LOCALHOST));

    let socket: SocketAddress = SocketAddress::new(IPAddress::V6(IPv6Address::LOCALHOST), 80);
    assert_eq!(socket.ip(), IPAddress::V6(IPv6Address::LOCALHOST));
}

#[test]
fn port() {
    let socket: SocketAddress = SocketAddress::new(IPAddress::V4(IPv4Address::LOCALHOST), 80);
    assert_eq!(socket.port(), 80);

    let socket: SocketAddress = SocketAddress::new(IPAddress::V6(IPv6Address::LOCALHOST), 80);
    assert_eq!(socket.port(), 80);
}
