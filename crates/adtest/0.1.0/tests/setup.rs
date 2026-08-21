use adtest::adtest;

#[adtest(
    setup = async async_setup_function
)]
fn test_with_setup() {
    println!("Do something")
}

fn setup_function() {
    println!("Doing setup");
}


#[adtest(setup = setup_function)]
async fn async_test_with_setup() {
    println!("Do something")
}

async fn async_setup_function() {
    println!("Doing setup");
}


#[adtest(setup = setup_with_return)]
fn test_setup_returning_value(){
    assert_eq!(_setup_, "Hello There")
}

fn setup_with_return() -> String{
    "Hello There".to_owned()
}