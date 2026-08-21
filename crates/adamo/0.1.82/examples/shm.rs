//! Consume an existing iceoryx2 shared-memory video service.
//!
//! Run with:
//!   ADAMO_API_KEY=<key> cargo run --features video --example shm
//!
//! Start a producer first. It must publish one complete frame per `[u8]`
//! sample on the service below, with no timestamps or metadata prepended.

use adamo::{Protocol, Robot, VideoOptions};

fn main() -> adamo::Result<()> {
    let api_key = std::env::var("ADAMO_API_KEY").expect("set ADAMO_API_KEY");
    let name = std::env::var("ADAMO_ROBOT_NAME").unwrap_or_else(|_| "shm-rust-example".into());
    let service = std::env::var("ADAMO_SHM_SERVICE").unwrap_or_else(|_| "camera/front".into());

    let mut robot = Robot::new(&api_key, Some(&name), Protocol::Quic)?;
    let options = VideoOptions {
        width: 1280,
        height: 720,
        pixel_format: Some("BGRA".into()),
        bitrate_kbps: 4000,
        fps: 30,
        ..Default::default()
    };
    robot.attach_shm_with_options("front", &service, &options)?;

    println!("streaming SHM service `{service}` as `front` 1280x720@30 BGRA");
    robot.run()
}
