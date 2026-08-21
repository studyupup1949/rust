use adamo::{Protocol, Robot};

fn main() -> adamo::Result<()> {
    let api_key = std::env::var("ADAMO_API_KEY").expect("set ADAMO_API_KEY");
    let mut robot = Robot::new(&api_key, Some("rust-example"), Protocol::Quic)?;

    // Use a synthetic GStreamer source so this runs without a real camera.
    robot.attach_gst(
        "test",
        "videotestsrc is-live=true pattern=ball",
        640,
        480,
        30,
        2000,
    )?;

    // Blocks forever — drive the encoding + transport pipeline.
    robot.run()
}
