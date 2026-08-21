use crate::{Authority, Domain, Endpoint};

#[test]
fn to_authority() {
    let result: Authority = Endpoint::new(Domain::localhost(), 80).to_authority();
    let expected: Authority = Authority::new(Domain::localhost().to_host(), 80);
    assert_eq!(result, expected);
}

#[test]
fn from_tuple() {
    let result: Endpoint = (Domain::localhost(), 80).into();
    let expected: Endpoint = Endpoint::new(Domain::localhost(), 80);
    assert_eq!(result, expected);
}

#[test]
fn tuple_from_endpoint() {
    let result: (Domain, u16) = Endpoint::new(Domain::localhost(), 80).into();
    assert_eq!(result.0, Domain::localhost());
    assert_eq!(result.1, 80);
}
