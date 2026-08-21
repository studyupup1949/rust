# abs_sync

Abstraction of synchronization for Rust async/await.

This crate provide traits about locks in sync and/or async environment.

# Example

```rust
use core::{
    borrow::BorrowMut,
    ops::{ControlFlow, Deref, DerefMut},
};

use pin_utils::pin_mut;
use abs_sync::{
    cancellation::{NonCancellableToken, TrIntoFutureMayCancel},
    x_deps::pin_utils,
};

async fn demo<B, L, T>(rwlock: B)
where
    B: BorrowMut<L>,
    L: TrAsyncRwLock<Target = T>,
{
    let acq = rwlock.borrow().acquire();
    pin_mut!(acq);
    let read_async = acq.as_mut().read_async();
    // let write_async = acq.write_async(); // illegal
    let ControlFlow::Continue(read_guard) = read_async
        .may_cancel_with(NonCancellableToken::pinned())
        .await
        .branch()
    else {
        panic!()
    };
    let _ = read_guard.deref();
    // let write_async = acq.write_async(); // illegal
    drop(read_guard);
    let ControlFlow::Continue(upgradable) = acq
        .as_mut()
        .upgradable_read_async()
        .may_cancel_with(NonCancellableToken::pinned())
        .await
        .branch()
    else {
        panic!()
    };
    let _ = upgradable.deref();
    let upgrade = upgradable.upgrade();
    pin_mut!(upgrade);
    let ControlFlow::Continue(mut write_guard) = upgrade
        .upgrade_async()
        .may_cancel_with(NonCancellableToken::pinned())
        .await
        .branch()
    else {
        panic!()
    };
    let _ = write_guard.deref_mut();
    let upgradable = write_guard.downgrade_to_upgradable();
    drop(upgradable)
}
```