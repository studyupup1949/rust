use address::{
    Authority, Domain, Endpoint, Host, IPAddress, IPv4Address, IPv6Address, SocketAddress, SocketAddressV4,
    SocketAddressV6,
};
use std::fmt::{Debug, Display};
use std::str::FromStr;

/// Parses each canonical string, checks the value displays as the exact same string, then checks the displayed string
/// parses back to an equal value.
fn assert_round_trips<T>(canonical: &[&str])
where
    T: FromStr + Display + PartialEq + Debug,
    T::Err: Debug,
{
    for s in canonical {
        let parsed: T = match T::from_str(s) {
            Ok(parsed) => parsed,
            Err(error) => panic!("failed to parse {:?}: {:?}", s, error),
        };
        let displayed: String = parsed.to_string();
        assert_eq!(displayed.as_str(), *s, "display was not canonical for {:?}", s);

        let reparsed: T = T::from_str(displayed.as_str()).unwrap();
        assert_eq!(reparsed, parsed, "reparse changed the value for {:?}", s);
    }
}

/// Displays each value and checks the displayed string parses back to an equal value.
fn assert_display_parses<T>(values: &[T])
where
    T: FromStr + Display + PartialEq + Debug,
    T::Err: Debug,
{
    for value in values {
        let displayed: String = value.to_string();
        let reparsed: T = match T::from_str(displayed.as_str()) {
            Ok(reparsed) => reparsed,
            Err(error) => panic!("failed to reparse {:?} from {:?}: {:?}", value, displayed, error),
        };
        assert_eq!(&reparsed, value, "value={:?}", value);
    }
}

#[test]
fn ipv4() {
    assert_round_trips::<IPv4Address>(&["0.0.0.0", "127.0.0.1", "1.2.3.4", "255.255.255.255"]);
}

#[test]
fn ipv6() {
    assert_round_trips::<IPv6Address>(&[
        "::",
        "::1",
        "1::",
        "1::1",
        "1:0:0:1::",
        "1:2:3:4:5:6:7:8",
        "fe80::1",
        "::ffff:1.2.3.4",
        "2001:db8::8a2e:370:7334",
        "ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff",
    ]);
}

#[test]
fn ipv6_constructed() {
    assert_display_parses(&[
        IPv6Address::from(0u128),
        IPv6Address::from(1u128),
        IPv6Address::from(u128::MAX),
        IPv6Address::from([1, 0, 0, 1, 0, 0, 0, 0]),
        IPv6Address::from([0, 0, 1, 0, 0, 0, 0, 1]),
        IPv4Address::LOCALHOST.to_v6_compatible(),
        IPv4Address::LOCALHOST.to_v6_mapped(),
    ]);
}

#[test]
fn ip() {
    assert_round_trips::<IPAddress>(&["127.0.0.1", "255.255.255.255", "::1", "fe80::1"]);
}

#[test]
fn socket_v4() {
    assert_round_trips::<SocketAddressV4>(&["0.0.0.0:0", "127.0.0.1:80", "255.255.255.255:65535"]);
}

#[test]
fn socket_v6() {
    assert_round_trips::<SocketAddressV6>(&["[::]:0", "[::1]:80", "[::ffff:1.2.3.4]:443", "[fe80::1]:65535"]);
}

#[test]
fn socket() {
    assert_round_trips::<SocketAddress>(&["127.0.0.1:80", "[::1]:443", "[fe80::1]:0"]);
}

#[test]
fn sockets_constructed() {
    assert_display_parses(&[
        IPv6Address::from([0, 0, 1, 0, 0, 0, 0, 1]).to_socket(0),
        IPv6Address::from(u128::MAX).to_socket(65535),
    ]);
    assert_display_parses(&[
        IPv4Address::BROADCAST.to_socket(65535).to_socket(),
        IPv6Address::LOCALHOST.to_socket(1).to_socket(),
    ]);
}

#[test]
fn domain() {
    assert_round_trips::<Domain>(&[
        "localhost",
        "example.com",
        "a-b.c--d.example",
        "xn--bcher-kva.example",
        "123.example",
    ]);
}

#[test]
fn endpoint() {
    assert_round_trips::<Endpoint>(&["localhost:80", "example.com:443", "a.b.c:65535", "x:0"]);
}

#[test]
fn host() {
    assert_round_trips::<Host>(&["localhost", "example.com", "127.0.0.1", "::1", "fe80::1"]);
}

#[test]
fn authority() {
    assert_round_trips::<Authority>(&[
        "localhost:80",
        "example.com:443",
        "127.0.0.1:80",
        "[::1]:443",
        "[fe80::1]:0",
    ]);
}
