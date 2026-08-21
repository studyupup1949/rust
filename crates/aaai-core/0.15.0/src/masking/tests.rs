//! Integration tests for the masking engine.

use super::engine::MaskingEngine;

#[test]
fn connection_string_password_masked() {
    let engine = MaskingEngine::builtin();
    let text = "postgres://admin:my_secret_password@db.example.com:5432/mydb";
    let masked = engine.mask(text);
    assert!(masked.contains("***MASKED***"), "password in URL should be masked");
    assert!(!masked.contains("my_secret_password"));
}

#[test]
fn bearer_token_masked() {
    let engine = MaskingEngine::builtin();
    let text = "Authorization: Bearer eyJhbGciOiJSUzI1NiJ9.payload.signature";
    let masked = engine.mask(text);
    assert!(masked.contains("***MASKED***"));
    assert!(!masked.contains("eyJhbGciOiJSUzI1NiJ9"));
}

#[test]
fn multiple_secrets_all_masked() {
    let engine = MaskingEngine::builtin();
    let text = "api_key = 'abcdefghijklmnop1234'\npassword = secret123word\n";
    let masked = engine.mask(text);
    let mask_count = masked.matches("***MASKED***").count();
    assert!(mask_count >= 2, "both secrets should be masked, got {mask_count}");
}
