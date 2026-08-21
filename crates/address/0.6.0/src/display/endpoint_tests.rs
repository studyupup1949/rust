use crate::Domain;

#[test]
fn display_endpoint() {
    let result: String = Domain::localhost().to_endpoint(80).to_string();
    assert_eq!(result, "localhost:80");

    let result: String = Domain::example().to_endpoint(80).to_string();
    assert_eq!(result, "example.com:80");
}
