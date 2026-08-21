use adder_lro as adder;

mod common;

#[test]
fn it_adds_two() {
  common::setup();
  assert_eq!(4, adder::add_two(2));
}