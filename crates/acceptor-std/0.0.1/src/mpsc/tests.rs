use accepts::{Accepts, AsyncAccepts};
use std::sync::mpsc;

use crate::support::{Recorder, block_on};

use super::MpscSender;

#[test]
fn mpsc_sender_records_send_result_and_delivers_value() {
    let (tx, rx) = mpsc::channel();
    let results = Recorder::new();
    let sender = MpscSender::new(tx, results.clone());

    sender.accept(7);

    assert_eq!(rx.recv().unwrap(), 7);

    let mut recorded = results.take();
    assert_eq!(recorded.len(), 1);
    assert!(recorded.pop().unwrap().is_ok());
}

#[test]
fn mpsc_sender_async_records_error_when_channel_closed() {
    let (tx, rx) = mpsc::channel::<&str>();
    drop(rx);

    let results = Recorder::new();
    let sender = MpscSender::new(tx, results.clone());

    block_on(sender.accept_async("ping"));

    let mut recorded = results.take();
    let err = recorded
        .pop()
        .expect("result recorded")
        .expect_err("expected send error");
    match err {
        mpsc::SendError(value) => assert_eq!(value, "ping"),
    }
}
