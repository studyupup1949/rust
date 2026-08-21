use accessor_macro::Accessor;
use uintx::u24;

#[derive(Accessor, Debug)]
struct Person {
    #[accessor(get, set)]
    name: String,
    #[accessor(get, set, range=[0, 200])]
    age: i32,
    #[accessor(get, set, range=[u24::from(0), u24::from(100)])]
    number: u24,

    self_test_min: i32,
    #[accessor(get, set, range=[self.self_test_min, self.get_test_max()])]
    self_test: i32,
}

impl Person {
    pub fn get_test_max(&self) -> i32 {
        100
    }
}

fn main() {
    let mut person = Person {
        name: "Alice".to_string(),
        age: 25,
        number: u24::from(50),
        self_test_min: 0,
        self_test: 50,
    };

    assert!(person.get_name().eq("Alice"));
    person.set_name("Bob".to_string());
    assert!(person.get_name().eq("Bob"));

    println!("person: {:?}", person);
    println!("--------------------------------------");
    println!("设置age为-1, -1 不在范围, 返回false");
    #[cfg(not(any(debug_assertions, feature = "debug_panic")))]
    assert!(!person.set_age(-1));
    println!("person: {:?}", person);
    println!("--------------------------------------");
    println!("设置age为18, 18 在范围, 返回true");
    assert!(person.set_age(18));
    println!("person: {:?}", person);
    println!("--------------------------------------");
    println!("设置number为0, 0 在范围, 返回true");
    assert!(person.set_number(u24::from(0)));
    println!("person: {:?}", person);
    println!("--------------------------------------");
    println!("设置number为101, 101 不在范围, 返回false");
    #[cfg(not(any(debug_assertions, feature = "debug_panic")))]
    assert!(!person.set_number(u24::from(101)));
    println!("person: {:?}", person);
    println!("--------------------------------------");
    println!("设置self_test为-1, -1 不在范围, 返回false");
    #[cfg(not(any(debug_assertions, feature = "debug_panic")))]
    assert!(!person.set_self_test(-1));
    println!("person:{:?}", person);
    println!("--------------------------------------");
    println!("动态修改self_test_min为-100");
    person.self_test_min = -100;
    println!("设置self_test为-1, -1 在范围, 返回true");
    assert!(person.set_self_test(-1));
    println!("persion:{:?}", person);
    println!("--------------------------------------");
}
