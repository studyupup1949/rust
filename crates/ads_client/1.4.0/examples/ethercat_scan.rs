
fn main() {
    //f().unwrap();
}

// fn f() -> Result<(), anyhow::Error> {
//     g().map_err(|e| e.context(format!("at {}:{}:{}", file!(), line!(), column!())))?;
//     Err(anyhow::anyhow!("some other error"))
// }

// fn g() -> Result<(), anyhow::Error> {
//     Err(anyhow::anyhow!("oh noes"))
// }

// fn main() -> Result<()>{
//     let rt = Runtime::new().unwrap();
//     let ads_client = rt.block_on(Client::new("5.80.201.232.1.1", 10000, AdsTimeout::DefaultTimeout)).unwrap();

//     let ads_state = rt.block_on(ads_client.read_state())?;
//     //println!("State: {:?}", ads_client.read_state().unwrap());
//     // match rt.block_on(ads_client.read_state()) {
//     //     Ok(state) => println!("State: {:?}", state),
//     //     Err(err) => println!("Error: {}", err.to_string())
//     // }
//     Ok(())
// }