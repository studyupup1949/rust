trait Foo {
    type Bar;
}

trait Baz<T>
where
    T: ?Sized + Foo,
    Self: Foo<Bar = <T as Foo>::Bar>
{}

trait Qux
where
    Self: Baz<Self>
{}

struct Qax {}

impl Foo for Qax {
    type Bar = u32;
}

impl Baz<Qax> for Qax {}

impl Qux for Qax {}
