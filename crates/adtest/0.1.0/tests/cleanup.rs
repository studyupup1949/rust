use adtest::adtest;

#[adtest(
    cleanup = async async_cleanup_function
)]
fn test_with_cleanup(){
    println!("Do something")
}

fn cleanup_function(){
    println!("Doing cleanup");
}

#[adtest(
    cleanup = cleanup_function
)]
async fn async_test_with_cleanup(){
    println!("Do something")
}

async fn async_cleanup_function(){
    println!("Doing cleanup");
}