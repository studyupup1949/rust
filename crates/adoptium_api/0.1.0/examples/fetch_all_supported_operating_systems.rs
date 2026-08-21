use adoptium_api::v3::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Build your request
    let endpoint = Adoptium::production(types::OperationgSystems::new());

    // Now you can get it's URL
    let url = endpoint.try_as_url()?;
    println!("URL: {url}");

    // Or get a parsed response body if endpoint implements [`GetRequest`] trait.
    let response = endpoint.get().await?;
    println!("Parsed response: {response:#?}");

    Ok(())
}
