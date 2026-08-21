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
  Caller-fed tracks return an independent sender that remains usable while
  `Robot::run()` blocks. Raw frames use direct in-process ingress; encoded
  H.264/H.265 passthrough accepts variable-size Annex-B access units:
  ```rust
  let track = robot.video("front", 1280, 720, "BGRA", 30, 4000, false)?;
  let frame = vec![0u8; 1280 * 720 * 4];
  std::thread::scope(|scope| {
      scope.spawn(|| robot.run().unwrap());
      track.send(&frame).unwrap();
      robot.stop().unwrap();
  });
  ```
  Existing iceoryx2 shared-memory producers can be consumed directly. Use
  `VideoOptions` when you need the same knobs exposed by the Python SDK:
  ```rust
  use adamo::{VideoBackend, VideoOptions};

  let options = VideoOptions {
      width: 1280,
      height: 720,
      pixel_format: Some("BGRA".into()),
      codec: "h264".into(),
      encoder: None,
      bitrate_kbps: 4000,
      adaptive_bitrate: true,
      min_bitrate_kbps: Some(1000),
      max_bitrate_kbps: Some(6000),
      bitrate_priority: 1.0,
      fps: 30,
      keyframe_distance: 2.0,
      stereo: false,
      backend: VideoBackend::Auto,
  };
  robot.attach_shm_with_options(
      "front",
      "camera/front",
      &options,
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
