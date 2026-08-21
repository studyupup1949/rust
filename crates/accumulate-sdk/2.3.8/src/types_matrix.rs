//! Type matrix for comprehensive roundtrip testing
//!
//! This module contains all protocol types that need to be tested for
//! encode → decode → re-encode roundtrip consistency.

use serde::{Deserialize, Serialize};
use crate::codec::transaction_codec::{TransactionHeader, TransactionSignature};

/// All protocol type names that must pass roundtrip tests
pub const TYPE_NAMES: &[&str] = &[
    // Core transaction types
    "TransactionEnvelope",
    "TransactionHeader",
    "TransactionSignature",
    "TransactionKeyPage",

    // Transaction body builders and components
    "TokenRecipient",
    "KeySpec",

    // API response types
    "StatusResponse",
    "NodeInfo",
    "QueryResponse",
    "TransactionResponse",
    "TransactionResult",
    "Event",
    "Attribute",
    "SignedTransaction",
    "Signature",
    "Account",
    "FaucetResponse",

    // V3 specific types
    "V3SubmitRequest",
    "V3SubmitResponse",
    "SubmitResult",
    "V3Signature",

    // Protocol types
    "ProtocolTransactionEnvelope",
    "ProtocolTransactionSignature",
    "ProtocolTransactionHeader",

    // Codec support types
    "BinaryReader",
    "BinaryWriter",
    "EncodingError",
    "DecodingError",
    "FieldReader",

    // Crypto types
    "Ed25519Signer",

    // Client configuration
    "AccOptions",
];

/// Trait for types that can generate safe sample instances for testing
pub trait SampleGenerator {
    /// Generate a safe sample instance suitable for roundtrip testing
    fn generate_sample() -> Self;

    /// Generate multiple sample instances with different characteristics
    fn generate_samples() -> Vec<Self> where Self: Sized {
        vec![Self::generate_sample()]
    }
}

/// Trait for types that support roundtrip encoding/decoding
pub trait RoundtripTestable: Serialize + for<'de> Deserialize<'de> + Clone + PartialEq {
    /// Test JSON roundtrip: serialize → deserialize → re-serialize
    fn test_json_roundtrip(&self) -> Result<(), String> {
        // Serialize to JSON
        let json = serde_json::to_string(self)
            .map_err(|e| format!("Failed to serialize to JSON: {}", e))?;

        // Deserialize from JSON
        let deserialized: Self = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to deserialize from JSON: {}", e))?;

        // Re-serialize to JSON
        let json2 = serde_json::to_string(&deserialized)
            .map_err(|e| format!("Failed to re-serialize to JSON: {}", e))?;

        // Compare original and deserialized objects
        if self != &deserialized {
            return Err("Deserialized object differs from original".to_string());
        }

        // Compare JSON strings
        if json != json2 {
            return Err(format!(
                "Re-serialized JSON differs from original\nOriginal: {}\nRe-serialized: {}",
                json, json2
            ));
        }

        Ok(())
    }

    /// Test binary roundtrip if the type supports binary encoding
    fn test_binary_roundtrip(&self) -> Result<(), String> {
        // Default implementation - override for types with binary encoding
        Ok(())
    }
}

// Implement SampleGenerator for core transaction types

impl SampleGenerator for crate::codec::TransactionEnvelope {
    fn generate_sample() -> Self {
        use crate::codec::{TransactionHeader, TransactionSignature};

        Self {
            header: TransactionHeader {
                principal: "acc://alice.acme/tokens".to_string(),
                initiator: Some("acc://alice.acme".to_string()),
                timestamp: 1234567890123,
                nonce: Some(42),
                memo: Some("Test transaction".to_string()),
                metadata: Some(serde_json::json!({"test": "metadata"})),
            },
            body: serde_json::json!({
                "type": "send-tokens",
                "to": [{
                    "url": "acc://bob.acme/tokens",
                    "amount": "1000"
                }]
            }),
            signatures: vec![TransactionSignature::generate_sample()],
        }
    }

