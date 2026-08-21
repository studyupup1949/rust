# adjusting_clock - smoothly adjusting for clock drift

A Rust crate for measuring time, while synchronizing it with an external source,
for example adjusting the clock of a client to match the time one of a server.

The [`Clock`] struct works as a stopwatch. After you create an instance of it, you can query how much time has passed since it started. The clock returns this as a [`std::time::Duration`].

```rust
use adjusting_clock::{Clock, ConstantRate};
use std::time::Instant;

let clock = Clock::new(Instant::now(), ConstantRate(0.001));
// ... after a while:
let current = clock.elapsed(Instant::now());
```

The usefulness of `Clock` comes from its ability to smoothly adjust its current time to match the time of another source:

```rust no_run
use std::thread;
use adjusting_clock::{Clock, ConstantRate};
use std::time::{Instant, Duration};

// Create a clock that adjusts at a speed of 10 ms per second:
let t0 = Instant::now();
let mut clock = Clock::new(t0, ConstantRate(0.01));
// Request the current time to be 10 ms.
// As the clock just started at 0, this is an adjustment of +10 ms from the current time:
clock.set_target(t0, Duration::from_millis(10));
// After taking time to adjust at a rate of 10 ms per second,
// the current time will have been adjusted:
thread::sleep(Duration::from_millis(1000));
let elapsed = clock.elapsed(Instant::now());
//assert_eq!(elapsed, Duration::from_millis(1010)); // approx.
println!("Current time: {0} ms", elapsed.as_millis());
// Will print approximately 1010 ms, which is the sum of
// 10 because that's what the `set_target` call requested,
// and 1000 because that's how long time has passed since then.
```

See the [test cases](src/test.rs) for more examples.
