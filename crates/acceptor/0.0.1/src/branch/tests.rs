use accepts::{Accepts, AsyncAccepts};

use crate::support::{Recorder, block_on};

use super::{AsyncBranch, Branch};

#[test]
fn routes_to_true_or_false() {
    let true_recorder = Recorder::new();
    let false_recorder = Recorder::new();

    let branch = Branch::new(
        |v: &i32| *v >= 0,
        true_recorder.clone(),
        false_recorder.clone(),
    );

    branch.accept(5);
    branch.accept(-3);

    assert_eq!(true_recorder.take(), vec![5]);
    assert_eq!(false_recorder.take(), vec![-3]);
}

#[test]
fn routes_async_to_true_or_false() {
    let true_recorder = Recorder::new();
    let false_recorder = Recorder::new();

    let branch = AsyncBranch::new(
        |v: &i32| {
            let v = *v;
            async move { v % 2 == 0 }
        },
        true_recorder.clone(),
        false_recorder.clone(),
    );

    block_on(branch.accept_async(2));
    block_on(branch.accept_async(3));

    assert_eq!(true_recorder.take(), vec![2]);
    assert_eq!(false_recorder.take(), vec![3]);
}
