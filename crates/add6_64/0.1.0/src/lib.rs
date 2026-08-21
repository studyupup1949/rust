extern crate add3;

pub fn add6(x: i64) -> i64 {
    add3::add3(add3::add3(x))
}

#[test]
fn it_works() {
    assert_eq!(13_i64, add6(7));
}
