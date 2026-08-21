//! Drive the default macOS webcam through the Adamo SDK so the frontend
//! can subscribe to the resulting H.264 stream.
//!
//! Run with:
//!   ADAMO_API_KEY=<key> cargo run --features video --example webcam
//!
//! Requires `libadamo` built with `--features video` (which enables the
//! hw_pipeline backend: AVFoundation → VideoToolbox on macOS, direct
//! V4L2 → NVENC on Linux).

use adamo::{Protocol, Robot, detect_encoder};

fn main() -> adamo::Result<()> {
    let api_key = std::env::var("ADAMO_API_KEY").expect("set ADAMO_API_KEY");
    let name = std::env::var("ADAMO_ROBOT_NAME").unwrap_or_else(|_| "macbook".into());

    println!("detected encoder: {}", detect_encoder()?);

    let mut robot = Robot::new(&api_key, Some(&name), Protocol::Quic)?;

    // `attach_v4l2` is the hw_pipeline entry point. On macOS the `device`
    // string is reinterpreted as an AVFoundation camera UID — "" means
    // "default camera" (see src/hw_pipeline/macos/macos_capture.m:88).
    robot.attach_v4l2("webcam", "", 1280, 720, 30, 2000, false)?;

    println!("streaming `webcam` 1280x720@30 (2000 kbps) — robot name: {name}");
    println!("subscribe from the frontend to view.");
    robot.run()
}
