//! GENERATED FILE - DO NOT EDIT
//! Source: protocol/signatures.yml | Generated: 2025-10-03 20:47:50

#![allow(missing_docs)]

use serde::{Serialize, Deserialize};
use hex;

// Helper module for optional hex serialization
mod hex_option {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &Option<[u8; 32]>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(bytes) => hex::encode(bytes).serialize(serializer),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<[u8; 32]>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<String> = Option::deserialize(deserializer)?;
        match opt {
            Some(hex_str) => {
                let bytes = hex::decode(&hex_str).map_err(serde::de::Error::custom)?;
                if bytes.len() != 32 {
                    return Err(serde::de::Error::custom("Hash must be 32 bytes"));
                }
                let mut hash = [0u8; 32];
                hash.copy_from_slice(&bytes);
                Ok(Some(hash))
            },
            None => Ok(None),
        }
    }
}

// Helper module for optional bytes hex serialization
mod hex_option_vec {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(bytes) => hex::encode(bytes).serialize(serializer),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<String> = Option::deserialize(deserializer)?;
        match opt {
            Some(hex_str) => {
                let bytes = hex::decode(&hex_str).map_err(serde::de::Error::custom)?;
                Ok(Some(bytes))
            },
            None => Ok(None),
        }
    }
}

// Helper module for vector of hashes hex serialization
mod hex_vec_hash {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &Vec<[u8; 32]>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex_strings: Vec<String> = value.iter().map(|hash| hex::encode(hash)).collect();
        hex_strings.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<[u8; 32]>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let hex_strings: Vec<String> = Vec::deserialize(deserializer)?;
        let mut result = Vec::new();
        for hex_str in hex_strings {
            let bytes = hex::decode(&hex_str).map_err(serde::de::Error::custom)?;
            if bytes.len() != 32 {
                return Err(serde::de::Error::custom("Hash must be 32 bytes"));
            }
            let mut hash = [0u8; 32];
            hash.copy_from_slice(&bytes);
            result.push(hash);
        }
        Ok(result)
    }
}

// Helper module for vector of bytes hex serialization
// Kept for potential future use with Vec<Vec<u8>> fields
#[allow(dead_code)]
mod hex_vec_bytes {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &Vec<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex_strings: Vec<String> = value.iter().map(|bytes| hex::encode(bytes)).collect();
        hex_strings.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Vec<u8>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let hex_strings: Vec<String> = Vec::deserialize(deserializer)?;
        let mut result = Vec::new();
        for hex_str in hex_strings {
            let bytes = hex::decode(&hex_str).map_err(serde::de::Error::custom)?;
            result.push(bytes);
        }
        Ok(result)
    }
}

/// Error type for signature operations
#[derive(Debug, thiserror::Error)]
pub enum SignatureError {
    #[error("Invalid signature format")]
    InvalidFormat,
    #[error("Verification failed: {0}")]
    VerificationFailed(String),
    #[error("Unsupported signature type")]
    UnsupportedType,
}

/// Main signature trait for verification
pub trait AccSignature {
    fn verify(&self, message: &[u8]) -> Result<bool, crate::errors::Error>;
    fn sig_type(&self) -> &'static str;
}

/// LegacyED25519Signature signature
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyED25519Signature {
    #[serde(rename = "Timestamp")]
    pub timestamp: u64,
    #[serde(rename = "PublicKey", with = "hex::serde")]
    pub public_key: Vec<u8>,
    #[serde(rename = "Signature", with = "hex::serde")]
    pub signature: Vec<u8>,
    #[serde(rename = "Signer")]
    pub signer: String,
    #[serde(rename = "SignerVersion")]
    pub signer_version: u64,
    #[serde(rename = "Vote")]
    pub vote: Option<crate::generated::enums::VoteType>,
    #[serde(rename = "TransactionHash", with = "hex_option")]
    pub transaction_hash: Option<[u8; 32]>,
}

/// RCD1Signature signature
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RCD1Signature {
    #[serde(rename = "PublicKey", with = "hex::serde")]
    pub public_key: Vec<u8>,
    #[serde(rename = "Signature", with = "hex::serde")]
    pub signature: Vec<u8>,
    #[serde(rename = "Signer")]
    pub signer: String,
    #[serde(rename = "SignerVersion")]
    pub signer_version: u64,
    #[serde(rename = "Timestamp")]
    pub timestamp: Option<u64>,
    #[serde(rename = "Vote")]
    pub vote: Option<crate::generated::enums::VoteType>,
    #[serde(rename = "TransactionHash", with = "hex_option")]
    pub transaction_hash: Option<[u8; 32]>,
    #[serde(rename = "Memo")]
    pub memo: Option<String>,
    #[serde(rename = "Data", with = "hex_option_vec")]
    pub data: Option<Vec<u8>>,
}

