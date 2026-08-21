# ADTEST

[![codecov](https://codecov.io/gh/0xMimir/adtest/graph/badge.svg?token=w6KaZt9nuN)](https://codecov.io/gh/0xMimir/adtest)

This crate allows you to easily create tests with setup and cleanup functions, like `beforeEach` and `afterEach` functions in jest,
it offers this functionality for both async and non-async test.

To use simply add to your crate in `lib.rs` or `main.rs`
```rust
#[macro_use] extern crate adtest;
```

After that add `#[adtest]` to desired function
```rust
#[adtest]
fn complex_test(){
    println!("I like to test things");
}
```

If used solely it behaves as `#[test]` on non async function and on async functions as `#[tokio::test]`.
But unlike those, `#[adtest]` allows you to add setup and clean up functions to run before/after your tests.

Example of test with setup
```rust
#[adtest(setup = setup_function)]
fn complex_test(){
    println!("I like to test things");
}

fn setup_function(){
    println!("I will do some setup things");
}
```

If your setup/cleanup function is async you must specify it with `async` keyword before test name:
```rust
#[adtest(
    setup = setup_function,
    cleanup = async cleanup_function
)]
fn complex_test(){
    println!("I like to test things");
}

fn setup_function(){
    println!("I will do some setup things");
}

async fn cleanup_function(){
    println!("I am async function")
}
```