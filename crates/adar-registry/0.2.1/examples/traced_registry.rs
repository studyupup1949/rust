use adar_registry::prelude::*;

fn main() {
    let registry = TracedRegistry::<&'static str>::new();
    let _observer = registry.register_observer(|(event, entry, value): &_| {
        println!("{:?}, {:?}, {}", event, entry, value)
    });

    let foo = registry.register("one");
    let bar = registry.register("two");
    drop(foo);
    let baz = registry.register("three");
    drop(bar);
    drop(baz);
}
