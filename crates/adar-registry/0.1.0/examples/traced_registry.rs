use adar_registry::prelude::*;

fn main() {
    let registry = TracedRegistry::<&'static str>::new();
    let _observer = registry.register_observer(|(event, entry, value): &_| {
        println!("{:?}, {:?}, {}", event, entry, value)
    });

    let foo = registry.register("foo");
    let bar = registry.register("bar");
    drop(foo);
    let baz = registry.register("baz");
    drop(bar);
    drop(baz);
}