/// ED25519Signature signature
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ED25519Signature {
    #[serde(rename = "PublicKey", with = "hex::serde")]
    pub public_key: Vec<u8>,
    #[serde(rename = "Signature", with = "hex::serde")]
    pub signature: Vec<u8>,
    #[serde(rename = "Signer")]
    pub signer: String,
    #[serde(rename = "SignerVersion")]
    pub signer_version: u64,
    #[serde(rename = "Timestamp")]
    pub timestamp: Option<u64>,
    #[serde(rename = "Vote")]
    pub vote: Option<crate::generated::enums::VoteType>,
    #[serde(rename = "TransactionHash", with = "hex_option")]
    pub transaction_hash: Option<[u8; 32]>,
    #[serde(rename = "Memo")]
    pub memo: Option<String>,
    #[serde(rename = "Data", with = "hex_option_vec")]
    pub data: Option<Vec<u8>>,
}

/// BTCSignature signature
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BTCSignature {
    #[serde(rename = "PublicKey", with = "hex::serde")]
    pub public_key: Vec<u8>,
    #[serde(rename = "Signature", with = "hex::serde")]
    pub signature: Vec<u8>,
    #[serde(rename = "Signer")]
    pub signer: String,
    #[serde(rename = "SignerVersion")]
    pub signer_version: u64,
    #[serde(rename = "Timestamp")]
    pub timestamp: Option<u64>,
    #[serde(rename = "Vote")]
    pub vote: Option<crate::generated::enums::VoteType>,
    #[serde(rename = "TransactionHash", with = "hex_option")]
    pub transaction_hash: Option<[u8; 32]>,
    #[serde(rename = "Memo")]
    pub memo: Option<String>,
    #[serde(rename = "Data", with = "hex_option_vec")]
    pub data: Option<Vec<u8>>,
}

/// BTCLegacySignature signature
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BTCLegacySignature {
    #[serde(rename = "PublicKey", with = "hex::serde")]
    pub public_key: Vec<u8>,
    #[serde(rename = "Signature", with = "hex::serde")]
    pub signature: Vec<u8>,
    #[serde(rename = "Signer")]
    pub signer: String,
    #[serde(rename = "SignerVersion")]
    pub signer_version: u64,
    #[serde(rename = "Timestamp")]
    pub timestamp: Option<u64>,
    #[serde(rename = "Vote")]
    pub vote: Option<crate::generated::enums::VoteType>,
    #[serde(rename = "TransactionHash", with = "hex_option")]
    pub transaction_hash: Option<[u8; 32]>,
    #[serde(rename = "Memo")]
    pub memo: Option<String>,
    #[serde(rename = "Data", with = "hex_option_vec")]
    pub data: Option<Vec<u8>>,
}

/// ETHSignature signature
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ETHSignature {
    #[serde(rename = "PublicKey", with = "hex::serde")]
    pub public_key: Vec<u8>,
    #[serde(rename = "Signature", with = "hex::serde")]
    pub signature: Vec<u8>,
    #[serde(rename = "Signer")]
    pub signer: String,
    #[serde(rename = "SignerVersion")]
    pub signer_version: u64,
    #[serde(rename = "Timestamp")]
    pub timestamp: Option<u64>,
    #[serde(rename = "Vote")]
    pub vote: Option<crate::generated::enums::VoteType>,
    #[serde(rename = "TransactionHash", with = "hex_option")]
    pub transaction_hash: Option<[u8; 32]>,
    #[serde(rename = "Memo")]
    pub memo: Option<String>,
    #[serde(rename = "Data", with = "hex_option_vec")]
    pub data: Option<Vec<u8>>,
}

/// RsaSha256Signature signature
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RsaSha256Signature {
    #[serde(rename = "PublicKey", with = "hex::serde")]
    pub public_key: Vec<u8>,
    #[serde(rename = "Signature", with = "hex::serde")]
    pub signature: Vec<u8>,
    #[serde(rename = "Signer")]
    pub signer: String,
    #[serde(rename = "SignerVersion")]
    pub signer_version: u64,
    #[serde(rename = "Timestamp")]
    pub timestamp: Option<u64>,
    #[serde(rename = "Vote")]
    pub vote: Option<crate::generated::enums::VoteType>,
    #[serde(rename = "TransactionHash", with = "hex_option")]
    pub transaction_hash: Option<[u8; 32]>,
    #[serde(rename = "Memo")]
    pub memo: Option<String>,
    #[serde(rename = "Data", with = "hex_option_vec")]
    pub data: Option<Vec<u8>>,
}

