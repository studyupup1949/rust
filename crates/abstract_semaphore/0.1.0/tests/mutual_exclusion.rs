use abstract_semaphore::Semaphore;

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use std::thread;

const THREADS: usize = 8;
const ITERATIONS: usize = 10_000;

#[test]
fn mutual_exclusion() {
    let semaphore = Arc::new(Semaphore::new(1).unwrap());

    let counter = Arc::new(AtomicUsize::new(0));

    let mut handles = Vec::new();

    for _ in 0..THREADS {
        let semaphore = Arc::clone(&semaphore);
        let counter = Arc::clone(&counter);

        handles.push(thread::spawn(move || {
            for _ in 0..ITERATIONS {
                semaphore.wait().unwrap();

                let value = counter.load(Ordering::Relaxed);
                counter.store(value + 1, Ordering::Relaxed);

                semaphore.post().unwrap();
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(counter.load(Ordering::Relaxed), THREADS * ITERATIONS);

    Arc::into_inner(semaphore).unwrap().destroy().unwrap();
}
