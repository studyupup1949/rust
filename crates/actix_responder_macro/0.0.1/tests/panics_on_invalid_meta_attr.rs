extern crate trybuild;

use trybuild::TestCases;

#[test]
fn it_works_without_meta_attr() {
    let t = TestCases::new();
    t.compile_fail("./panics/panics_on_invalid_meta_attr.rs");
}
