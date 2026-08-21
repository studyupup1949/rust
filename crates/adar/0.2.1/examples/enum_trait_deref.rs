use adar::prelude::*;

trait MyTrait {
    fn my_func(&self);
}

#[EnumTraitDeref(MyTrait)]
enum MyEnum {
    A(A),
    B(B),
}

#[derive(Clone)]
struct A;

#[derive(Clone)]
struct B;

impl MyTrait for A {
    fn my_func(&self) {
        println!("Hello A");
    }
}

impl MyTrait for B {
    fn my_func(&self) {
        println!("Hello B");
    }
}

fn main() {
    for e in [MyEnum::A(A), MyEnum::B(B)] {
        e.my_func();
    }
}
