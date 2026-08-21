use std::fs;
use std::path::PathBuf;

/// Test 17: test_inline_attributes_present
/// Ensure that `#[inline(always)]` is present on hot path functions.
/// We do this by scanning the source files, as Rust doesn't provide reflection
/// for inline attributes at runtime.
#[test]
fn test_inline_attributes_present() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let service_rs_path = manifest_dir.join("src").join("compat").join("tower").join("service.rs");
    
    let content = fs::read_to_string(service_rs_path).expect("Could not read service.rs");

    // We expect several #[inline(always)] in service.rs
    let inline_count = content.matches("#[inline(always)]").count();
    assert!(inline_count >= 5, "Expected at least 5 #[inline(always)] annotations in service.rs, found {}", inline_count);
}

/// Test 18: test_lto_codegen_units
/// Parse Cargo.toml to ensure `lto = "thin"` and `codegen-units = 1` are set for the release profile.
#[test]
fn test_lto_codegen_units() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cargo_toml_path = manifest_dir.join("Cargo.toml");
    
    let content = fs::read_to_string(cargo_toml_path).expect("Could not read Cargo.toml");

    let has_lto = content.contains(r#"lto = "thin""#);
    let has_codegen_units = content.contains("codegen-units = 1");

    assert!(has_lto, "Cargo.toml is missing `lto = \"thin\"` in [profile.release]");
    assert!(has_codegen_units, "Cargo.toml is missing `codegen-units = 1` in [profile.release]");
}
