extern crate add3;

pub fn add6(x: i64) -> i64 {
    add3::add3(add3::add3(x))
}

pub fn twice(a: add3::Adder, x: i64) -> i64 {
    a.add(a.add(x))
}

pub fn make_adder() -> add3::Adder {
    add3::Adder::new()
}

#[test]
fn it_works() {
    assert_eq!(13_i64, add6(7));
}

#[test]
fn twice_works() {
    assert_eq!(twice(make_adder(), 7), 13);
}
