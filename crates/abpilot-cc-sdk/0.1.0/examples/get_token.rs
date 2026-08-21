use abpilot_cc_sdk::AbpilotClient;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ABPilot CC - Get Authentication Token ===\n");

    let client = AbpilotClient::new();

    // Step 1: Get email
    print!("Enter your email: ");
    io::stdout().flush()?;
    let mut email = String::new();
    io::stdin().read_line(&mut email)?;
    let email = email.trim();

    // Step 2: Send verification code
    println!("\nSending verification code to {}...", email);
    match client.mp().send_verification_code(email).await {
        Ok(_) => {
            println!("✅ Verification code sent!");
            println!("   Check your email (including spam folder)");
        }
        Err(e) => {
            eprintln!("❌ Failed to send code: {}", e);
            eprintln!("\nPossible reasons:");
            eprintln!("  - SMTP not configured on backend");
            eprintln!("  - Invalid email address");
            eprintln!("  - Network issues");
            return Err(e.into());
        }
    }

    // Step 3: Get verification code
    print!("\nEnter the 6-digit code from your email: ");
    io::stdout().flush()?;
    let mut code = String::new();
    io::stdin().read_line(&mut code)?;
    let code = code.trim();

    // Step 4: Verify code
    println!("Verifying code...");
    match client.mp().verify_code(email, code).await {
        Ok(token) => {
            println!("\n✅ Authentication successful!");
            println!("\n========================================");
            println!("Your JWT Token:");
            println!("========================================");
            println!("{}", token.token);
            println!("========================================");
            println!("\nUser ID: {}", token.user_id);
            println!("\nTo use this token, run:");
            println!("  export ABPILOT_TOKEN=\"{}\"", token.token);
            println!("  cargo run --example full_test_with_token --all-features");
            println!("\nOr create an API key:");
            println!("  ABPILOT_TOKEN=\"{}\" cargo run --example create_api_key --all-features", token.token);
        }
        Err(e) => {
            eprintln!("\n❌ Verification failed: {}", e);
            eprintln!("\nPossible reasons:");
            eprintln!("  - Invalid code");
            eprintln!("  - Code expired (5 minutes)");
            eprintln!("  - Code already used");
            return Err(e.into());
        }
    }

    Ok(())
}