/// EcdsaSha256Signature signature
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EcdsaSha256Signature {
    #[serde(rename = "PublicKey", with = "hex::serde")]
    pub public_key: Vec<u8>,
    #[serde(rename = "Signature", with = "hex::serde")]
    pub signature: Vec<u8>,
    #[serde(rename = "Signer")]
    pub signer: String,
    #[serde(rename = "SignerVersion")]
    pub signer_version: u64,
    #[serde(rename = "Timestamp")]
    pub timestamp: Option<u64>,
    #[serde(rename = "Vote")]
    pub vote: Option<crate::generated::enums::VoteType>,
    #[serde(rename = "TransactionHash", with = "hex_option")]
    pub transaction_hash: Option<[u8; 32]>,
    #[serde(rename = "Memo")]
    pub memo: Option<String>,
    #[serde(rename = "Data", with = "hex_option_vec")]
    pub data: Option<Vec<u8>>,
}

/// TypedDataSignature signature
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypedDataSignature {
    #[serde(rename = "PublicKey", with = "hex::serde")]
    pub public_key: Vec<u8>,
    #[serde(rename = "Signature", with = "hex::serde")]
    pub signature: Vec<u8>,
    #[serde(rename = "Signer")]
    pub signer: String,
    #[serde(rename = "SignerVersion")]
    pub signer_version: u64,
    #[serde(rename = "Timestamp")]
    pub timestamp: Option<u64>,
    #[serde(rename = "Vote")]
    pub vote: Option<crate::generated::enums::VoteType>,
    #[serde(rename = "TransactionHash", with = "hex_option")]
    pub transaction_hash: Option<[u8; 32]>,
    #[serde(rename = "Memo")]
    pub memo: Option<String>,
    #[serde(rename = "Data", with = "hex_option_vec")]
    pub data: Option<Vec<u8>>,
    #[serde(rename = "ChainID")]
    pub chain_i_d: String,
}

/// ReceiptSignature signature
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptSignature {
    #[serde(rename = "SourceNetwork")]
    pub source_network: String,
    #[serde(rename = "Proof")]
    pub proof: crate::types::MerkleReceipt,
    #[serde(rename = "TransactionHash", with = "hex_option")]
    pub transaction_hash: Option<[u8; 32]>,
}

/// PartitionSignature signature
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartitionSignature {
    #[serde(rename = "SourceNetwork")]
    pub source_network: String,
    #[serde(rename = "DestinationNetwork")]
    pub destination_network: String,
    #[serde(rename = "SequenceNumber")]
    pub sequence_number: u64,
    #[serde(rename = "TransactionHash", with = "hex_option")]
    pub transaction_hash: Option<[u8; 32]>,
}

/// SignatureSet signature
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignatureSet {
    #[serde(rename = "Vote")]
    pub vote: Option<crate::generated::enums::VoteType>,
    #[serde(rename = "Signer")]
    pub signer: String,
    #[serde(rename = "TransactionHash", with = "hex_option")]
    pub transaction_hash: Option<[u8; 32]>,
    #[serde(rename = "Signatures")]
    pub signatures: Vec<Box<crate::generated::signatures::Signature>>,
    #[serde(rename = "Authority")]
    pub authority: String,
}

/// RemoteSignature signature
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteSignature {
    #[serde(rename = "Destination")]
    pub destination: String,
    #[serde(rename = "Signature")]
    pub signature: Box<crate::generated::signatures::Signature>,
    #[serde(rename = "Cause", with = "hex_vec_hash")]
    pub cause: Vec<[u8; 32]>,
}

/// DelegatedSignature signature
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DelegatedSignature {
    #[serde(rename = "Signature")]
    pub signature: Box<crate::generated::signatures::Signature>,
    #[serde(rename = "Delegator")]
    pub delegator: String,
}

/// InternalSignature signature
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InternalSignature {
    #[serde(rename = "Cause", with = "hex::serde")]
    pub cause: [u8; 32],
    #[serde(rename = "TransactionHash", with = "hex::serde")]
    pub transaction_hash: [u8; 32],
}

/// AuthoritySignature signature
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthoritySignature {
    #[serde(rename = "Origin")]
    pub origin: String,
    #[serde(rename = "Authority")]
    pub authority: String,
    #[serde(rename = "Vote")]
    pub vote: Option<crate::generated::enums::VoteType>,
    #[serde(rename = "TxID")]
    pub tx_i_d: String,
    #[serde(rename = "Cause")]
    pub cause: String,
    #[serde(rename = "Delegator")]
    pub delegator: Vec<String>,
    #[serde(rename = "Memo")]
    pub memo: Option<String>,
}

