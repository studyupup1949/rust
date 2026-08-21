//! Using [`Clock`] as a stopwatch. No adjustments made.

use adjusting_clock::{Clock, ConstantRate};
use std::thread;
use std::time::{Duration, Instant};

pub fn main() {
    let clock = Clock::new(Instant::now(), ConstantRate(0.01));
    loop {
        let elapsed = clock.elapsed(Instant::now());
        println!("Current time: {0} ms", elapsed.as_millis());
        thread::sleep(Duration::from_millis(100));
    }
}
