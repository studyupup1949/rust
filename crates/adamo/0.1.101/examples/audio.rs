use adamo::{AudioOptions, Protocol, Robot};

// Source-driven audio send. Run with `--features audio` against a libadamo built
// with its `audio` feature (see src/audio.rs for the ADAMO_LIB_DIR note):
//
//   ADAMO_API_KEY=ak_... cargo run --example audio --features audio
fn main() -> adamo::Result<()> {
    let api_key = std::env::var("ADAMO_API_KEY").expect("set ADAMO_API_KEY");
    let mut robot = Robot::new(&api_key, Some("rust-audio"), Protocol::Quic)?;

    // Synthetic test tone — no microphone required.
    robot.attach_audio_test("mic")?;

    // A real mic from ALSA (prefer a plughw: device):
    //   robot.attach_audio_alsa("mic", Some("plughw:1,0"), 64, 2, 48_000, 20)?;
    //
    // Or the full option surface:
    //   let opts = AudioOptions::default()
    //       .with_source_type("alsa")
    //       .with_device("plughw:1,0")
    //       .with_bitrate_kbps(64);
    //   robot.attach_audio("mic", &opts)?;
    let _ = AudioOptions::default();

    // Blocks forever — drive the capture + Opus encode + transport pipeline.
    robot.run()
}
