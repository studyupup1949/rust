use adar::prelude::*;

#[ReflectEnum]
#[repr(u32)]
#[derive(Debug)]
#[allow(dead_code)]
enum MyEnum {
    Value1 = 33,
    Value2(i32),
    Value3 { a: String },
}

fn main() {
    println!("Variants count: {}", MyEnum::count());
    for variant in MyEnum::variants() {
        println!("{}, {:?}", variant.name, variant.value,);
    }
}
