use crate::config::{Config, save_config};
use abpilot_cc_sdk::AbpilotClient;
use std::io::{self, Write};

pub async fn login() -> Result<(), Box<dyn std::error::Error>> {
    println!("Login to ABPilot");
    
    print!("Email: ");
    io::stdout().flush()?;
    let mut email = String::new();
    io::stdin().read_line(&mut email)?;
    let email = email.trim().to_string();
    
    println!("Sending verification code...");
    let client = AbpilotClient::new();
    client.mp().send_verification_code(&email).await?;
    
    print!("Verification code: ");
    io::stdout().flush()?;
    let mut code = String::new();
    io::stdin().read_line(&mut code)?;
    let code = code.trim().to_string();
    
    println!("Verifying...");
    let auth_token = client.mp().verify_code(&email, &code).await?;
    
    let mut authed_client = client.clone();
    authed_client.mp_mut().set_auth(abpilot_cc_sdk::AuthMethod::jwt(auth_token.token));
    
    println!("Fetching API keys...");
    let apikeys = authed_client.mp().list_api_keys().await?;
    
    let apikey = if let Some(first_key) = apikeys.first() {
        println!("Using existing API key: {}", first_key.name);
        first_key.apikey.clone()
    } else {
        println!("No API key found, creating new one...");
        let machine_name = hostname::get()?
            .to_string_lossy()
            .to_string();
        let new_key = authed_client.mp().create_api_key(&machine_name).await?;
        println!("Created API key: {}", machine_name);
        new_key.apikey
    };
    
    let config = Config { apikey };
    save_config(&config)?;
    
    println!("✓ Login successful!");
    Ok(())
}
