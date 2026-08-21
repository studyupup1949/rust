//! Consume an existing iceoryx2 shared-memory video service.
//!
//! Run with:
//!   ADAMO_API_KEY=<key> cargo run --features video --example shm
//!
//! Start a producer first. It must publish one complete raw frame or encoded
//! access unit per `[u8]` sample, with no timestamps or metadata prepended.
//! Set `ADAMO_SHM_SOURCE_FORMAT=h265` to transcode H.265 input to the default
//! H.264 output (or set `ADAMO_SHM_OUTPUT_CODEC` to choose another output).

use adamo::{Protocol, Robot, VideoOptions};

fn main() -> adamo::Result<()> {
    let api_key = std::env::var("ADAMO_API_KEY").expect("set ADAMO_API_KEY");
    let name = std::env::var("ADAMO_ROBOT_NAME").unwrap_or_else(|_| "shm-rust-example".into());
    let service = std::env::var("ADAMO_SHM_SERVICE").unwrap_or_else(|_| "camera/front".into());
    let source_format =
        std::env::var("ADAMO_SHM_SOURCE_FORMAT").unwrap_or_else(|_| "BGRA".into());
    let output_codec =
        std::env::var("ADAMO_SHM_OUTPUT_CODEC").unwrap_or_else(|_| "h264".into());

    let mut robot = Robot::new(&api_key, Some(&name), Protocol::Quic)?;
    let options = VideoOptions {
        width: 1280,
        height: 720,
        pixel_format: Some(source_format.clone()),
        codec: output_codec.clone(),
        bitrate_kbps: 4000,
        fps: 30,
        ..Default::default()
    };
    robot.attach_shm_with_options("front", &service, &options)?;

    println!(
        "streaming SHM service `{service}` as `front` 1280x720@30 {source_format} -> {output_codec}"
    );
    robot.run()
}
