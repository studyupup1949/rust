use accepts::{Accepts, AsyncAccepts};

use crate::support::{Recorder, block_on};

use super::{AsyncFilter, Filter};

#[test]
fn filters_values_synchronously() {
    let recorder = Recorder::new();
    let filter = Filter::new(|v: &u32| *v % 2 == 0, recorder.clone());

    filter.accept(1);
    filter.accept(2);
    filter.accept(3);
    filter.accept(4);

    assert_eq!(recorder.take(), vec![2, 4]);
}

#[test]
fn filters_values_asynchronously() {
    let recorder = Recorder::new();
    let filter = AsyncFilter::new(
        |v: &u32| {
            let v = *v;
            async move { v > 5 }
        },
        recorder.clone(),
    );

    block_on(filter.accept_async(4));
    block_on(filter.accept_async(6));
    block_on(filter.accept_async(7));

    assert_eq!(recorder.take(), vec![6, 7]);
}
