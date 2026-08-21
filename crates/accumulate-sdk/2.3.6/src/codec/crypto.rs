//! Cryptographic utilities for Accumulate protocol codec
//!
//! This module re-exports the main crypto utilities from the crypto module.
//! For Ed25519 operations, use `crate::crypto::ed25519` or `crate::crypto::ed25519_helper`.
//! For hash operations, use `crate::codec::hash_helper`.

// This module exists for backwards compatibility.
// The main crypto implementations are in:
// - crate::crypto::ed25519 - Ed25519 signing and verification
// - crate::crypto::ed25519_helper - Key derivation utilities
// - crate::codec::hash_helper - Hash computation utilities
