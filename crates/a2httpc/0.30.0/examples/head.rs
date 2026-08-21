fn main() -> a2httpc::Result {
    env_logger::init();

    let resp = a2httpc::head("http://httpbin.org").send()?;
    println!("Status: {:?}", resp.status());
    println!("Headers:\n{:#?}", resp.headers());

    Ok(())
}
