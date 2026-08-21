use std::time::{Duration, Instant};

use crate::{Clock, ConstantRate};

fn test_clock() -> (Instant, Clock<ConstantRate>) {
    let now = Instant::now();
    (now, Clock::new(now, ConstantRate(0.25)))
}

const fn ms(ms: u64) -> Duration {
    Duration::from_millis(ms)
}

#[test]
fn new_clock_starts_at_zero() {
    let (now, c) = test_clock();
    assert_eq!(c.elapsed(now), Duration::ZERO);
}

#[test]
fn unadjusted_clock_acts_as_stopwatch() {
    let (mut now, c) = test_clock();
    now += ms(10);
    assert_eq!(c.elapsed(now), ms(10));
}

#[test]
fn setting_target_changes_elapsed_time() {
    let (mut now, mut c) = test_clock();
    const TARGET: Duration = ms(1000);
    const ADJUSTMENT_DELAY: Duration = ms(4000); // as rate is 0.25, it takes 4 seconds to adjust 1 second
    c.set_target(now, TARGET);
    now += ADJUSTMENT_DELAY; // Give clock time to adjust
    assert_eq!(c.elapsed(now), TARGET + ADJUSTMENT_DELAY);
}

#[test]
fn setting_target_does_not_changed_elapse_time_immediately() {
    let (now, mut c) = test_clock();
    // Request current elapsed time to be 1000
    c.set_target(now, ms(1000));
    assert_eq!(c.elapsed(now), ms(0));
}

#[test]
fn setting_target_to_later_speeds_up_clock() {
    // Start the clock and let it run for one second
    let (mut now, mut c) = test_clock();
    now += ms(1000);
    assert_eq!(c.elapsed(now), ms(1000));

    // Adjust clock so the current time is 1300 instead of 1000.
    // Adjusting the clock +300 ms will take 300 / `rate` = 1200 ms.
    c.set_target(now, ms(1300));

    // no change directly after set_target
    assert_eq!(c.elapsed(now), ms(1000));
    // After another 100 ms, time has advanced 100 ms, plus additional 25 ms (100 * rate) to catch up with the target
    assert_eq!(c.elapsed(now + ms(100)), ms(1000 + 100 + 25));

    // ...and so on: For each elapsed 100 ms, the clock is adjusted by 25 ms until the adjustment is 300 ms:
    assert_eq!(c.elapsed(now + ms(200)), ms(1000 + 200 + 50));
    assert_eq!(c.elapsed(now + ms(300)), ms(1000 + 300 + 75));
    assert_eq!(c.elapsed(now + ms(400)), ms(1000 + 400 + 100));
    // ... until we reach the target at 1200 ms:
    assert_eq!(c.elapsed(now + ms(1100)), ms(1000 + 1100 + 275));
    assert_eq!(c.elapsed(now + ms(1200)), ms(1000 + 1200 + 300));
    // ...and once the target is reached, there are no more adjustments:
    assert_eq!(c.elapsed(now + ms(1300)), ms(1000 + 1300 + 300));
    assert_eq!(c.elapsed(now + ms(1400)), ms(1000 + 1400 + 300));
}

#[test]
fn setting_target_to_earlier_slows_down_clock() {
    // Start the clock and let it run for one step
    let (start, mut c) = test_clock();
    let t = start + ms(1000);
    // Adjust clock so the current time is 700 instead of 1000.
    // Adjusting the clock -300 ms will take 300 / `rate` = 1200 ms.
    c.set_target(t, ms(700));

    // No change directly after setting target
    assert_eq!(c.elapsed(t), ms(1000));
    // After another 100 ms, time has advanced 100 ms, plus additional 25 ms (100 * rate) to catch up with the target
    assert_eq!(c.elapsed(t + ms(100)), ms(1000 + 100 - 25));
    // ...and so on: For each elapsed 100 ms, the clock is adjusted by 25 ms until the adjustment is 300 ms:
    assert_eq!(c.elapsed(t + ms(200)), ms(1000 + 200 - 50));
    assert_eq!(c.elapsed(t + ms(300)), ms(1000 + 300 - 75));
    assert_eq!(c.elapsed(t + ms(400)), ms(1000 + 400 - 100));
    // ... until we reach the target at 1200 ms:
    assert_eq!(c.elapsed(t + ms(1100)), ms(1000 + 1100 - 275));
    assert_eq!(c.elapsed(t + ms(1200)), ms(1000 + 1200 - 300));
    // ...and once the target is reached, there are no more adjustments:
    assert_eq!(c.elapsed(t + ms(1300)), ms(1000 + 1300 - 300));
}

#[test]
fn high_rate_forwards() {
    let start = Instant::now();
    let mut c = Clock::new(start, ConstantRate(100.0));
    c.set_target(start, ms(100)); // adjust by +100 ms, will take 1 ms
    assert_eq!(c.elapsed(start), ms(0), "no change at first");
    assert_eq!(
        c.elapsed(start + ms(1)),
        ms(101),
        "adjustment done after short time"
    );
}

#[test]
fn high_rate_backwards() {
    let base = Instant::now();
    let mut c = Clock::new(base, ConstantRate(100.0));
    let now = base + ms(1000);
    c.set_target(now, ms(900)); // adjust by -100 ms, will take 1 ms
    assert_eq!(c.elapsed(now), ms(1000), "no change at first");
    assert_eq!(
        c.elapsed(now + ms(1)),
        ms(901),
        "adjustment done after short time"
    );
}
