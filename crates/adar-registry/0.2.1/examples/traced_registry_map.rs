use adar_registry::prelude::*;

fn main() {
    let registry = TracedRegistryMap::<&'static str, i32>::new();
    let _observer = registry.register_observer(|(event, entry, key, value): &_| {
        println!("{:?}, {:?}, {}, {}", event, entry, key, value)
    });

    let foo = registry.register("one", 1);
    let bar = registry.register("two", 2);
    drop(foo);
    let baz = registry.register("three", 3);
    drop(bar);
    drop(baz);
}
