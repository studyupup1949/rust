use std::str::FromStr;

use crate::{IPv4Address, IPv6Address, SocketAddress, SocketAddressV4, SocketAddressV6};

#[test]
fn parse_socket_v4() {
    let test_cases: &[(&str, Result<SocketAddressV4, ()>, &str)] = &[
        ("127.0.0.1", Err(()), "no port"),
        ("127.0.0.1:", Err(()), "empty port"),
        ("127.0.0.1:invalid", Err(()), "invalid port"),
        ("80", Err(()), "no ip"),
        (":80", Err(()), "empty ip"),
        ("invalid:80", Err(()), "invalid ip"),
        ("[127.0.0.1]:80", Err(()), "bracketed ip"),
        ("::1:80", Err(()), "ipv6 unbracketed"),
        ("[::1]:80", Err(()), "ipv6 bracketed"),
        (
            "127.0.0.1:80",
            Ok(IPv4Address::LOCALHOST.to_socket_v4(80)),
            "valid",
        ),
    ];
    for (s, expected, message) in test_cases {
        let result: Result<SocketAddressV4, ()> = SocketAddressV4::from_str(s);
        assert_eq!(result, *expected, "{} :: {}", s, message);
    }
}

#[test]
fn parse_socket_v6() {
    let test_cases: &[(&str, Result<SocketAddressV6, ()>, &str)] = &[
        ("[::1]", Err(()), "no port"),
        ("[::1]:", Err(()), "empty port"),
        ("[::1]:invalid", Err(()), "invalid port"),
        ("80", Err(()), "no ip"),
        (":80", Err(()), "empty ip"),
        ("invalid:80", Err(()), "invalid ip"),
        ("::1:80", Err(()), "unbracketed ip"),
        (
            "[::1]:80",
            Ok(IPv6Address::LOCALHOST.to_socket_v6(80)),
            "valid",
        ),
        ("[127.0.0.1]:80", Err(()), "bracketed v4"),
        ("127.0.0.1:80", Err(()), "unbracketed v4"),
    ];
    for (s, expected, message) in test_cases {
        let result: Result<SocketAddressV6, ()> = SocketAddressV6::from_str(s);
        assert_eq!(result, *expected, "{} :: {}", s, message);
    }
}

#[test]
fn parse_socket() {
    let test_cases: &[(&str, Result<SocketAddress, ()>, &str)] = &[
        ("127.0.0.1", Err(()), "no port"),
        ("127.0.0.1:", Err(()), "empty port"),
        ("127.0.0.1:invalid", Err(()), "invalid port"),
        ("80", Err(()), "no ip"),
        (":80", Err(()), "empty ip"),
        ("invalid:80", Err(()), "invalid ip"),
        (
            "127.0.0.1:80",
            Ok(IPv4Address::LOCALHOST.to_socket(80)),
            "ipv4",
        ),
        ("[::1]:80", Ok(IPv6Address::LOCALHOST.to_socket(80)), "ipv6"),
        ("::1:80", Err(()), "ipv6 no brackets"),
    ];
    for (s, expected, message) in test_cases {
        let result: Result<SocketAddress, ()> = SocketAddress::from_str(s);
        assert_eq!(result, *expected, "{} :: {}", s, message);
    }
}
