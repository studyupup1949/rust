use serde_json::json;

fn main() -> a2httpc::Result {
    env_logger::init();

    let body = json!({
        "hello": "world",
    });

    let resp = a2httpc::post("http://httpbin.org/post").json(&body)?.send()?;
    println!("Status: {:?}", resp.status());
    println!("Headers:\n{:#?}", resp.headers());
    println!("Body:\n{}", resp.text_utf8()?);

    Ok(())
}
