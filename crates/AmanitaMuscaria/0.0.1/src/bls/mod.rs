//! # BLS Signatures
//! 
//! BLS Signatures are elliptic curve based signature schemes that are simple to understand, can aggregate signatures into a single signature, are secure, and are fast.
//! 
//! ## Aggregation
//! 
//! When we aggregate the signatures, you must be sure to keep the **order** of the messages and public keys the same.

pub mod bls_signature;