    fn generate_samples() -> Vec<Self> {
        vec![
            Self::generate_sample(),
            // Minimal envelope
            Self {
                header: TransactionHeader {
                    principal: "acc://test.acme".to_string(),
                    initiator: None,
                    timestamp: 1000000000000,
                    nonce: None,
                    memo: None,
                    metadata: None,
                },
                body: serde_json::json!({"type": "create-identity"}),
                signatures: vec![],
            },
            // Complex envelope with multiple signatures
            Self {
                header: TransactionHeader {
                    principal: "acc://complex.acme/tokens".to_string(),
                    initiator: Some("acc://initiator.acme".to_string()),
                    timestamp: 9999999999999,
                    nonce: Some(999999),
                    memo: Some("Complex test with unicode: test nono".to_string()),
                    metadata: Some(serde_json::json!({
                        "version": "1.0",
                        "flags": ["test", "complex"],
                        "nested": {"deep": {"value": 42}}
                    })),
                },
                body: serde_json::json!({
                    "type": "send-tokens",
                    "to": [
                        {"url": "acc://recipient1.acme/tokens", "amount": "100"},
                        {"url": "acc://recipient2.acme/tokens", "amount": "200"},
                        {"url": "acc://recipient3.acme/tokens", "amount": "300"}
                    ]
                }),
                signatures: vec![
                    TransactionSignature::generate_sample(),
                    TransactionSignature {
                        signature: vec![0x99; 64],
                        signer: "acc://signer2.acme/book/1".to_string(),
                        timestamp: 1234567890124,
                        vote: Some("approve".to_string()),
                        public_key: Some(vec![0xAA; 32]),
                        key_page: None,
                    }
                ],
            }
        ]
    }
}

impl SampleGenerator for crate::codec::TransactionHeader {
    fn generate_sample() -> Self {
        Self {
            principal: "acc://sample.acme/tokens".to_string(),
            initiator: Some("acc://sample.acme".to_string()),
            timestamp: 1234567890123,
            nonce: Some(1),
            memo: Some("Sample header".to_string()),
            metadata: Some(serde_json::json!({"sample": true})),
        }
    }

    fn generate_samples() -> Vec<Self> {
        vec![
            Self::generate_sample(),
            // Minimal header
            Self {
                principal: "acc://minimal.acme".to_string(),
                initiator: None,
                timestamp: 0,
                nonce: None,
                memo: None,
                metadata: None,
            },
            // Unicode and special characters
            Self {
                principal: "acc://üñíçødé.acme/tøkeñs".to_string(),
                initiator: Some("acc://spëçîál.acme".to_string()),
                timestamp: u64::MAX,
                nonce: Some(u64::MAX),
                memo: Some("Unicode test: star nono cafe resume".to_string()),
                metadata: Some(serde_json::json!({
                    "unicode": "test",
                    "special": "special chars: !@#$%^&*()_+-=[]{}|;':\",./<>?",
                    "nested": {"array": [1, 2, 3], "object": {"key": "value"}}
                })),
            }
        ]
    }
}

impl SampleGenerator for crate::codec::TransactionSignature {
    fn generate_sample() -> Self {
        Self {
            signature: vec![0x42; 64], // 64-byte signature
            signer: "acc://signer.acme/book/0".to_string(),
            timestamp: 1234567890000,
            vote: Some("approve".to_string()),
            public_key: Some(vec![0x33; 32]), // 32-byte public key
            key_page: Some(crate::codec::TransactionKeyPage {
                height: 1000,
                index: 0,
            }),
        }
    }

