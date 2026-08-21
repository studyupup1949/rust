use abpilot_cc_sdk::{AbpilotClient, Config};

#[cfg(feature = "mp")]
#[tokio::test]
async fn test_mp_client_creation() {
    let client = AbpilotClient::new();
    assert!(client.mp().list_apps().await.is_err()); // Should fail without auth
}

#[cfg(feature = "app")]
#[tokio::test]
async fn test_app_client_creation() {
    let client = AbpilotClient::new();
    // Client should be created successfully
    let result = client.app()
        .list_assets("test_app", "test_secret", "test_device", "test_world")
        .await;
    assert!(result.is_err()); // Should fail with invalid credentials
}

#[test]
fn test_config_builder() {
    let config = Config::new();
    
    #[cfg(feature = "mp")]
    assert!(!config.mp_base_url.is_empty());
    
    #[cfg(feature = "app")]
    assert!(!config.app_base_url.is_empty());
    
    assert_eq!(config.timeout.as_secs(), 30);
    assert_eq!(config.max_retries, 3);
}

#[test]
#[cfg(feature = "mp")]
fn test_config_with_custom_mp_url() {
    let config = Config::new()
        .with_mp_base_url("https://custom.example.com");
    
    assert_eq!(config.mp_base_url, "https://custom.example.com");
}

#[test]
#[cfg(feature = "app")]
fn test_config_with_custom_app_url() {
    let config = Config::new()
        .with_app_base_url("https://custom.example.com");
    
    assert_eq!(config.app_base_url, "https://custom.example.com");
}

#[test]
#[cfg(feature = "app")]
fn test_signature_generation() {
    use abpilot_cc_sdk::SignatureGenerator;
    
    let generator = SignatureGenerator::new("test_secret");
    let (sig1, ts1) = generator.generate_app_signature("test_app_id");
    
    // Wait a moment to ensure different timestamp
    std::thread::sleep(std::time::Duration::from_millis(1001));
    
    let (sig2, ts2) = generator.generate_app_signature("test_app_id");
    
    // Signatures should be different due to different timestamps
    assert_ne!(sig1, sig2);
    assert!(ts2 > ts1);
    
    // Signature should be 64 characters (SHA256 hex)
    assert_eq!(sig1.len(), 64);
}

#[test]
#[cfg(feature = "mp")]
fn test_auth_method_jwt() {
    use abpilot_cc_sdk::AuthMethod;
    
    let auth = AuthMethod::jwt("test_token");
    match auth {
        AuthMethod::JwtToken(token) => assert_eq!(token, "test_token"),
        _ => panic!("Expected JwtToken"),
    }
}

#[test]
#[cfg(feature = "mp")]
fn test_auth_method_api_key() {
    use abpilot_cc_sdk::AuthMethod;
    
    let auth = AuthMethod::api_key("sk_test_key");
    match auth {
        AuthMethod::ApiKey(key) => assert_eq!(key, "sk_test_key"),
        _ => panic!("Expected ApiKey"),
    }
}

#[test]
#[cfg(feature = "app")]
fn test_auth_method_app_signature() {
    use abpilot_cc_sdk::AuthMethod;
    
    let auth = AuthMethod::app_signature("app_id", "secret");
    match auth {
        AuthMethod::AppSignature { app_id, secret } => {
            assert_eq!(app_id, "app_id");
            assert_eq!(secret, "secret");
        }
        _ => panic!("Expected AppSignature"),
    }
}

#[test]
#[cfg(feature = "app")]
fn test_auth_method_world_signature() {
    use abpilot_cc_sdk::AuthMethod;
    
    let auth = AuthMethod::world_signature("world_id", "secret");
    match auth {
        AuthMethod::WorldSignature { world_id, secret } => {
            assert_eq!(world_id, "world_id");
            assert_eq!(secret, "secret");
        }
        _ => panic!("Expected WorldSignature"),
    }
}
