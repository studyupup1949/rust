# adamo

Rust SDK for the [Adamo Network](https://adamohq.com) — low-latency
robotics pub/sub and video streaming.

```rust
use adamo::Session;

let session = Session::open_default("ak_…")?;
session.put("telemetry/heartbeat", b"hi", Default::default())?;

let sub = session.subscribe("commands/move")?;
let sample = sub.recv(Some(std::time::Duration::from_secs(5)))?;
```

## Features

- **Session** — auth-aware Zenoh wrapper (`open`, `put`, `publisher`, `subscribe`).
- **Robot** (behind `video` feature) — declare hardware-accelerated video
  tracks against the local camera and stream them over Zenoh:
  ```rust
  use adamo::Robot;
  let mut robot = Robot::new_default("ak_…", Some("rover-01"))?;
  robot.attach_v4l2("front", "/dev/video0", 1280, 720, 30, 4000, false)?;
  robot.run()?; // blocks; capture → encode → publish
  ```
  Existing iceoryx2 shared-memory producers can be consumed directly:
  ```rust
  robot.attach_shm(
      "front",
      "camera/front",
      1280,
      720,
      "BGRA",
      30,
      4000,
      false,
  )?;
  ```
- **Adamo time** — `adamo::fabric_now_us()` returns microseconds on a
  shared clock derived from the relays' `zenoh-plugin-adamo-time`. Stamp
  every cross-node timestamp with this so glass-to-glass latency
  measurements aren't poisoned by NTP drift between machines.

## How the binary is shipped

The crate's [`adamo-sys`](https://crates.io/crates/adamo-sys) dependency
bundles a precompiled `libadamo.so` for the supported target (currently
`aarch64-unknown-linux-gnu` — Jetson / Ubuntu 22.04 arm64) inside its
crate tarball. No network access is required at build time.

For local development against a freshly-built `libadamo`, set
`ADAMO_LIB_DIR=/path/to/dir/with/libadamo.so` to override the bundled
binary.

## License

Apache-2.0. The bundled `libadamo` binary is proprietary and covered by
the Adamo SDK license.