    fn generate_samples() -> Vec<Self> {
        vec![
            Self::generate_sample(),
            // Minimal signature
            Self {
                signature: vec![],
                signer: "acc://min.acme/book/0".to_string(),
                timestamp: 0,
                vote: None,
                public_key: None,
                key_page: None,
            },
            // Maximum values
            Self {
                signature: vec![0xFF; 128], // Large signature
                signer: "acc://very-long-signer-name-with-many-characters.acme/book/999".to_string(),
                timestamp: u64::MAX,
                vote: Some("reject".to_string()),
                public_key: Some(vec![0x00; 64]), // Large public key
                key_page: Some(crate::codec::TransactionKeyPage {
                    height: u64::MAX,
                    index: u32::MAX,
                }),
            }
        ]
    }
}

impl SampleGenerator for crate::codec::TransactionKeyPage {
    fn generate_sample() -> Self {
        Self {
            height: 12345,
            index: 42,
        }
    }

    fn generate_samples() -> Vec<Self> {
        vec![
            Self::generate_sample(),
            Self { height: 0, index: 0 },
            Self { height: u64::MAX, index: u32::MAX },
        ]
    }
}

impl SampleGenerator for crate::codec::TokenRecipient {
    fn generate_sample() -> Self {
        Self {
            url: "acc://recipient.acme/tokens".to_string(),
            amount: "1000".to_string(),
        }
    }

    fn generate_samples() -> Vec<Self> {
        vec![
            Self::generate_sample(),
            Self {
                url: "acc://zero.acme/tokens".to_string(),
                amount: "0".to_string(),
            },
            Self {
                url: "acc://big.acme/tokens".to_string(),
                amount: "999999999999999999999999".to_string(),
            },
            Self {
                url: "acc://üñíçødé.acme/tøkeñs".to_string(),
                amount: "42.123456789".to_string(),
            }
        ]
    }
}

impl SampleGenerator for crate::codec::transaction_codec::KeySpec {
    fn generate_sample() -> Self {
        Self {
            public_key_hash: "abcdef1234567890abcdef1234567890abcdef12".to_string(),
            delegate: None,
        }
    }

    fn generate_samples() -> Vec<Self> {
        vec![
            Self::generate_sample(),
            Self {
                public_key_hash: "0000000000000000000000000000000000000000".to_string(),
                delegate: None,
            },
            Self {
                public_key_hash: "ffffffffffffffffffffffffffffffffffffffff".to_string(),
                delegate: Some("acc://example.acme/delegate".to_string()),
            }
        ]
    }
}

// Implement RoundtripTestable for all the main types
impl RoundtripTestable for crate::codec::TransactionEnvelope {}
impl RoundtripTestable for crate::codec::TransactionHeader {}
impl RoundtripTestable for crate::codec::TransactionSignature {}
impl RoundtripTestable for crate::codec::TransactionKeyPage {}
impl RoundtripTestable for crate::codec::TokenRecipient {}
impl RoundtripTestable for crate::codec::KeySpec {}

// Also implement for types from the types.rs file
impl RoundtripTestable for crate::types::StatusResponse {}
impl RoundtripTestable for crate::types::NodeInfo {}
impl RoundtripTestable for crate::types::TransactionResponse {}
impl RoundtripTestable for crate::types::TransactionResult {}
impl RoundtripTestable for crate::types::Event {}
impl RoundtripTestable for crate::types::Attribute {}
impl RoundtripTestable for crate::types::SignedTransaction {}
impl RoundtripTestable for crate::types::Signature {}
impl RoundtripTestable for crate::types::Account {}
impl RoundtripTestable for crate::types::FaucetResponse {}
impl RoundtripTestable for crate::types::V3SubmitRequest {}
impl RoundtripTestable for crate::types::V3SubmitResponse {}
impl RoundtripTestable for crate::types::SubmitResult {}
impl RoundtripTestable for crate::types::TransactionEnvelope {}
impl RoundtripTestable for crate::types::V3Signature {}

/// Get type name for a given type (for debugging and reporting)
pub fn get_type_name<T>() -> &'static str {
    std::any::type_name::<T>()
}

