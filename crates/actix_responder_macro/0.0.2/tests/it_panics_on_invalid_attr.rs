extern crate trybuild;

use trybuild::TestCases;

#[test]
fn it_panics_on_invalid_attr() {
    let t = TestCases::new();
    t.compile_fail("./panics/panics_on_invalid_meta_attr.rs");
    t.compile_fail("./panics/panics_on_unsupported_attr.rs");
}
