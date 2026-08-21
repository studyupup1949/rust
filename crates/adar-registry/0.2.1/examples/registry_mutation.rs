use adar_registry::prelude::*;

fn main() {
    let registry = Registry::<i32>::new();
    let entry1 = registry.register(0);
    let entry2 = registry.register(100);

    println!("{:?}", registry); // Prints: {0: 0, 1: 100}

    println!("Mutation via Registry...");
    for (_, value) in registry.write().iter_mut() {
        *value += 1;
    }
    println!("{:?}", registry); // Prints: {0: 1, 1: 101}

    println!("Mutation via typed Entry...");
    *entry1.write().unwrap().get_mut() += 10;
    *entry2.write().unwrap().get_mut() += 10;
    println!("{:?}", registry); // Prints: {0: 11, 1: 111}
}