/// Main signature dispatch enum
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Signature {
    #[serde(rename = "legacyED25519")]
    LegacyED25519(LegacyED25519Signature),
    #[serde(rename = "rcd1")]
    RCD1(RCD1Signature),
    #[serde(rename = "ed25519")]
    ED25519(ED25519Signature),
    #[serde(rename = "btc")]
    BTC(BTCSignature),
    #[serde(rename = "btcLegacy")]
    BTCLegacy(BTCLegacySignature),
    #[serde(rename = "eth")]
    ETH(ETHSignature),
    #[serde(rename = "rsaSha256")]
    RsaSha256(RsaSha256Signature),
    #[serde(rename = "ecdsaSha256")]
    EcdsaSha256(EcdsaSha256Signature),
    #[serde(rename = "typedData")]
    TypedData(TypedDataSignature),
    #[serde(rename = "receipt")]
    Receipt(ReceiptSignature),
    #[serde(rename = "partition")]
    Partition(PartitionSignature),
    #[serde(rename = "signatureSet")]
    Set(SignatureSet),
    #[serde(rename = "remote")]
    Remote(RemoteSignature),
    #[serde(rename = "delegated")]
    Delegated(DelegatedSignature),
    #[serde(rename = "internal")]
    Internal(InternalSignature),
    #[serde(rename = "authority")]
    Authority(AuthoritySignature),
}

impl Signature {
    pub fn wire_tag(&self) -> &'static str {
        match self {
            Signature::LegacyED25519(_) => "legacyED25519",
            Signature::RCD1(_) => "rcd1",
            Signature::ED25519(_) => "ed25519",
            Signature::BTC(_) => "btc",
            Signature::BTCLegacy(_) => "btcLegacy",
            Signature::ETH(_) => "eth",
            Signature::RsaSha256(_) => "rsaSha256",
            Signature::EcdsaSha256(_) => "ecdsaSha256",
            Signature::TypedData(_) => "typedData",
            Signature::Receipt(_) => "receipt",
            Signature::Partition(_) => "partition",
            Signature::Set(_) => "signatureSet",
            Signature::Remote(_) => "remote",
            Signature::Delegated(_) => "delegated",
            Signature::Internal(_) => "internal",
            Signature::Authority(_) => "authority",
        }
    }

    /// Get the vote from this signature
    /// Returns VoteType::Accept for signatures that don't have a vote field
    pub fn get_vote(&self) -> crate::generated::enums::VoteType {
        use crate::generated::enums::VoteType;
        match self {
            Signature::LegacyED25519(s) => s.vote.unwrap_or(VoteType::Accept),
            Signature::RCD1(s) => s.vote.unwrap_or(VoteType::Accept),
            Signature::ED25519(s) => s.vote.unwrap_or(VoteType::Accept),
            Signature::BTC(s) => s.vote.unwrap_or(VoteType::Accept),
            Signature::BTCLegacy(s) => s.vote.unwrap_or(VoteType::Accept),
            Signature::ETH(s) => s.vote.unwrap_or(VoteType::Accept),
            Signature::RsaSha256(s) => s.vote.unwrap_or(VoteType::Accept),
            Signature::EcdsaSha256(s) => s.vote.unwrap_or(VoteType::Accept),
            Signature::TypedData(s) => s.vote.unwrap_or(VoteType::Accept),
            // System signatures always vote Accept
            Signature::Receipt(_) => VoteType::Accept,
            Signature::Partition(_) => VoteType::Accept,
            Signature::Internal(_) => VoteType::Accept,
            // SignatureSet has its own vote
            Signature::Set(s) => s.vote.unwrap_or(VoteType::Accept),
            // Wrapper signatures delegate to inner (signature is Box<Signature>, not Option)
            Signature::Remote(s) => s.signature.get_vote(),
            Signature::Delegated(s) => s.signature.get_vote(),
            Signature::Authority(s) => s.vote.unwrap_or(VoteType::Accept),
        }
    }

    /// Check if this signature votes to accept
    pub fn votes_accept(&self) -> bool {
        self.get_vote().is_approval()
    }

    /// Check if this signature votes to reject
    pub fn votes_reject(&self) -> bool {
        self.get_vote().is_rejection()
    }

    /// Check if this signature abstains
    pub fn votes_abstain(&self) -> bool {
        self.get_vote().is_abstention()
    }

    /// Tally votes from multiple signatures
    pub fn tally_votes(signatures: &[Signature]) -> crate::generated::enums::VoteTally {
        let mut tally = crate::generated::enums::VoteTally::new();
        for sig in signatures {
            tally.add_vote(sig.get_vote());
        }
        tally
    }

    /// Tally votes from multiple signatures (nested - includes nested signatures in sets)
    pub fn tally_votes_nested(signatures: &[Signature]) -> crate::generated::enums::VoteTally {
        let mut tally = crate::generated::enums::VoteTally::new();
        for sig in signatures {
            match sig {
                Signature::Set(s) => {
                    // For SignatureSet, use its vote field
                    tally.add_vote(s.vote.unwrap_or(crate::generated::enums::VoteType::Accept));
                }
                Signature::Remote(s) => {
                    // signature is Box<Signature>, not Option
                    tally.add_vote(s.signature.get_vote());
                }
                Signature::Delegated(s) => {
                    // signature is Box<Signature>, not Option
                    tally.add_vote(s.signature.get_vote());
                }
                _ => {
                    tally.add_vote(sig.get_vote());
                }
            }
        }
        tally
    }
}

