use access::{Access, AccessMut};

fn main() {
    let mut int = 0;
    let mut string_access = (&mut int).map_mut(|i| i.to_string(), |s| s.parse().unwrap());

    println!("string: {}", string_access.get());
    string_access.set("42".to_string());
    println!("string: {}", string_access.get());
    println!("int: {}", int);
}