/// Check if all types in TYPE_NAMES are covered by tests
pub fn verify_type_coverage() -> Result<(), Vec<String>> {
    let mut missing_types = Vec::new();

    // Check that each type in TYPE_NAMES has some form of test coverage
    // This is a basic implementation - could be enhanced with more sophisticated checks
    for type_name in TYPE_NAMES {
        match *type_name {
            // Core types that have SampleGenerator implementations
            "TransactionEnvelope" | "TransactionHeader" | "TransactionSignature"
            | "TransactionKeyPage" | "TokenRecipient" | "KeySpec" => {
                // These are covered by SampleGenerator
            }

            // Types that have manual test implementations
            "StatusResponse" | "NodeInfo" | "TransactionResponse" | "TransactionResult"
            | "Event" | "Attribute" | "Account" | "FaucetResponse" | "V3Signature" => {
                // These are covered by manual tests
            }

            // Types that might need implementation
            "QueryResponse" | "SignedTransaction" | "Signature" | "V3SubmitRequest"
            | "V3SubmitResponse" | "SubmitResult" | "ProtocolTransactionEnvelope"
            | "ProtocolTransactionSignature" | "ProtocolTransactionHeader"
            | "BinaryReader" | "BinaryWriter" | "EncodingError" | "DecodingError"
            | "FieldReader" | "Ed25519Signer" | "AccOptions" => {
                // These types might need test implementations
                missing_types.push(type_name.to_string());
            }

            _ => {
                // Unknown type
                missing_types.push(format!("Unknown type: {}", type_name));
            }
        }
    }

    if missing_types.is_empty() {
        Ok(())
    } else {
        Err(missing_types)
    }
}

/// Generate a comprehensive test report for all types
pub fn generate_type_test_report() -> String {
    let mut report = String::new();

    report.push_str("# Type Matrix Test Coverage Report\n\n");
    report.push_str(&format!("Total types in matrix: {}\n\n", TYPE_NAMES.len()));

    report.push_str("## Core Transaction Types\n");
    let core_types = [
        "TransactionEnvelope", "TransactionHeader", "TransactionSignature",
        "TransactionKeyPage", "TokenRecipient", "KeySpec"
    ];

    for type_name in core_types {
        if TYPE_NAMES.contains(&type_name) {
            report.push_str(&format!("- [OK] {}\n", type_name));
        } else {
            report.push_str(&format!("- [MISSING] {} (missing from TYPE_NAMES)\n", type_name));
        }
    }

    report.push_str("\n## API Response Types\n");
    let api_types = [
        "StatusResponse", "NodeInfo", "QueryResponse", "TransactionResponse",
        "TransactionResult", "Event", "Attribute", "Account", "FaucetResponse"
    ];

    for type_name in api_types {
        if TYPE_NAMES.contains(&type_name) {
            report.push_str(&format!("- [OK] {}\n", type_name));
        } else {
            report.push_str(&format!("- [MISSING] {} (missing from TYPE_NAMES)\n", type_name));
        }
    }

    report.push_str("\n## V3 Protocol Types\n");
    let v3_types = [
        "V3SubmitRequest", "V3SubmitResponse", "SubmitResult", "V3Signature"
    ];

    for type_name in v3_types {
        if TYPE_NAMES.contains(&type_name) {
            report.push_str(&format!("- [OK] {}\n", type_name));
        } else {
            report.push_str(&format!("- [MISSING] {} (missing from TYPE_NAMES)\n", type_name));
        }
    }

    report.push_str("\n## Coverage Status\n");
    match verify_type_coverage() {
        Ok(()) => {
            report.push_str("[OK] All types have test coverage\n");
        }
        Err(missing) => {
            report.push_str(&format!("[MISSING] {} types need test implementations:\n", missing.len()));
            for missing_type in missing {
                report.push_str(&format!("  - {}\n", missing_type));
            }
        }
    }

    report
}

/// Utility to count the number of samples generated for a type
pub fn count_samples<T: SampleGenerator>() -> usize {
    T::generate_samples().len()
}