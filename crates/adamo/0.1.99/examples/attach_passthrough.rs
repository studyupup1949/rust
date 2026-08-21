//! Regression test for the AttachOptions composition refactor.
//!
//! `attach_camera` used to rebuild VideoOptions from a subset of AttachOptions,
//! silently dropping `passthrough`. Now AttachOptions embeds VideoOptions and
//! forwards it whole. We probe that via the H.265 encoder resolver: with
//! passthrough it auto-tags `nvh265enc`; without it, H.265 is rejected for
//! lacking an explicit encoder. So the presence/absence of that exact error
//! tells us whether passthrough survived the hop — no camera/stream needed.
//!
//!   ADAMO_API_KEY=<key> ADAMO_LIB_DIR=<dir> DYLD_LIBRARY_PATH=<dir> \
//!   cargo run --release --features video --example attach_passthrough

use std::collections::BTreeMap;

use adamo::{AttachOptions, CameraInfo, Protocol, Robot, VideoBackend, VideoOptions};

const ENCODER_ERR: &str = "requires explicit encoder";

fn fake_v4l2_camera() -> CameraInfo {
    CameraInfo {
        vendor: "v4l2".into(),
        model: "ZED".into(),
        serial: "test".into(),
        handle: "/dev/video0".into(),
        has_depth: false,
        has_imu: false,
        has_onboard_encode: true,
        extra: BTreeMap::new(),
    }
}

fn main() -> adamo::Result<()> {
    let api_key = std::env::var("ADAMO_API_KEY").expect("set ADAMO_API_KEY");
    let mut robot = Robot::new(&api_key, Some("attachopts-test"), Protocol::Quic)?;

    // Control: H.265 through the camera helper WITHOUT passthrough -> the
    // resolver must reject it (no explicit encoder).
    let control = AttachOptions {
        video: VideoOptions { codec: "h265".into(), passthrough: false, ..Default::default() },
        ..Default::default()
    };
    let rc = adamo::attach_camera(&mut robot, &fake_v4l2_camera(), Some("zed_no_pt"), &control);
    let cmsg = rc.as_ref().err().map(|e| e.to_string()).unwrap_or_default();
    println!("h265 passthrough=false -> {cmsg:?}");
    assert!(
        cmsg.contains(ENCODER_ERR),
        "control should hit the encoder-resolver error; got {cmsg:?}"
    );

    // Subject: SAME, WITH passthrough -> that error must NOT appear. If it does,
    // AttachOptions/attach_camera dropped passthrough (the bug we're fixing).
    let subject = AttachOptions {
        video: VideoOptions {
            codec: "h265".into(),
            passthrough: true,
            backend: VideoBackend::GStreamer,
            ..Default::default()
        },
        ..Default::default()
    };
    let rs = adamo::attach_camera(&mut robot, &fake_v4l2_camera(), Some("zed_pt"), &subject);
    let smsg = rs.as_ref().err().map(|e| e.to_string()).unwrap_or_default();
    println!(
        "h265 passthrough=true  -> {}",
        if rs.is_ok() { "Ok".to_string() } else { format!("Err({smsg:?})") }
    );
    assert!(
        !smsg.contains(ENCODER_ERR),
        "passthrough was dropped by AttachOptions->attach_camera: {smsg:?}"
    );

    println!("PASS: AttachOptions.video.passthrough reaches the resolver via attach_camera");
    Ok(())
}
