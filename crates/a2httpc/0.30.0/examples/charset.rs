fn main() -> Result<(), a2httpc::Error> {
    env_logger::init();

    let resp = a2httpc::get("https://rust-lang.org/").send()?;
    println!("{}", resp.text()?);
    Ok(())
}
