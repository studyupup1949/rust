use accepts::{Accepts, AsyncAccepts};

use crate::support::{Recorder, block_on};

use super::{AsyncStatefulCallback, StatefulCallback};

#[test]
fn stateful_callback_routes_via_state() {
    #[derive(Clone)]
    struct State {
        even: Recorder<u32>,
        odd: Recorder<u32>,
    }

    let even = Recorder::new();
    let odd = Recorder::new();
    let state = State {
        even: even.clone(),
        odd: odd.clone(),
    };

    let cb = StatefulCallback::new(state, |state: &State, value: u32| {
        if value % 2 == 0 {
            state.even.accept(value);
        } else {
            state.odd.accept(value);
        }
    });

    cb.accept(1);
    cb.accept(2);
    cb.accept(3);

    assert_eq!(even.take(), vec![2]);
    assert_eq!(odd.take(), vec![1, 3]);
}

#[test]
fn async_stateful_callback_can_use_multiple_nexts() {
    #[derive(Clone)]
    struct State {
        primary: Recorder<u32>,
        mirror: Recorder<u32>,
    }

    let primary = Recorder::new();
    let mirror = Recorder::new();
    let state = State {
        primary: primary.clone(),
        mirror: mirror.clone(),
    };

    let cb = AsyncStatefulCallback::new(state, |state: &State, value: u32| {
        let primary = state.primary.clone();
        let mirror = state.mirror.clone();
        async move {
            primary.accept(value);
            mirror.accept(value + 10);
        }
    });

    block_on(cb.accept_async(4));
    block_on(cb.accept_async(5));

    assert_eq!(primary.take(), vec![4, 5]);
    assert_eq!(mirror.take(), vec![14, 15]);
}