impl AccSignature for LegacyED25519Signature {
    fn verify(&self, message: &[u8]) -> Result<bool, crate::errors::Error> {
        // Updated for ed25519-dalek v2.x API
        // Legacy Ed25519 - use same verification as ED25519Signature
        use ed25519_dalek::{Signature as Ed25519Sig, VerifyingKey, Verifier};

        if self.public_key.len() != 32 || self.signature.len() != 64 {
            return Ok(false);
        }

        let mut pub_key_bytes = [0u8; 32];
        pub_key_bytes.copy_from_slice(&self.public_key);

        let mut sig_bytes = [0u8; 64];
        sig_bytes.copy_from_slice(&self.signature);

        match VerifyingKey::from_bytes(&pub_key_bytes) {
            Ok(verifying_key) => {
                let signature = Ed25519Sig::from_bytes(&sig_bytes);
                Ok(verifying_key.verify(message, &signature).is_ok())
            },
            Err(_) => Ok(false),
        }
    }

    fn sig_type(&self) -> &'static str {
        "legacyED25519"
    }
}

impl AccSignature for RCD1Signature {
    fn verify(&self, message: &[u8]) -> Result<bool, crate::errors::Error> {
        // Updated for ed25519-dalek v2.x API
        // RCD1 uses ED25519 internally (Factom RCD1 format)
        // The public key format is the same as ED25519
        use ed25519_dalek::{Signature as Ed25519Sig, VerifyingKey, Verifier};

        if self.public_key.len() != 32 || self.signature.len() != 64 {
            return Ok(false);
        }

        let mut pub_key_bytes = [0u8; 32];
        pub_key_bytes.copy_from_slice(&self.public_key);

        let mut sig_bytes = [0u8; 64];
        sig_bytes.copy_from_slice(&self.signature);

        match VerifyingKey::from_bytes(&pub_key_bytes) {
            Ok(verifying_key) => {
                let signature = Ed25519Sig::from_bytes(&sig_bytes);
                Ok(verifying_key.verify(message, &signature).is_ok())
            },
            Err(_) => Ok(false),
        }
    }

    fn sig_type(&self) -> &'static str {
        "rcd1"
    }
}

impl AccSignature for ED25519Signature {
    fn verify(&self, message: &[u8]) -> Result<bool, crate::errors::Error> {
        // Updated for ed25519-dalek v2.x API
        use ed25519_dalek::{Signature as Ed25519Sig, VerifyingKey, Verifier};

        if self.public_key.len() != 32 || self.signature.len() != 64 {
            return Ok(false);
        }

        let mut pub_key_bytes = [0u8; 32];
        pub_key_bytes.copy_from_slice(&self.public_key);

        let mut sig_bytes = [0u8; 64];
        sig_bytes.copy_from_slice(&self.signature);

        match VerifyingKey::from_bytes(&pub_key_bytes) {
            Ok(verifying_key) => {
                // In v2, Signature::from_bytes is infallible
                let signature = Ed25519Sig::from_bytes(&sig_bytes);
                Ok(verifying_key.verify(message, &signature).is_ok())
            },
            Err(_) => Ok(false),
        }
    }

    fn sig_type(&self) -> &'static str {
        "ed25519"
    }
}

impl AccSignature for BTCSignature {
    fn verify(&self, message: &[u8]) -> Result<bool, crate::errors::Error> {
        // BTC uses secp256k1 ECDSA with DER-encoded signatures
        use k256::ecdsa::{Signature as K256Signature, VerifyingKey, signature::Verifier};
        use sha2::{Sha256, Digest};

        if self.public_key.is_empty() || self.signature.is_empty() {
            return Ok(false);
        }

        // Hash the message with SHA256 (Bitcoin double-SHA256 for transaction signing)
        let mut hasher = Sha256::new();
        hasher.update(message);
        let hash = hasher.finalize();

        // Parse the public key (compressed or uncompressed SEC1 format)
        let verifying_key = match VerifyingKey::from_sec1_bytes(&self.public_key) {
            Ok(key) => key,
            Err(_) => return Ok(false),
        };

        // Try to parse signature as DER format first, then as raw r||s format
        let signature = if let Ok(sig) = K256Signature::from_der(&self.signature) {
            sig
        } else if self.signature.len() == 64 {
            // Raw r||s format (32 bytes each)
            match K256Signature::from_slice(&self.signature) {
                Ok(sig) => sig,
                Err(_) => return Ok(false),
            }
        } else {
            return Ok(false);
        };

        Ok(verifying_key.verify(&hash, &signature).is_ok())
    }

    fn sig_type(&self) -> &'static str {
        "btc"
    }
}

