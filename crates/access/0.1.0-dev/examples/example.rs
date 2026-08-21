use access::{Access, RefAccess};

fn main() {
    let mut int = 0;
    let mut string_access = RefAccess::new(&mut int).map(|i| i.to_string(), |s| s.parse().unwrap());

    println!("string: {}", string_access.get());
    string_access.set("42".to_string());
    println!("string: {}", string_access.get());
    println!("int: {}", int);
}
