#![feature(custom_test_frameworks)]
#![test_runner(admiral::runner::runner)]

use admiral_derive::rust_decorator_fn;
async fn foo_bar() {
    println!("doing foo");
}

#[rust_decorator_fn]
pub async fn test_one() {
    let mut _i = 3;
    foo_bar().await;
    _i = 4;
    assert!(1 + 1 == 2);
}

#[rust_decorator_fn]
#[should_panic]
pub async fn test_two() {
    let mut _i = 3;
    foo_bar().await;
    _i = 4;
    assert!(1 + 1 == 1);
}
