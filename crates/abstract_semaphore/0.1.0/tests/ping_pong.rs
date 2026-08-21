use abstract_semaphore::Semaphore;

use std::sync::{Arc, Mutex};

use std::thread;

const ITERATIONS: usize = 100;

#[test]
fn ping_pong() {
    let ping = Arc::new(Semaphore::new(1).unwrap());

    let pong = Arc::new(Semaphore::new(0).unwrap());

    let output = Arc::new(Mutex::new(String::new()));

    let ping_thread = {
        let ping = Arc::clone(&ping);
        let pong = Arc::clone(&pong);
        let output = Arc::clone(&output);

        thread::spawn(move || {
            for _ in 0..ITERATIONS {
                ping.wait().unwrap();

                output.lock().unwrap().push('A');

                pong.post().unwrap();
            }
        })
    };

    let pong_thread = {
        let ping = Arc::clone(&ping);
        let pong = Arc::clone(&pong);
        let output = Arc::clone(&output);

        thread::spawn(move || {
            for _ in 0..ITERATIONS {
                pong.wait().unwrap();

                output.lock().unwrap().push('B');

                ping.post().unwrap();
            }
        })
    };

    ping_thread.join().unwrap();
    pong_thread.join().unwrap();

    let expected = "AB".repeat(ITERATIONS);

    assert_eq!(*output.lock().unwrap(), expected);

    Arc::into_inner(ping).unwrap().destroy().unwrap();

    Arc::into_inner(pong).unwrap().destroy().unwrap();
}
