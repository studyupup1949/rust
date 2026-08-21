use std::collections::BTreeMap;

use accepts::{Accepts, AsyncAccepts};

use crate::{
    BTreeKVRouter,
    support::{Recorder, block_on},
};

#[test]
fn routes_to_matching_entry() {
    let recorder_a: Recorder<(char, i32)> = Recorder::default();
    let recorder_b: Recorder<(char, i32)> = Recorder::default();
    let fallback: Recorder<(char, i32)> = Recorder::default();

    let mut routes = BTreeMap::new();
    routes.insert('a', recorder_a.clone());
    routes.insert('b', recorder_b.clone());

    let router = BTreeKVRouter::with_fallback(routes, fallback.clone());

    router.accept(('a', 1));
    router.accept(('b', 2));
    router.accept(('c', 3));

    assert_eq!(recorder_a.take(), vec![('a', 1)]);
    assert_eq!(recorder_b.take(), vec![('b', 2)]);
    assert_eq!(fallback.take(), vec![('c', 3)]);
}

#[test]
fn routes_to_fallback_when_route_missing() {
    let fallback: Recorder<(char, u8)> = Recorder::default();
    let routes: BTreeMap<char, Recorder<(char, u8)>> = BTreeMap::new();

    let router = BTreeKVRouter::with_fallback(routes, fallback.clone());

    router.accept(('x', 9));

    assert_eq!(fallback.take(), vec![('x', 9)]);
}

#[test]
fn async_routing_works() {
    let recorder_a: Recorder<(u32, &'static str)> = Recorder::default();
    let fallback: Recorder<(u32, &'static str)> = Recorder::default();

    let mut routes = BTreeMap::new();
    routes.insert(1_u32, recorder_a.clone());

    let router = BTreeKVRouter::with_fallback(routes, fallback.clone());

    block_on(router.accept_async((1, "hit")));
    block_on(router.accept_async((2, "miss")));

    assert_eq!(recorder_a.take(), vec![(1, "hit")]);
    assert_eq!(fallback.take(), vec![(2, "miss")]);
}