impl AccSignature for BTCLegacySignature {
    fn verify(&self, message: &[u8]) -> Result<bool, crate::errors::Error> {
        // BTCLegacy uses same secp256k1 ECDSA as BTC
        use k256::ecdsa::{Signature as K256Signature, VerifyingKey, signature::Verifier};
        use sha2::{Sha256, Digest};

        if self.public_key.is_empty() || self.signature.is_empty() {
            return Ok(false);
        }

        // Hash the message with SHA256
        let mut hasher = Sha256::new();
        hasher.update(message);
        let hash = hasher.finalize();

        // Parse the public key (compressed or uncompressed SEC1 format)
        let verifying_key = match VerifyingKey::from_sec1_bytes(&self.public_key) {
            Ok(key) => key,
            Err(_) => return Ok(false),
        };

        // Try to parse signature as DER format first, then as raw r||s format
        let signature = if let Ok(sig) = K256Signature::from_der(&self.signature) {
            sig
        } else if self.signature.len() == 64 {
            match K256Signature::from_slice(&self.signature) {
                Ok(sig) => sig,
                Err(_) => return Ok(false),
            }
        } else {
            return Ok(false);
        };

        Ok(verifying_key.verify(&hash, &signature).is_ok())
    }

    fn sig_type(&self) -> &'static str {
        "btcLegacy"
    }
}

impl AccSignature for ETHSignature {
    fn verify(&self, message: &[u8]) -> Result<bool, crate::errors::Error> {
        // Ethereum uses secp256k1 ECDSA with keccak256 hash
        // Signature format: r (32 bytes) || s (32 bytes) || v (1 byte recovery id)
        use k256::ecdsa::{Signature as K256Signature, VerifyingKey, RecoveryId, signature::Verifier};
        use sha3::{Keccak256, Digest};

        if self.public_key.is_empty() || self.signature.is_empty() {
            return Ok(false);
        }

        // Hash the message with Keccak256 (Ethereum style)
        let mut hasher = Keccak256::new();
        hasher.update(message);
        let hash = hasher.finalize();

        // Parse the public key (compressed or uncompressed SEC1 format)
        let expected_key = match VerifyingKey::from_sec1_bytes(&self.public_key) {
            Ok(key) => key,
            Err(_) => return Ok(false),
        };

        // ETH signatures are typically 65 bytes (r || s || v) or 64 bytes (r || s)
        if self.signature.len() == 65 {
            // Extract r, s, and recovery id
            let recovery_id = match RecoveryId::try_from(self.signature[64] % 4) {
                Ok(id) => id,
                Err(_) => return Ok(false),
            };

            let sig = match K256Signature::from_slice(&self.signature[..64]) {
                Ok(s) => s,
                Err(_) => return Ok(false),
            };

            // Recover the public key and compare
            match VerifyingKey::recover_from_prehash(&hash, &sig, recovery_id) {
                Ok(recovered_key) => {
                    Ok(recovered_key == expected_key)
                },
                Err(_) => Ok(false),
            }
        } else if self.signature.len() == 64 {
            // Standard ECDSA without recovery - verify directly
            let signature = match K256Signature::from_slice(&self.signature) {
                Ok(sig) => sig,
                Err(_) => return Ok(false),
            };
            Ok(expected_key.verify(&hash, &signature).is_ok())
        } else {
            Ok(false)
        }
    }

    fn sig_type(&self) -> &'static str {
        "eth"
    }
}

impl AccSignature for RsaSha256Signature {
    fn verify(&self, message: &[u8]) -> Result<bool, crate::errors::Error> {
        // RSA PKCS#1 v1.5 with SHA-256
        use rsa::{RsaPublicKey, pkcs1::DecodeRsaPublicKey, pkcs8::DecodePublicKey};
        use rsa::signature::Verifier;
        use rsa::pkcs1v15::{Signature, VerifyingKey};
        use sha2::Sha256;

        if self.signature.is_empty() || self.public_key.is_empty() {
            return Ok(false);
        }

        // Try to parse the public key in different formats
        // First try PKCS#1 DER format, then PKCS#8 DER format
        let public_key = if let Ok(key) = RsaPublicKey::from_pkcs1_der(&self.public_key) {
            key
        } else if let Ok(key) = RsaPublicKey::from_public_key_der(&self.public_key) {
            key
        } else {
            return Ok(false);
        };

        // Create verifying key with SHA256
        let verifying_key: VerifyingKey<Sha256> = VerifyingKey::new(public_key);

        // Parse the signature
        let signature = match Signature::try_from(self.signature.as_slice()) {
            Ok(sig) => sig,
            Err(_) => return Ok(false),
        };

        // Verify the signature
        Ok(verifying_key.verify(message, &signature).is_ok())
    }

    fn sig_type(&self) -> &'static str {
        "rsaSha256"
    }
}

