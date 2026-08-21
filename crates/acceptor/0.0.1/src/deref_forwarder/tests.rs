use std::sync::Arc;

use accepts::{Accepts, AsyncAccepts};

use crate::support::{Recorder, block_on};

use super::DerefForwarder;

#[test]
fn forwards_through_deref() {
    let recorder = Recorder::new();
    let forwarder = DerefForwarder::<u32, _, _>::new(Arc::new(recorder.clone()));

    forwarder.accept(10);

    assert_eq!(recorder.take(), vec![10]);
}

#[test]
fn forwards_async_through_deref() {
    let recorder = Recorder::new();
    let forwarder = DerefForwarder::<u32, _, _>::new(Arc::new(recorder.clone()));

    block_on(forwarder.accept_async(42));

    assert_eq!(recorder.take(), vec![42]);
}
