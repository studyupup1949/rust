extern crate add3;

pub fn add6(x: i32) -> i32 {
    add3::add3(add3::add3(x))
}

#[test]
fn it_works() {
    assert_eq!(13, add6(7));
}
