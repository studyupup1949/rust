use accepts::{Accepts, AsyncAccepts};

use crate::support::{Recorder, block_on};

use super::{AsyncRepeat, Repeat};

#[test]
fn repeats_value_fixed_times_with_new() {
    let recorder: Recorder<i32> = Recorder::new();
    let repeat: Repeat<i32, _, _> = Repeat::new(3, recorder.clone());

    repeat.accept(7);

    assert_eq!(recorder.take(), vec![7, 7, 7]);
}

#[test]
fn repeats_value_fixed_times_with_fn() {
    let recorder: Recorder<u32> = Recorder::new();
    let repeat = Repeat::with_fn(|v: &u32| (*v % 2 + 1) as usize, recorder.clone());

    repeat.accept(4); // 4 % 2 + 1 == 1
    repeat.accept(5); // 5 % 2 + 1 == 2

    assert_eq!(recorder.take(), vec![4, 5, 5]);
}

#[test]
fn does_not_forward_when_repeat_count_is_zero() {
    let recorder: Recorder<i32> = Recorder::new();
    let repeat: Repeat<i32, _, _> = Repeat::new(0, recorder.clone());

    repeat.accept(42);

    assert_eq!(recorder.take(), Vec::<i32>::new());
}

#[test]
fn repeats_value_fixed_times_async_with_new() {
    let recorder: Recorder<i32> = Recorder::new();
    let repeat: AsyncRepeat<i32, _, _, _> = AsyncRepeat::new(2, recorder.clone());

    block_on(repeat.accept_async(9));

    assert_eq!(recorder.take(), vec![9, 9]);
}

#[test]
fn repeats_value_with_async_count() {
    let recorder: Recorder<u32> = Recorder::new();
    let repeat = AsyncRepeat::with_fn(
        |v: &u32| {
            let v = *v;
            async move { (v % 3) as usize }
        },
        recorder.clone(),
    );

    block_on(repeat.accept_async(4)); // repeats once
    block_on(repeat.accept_async(5)); // repeats twice

    assert_eq!(recorder.take(), vec![4, 5, 5]);
}
