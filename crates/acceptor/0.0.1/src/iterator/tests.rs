use accepts::{Accepts, AsyncAccepts};

use crate::support::{Recorder, block_on};

use super::{AsyncForEach, ForEach};

#[test]
fn forwards_all_items_from_iterator() {
    let recorder = Recorder::new();
    let for_each = ForEach::new(recorder.clone());

    for_each.accept(vec![1, 2, 3]);

    assert_eq!(recorder.take(), vec![1, 2, 3]);
}

#[test]
fn forwards_all_items_from_iterator_async() {
    let recorder = Recorder::new();
    let for_each = AsyncForEach::new(recorder.clone());

    block_on(for_each.accept_async(vec![4, 5]));

    assert_eq!(recorder.take(), vec![4, 5]);
}
