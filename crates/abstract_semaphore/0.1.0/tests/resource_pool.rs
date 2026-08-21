use abstract_semaphore::Semaphore;

use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::Duration,
};

const POOL_SIZE: usize = 3;
const THREADS: usize = 10;

#[test]
fn resource_pool() {
    let semaphore = Arc::new(Semaphore::new(POOL_SIZE as u32).unwrap());

    let active = Arc::new(AtomicUsize::new(0));

    let maximum = Arc::new(AtomicUsize::new(0));

    let mut handles = Vec::new();

    for _ in 0..THREADS {
        let semaphore = Arc::clone(&semaphore);
        let active = Arc::clone(&active);
        let maximum = Arc::clone(&maximum);

        handles.push(thread::spawn(move || {
            semaphore.wait().unwrap();

            let current = active.fetch_add(1, Ordering::SeqCst) + 1;

            maximum.fetch_max(current, Ordering::SeqCst);

            // Simulamos el uso del recurso compartido.
            thread::sleep(Duration::from_millis(25));

            active.fetch_sub(1, Ordering::SeqCst);

            semaphore.post().unwrap();
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(active.load(Ordering::SeqCst), 0);

    assert_eq!(maximum.load(Ordering::SeqCst), POOL_SIZE);

    Arc::into_inner(semaphore).unwrap().destroy().unwrap();
}
