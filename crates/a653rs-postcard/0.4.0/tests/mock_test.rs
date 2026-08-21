use core::str::FromStr;
use core::time::Duration;

use a653rs::bindings::QueuingDiscipline;
use a653rs::prelude::{Name, SystemTime};
use mock::MockHyp;

mod mock;

#[test]
fn mock_sampling_port() {
    MockHyp::run_test(|mut ctx| {
        let src_port = ctx
            .create_sampling_port_source(Name::from_str("").unwrap(), 4)
            .unwrap();
        let dest_port = ctx
            .create_sampling_port_destination(Name::from_str("").unwrap(), 4, Duration::ZERO)
            .unwrap();

        let msg = [0, 1, 2, 3];
        let mut buf = [0; 4];
        src_port.send(&msg).unwrap();
        dest_port.receive(&mut buf).unwrap();

        assert_eq!(msg, buf)
    })
}

#[test]
fn mock_const_sampling_port() {
    MockHyp::run_test(|mut ctx| {
        let src_port = ctx
            .create_const_sampling_port_source::<4>(Name::from_str("").unwrap())
            .unwrap();
        let dest_port = ctx
            .create_const_sampling_port_destination::<4>(
                Name::from_str("").unwrap(),
                Duration::ZERO,
            )
            .unwrap();

        let msg = [0, 1, 2, 3];
        let mut buf = [0; 4];
        src_port.send(&msg).unwrap();
        dest_port.receive(&mut buf).unwrap();

        assert_eq!(msg, buf)
    })
}

#[test]
fn mock_queuing_port() {
    MockHyp::run_test(|mut ctx| {
        let src_port = ctx
            .create_queuing_port_sender(Name::from_str("").unwrap(), 4, 0, QueuingDiscipline::Fifo)
            .unwrap();
        let dest_port = ctx
            .create_queuing_port_receiver(
                Name::from_str("").unwrap(),
                4,
                0,
                QueuingDiscipline::Fifo,
            )
            .unwrap();

        let msg = [0, 1, 2, 3];
        let mut buf = [0; 4];
        src_port.send(&msg, SystemTime::Infinite).unwrap();
        dest_port.receive(&mut buf, SystemTime::Infinite).unwrap();

        assert_eq!(msg, buf)
    })
}

#[test]
fn mock_const_queuing_port() {
    MockHyp::run_test(|mut ctx| {
        let src_port = ctx
            .create_const_queuing_port_sender::<4, 0>(
                Name::from_str("").unwrap(),
                QueuingDiscipline::Fifo,
            )
            .unwrap();
        let dest_port = ctx
            .create_const_queuing_port_receiver::<4, 0>(
                Name::from_str("").unwrap(),
                QueuingDiscipline::Fifo,
            )
            .unwrap();

        let msg = [0, 1, 2, 3];
        let mut buf = [0; 4];
        src_port.send(&msg, SystemTime::Infinite).unwrap();
        dest_port.receive(&mut buf, SystemTime::Infinite).unwrap();

        assert_eq!(msg, buf)
    })
}

#[test]
fn mock_queuing_port_msgs() {
    MockHyp::run_test(|mut ctx| {
        let src_port = ctx
            .create_queuing_port_sender(Name::from_str("").unwrap(), 4, 0, QueuingDiscipline::Fifo)
            .unwrap();
        let dest_port = ctx
            .create_queuing_port_receiver(
                Name::from_str("").unwrap(),
                4,
                0,
                QueuingDiscipline::Fifo,
            )
            .unwrap();

        let msg = [0, 1, 2, 3];
        let msg2 = [3, 2, 1, 0];
        let mut buf = [0; 4];
        src_port.send(&msg, SystemTime::Infinite).unwrap();
        src_port.send(&msg2, SystemTime::Infinite).unwrap();
        dest_port.receive(&mut buf, SystemTime::Infinite).unwrap();
        assert_eq!(msg, buf);
        dest_port.receive(&mut buf, SystemTime::Infinite).unwrap();
        assert_eq!(msg2, buf)
    })
}
