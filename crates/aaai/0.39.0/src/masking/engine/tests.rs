use super::*;

fn engine() -> MaskingEngine { MaskingEngine::builtin() }

#[test]
fn masks_api_key_assignment() {
    let text = r#"api_key = "sk-abcdefghijklmnop1234567890""#;
    let masked = engine().mask(text);
    assert!(masked.contains(MASK), "expected mask in: {masked}");
    assert!(!masked.contains("sk-abcdefghijklmnop"), "key should be masked");
}

#[test]
fn masks_aws_access_key() {
    let text = "AKIAIOSFODNN7EXAMPLE";
    let masked = engine().mask(text);
    assert!(masked.contains(MASK));
}

#[test]
fn safe_text_unchanged() {
    let text = "port = 8080";
    let masked = engine().mask(text);
    assert_eq!(masked, text);
}

#[test]
fn mask_if_needed_returns_none_for_safe_text() {
    assert!(engine().mask_if_needed("ordinary text").is_none());
}

#[test]
fn custom_pattern_applied() {
    let engine = MaskingEngine::with_custom(&["SECRET_VALUE".to_string()]);
    let masked = engine.mask("here is SECRET_VALUE exposed");
    assert!(masked.contains(MASK));
}

#[test]
fn private_key_header_masked() {
    let text = "-----BEGIN RSA PRIVATE KEY-----\nABC123\n-----END RSA PRIVATE KEY-----";
    let masked = engine().mask(text);
    assert!(masked.contains(MASK));
}

#[test]
fn github_token_masked() {
    let text = "token = ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdef123456";
    let masked = engine().mask(text);
    assert!(masked.contains(MASK));
}
