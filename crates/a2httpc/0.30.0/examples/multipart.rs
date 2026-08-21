fn main() -> a2httpc::Result {
    env_logger::init();

    let file = a2httpc::MultipartFile::new("file", b"Hello, world!")
        .with_type("text/plain")?
        .with_filename("hello.txt");
    let form = a2httpc::MultipartBuilder::new()
        .with_text("Hello", "world!")
        .with_file(file)
        .build()?;

    let resp = a2httpc::post("http://httpbin.org/post").body(form).send()?;

    println!("Status: {:?}", resp.status());
    println!("Headers:\n{:#?}", resp.headers());
    println!("Body:\n{}", resp.text()?);

    Ok(())
}
