//! LOW-1: CORE_VERSION constant sync test.
//!
//! Proves that `types::CORE_VERSION` matches the version declared in
//! `Cargo.toml`. The constant is used in the User-Agent header
//! (`ace-sdk-rust/{CORE_VERSION}`) so a stale value sends the wrong
//! version string to the server. The check is dynamic (compares against
//! `CARGO_PKG_VERSION`) so it stays correct across version bumps.

use ace_sdk_core::CORE_VERSION;

/// Version declared in Cargo.toml — embedded at compile time.
const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

#[test]
fn should_match_cargo_toml_when_core_version_constant_is_in_sync() {
    assert_eq!(
        CORE_VERSION, CARGO_PKG_VERSION,
        "CORE_VERSION constant ({}) must equal Cargo.toml version ({}). \
         Update types.rs to fix this drift.",
        CORE_VERSION, CARGO_PKG_VERSION
    );
}

#[test]
fn should_not_be_stale_0_5_0_placeholder() {
    assert_ne!(
        CORE_VERSION, "0.5.0",
        "CORE_VERSION is still the stale '0.5.0' placeholder — update \
         types.rs to match Cargo.toml version '{}'.",
        CARGO_PKG_VERSION
    );
}
