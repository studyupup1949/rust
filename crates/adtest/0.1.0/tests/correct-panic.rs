#[adtest::adtest]
fn test_this_doesnt_panic() {
    println!("No panic")
}

#[adtest::adtest]
fn test_this_doesnt_panic_error() -> Result<(), String> {
    println!("No panic");
    Ok(())
}

#[adtest::adtest]
async fn async_test_this_doesnt_panic() {
    println!("No panic")
}

#[adtest::adtest]
async fn async_test_this_doesnt_panic_error() -> Result<(), String> {
    println!("No panic");
    Ok(())
}

#[adtest::adtest]
#[should_panic]
fn test_panic_without_result() {
    panic!("Just a regular panic")
}

#[adtest::adtest]
#[should_panic]
fn test_panic_result() -> Result<(), &str> {
    Err("Error variant returned")
}

