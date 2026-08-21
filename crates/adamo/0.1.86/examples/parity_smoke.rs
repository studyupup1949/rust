use std::time::Duration;

use adamo::{
    AttachOptions, ControlMessage, JointState, Joy, JoystickCommand, Priority, Protocol,
    PublishOptions, PublisherOptions, Robot, Session, attach_all, attach_camera, decode_control,
    discover_v4l2,
};

fn main() -> adamo::Result<()> {
    assert_eq!(PublishOptions::default().priority, Priority::DATA);
    assert!(!PublishOptions::default().express);
    assert_eq!(PublisherOptions::default().priority, Priority::DATA);
    assert!(!PublisherOptions::default().express);
    assert!(!PublisherOptions::default().reliable);

    let joint = JointState {
        names: vec!["joint_1".to_owned()],
        positions: vec![1.0],
        velocity: vec![0.5],
        effort: vec![0.25],
        frame_id: "base".to_owned(),
        ..Default::default()
    };
    match decode_control(&joint.to_json()?)? {
        ControlMessage::JointState(decoded) => {
            assert_eq!(decoded.names, joint.names);
            assert_eq!(decoded.positions, joint.positions);
            assert_eq!(decoded.velocity, joint.velocity);
            assert_eq!(decoded.effort, joint.effort);
            assert_eq!(decoded.frame_id, joint.frame_id);
        }
        other => panic!("expected JointState, got {other:?}"),
    }

    let joy = Joy {
        axes: vec![0.1, -0.2],
        buttons: vec![1, 0],
        ..Default::default()
    };
    match decode_control(&joy.to_json()?)? {
        ControlMessage::Joy(decoded) => {
            assert_eq!(decoded.axes, joy.axes);
            assert_eq!(decoded.buttons, joy.buttons);
        }
        other => panic!("expected Joy, got {other:?}"),
    }

    let command = JoystickCommand {
        sequence_id: 42,
        axes: vec![0.3],
        buttons: vec![1],
        ..Default::default()
    };
    match decode_control(&command.to_json()?)? {
        ControlMessage::JoystickCommand(decoded) => {
            assert_eq!(decoded.sequence_id, command.sequence_id);
            assert_eq!(decoded.axes, command.axes);
            assert_eq!(decoded.buttons, command.buttons);
        }
        other => panic!("expected JoystickCommand, got {other:?}"),
    }

    let cameras = discover_v4l2();
    println!("discovered {} V4L2 camera(s)", cameras.len());

    if let Ok(api_key) = std::env::var("ADAMO_PARITY_OPEN") {
        let session = Session::open(&api_key, Protocol::Quic)?;
        let _ = session.get("parity/rust/**", Duration::from_millis(1))?;
        let _token = session.alive("parity/rust")?;
        let _ = session.live_tokens("parity/**")?;
        let _watch = session.on_liveliness("parity/**", false, |_, _| {})?;
        let publisher = session.publisher("parity/rust/smoke", PublisherOptions::default())?;
        publisher.put(b"hello")?;
        session.put("parity/rust/put", b"hello", PublishOptions::default())?;
    }

    if let Ok(api_key) = std::env::var("ADAMO_PARITY_OPEN_MTLS") {
        let _ = Session::open_mtls(&api_key, Protocol::Quic)?;
    }

    if let Ok(api_key) = std::env::var("ADAMO_PARITY_ATTACH_V4L2") {
        let mut robot = Robot::new_default(&api_key, Some("rust_parity_smoke"))?;
        let options = AttachOptions::default();
        if let Some(camera) = cameras.first() {
            attach_camera(&mut robot, camera, None, &options)?;
        }
        let _attach_all: fn(&mut Robot, &AttachOptions) -> Vec<adamo::CameraInfo> = attach_all;
        let _ = _attach_all;
    }

    Ok(())
}
