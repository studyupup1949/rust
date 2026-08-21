use crate::Domain;

#[test]
fn display_domain() {
    let result: String = Domain::localhost().to_string();
    assert_eq!(result, "localhost");

    let result: String = Domain::example().to_string();
    assert_eq!(result, "example.com");
}
