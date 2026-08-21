use abpilot_cc_sdk::AbpilotClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AbpilotClient::new();
    let email = "hailongz@qq.com";

    println!("Testing send_verification_code...");
    println!("Email: {}", email);
    println!("URL: https://wpyi6ctkdvfcxbqtmy6d6tkesi0yzzid.lambda-url.us-east-1.on.aws/auth/send-code");
    
    // Test with reqwest directly to see what's happening
    let http_client = reqwest::Client::new();
    let url = "https://wpyi6ctkdvfcxbqtmy6d6tkesi0yzzid.lambda-url.us-east-1.on.aws/auth/send-code";
    
    let body = serde_json::json!({
        "email": email
    });
    
    println!("\nSending request with body: {}", body);
    
    let response = http_client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;
    
    let status = response.status();
    let response_text = response.text().await?;
    
    println!("\nResponse status: {}", status);
    println!("Response body: {}", response_text);
    
    if status.is_success() {
        println!("\n✅ Request successful!");
        println!("Check your email at: {}", email);
    } else {
        println!("\n❌ Request failed!");
    }
    
    // Now test with SDK
    println!("\n\nTesting with SDK...");
    match client.mp().send_verification_code(email).await {
        Ok(_) => println!("✅ SDK request successful!"),
        Err(e) => println!("❌ SDK request failed: {}", e),
    }

    Ok(())
}
