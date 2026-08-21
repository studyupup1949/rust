use abpilot_cc_sdk::{AbpilotClient, AuthMethod};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Create API Key ===\n");

    let token = env::var("ABPILOT_TOKEN")
        .expect("ABPILOT_TOKEN environment variable not set");

    let mut client = AbpilotClient::new();
    client.mp_mut().set_auth(AuthMethod::jwt(token));

    println!("Creating API key...");
    let api_key = client.mp().create_api_key("My API Key").await?;
    
    println!("\n✅ API Key created successfully!");
    println!("\n========================================");
    println!("Your API Key:");
    println!("========================================");
    println!("{}", api_key.apikey);
    println!("========================================");
    println!("\nName: {}", api_key.name);
    println!("\nTo use this API key, run:");
    println!("  export ABPILOT_API_KEY=\"{}\"", api_key.apikey);
    println!("  cargo run --example full_test_with_token --all-features");

    Ok(())
}
