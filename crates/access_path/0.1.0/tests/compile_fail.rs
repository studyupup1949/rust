#[test]
fn compile_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/fail/not_a_struct.rs");
    t.compile_fail("tests/fail/tuple_struct.rs");
}
