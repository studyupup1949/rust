use trybuild::TestCases;

#[test]
fn test_error_handling(){
    let tests = TestCases::new();
    tests.compile_fail("tests/errors/multiple-setups.rs");
    tests.compile_fail("tests/errors/multiple-cleanup.rs");
    tests.compile_fail("tests/errors/invalid-attrs.rs");
}
