#[derive(PartialEq)]
struct Wrapper<T>(T);

impl<T: PartialEq> PartialEq<T> for Wrapper<T> {
    fn eq(&self, other: &T) -> bool {
        self.0.eq(other)
    }
}

/*
impl<T: PartialEq> PartialEq<Wrapper<T>> for T {
    fn eq(&self, other: &Wrapper<T>) -> bool {
        self.eq(other.0)
    }
}
*/

struct Foo;

impl<T> PartialEq<T> for Foo {
    fn eq(&self, other: &T) -> bool {
        false
    }
}

fn main() {}
