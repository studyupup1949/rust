use core::sync::atomic::{AtomicUsize, Ordering};

use accepts::{Accepts, AsyncAccepts};

use crate::support::{Recorder, block_on};

use super::{Around, AsyncAround};

#[test]
fn around_runs_hooks_and_forwards() {
    let before = AtomicUsize::new(0);
    let after = AtomicUsize::new(0);
    let rec = Recorder::new();

    let adapter = Around::new(
        (),
        |_: &()| {
            before.fetch_add(1, Ordering::SeqCst);
        },
        |_: &()| {
            after.fetch_add(1, Ordering::SeqCst);
        },
        rec.clone(),
    );

    adapter.accept(7u32);

    assert_eq!(before.load(Ordering::SeqCst), 1);
    assert_eq!(after.load(Ordering::SeqCst), 1);
    assert_eq!(rec.take(), vec![7]);
}

#[test]
fn async_around_runs_hooks_and_forwards() {
    let before = AtomicUsize::new(0);
    let after = AtomicUsize::new(0);
    let rec = Recorder::new();

    let adapter = AsyncAround::new(
        (),
        |_: &()| {
            let before = &before;
            async move {
                before.fetch_add(1, Ordering::SeqCst);
            }
        },
        |_: &()| {
            let after = &after;
            async move {
                after.fetch_add(1, Ordering::SeqCst);
            }
        },
        rec.clone(),
    );

    block_on(adapter.accept_async(9u32));

    assert_eq!(before.load(Ordering::SeqCst), 1);
    assert_eq!(after.load(Ordering::SeqCst), 1);
    assert_eq!(rec.take(), vec![9]);
}
