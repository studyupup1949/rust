fn main() -> a2httpc::Result {
    env_logger::init();

    let resp = a2httpc::get("https://statsapi.web.nhl.com/api/v1/schedule").send()?;
    println!("Status: {:?}", resp.status());
    println!("Headers:\n{:#?}", resp.headers());
    println!("Body:\n{}", resp.text()?);

    Ok(())
}
