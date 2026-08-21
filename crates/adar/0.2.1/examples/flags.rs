use adar::prelude::*;

#[FlagEnum]
#[repr(u32)]
enum MyFlag {
    F1,
    F2,
    F3,
}

fn main() {
    let mut a = Flags::from(MyFlag::F1);
    a.set(MyFlag::F2);
    let mut b = MyFlag::F1 | MyFlag::F2 | MyFlag::F3;
    b.reset(MyFlag::F1);

    println!("a: {:?}, {:03b}", a, a.into_raw()); // Prints: (F1|F2), 011
    println!("b: {:?}, {:03b}", b, b.into_raw()); // Prints: (F2|F3), 110

    println!("a.any(F1,F3): {:?}", a.any(MyFlag::F1 | MyFlag::F3)); // Prints: true
    println!("a.all(F1,F3): {:?}", a.all(MyFlag::F1 | MyFlag::F3)); // Prints: false

    println!("a.intersect(b): {:?}", a.intersect(b)); // Prints: (F2)
    println!("a.union(b): {:?}", a.union(b)); // Prints: (F1|F2|F3)
    println!("full(): {:?}", Flags::<MyFlag>::full()); // Prints: (F1|F2|F3)
}
