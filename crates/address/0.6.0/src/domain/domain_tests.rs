use crate::Domain;

#[test]
fn specials() {
    assert_eq!(Domain::localhost().name(), "localhost");
    assert_eq!(Domain::example().name(), "example.com");
}

#[test]
fn new() {
    let result: Domain = unsafe { Domain::new("localhost".to_string()) };
    assert_eq!(result, Domain::localhost());
}

#[test]
fn name() {
    let domain: Domain = Domain::localhost();
    assert_eq!(domain.name(), "localhost");
}

#[test]
fn export_name() {
    let domain: Domain = Domain::localhost();
    assert_eq!(domain.export_name(), "localhost".to_string());
}
