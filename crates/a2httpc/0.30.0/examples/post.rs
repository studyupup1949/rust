fn main() -> a2httpc::Result {
    env_logger::init();

    let resp = a2httpc::post("https://httpbin.org/post").text("hello, world!").send()?;

    println!("Status: {:?}", resp.status());
    println!("Headers:\n{:#?}", resp.headers());
    println!("Body:\n{}", resp.text()?);

    Ok(())
}
