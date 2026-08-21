//! Simulating a time source that has a clock that differs from the local one.

use adjusting_clock::{Clock, ConstantRate};
use std::thread;
use std::time::{Duration, Instant};

fn run(rate: f64, realtime: bool) {
    let mut clock = Clock::new(Instant::now(), ConstantRate(rate));
    let start = Instant::now();
    let mut ahead = false;
    for iteration in 0..200 {
        let now = if realtime {
            Instant::now()
        } else {
            // Simulate that the local clock advances 100 ms per iteration.
            start + Duration::from_millis(100 * iteration)
        };

        let elapsed = clock.elapsed(now);
        println!("{iteration}: Current time is {0} ms", elapsed.as_millis(),);

        match iteration % 100 {
            50 | 90 | 95 => {
                // In our simulation, the clock source's time fluctuates
                let request = if ahead {
                    now.duration_since(start) + Duration::from_millis(10)
                } else {
                    now.duration_since(start) - Duration::from_millis(10)
                };
                println!(
                    "Clock source says time is {0}. Adjusting...",
                    request.as_millis()
                );
                clock.set_target(now, request);
                ahead = !ahead;
            }
            _ => {}
        }
        if realtime {
            thread::sleep(Duration::from_millis(100));
        }
    }
    println!("Example done");
}

pub fn main() {
    run(0.01, false);
}