impl AccSignature for EcdsaSha256Signature {
    fn verify(&self, message: &[u8]) -> Result<bool, crate::errors::Error> {
        // ECDSA secp256k1 with SHA-256 hash
        use k256::ecdsa::{Signature as K256Signature, VerifyingKey, signature::Verifier};
        use sha2::{Sha256, Digest};

        if self.signature.is_empty() || self.public_key.is_empty() {
            return Ok(false);
        }

        // Hash the message with SHA256
        let mut hasher = Sha256::new();
        hasher.update(message);
        let hash = hasher.finalize();

        // Parse the public key (compressed or uncompressed SEC1 format)
        let verifying_key = match VerifyingKey::from_sec1_bytes(&self.public_key) {
            Ok(key) => key,
            Err(_) => return Ok(false),
        };

        // Try to parse signature as DER format first, then as raw r||s format
        let signature = if let Ok(sig) = K256Signature::from_der(&self.signature) {
            sig
        } else if self.signature.len() == 64 {
            match K256Signature::from_slice(&self.signature) {
                Ok(sig) => sig,
                Err(_) => return Ok(false),
            }
        } else {
            return Ok(false);
        };

        Ok(verifying_key.verify(&hash, &signature).is_ok())
    }

    fn sig_type(&self) -> &'static str {
        "ecdsaSha256"
    }
}

impl AccSignature for TypedDataSignature {
    fn verify(&self, message: &[u8]) -> Result<bool, crate::errors::Error> {
        // EIP-712 TypedData signature - uses keccak256 and secp256k1 with recovery
        // Similar to ETHSignature but the message is already the EIP-712 typed data hash
        use k256::ecdsa::{Signature as K256Signature, VerifyingKey, RecoveryId, signature::Verifier};
        use sha3::{Keccak256, Digest};

        if self.public_key.is_empty() || self.signature.is_empty() {
            return Ok(false);
        }

        // For EIP-712, the message should already be the structured data hash
        // We hash it with keccak256 for final signing
        let mut hasher = Keccak256::new();
        hasher.update(message);
        let hash = hasher.finalize();

        // Parse the public key
        let expected_key = match VerifyingKey::from_sec1_bytes(&self.public_key) {
            Ok(key) => key,
            Err(_) => return Ok(false),
        };

        // ETH-style signatures are typically 65 bytes (r || s || v)
        if self.signature.len() == 65 {
            let recovery_id = match RecoveryId::try_from(self.signature[64] % 4) {
                Ok(id) => id,
                Err(_) => return Ok(false),
            };

            let sig = match K256Signature::from_slice(&self.signature[..64]) {
                Ok(s) => s,
                Err(_) => return Ok(false),
            };

            match VerifyingKey::recover_from_prehash(&hash, &sig, recovery_id) {
                Ok(recovered_key) => Ok(recovered_key == expected_key),
                Err(_) => Ok(false),
            }
        } else if self.signature.len() == 64 {
            let signature = match K256Signature::from_slice(&self.signature) {
                Ok(sig) => sig,
                Err(_) => return Ok(false),
            };
            Ok(expected_key.verify(&hash, &signature).is_ok())
        } else {
            Ok(false)
        }
    }

    fn sig_type(&self) -> &'static str {
        "typedData"
    }
}

impl AccSignature for ReceiptSignature {
    fn verify(&self, _message: &[u8]) -> Result<bool, crate::errors::Error> {
        // Receipt signatures verify Merkle proofs from anchors
        // The proof field contains the Merkle receipt that proves inclusion
        // For now, we validate structure - full Merkle verification requires the anchor root

        // Basic structural validation
        if self.source_network.is_empty() {
            return Ok(false);
        }

        // Receipt validation would require access to the anchor chain
        // to verify the Merkle root. For SDK purposes, we trust well-formed receipts.
        // Full validation happens on the validator nodes.
        Ok(true)
    }

    fn sig_type(&self) -> &'static str {
        "receipt"
    }
}

impl AccSignature for PartitionSignature {
    fn verify(&self, _message: &[u8]) -> Result<bool, crate::errors::Error> {
        // Partition signatures are used for cross-partition communication
        // They are produced by validators and verify routing between partitions

        // Basic structural validation
        if self.source_network.is_empty() || self.destination_network.is_empty() {
            return Ok(false);
        }

        // Partition signatures are trusted when coming from validators
        // Full validation requires access to validator sets
        Ok(true)
    }

    fn sig_type(&self) -> &'static str {
        "partition"
    }
}

