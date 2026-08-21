use core::sync::atomic::{AtomicUsize, Ordering};

use accepts::{Accepts, AsyncAccepts};

use crate::support::{Recorder, block_on};

use super::{AsyncInspect, Inspect};

#[test]
fn inspect_runs_before_forwarding() {
    let recorder = Recorder::new();
    let seen = AtomicUsize::new(0);

    let inspector = Inspect::new(
        |_: &u32| {
            seen.fetch_add(1, Ordering::SeqCst);
        },
        recorder.clone(),
    );

    inspector.accept(1);
    inspector.accept(2);

    assert_eq!(seen.load(Ordering::SeqCst), 2);
    assert_eq!(recorder.take(), vec![1, 2]);
}

#[test]
fn async_inspect_runs_before_forwarding() {
    let recorder = Recorder::new();
    let seen = AtomicUsize::new(0);

    let inspector = AsyncInspect::new(
        |_: &u32| {
            let seen = &seen;
            async move {
                seen.fetch_add(1, Ordering::SeqCst);
            }
        },
        recorder.clone(),
    );

    block_on(inspector.accept_async(3));
    block_on(inspector.accept_async(4));

    assert_eq!(seen.load(Ordering::SeqCst), 2);
    assert_eq!(recorder.take(), vec![3, 4]);
}
