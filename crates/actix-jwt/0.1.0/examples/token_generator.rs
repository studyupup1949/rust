// Run: cargo run --example token_generator
//
// Demonstrates TokenGenerator functionality without HTTP server.
// Generates access + refresh token pairs and shows token rotation.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use actix_jwt::{ActixJwtMiddleware, JwtError};
use serde_json::Value;

#[tokio::main]
async fn main() {
    let mut jwt = ActixJwtMiddleware::new();
    jwt.realm = "example zone".to_string();
    jwt.key = b"secret key".to_vec();
    jwt.timeout = Duration::from_secs(3600);
    jwt.max_refresh = Duration::from_secs(86400);
    jwt.refresh_token_timeout = Duration::from_secs(86400);

    jwt.authenticator = Some(Arc::new(
        |_req: &actix_web::HttpRequest, _body: &[u8]| -> Result<Value, JwtError> {
            Err(JwtError::MissingLoginValues)
        },
    ));
    jwt.payload_func = Some(Arc::new(|data: &Value| {
        let mut claims = HashMap::new();
        claims.insert("user_id".to_string(), data.clone());
        claims
    }));
    jwt.init().expect("JWT init failed");

    let user_data = serde_json::json!("user123");

    // Generate a complete token pair (access + refresh tokens)
    println!("=== Generating Token Pair ===");
    let token_pair = jwt.token_generator(&user_data).await.unwrap();

    println!(
        "Access Token: {}...",
        &token_pair.access_token[..50.min(token_pair.access_token.len())]
    );
    println!("Token Type: {}", token_pair.token_type);
    println!(
        "Refresh Token: {}",
        token_pair.refresh_token.as_deref().unwrap_or("(none)")
    );
    println!("Expires At: {}", token_pair.expires_at);
    println!("Created At: {}", token_pair.created_at);
    println!("Expires In: {} seconds", token_pair.expires_in());

    // Simulate refresh token usage
    println!("\n=== Refreshing Token Pair ===");
    let old_refresh = token_pair.refresh_token.as_ref().unwrap();
    let new_token_pair = jwt
        .token_generator_with_revocation(&user_data, old_refresh)
        .await
        .unwrap();

    println!(
        "New Access Token: {}...",
        &new_token_pair.access_token[..50.min(new_token_pair.access_token.len())]
    );
    println!(
        "New Refresh Token: {}",
        new_token_pair.refresh_token.as_deref().unwrap_or("(none)")
    );
    println!(
        "Old refresh token revoked: {}",
        old_refresh != new_token_pair.refresh_token.as_deref().unwrap_or("")
    );

    // Verify old refresh token is invalid
    println!("\n=== Verifying Old Token Revocation ===");
    let result = jwt
        .token_generator_with_revocation(&user_data, old_refresh)
        .await;
    match result {
        Err(e) => println!("Old refresh token correctly rejected: {}", e),
        Ok(_) => println!("WARNING: Old refresh token was NOT revoked!"),
    }

    println!("\n=== Token Generation Complete! ===");
    println!("You can now use these tokens without needing middleware handlers!");
}
