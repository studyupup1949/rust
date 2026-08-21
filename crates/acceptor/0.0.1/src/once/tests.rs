use accepts::{Accepts, AsyncAccepts};

use crate::support::{Recorder, block_on};

use super::{AsyncOnce, Once};

#[test]
fn forwards_only_first_value() {
    let recorder = Recorder::new();
    let once = Once::new(recorder.clone());

    once.accept(1);
    once.accept(2);
    once.accept(3);

    assert_eq!(recorder.take(), vec![1]);
}

#[test]
fn forwards_only_first_value_async() {
    let recorder = Recorder::new();
    let once = AsyncOnce::new(recorder.clone());

    block_on(once.accept_async(4));
    block_on(once.accept_async(5));

    assert_eq!(recorder.take(), vec![4]);
}
