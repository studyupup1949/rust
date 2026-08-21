use accepts::Accepts;

use crate::support::Recorder;

use super::Batch;

#[test]
fn batches_based_on_predicate_and_flushes_remaining() {
    let recorder = Recorder::new();

    let batch =
        Batch::<u32, Vec<u32>, _, _>::new(|buffer, _last| buffer.len() >= 3, recorder.clone());

    batch.accept(1);
    batch.accept(2);
    // third element satisfies the predicate and flushes
    batch.accept(3);
    // stays buffered until flushed manually
    batch.accept(4);

    batch.flush();

    assert_eq!(recorder.take(), vec![vec![1, 2, 3], vec![4]]);
}
