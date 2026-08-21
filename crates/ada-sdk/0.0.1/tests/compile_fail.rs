#[test]
fn event_marker_and_payload_types_are_enforced() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/*.rs");
}
