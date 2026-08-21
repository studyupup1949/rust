use std::convert::TryFrom;

use crate::{Authority, Domain, Endpoint, Host};

#[test]
fn to_endpoint() {
    let result: Endpoint = Domain::localhost().to_endpoint(80);
    let expected: Endpoint = Endpoint::new(Domain::localhost(), 80);
    assert_eq!(result, expected);
}

#[test]
fn to_host() {
    let result: Host = Domain::localhost().to_host();
    let expected: Host = Host::Name(Domain::localhost());
    assert_eq!(result, expected);
}

#[test]
fn to_authority() {
    let result: Authority = Domain::localhost().to_authority(80);
    let expected: Authority = Authority::new(Domain::localhost().to_host(), 80);
    assert_eq!(result, expected);
}

#[test]
fn try_from_string() {
    let result: Result<Domain, ()> = Domain::try_from("localhost".to_string());
    assert_eq!(result, Ok(Domain::localhost()));

    let result: Result<Domain, ()> = Domain::try_from("LOCALHOST".to_string());
    assert_eq!(result, Err(()));
}

#[test]
fn string_from_domain() {
    let s: String = Domain::localhost().into();
    assert_eq!(s, "localhost");
}

#[test]
fn try_from_str() {
    let result: Result<Domain, ()> = Domain::try_from("localhost");
    assert_eq!(result, Ok(Domain::localhost()));

    let result: Result<Domain, ()> = Domain::try_from("LOCALHOST");
    assert_eq!(result, Err(()));
}

#[test]
fn try_from_vec() {
    let result: Result<Domain, ()> = Domain::try_from(b"localhost".to_vec());
    assert_eq!(result, Ok(Domain::localhost()));

    let result: Result<Domain, ()> = Domain::try_from(b"LOCALHOST".to_vec());
    assert_eq!(result, Err(()));
}

#[test]
fn try_from_u8_slice() {
    let result: Result<Domain, ()> = Domain::try_from("localhost".as_bytes());
    assert_eq!(result, Ok(Domain::localhost()));

    let result: Result<Domain, ()> = Domain::try_from("LOCALHOST".as_bytes());
    assert_eq!(result, Err(()));
}
