use abstract_semaphore::Semaphore;

use std::{
    sync::{Arc, Mutex},
    thread,
};

#[test]
fn producer_consumer() {
    const ITEMS: usize = 10_000;

    let produced = Arc::new(Semaphore::new(0).unwrap());
    let consumed = Arc::new(Semaphore::new(1).unwrap());

    let value = Arc::new(Mutex::new(None::<usize>));

    let producer_value = Arc::clone(&value);
    let producer_produced = Arc::clone(&produced);
    let producer_consumed = Arc::clone(&consumed);

    let producer = thread::spawn(move || {
        for i in 0..ITEMS {
            producer_consumed.wait().unwrap();

            *producer_value.lock().unwrap() = Some(i);

            producer_produced.post().unwrap();
        }
    });

    let consumer_value = Arc::clone(&value);
    let consumer_produced = Arc::clone(&produced);
    let consumer_consumed = Arc::clone(&consumed);

    let consumer = thread::spawn(move || {
        for expected in 0..ITEMS {
            consumer_produced.wait().unwrap();

            let value = consumer_value.lock().unwrap().take();

            assert_eq!(value, Some(expected));

            consumer_consumed.post().unwrap();
        }
    });

    producer.join().unwrap();
    consumer.join().unwrap();
}
