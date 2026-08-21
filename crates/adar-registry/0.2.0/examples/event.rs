use adar_registry::prelude::*;

fn main() {
    // Create an event and register two observers.
    let event: Event<(u32, String)> = Event::new();
    let _entry1 = event.register_observer(|data: &(u32, String)| {
        println!("Observer #1 called: {:?}", data);
    });
    let entry2 = event.register_observer(|data: &(u32, String)| {
        println!("Observer #2 called: {:?}", data);
    });

    // Since all entries are still in scope all the observers will be called.
    event.dispatch((1, "First event".into()));

    // After dropping entry2. The associated observer will be dropped.
    drop(entry2);
    event.dispatch((2, "Second event".into()));
}
