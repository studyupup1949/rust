use accepts::{Accepts, AsyncAccepts};

use crate::support::{Recorder, block_on};

use super::{AsyncMap, Map};

#[test]
fn maps_and_forwards_values() {
    let recorder = Recorder::new();
    let mapper = Map::new(|v: u32| v * 2, recorder.clone());

    mapper.accept(3);
    mapper.accept(4);

    assert_eq!(recorder.take(), vec![6, 8]);
}

#[test]
fn maps_asynchronously_and_forwards_values() {
    let recorder = Recorder::new();
    let mapper = AsyncMap::new(|v: u32| async move { v + 1 }, recorder.clone());

    block_on(mapper.accept_async(10));
    block_on(mapper.accept_async(20));

    assert_eq!(recorder.take(), vec![11, 21]);
}
