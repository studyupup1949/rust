use abpilot_cc_sdk::{AbpilotClient, AuthMethod};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AbpilotClient::new();
    
    // Step 1: Send verification code
    println!("Sending verification code...");
    client.mp().send_verification_code("user@example.com").await?;
    println!("Code sent! Check your email.");
    
    // Step 2: Verify code (in real scenario, user would input the code)
    println!("\nVerifying code...");
    let auth_token = client.mp()
        .verify_code("user@example.com", "123456")
        .await?;
    
    println!("Authentication successful!");
    println!("User ID: {}", auth_token.user_id);
    println!("Token: {}", auth_token.token);
    
    // Step 3: Create authenticated client
    let mut authed_client = client.clone();
    authed_client.mp_mut().set_auth(AuthMethod::jwt(auth_token.token));
    
    // Step 4: Create an API key for future use
    println!("\nCreating API key...");
    let api_key = authed_client.mp().create_api_key("My API Key").await?;
    println!("API Key created: {}", api_key.apikey);
    
    // Step 5: List all API keys
    println!("\nListing all API keys...");
    let api_keys = authed_client.mp().list_api_keys().await?;
    for key in api_keys {
        println!("  - {} ({})", key.name, key.apikey);
    }
    
    Ok(())
}
