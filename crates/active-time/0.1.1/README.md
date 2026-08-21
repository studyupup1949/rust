# active-time
A Rust crate to find out the current amount of time the system has been active, excluding time spent hibernating/sleeping.

# Windows
This currently only works on Windows and uses the [`QueryUnbiasedInterruptTime`](https://learn.microsoft.com/en-us/windows/win32/api/realtimeapiset/nf-realtimeapiset-queryunbiasedinterrupttime) method to query the current "ticks", which represent `100ns` each.

# Example
```rust
let active_time: Duration = active_time::active_time()?;
```
This prints the current active time. Check out the [example](https://github.com/M1ngXU/active-time/blob/main/examples/compare_time.rs) for more information and also a comparation with `Instant::elapsed` when hibernating Windows while the program is running.