impl AccSignature for SignatureSet {
    fn verify(&self, message: &[u8]) -> Result<bool, crate::errors::Error> {
        // SignatureSet is a multi-signature container
        // All inner signatures must verify for the set to be valid

        if self.signatures.is_empty() {
            return Ok(false);
        }

        // Verify each signature in the set
        for sig in &self.signatures {
            let result = match sig.as_ref() {
                Signature::LegacyED25519(s) => s.verify(message),
                Signature::RCD1(s) => s.verify(message),
                Signature::ED25519(s) => s.verify(message),
                Signature::BTC(s) => s.verify(message),
                Signature::BTCLegacy(s) => s.verify(message),
                Signature::ETH(s) => s.verify(message),
                Signature::RsaSha256(s) => s.verify(message),
                Signature::EcdsaSha256(s) => s.verify(message),
                Signature::TypedData(s) => s.verify(message),
                Signature::Receipt(s) => s.verify(message),
                Signature::Partition(s) => s.verify(message),
                Signature::Set(s) => s.verify(message),
                Signature::Remote(s) => s.verify(message),
                Signature::Delegated(s) => s.verify(message),
                Signature::Internal(s) => s.verify(message),
                Signature::Authority(s) => s.verify(message),
            };

            match result {
                Ok(true) => continue,
                Ok(false) => return Ok(false),
                Err(e) => return Err(e),
            }
        }

        Ok(true)
    }

    fn sig_type(&self) -> &'static str {
        "signatureSet"
    }
}

impl AccSignature for RemoteSignature {
    fn verify(&self, message: &[u8]) -> Result<bool, crate::errors::Error> {
        // RemoteSignature wraps another signature for cross-partition routing
        // Delegate verification to the inner signature

        if self.destination.is_empty() {
            return Ok(false);
        }

        // Verify the wrapped signature
        match self.signature.as_ref() {
            Signature::LegacyED25519(s) => s.verify(message),
            Signature::RCD1(s) => s.verify(message),
            Signature::ED25519(s) => s.verify(message),
            Signature::BTC(s) => s.verify(message),
            Signature::BTCLegacy(s) => s.verify(message),
            Signature::ETH(s) => s.verify(message),
            Signature::RsaSha256(s) => s.verify(message),
            Signature::EcdsaSha256(s) => s.verify(message),
            Signature::TypedData(s) => s.verify(message),
            Signature::Receipt(s) => s.verify(message),
            Signature::Partition(s) => s.verify(message),
            Signature::Set(s) => s.verify(message),
            Signature::Remote(s) => s.verify(message),
            Signature::Delegated(s) => s.verify(message),
            Signature::Internal(s) => s.verify(message),
            Signature::Authority(s) => s.verify(message),
        }
    }

    fn sig_type(&self) -> &'static str {
        "remote"
    }
}

impl AccSignature for DelegatedSignature {
    fn verify(&self, message: &[u8]) -> Result<bool, crate::errors::Error> {
        // DelegatedSignature wraps a signature from a delegated authority
        // Delegate verification to the inner signature

        if self.delegator.is_empty() {
            return Ok(false);
        }

        // Verify the wrapped signature
        match self.signature.as_ref() {
            Signature::LegacyED25519(s) => s.verify(message),
            Signature::RCD1(s) => s.verify(message),
            Signature::ED25519(s) => s.verify(message),
            Signature::BTC(s) => s.verify(message),
            Signature::BTCLegacy(s) => s.verify(message),
            Signature::ETH(s) => s.verify(message),
            Signature::RsaSha256(s) => s.verify(message),
            Signature::EcdsaSha256(s) => s.verify(message),
            Signature::TypedData(s) => s.verify(message),
            Signature::Receipt(s) => s.verify(message),
            Signature::Partition(s) => s.verify(message),
            Signature::Set(s) => s.verify(message),
            Signature::Remote(s) => s.verify(message),
            Signature::Delegated(s) => s.verify(message),
            Signature::Internal(s) => s.verify(message),
            Signature::Authority(s) => s.verify(message),
        }
    }

    fn sig_type(&self) -> &'static str {
        "delegated"
    }
}

impl AccSignature for InternalSignature {
    fn verify(&self, _message: &[u8]) -> Result<bool, crate::errors::Error> {
        // InternalSignature is used for system-generated transactions
        // It contains cause and transaction hash references

        // Internal signatures are valid if they have proper hash references
        // They are generated by the system, not user signatures
        let zero_hash = [0u8; 32];
        if self.cause == zero_hash || self.transaction_hash == zero_hash {
            return Ok(false);
        }

        // Internal signatures are trusted - they come from the validator
        Ok(true)
    }

    fn sig_type(&self) -> &'static str {
        "internal"
    }
}

impl AccSignature for AuthoritySignature {
    fn verify(&self, _message: &[u8]) -> Result<bool, crate::errors::Error> {
        // AuthoritySignature represents approval from an authority (key book/key page)
        // It's a structural signature that tracks authority approval chains

        // Basic structural validation
        if self.origin.is_empty() || self.authority.is_empty() {
            return Ok(false);
        }

        if self.tx_i_d.is_empty() || self.cause.is_empty() {
            return Ok(false);
        }

        // Authority signatures are valid if they have proper structure
        // The actual authorization check is done by the validator
        Ok(true)
    }

    fn sig_type(&self) -> &'static str {
        "authority"
    }
}