use actix_web::client::Client;
use actix_webfinger::Webfinger;
use std::error::Error;

#[actix_rt::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::default();
    let wf = Webfinger::fetch(&client, "asonix@asonix.dog", "localhost:8000", false).await?;

    println!("asonix's webfinger:\n{:#?}", wf);
    Ok(())
}
