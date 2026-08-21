trait Foo: PartialOrd + PartialOrd<<Self as Foo>::A> {
    type A;
}

trait Bar {
    type B: Foo;

    fn bar(&self) -> Self::B;
}

fn baz<T: Bar>(x: T, y: T) -> bool {
    x.bar() < y.bar()
}
