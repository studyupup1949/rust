//! Transaction signing utilities for Accumulate protocol
//!
//! This module implements proper binary encoding for transaction signing
//! matching the Go core and Dart SDK implementations.

use super::writer::BinaryWriter;
use sha2::{Digest, Sha256};

/// Signature type enum values matching Go core
pub mod signature_types {
    pub const UNKNOWN: u64 = 0;
    pub const LEGACY_ED25519: u64 = 1;
    pub const ED25519: u64 = 2;
    pub const RCD1: u64 = 3;
    pub const BTC: u64 = 4;
    pub const BTC_LEGACY: u64 = 5;
    pub const ETH: u64 = 6;
    pub const DELEGATED: u64 = 7;
    pub const INTERNAL: u64 = 8;
    pub const RSA_SHA256: u64 = 9;
    pub const ECDSA_SHA256: u64 = 10;
    pub const TYPED_DATA: u64 = 11;
    pub const REMOTE: u64 = 12;
    pub const RECEIPT: u64 = 13;
    pub const PARTITION: u64 = 14;
    pub const SET: u64 = 15;
    pub const AUTHORITY: u64 = 16;
}

/// Transaction type enum values matching Go core
pub mod tx_types {
    pub const CREATE_IDENTITY: u64 = 0x01;
    pub const CREATE_TOKEN_ACCOUNT: u64 = 0x02;
    pub const SEND_TOKENS: u64 = 0x03;
    pub const CREATE_DATA_ACCOUNT: u64 = 0x04;
    pub const WRITE_DATA: u64 = 0x05;
    pub const WRITE_DATA_TO: u64 = 0x06;
    pub const ACME_FAUCET: u64 = 0x07;
    pub const CREATE_TOKEN: u64 = 0x08;
    pub const ISSUE_TOKENS: u64 = 0x09;
    pub const BURN_TOKENS: u64 = 0x0A;
    pub const CREATE_LITE_TOKEN_ACCOUNT: u64 = 0x0B;
    pub const CREATE_KEY_PAGE: u64 = 0x0C;
    pub const CREATE_KEY_BOOK: u64 = 0x0D;
    pub const ADD_CREDITS: u64 = 0x0E;
    pub const UPDATE_KEY_PAGE: u64 = 0x0F;
    pub const UPDATE_ACCOUNT_AUTH: u64 = 0x10;
    pub const UPDATE_KEY: u64 = 0x11;
    pub const LOCK_ACCOUNT: u64 = 0x12;
    pub const TRANSFER_CREDITS: u64 = 0x15;
    pub const BURN_CREDITS: u64 = 0x16;
}

/// KeyPageOperation type enum values matching Go core
/// From protocol/key_page_operations.yml
pub mod key_page_op_types {
    pub const UNKNOWN: u64 = 0;
    pub const UPDATE: u64 = 1;     // UpdateKeyOperation
    pub const REMOVE: u64 = 2;     // RemoveKeyOperation
    pub const ADD: u64 = 3;        // AddKeyOperation
    pub const SET_THRESHOLD: u64 = 4;  // SetThresholdKeyPageOperation
    pub const UPDATE_ALLOWED: u64 = 5; // UpdateAllowedKeyPageOperation
    pub const SET_REJECT_THRESHOLD: u64 = 6;   // SetRejectThresholdKeyPageOperation
    pub const SET_RESPONSE_THRESHOLD: u64 = 7; // SetResponseThresholdKeyPageOperation
}

/// Compute signature metadata hash for ED25519 signatures
///
/// This computes the SHA256 of the binary-encoded signature metadata,
/// which is used as both the transaction initiator AND part of the signing preimage.
///
/// Field order matches Go: protocol/types_gen.go ED25519Signature.MarshalBinary
/// - Field 1: Type (enum = 2 for ED25519)
/// - Field 2: PublicKey (bytes)
/// - Field 3: Signature (bytes) - OMITTED for metadata
/// - Field 4: Signer (URL as string)
/// - Field 5: SignerVersion (uint)
/// - Field 6: Timestamp (uint)
/// - Field 7: Vote (enum)
/// - Field 8: TransactionHash (hash) - OMITTED for metadata (zeros)
/// - Field 9: Memo (string)
/// - Field 10: Data (bytes)
pub fn compute_ed25519_signature_metadata_hash(
    public_key: &[u8],
    signer: &str,
    signer_version: u64,
    timestamp: u64,
) -> [u8; 32] {
    compute_signature_metadata_hash(
        signature_types::ED25519,
        public_key,
        signer,
        signer_version,
        timestamp,
        0, // vote
        None, // memo
        None, // data
    )
}

/// Compute signature metadata hash for any signature type
pub fn compute_signature_metadata_hash(
    signature_type: u64,
    public_key: &[u8],
    signer: &str,
    signer_version: u64,
    timestamp: u64,
    vote: u64,
    memo: Option<&str>,
    data: Option<&[u8]>,
) -> [u8; 32] {
    let mut writer = BinaryWriter::new();

    // Field 1: Type (enum)
    let _ = writer.write_uvarint(1);
    let _ = writer.write_uvarint(signature_type);

    // Field 2: PublicKey (bytes with length prefix)
    if !public_key.is_empty() {
        let _ = writer.write_uvarint(2);
        let _ = writer.write_uvarint(public_key.len() as u64);
        let _ = writer.write_bytes(public_key);
    }

    // Field 3: Signature - OMITTED for metadata

    // Field 4: Signer URL (string with length prefix)
    if !signer.is_empty() {
        let _ = writer.write_uvarint(4);
        let signer_bytes = signer.as_bytes();
        let _ = writer.write_uvarint(signer_bytes.len() as u64);
        let _ = writer.write_bytes(signer_bytes);
    }

    // Field 5: SignerVersion (uint)
    if signer_version != 0 {
        let _ = writer.write_uvarint(5);
        let _ = writer.write_uvarint(signer_version);
    }

    // Field 6: Timestamp (uint)
    if timestamp != 0 {
        let _ = writer.write_uvarint(6);
        let _ = writer.write_uvarint(timestamp);
    }

    // Field 7: Vote (enum)
    if vote != 0 {
        let _ = writer.write_uvarint(7);
        let _ = writer.write_uvarint(vote);
    }

    // Field 8: TransactionHash - OMITTED for metadata (zeros)

    // Field 9: Memo (string)
    if let Some(memo_str) = memo {
        if !memo_str.is_empty() {
            let _ = writer.write_uvarint(9);
            let memo_bytes = memo_str.as_bytes();
            let _ = writer.write_uvarint(memo_bytes.len() as u64);
            let _ = writer.write_bytes(memo_bytes);
        }
    }

    // Field 10: Data (bytes)
    if let Some(data_bytes) = data {
        if !data_bytes.is_empty() {
            let _ = writer.write_uvarint(10);
            let _ = writer.write_uvarint(data_bytes.len() as u64);
            let _ = writer.write_bytes(data_bytes);
        }
    }

    // Hash the encoded metadata
    sha256_bytes(writer.bytes())
}

/// Marshal transaction header to binary format
///
/// Field order matches Go: protocol/types_gen.go TransactionHeader.MarshalBinary
/// - Field 1: Principal (URL as string)
/// - Field 2: Initiator (hash, 32 bytes, no length prefix)
/// - Field 3: Memo (string)
/// - Field 4: Metadata (bytes)
pub fn marshal_transaction_header(
    principal: &str,
    initiator: &[u8; 32],
    memo: Option<&str>,
    metadata: Option<&[u8]>,
) -> Vec<u8> {
    let mut writer = BinaryWriter::new();

    // Field 1: Principal URL
    let _ = writer.write_uvarint(1);
    let principal_bytes = principal.as_bytes();
    let _ = writer.write_uvarint(principal_bytes.len() as u64);
    let _ = writer.write_bytes(principal_bytes);

    // Field 2: Initiator (32 byte hash, no length prefix)
    // Only write if not all zeros
    let is_zero = initiator.iter().all(|&b| b == 0);
    if !is_zero {
        let _ = writer.write_uvarint(2);
        let _ = writer.write_bytes(initiator);
    }

    // Field 3: Memo
    if let Some(memo_str) = memo {
        if !memo_str.is_empty() {
            let _ = writer.write_uvarint(3);
            let memo_bytes = memo_str.as_bytes();
            let _ = writer.write_uvarint(memo_bytes.len() as u64);
            let _ = writer.write_bytes(memo_bytes);
        }
    }

    // Field 4: Metadata
    if let Some(metadata_bytes) = metadata {
        if !metadata_bytes.is_empty() {
            let _ = writer.write_uvarint(4);
            let _ = writer.write_uvarint(metadata_bytes.len() as u64);
            let _ = writer.write_bytes(metadata_bytes);
        }
    }

    writer.into_bytes()
}

/// Marshal AddCredits transaction body to binary format
///
/// Field order matches Go: protocol/types_gen.go AddCredits.MarshalBinary
/// - Field 1: Type (enum)
/// - Field 2: Recipient (URL as string)
/// - Field 3: Amount (BigInt)
/// - Field 4: Oracle (uint)
pub fn marshal_add_credits_body(
    recipient: &str,
    amount: u64,
    oracle: u64,
) -> Vec<u8> {
    let mut writer = BinaryWriter::new();

    // Field 1: Type (AddCredits = 0x0E)
    let _ = writer.write_uvarint(1);
    let _ = writer.write_uvarint(tx_types::ADD_CREDITS);

    // Field 2: Recipient URL
    let _ = writer.write_uvarint(2);
    let recipient_bytes = recipient.as_bytes();
    let _ = writer.write_uvarint(recipient_bytes.len() as u64);
    let _ = writer.write_bytes(recipient_bytes);

    // Field 3: Amount (BigInt - encode as length-prefixed big-endian bytes)
    if amount > 0 {
        let _ = writer.write_uvarint(3);
        let amount_bytes = amount_to_bigint_bytes(amount);
        let _ = writer.write_uvarint(amount_bytes.len() as u64);
        let _ = writer.write_bytes(&amount_bytes);
    }

    // Field 4: Oracle
    if oracle > 0 {
        let _ = writer.write_uvarint(4);
        let _ = writer.write_uvarint(oracle);
    }

    writer.into_bytes()
}

/// Marshal SendTokens transaction body to binary format
///
/// Field order matches Go: protocol/types_gen.go SendTokens.MarshalBinary
/// - Field 1: Type (enum)
/// - Field 2: Hash (optional) - skipped
/// - Field 3: Meta (optional) - skipped
/// - Field 4: To (repeated TokenRecipient)
pub fn marshal_send_tokens_body(
    recipients: &[(String, u64)], // (url, amount)
) -> Vec<u8> {
    let mut writer = BinaryWriter::new();

    // Field 1: Type (SendTokens = 0x03)
    let _ = writer.write_uvarint(1);
    let _ = writer.write_uvarint(tx_types::SEND_TOKENS);

    // Field 4: To (repeated TokenRecipient)
    for (url, amount) in recipients {
        let recipient_bytes = marshal_token_recipient(url, *amount);
        let _ = writer.write_uvarint(4);
        let _ = writer.write_uvarint(recipient_bytes.len() as u64);
        let _ = writer.write_bytes(&recipient_bytes);
    }

    writer.into_bytes()
}

/// Marshal CreateIdentity transaction body to binary format
///
/// Field order matches Go: protocol/types_gen.go CreateIdentity.MarshalBinary
/// - Field 1: Type (enum)
/// - Field 2: Url (URL as string)
/// - Field 3: KeyHash (bytes)
/// - Field 4: KeyBookUrl (URL as string)
pub fn marshal_create_identity_body(
    url: &str,
    key_hash: &[u8],
    key_book_url: &str,
) -> Vec<u8> {
    let mut writer = BinaryWriter::new();

    // Field 1: Type (CreateIdentity = 0x01)
    let _ = writer.write_uvarint(1);
    let _ = writer.write_uvarint(tx_types::CREATE_IDENTITY);

    // Field 2: Url
    let _ = writer.write_uvarint(2);
    let url_bytes = url.as_bytes();
    let _ = writer.write_uvarint(url_bytes.len() as u64);
    let _ = writer.write_bytes(url_bytes);

    // Field 3: KeyHash (bytes with length prefix)
    if !key_hash.is_empty() {
        let _ = writer.write_uvarint(3);
        let _ = writer.write_uvarint(key_hash.len() as u64);
        let _ = writer.write_bytes(key_hash);
    }

    // Field 4: KeyBookUrl
    let _ = writer.write_uvarint(4);
    let book_bytes = key_book_url.as_bytes();
    let _ = writer.write_uvarint(book_bytes.len() as u64);
    let _ = writer.write_bytes(book_bytes);

    writer.into_bytes()
}

/// Marshal CreateDataAccount transaction body to binary format
///
/// Field order matches Go: protocol/types_gen.go CreateDataAccount.MarshalBinary
/// - Field 1: Type (enum)
/// - Field 2: Url (URL as string)
/// - Field 3: Authorities (repeated URLs)
pub fn marshal_create_data_account_body(url: &str) -> Vec<u8> {
    let mut writer = BinaryWriter::new();

    // Field 1: Type (CreateDataAccount = 0x04)
    let _ = writer.write_uvarint(1);
    let _ = writer.write_uvarint(tx_types::CREATE_DATA_ACCOUNT);

    // Field 2: Url
    if !url.is_empty() {
        let _ = writer.write_uvarint(2);
        let url_bytes = url.as_bytes();
        let _ = writer.write_uvarint(url_bytes.len() as u64);
        let _ = writer.write_bytes(url_bytes);
    }

    writer.into_bytes()
}

/// Marshal WriteData transaction body to binary format
///
/// Field order matches Go: protocol/types_gen.go WriteData.MarshalBinary
/// - Field 1: Type (enum)
/// - Field 2: Entry (nested DataEntry)
/// - Field 3: Scratch (bool, optional)
/// - Field 4: WriteToState (bool, optional)
pub fn marshal_write_data_body(entries_hex: &[String], scratch: bool, write_to_state: bool) -> Vec<u8> {
    let mut writer = BinaryWriter::new();

    // Field 1: Type (WriteData = 0x05)
    let _ = writer.write_uvarint(1);
    let _ = writer.write_uvarint(tx_types::WRITE_DATA);

    // Field 2: Entry (nested DataEntry)
    if !entries_hex.is_empty() {
        let entry_bytes = marshal_data_entry(entries_hex);
        let _ = writer.write_uvarint(2);
        let _ = writer.write_uvarint(entry_bytes.len() as u64);
        let _ = writer.write_bytes(&entry_bytes);
    }

    // Field 3: Scratch (only if true)
    if scratch {
        let _ = writer.write_uvarint(3);
        let _ = writer.write_uvarint(1); // true = 1
    }

    // Field 4: WriteToState (only if true)
    if write_to_state {
        let _ = writer.write_uvarint(4);
        let _ = writer.write_uvarint(1); // true = 1
    }

    writer.into_bytes()
}

/// Marshal a DataEntry (DoubleHash type)
///
/// Field order:
/// - Field 1: Type (enum = 3 for DoubleHash)
/// - Field 2: Data (repeated bytes)
fn marshal_data_entry(entries_hex: &[String]) -> Vec<u8> {
    let mut writer = BinaryWriter::new();

    // Field 1: Type (DoubleHash = 3)
    let _ = writer.write_uvarint(1);
    let _ = writer.write_uvarint(3); // DataEntryType.DoubleHash

    // Field 2: Data (repeated bytes)
    for entry_hex in entries_hex {
        if let Ok(data) = hex::decode(entry_hex) {
            let _ = writer.write_uvarint(2);
            let _ = writer.write_uvarint(data.len() as u64);
            let _ = writer.write_bytes(&data);
        }
    }

    writer.into_bytes()
}

/// Marshal CreateTokenAccount transaction body to binary format
///
/// Field order matches Go: protocol/types_gen.go CreateTokenAccount.MarshalBinary
/// - Field 1: Type (enum)
/// - Field 2: Url (URL as string)
/// - Field 3: TokenUrl (URL as string)
/// - Field 4: Authorities (repeated URLs)
pub fn marshal_create_token_account_body(
    url: &str,
    token_url: &str,
) -> Vec<u8> {
    let mut writer = BinaryWriter::new();

    // Field 1: Type (CreateTokenAccount = 0x02)
    let _ = writer.write_uvarint(1);
    let _ = writer.write_uvarint(tx_types::CREATE_TOKEN_ACCOUNT);

    // Field 2: Url
    if !url.is_empty() {
        let _ = writer.write_uvarint(2);
        let url_bytes = url.as_bytes();
        let _ = writer.write_uvarint(url_bytes.len() as u64);
        let _ = writer.write_bytes(url_bytes);
    }

    // Field 3: TokenUrl
    if !token_url.is_empty() {
        let _ = writer.write_uvarint(3);
        let token_bytes = token_url.as_bytes();
        let _ = writer.write_uvarint(token_bytes.len() as u64);
        let _ = writer.write_bytes(token_bytes);
    }

    writer.into_bytes()
}

/// Marshal a TokenRecipient
fn marshal_token_recipient(url: &str, amount: u64) -> Vec<u8> {
    let mut writer = BinaryWriter::new();

    // Field 1: URL
    let _ = writer.write_uvarint(1);
    let url_bytes = url.as_bytes();
    let _ = writer.write_uvarint(url_bytes.len() as u64);
    let _ = writer.write_bytes(url_bytes);

    // Field 2: Amount
    if amount > 0 {
        let _ = writer.write_uvarint(2);
        let amount_bytes = amount_to_bigint_bytes(amount);
        let _ = writer.write_uvarint(amount_bytes.len() as u64);
        let _ = writer.write_bytes(&amount_bytes);
    }

    writer.into_bytes()
}

/// Convert u64 amount to big-endian bytes (minimal encoding)
fn amount_to_bigint_bytes(mut value: u64) -> Vec<u8> {
    if value == 0 {
        return vec![];
    }

    let mut bytes = Vec::new();
    while value > 0 {
        bytes.push((value & 0xFF) as u8);
        value >>= 8;
    }
    bytes.reverse(); // Convert to big-endian
    bytes
}

/// Marshal CreateToken transaction body to binary format
///
/// Field order matches Go: protocol/user_transactions.yml CreateToken
/// - Field 1: Type (enum 0x08)
/// - Field 2: Url (URL)
/// - Field 4: Symbol (string) - note: field 3 is skipped per Go spec
/// - Field 5: Precision (uint)
/// - Field 6: Properties (URL, optional)
/// - Field 7: SupplyLimit (BigInt, optional)
pub fn marshal_create_token_body(url: &str, symbol: &str, precision: u64, supply_limit: Option<u64>) -> Vec<u8> {
    let mut writer = BinaryWriter::new();

    // Field 1: Type (CreateToken = 0x08)
    let _ = writer.write_uvarint(1);
    let _ = writer.write_uvarint(tx_types::CREATE_TOKEN);

    // Field 2: Url
    if !url.is_empty() {
        let _ = writer.write_uvarint(2);
        let url_bytes = url.as_bytes();
        let _ = writer.write_uvarint(url_bytes.len() as u64);
        let _ = writer.write_bytes(url_bytes);
    }

    // Field 4: Symbol (note: field 3 is not used)
    if !symbol.is_empty() {
        let _ = writer.write_uvarint(4);
        let symbol_bytes = symbol.as_bytes();
        let _ = writer.write_uvarint(symbol_bytes.len() as u64);
        let _ = writer.write_bytes(symbol_bytes);
    }

    // Field 5: Precision (always write, even if 0 - it's a valid value)
    let _ = writer.write_uvarint(5);
    let _ = writer.write_uvarint(precision);

    // Field 7: SupplyLimit (optional)
    if let Some(limit) = supply_limit {
        if limit > 0 {
            let _ = writer.write_uvarint(7);
            let limit_bytes = amount_to_bigint_bytes(limit);
            let _ = writer.write_uvarint(limit_bytes.len() as u64);
            let _ = writer.write_bytes(&limit_bytes);
        }
    }

    writer.into_bytes()
}

/// Marshal IssueTokens transaction body to binary format
///
/// Field order matches Go: protocol/user_transactions.yml IssueTokens
/// - Field 1: Type (enum 0x09)
/// - Field 4: To (repeated TokenRecipient)
pub fn marshal_issue_tokens_body(recipients: &[(&str, u64)]) -> Vec<u8> {
    let mut writer = BinaryWriter::new();

    // Field 1: Type (IssueTokens = 0x09)
    let _ = writer.write_uvarint(1);
    let _ = writer.write_uvarint(tx_types::ISSUE_TOKENS);

    // Field 4: To (repeated TokenRecipient)
    for (url, amount) in recipients {
        let recipient_bytes = marshal_token_recipient(url, *amount);
        let _ = writer.write_uvarint(4);
        let _ = writer.write_uvarint(recipient_bytes.len() as u64);
        let _ = writer.write_bytes(&recipient_bytes);
    }

    writer.into_bytes()
}

/// Compute transaction hash using binary encoding
///
/// Based on Go: protocol/transaction_hash.go:27-71
/// Transaction hash = SHA256(SHA256(header_binary) + SHA256(body_binary))
pub fn compute_transaction_hash(header_bytes: &[u8], body_bytes: &[u8]) -> [u8; 32] {
    let header_hash = sha256_bytes(header_bytes);
    let body_hash = sha256_bytes(body_bytes);

    let mut combined = Vec::with_capacity(64);
    combined.extend_from_slice(&header_hash);
    combined.extend_from_slice(&body_hash);

    sha256_bytes(&combined)
}

/// Create signing preimage
///
/// Based on Go: protocol/signature_utils.go:50-57
/// signingHash = SHA256(sigMdHash + txnHash)
pub fn create_signing_preimage(
    signature_metadata_hash: &[u8; 32],
    transaction_hash: &[u8; 32],
) -> [u8; 32] {
    let mut combined = Vec::with_capacity(64);
    combined.extend_from_slice(signature_metadata_hash);
    combined.extend_from_slice(transaction_hash);

    sha256_bytes(&combined)
}

/// SHA256 hash helper
pub fn sha256_bytes(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut output = [0u8; 32];
    output.copy_from_slice(&result);
    output
}

// =============================================================================
// WRITEDATA SPECIAL HASH COMPUTATION
// =============================================================================

/// Compute WriteData body hash using special Merkle algorithm
///
/// Based on Go: protocol/transaction_hash.go:91-114
/// 1. Marshal WriteData body with Entry=nil (only Type, Scratch, WriteToState)
/// 2. Compute Merkle hash of [SHA256(marshaledBody), entryHash]
pub fn compute_write_data_body_hash(entries_hex: &[String], scratch: bool, write_to_state: bool) -> [u8; 32] {
    // Marshal body WITHOUT entry
    let body_without_entry = marshal_write_data_body_without_entry(scratch, write_to_state);
    let body_part_hash = sha256_bytes(&body_without_entry);

    // Compute entry hash
    let entry_hash = if entries_hex.is_empty() {
        [0u8; 32]
    } else {
        compute_data_entry_hash(entries_hex)
    };

    // Merkle hash of [bodyPartHash, entryHash]
    merkle_hash(&[body_part_hash, entry_hash])
}

/// Marshal WriteData body without entry (Entry = nil)
fn marshal_write_data_body_without_entry(scratch: bool, write_to_state: bool) -> Vec<u8> {
    let mut writer = BinaryWriter::new();

    // Field 1: Type (WriteData = 0x05)
    let _ = writer.write_uvarint(1);
    let _ = writer.write_uvarint(tx_types::WRITE_DATA);

    // Field 2: Entry - OMITTED (nil)

    // Field 3: Scratch
    if scratch {
        let _ = writer.write_uvarint(3);
        let _ = writer.write_uvarint(1);
    }

    // Field 4: WriteToState
    if write_to_state {
        let _ = writer.write_uvarint(4);
        let _ = writer.write_uvarint(1);
    }

    writer.into_bytes()
}

/// Compute hash for a DataEntry
///
/// Based on Go: protocol/data_entry.go
/// DoubleHashDataEntry: SHA256(MerkleHash(SHA256(data1), SHA256(data2), ...))
fn compute_data_entry_hash(entries_hex: &[String]) -> [u8; 32] {
    if entries_hex.is_empty() {
        return [0u8; 32];
    }

    // Collect SHA256 hashes of each data item
    let mut data_hashes: Vec<[u8; 32]> = Vec::new();
    for entry_hex in entries_hex {
        if let Ok(data) = hex::decode(entry_hex) {
            data_hashes.push(sha256_bytes(&data));
        }
    }

    if data_hashes.is_empty() {
        return [0u8; 32];
    }

    // Compute Merkle hash of data hashes
    let merkle_root = merkle_hash(&data_hashes);

    // For DoubleHash: return SHA256(merkleRoot)
    sha256_bytes(&merkle_root)
}

/// Compute Merkle hash of a list of hashes
///
/// Based on Go: pkg/database/merkle/hasher.go MerkleHash()
/// Uses a cascading binary tree algorithm
fn merkle_hash(hashes: &[[u8; 32]]) -> [u8; 32] {
    if hashes.is_empty() {
        return [0u8; 32];
    }

    if hashes.len() == 1 {
        return hashes[0];
    }

    // Use the Merkle cascade algorithm from Go
    let mut pending: Vec<Option<[u8; 32]>> = Vec::new();

    for hash in hashes {
        let mut current = *hash;
        let mut i = 0;
        loop {
            // Extend pending if needed
            if i >= pending.len() {
                pending.push(Some(current));
                break;
            }

            // If slot is empty, put hash there
            if pending[i].is_none() {
                pending[i] = Some(current);
                break;
            }

            // Combine hashes and carry to next level
            current = combine_hashes(&pending[i].unwrap(), &current);
            pending[i] = None;
            i += 1;
        }
    }

    // Combine remaining pending hashes
    let mut anchor: Option<[u8; 32]> = None;
    for v in &pending {
        if anchor.is_none() {
            anchor = *v;
        } else if let Some(val) = v {
            anchor = Some(combine_hashes(val, &anchor.unwrap()));
        }
    }

    anchor.unwrap_or([0u8; 32])
}

/// Combine two hashes: SHA256(left + right)
fn combine_hashes(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut combined = Vec::with_capacity(64);
    combined.extend_from_slice(left);
    combined.extend_from_slice(right);
    sha256_bytes(&combined)
}

// =============================================================================
// UPDATE KEY PAGE ENCODING
// =============================================================================

/// Marshal KeySpecParams to binary format
///
/// Field order matches Go: protocol/key_page_operations.yml KeySpecParams
/// - Field 1: KeyHash (bytes)
/// - Field 2: Delegate (URL, optional)
pub fn marshal_key_spec_params(key_hash: &[u8], delegate: Option<&str>) -> Vec<u8> {
    let mut writer = BinaryWriter::new();

    // Field 1: KeyHash (bytes with length prefix)
    if !key_hash.is_empty() {
        let _ = writer.write_uvarint(1);
        let _ = writer.write_uvarint(key_hash.len() as u64);
        let _ = writer.write_bytes(key_hash);
    }

    // Field 2: Delegate URL (optional)
    if let Some(delegate_url) = delegate {
        if !delegate_url.is_empty() {
            let _ = writer.write_uvarint(2);
            let delegate_bytes = delegate_url.as_bytes();
            let _ = writer.write_uvarint(delegate_bytes.len() as u64);
            let _ = writer.write_bytes(delegate_bytes);
        }
    }

    writer.into_bytes()
}

/// Marshal a KeyPageOperation to binary format
///
/// Each operation type has different fields after the type enum:
/// - AddKeyOperation (type=3): Field 2: Entry (KeySpecParams)
/// - RemoveKeyOperation (type=2): Field 2: Entry (KeySpecParams)
/// - UpdateKeyOperation (type=1): Field 2: OldEntry, Field 3: NewEntry
/// - SetThresholdKeyPageOperation (type=4): Field 2: Threshold (uint)
/// - SetRejectThresholdKeyPageOperation (type=6): Field 2: Threshold (uint)
/// - SetResponseThresholdKeyPageOperation (type=7): Field 2: Threshold (uint)
/// - UpdateAllowedKeyPageOperation (type=5): Field 2: Allow[], Field 3: Deny[]
pub fn marshal_key_page_operation(
    op_type: &str,
    key_hash: Option<&[u8]>,
    delegate: Option<&str>,
    old_key_hash: Option<&[u8]>,
    new_key_hash: Option<&[u8]>,
    threshold: Option<u64>,
) -> Vec<u8> {
    let mut writer = BinaryWriter::new();

    // Map operation type string to enum value
    let op_type_num = match op_type {
        "add" => key_page_op_types::ADD,
        "remove" => key_page_op_types::REMOVE,
        "update" => key_page_op_types::UPDATE,
        "setThreshold" => key_page_op_types::SET_THRESHOLD,
        "setRejectThreshold" => key_page_op_types::SET_REJECT_THRESHOLD,
        "setResponseThreshold" => key_page_op_types::SET_RESPONSE_THRESHOLD,
        "updateAllowed" => key_page_op_types::UPDATE_ALLOWED,
        _ => key_page_op_types::UNKNOWN,
    };

    // Field 1: Type (enum)
    let _ = writer.write_uvarint(1);
    let _ = writer.write_uvarint(op_type_num);

    match op_type {
        "add" | "remove" => {
            // Field 2: Entry (KeySpecParams)
            if let Some(hash) = key_hash {
                let entry_bytes = marshal_key_spec_params(hash, delegate);
                let _ = writer.write_uvarint(2);
                let _ = writer.write_uvarint(entry_bytes.len() as u64);
                let _ = writer.write_bytes(&entry_bytes);
            }
        }
        "update" => {
            // Field 2: OldEntry (KeySpecParams)
            if let Some(old_hash) = old_key_hash {
                let old_entry_bytes = marshal_key_spec_params(old_hash, None);
                let _ = writer.write_uvarint(2);
                let _ = writer.write_uvarint(old_entry_bytes.len() as u64);
                let _ = writer.write_bytes(&old_entry_bytes);
            }
            // Field 3: NewEntry (KeySpecParams)
            if let Some(new_hash) = new_key_hash {
                let new_entry_bytes = marshal_key_spec_params(new_hash, delegate);
                let _ = writer.write_uvarint(3);
                let _ = writer.write_uvarint(new_entry_bytes.len() as u64);
                let _ = writer.write_bytes(&new_entry_bytes);
            }
        }
        "setThreshold" | "setRejectThreshold" | "setResponseThreshold" => {
            // Field 2: Threshold (uint)
            if let Some(thresh) = threshold {
                let _ = writer.write_uvarint(2);
                let _ = writer.write_uvarint(thresh);
            }
        }
        "updateAllowed" => {
            // TODO: Implement Allow/Deny arrays when needed
            // Field 2: Allow (repeated TransactionType)
            // Field 3: Deny (repeated TransactionType)
        }
        _ => {}
    }

    writer.into_bytes()
}

/// Marshal UpdateKeyPage transaction body to binary format
///
/// Field order matches Go: protocol/user_transactions.yml UpdateKeyPage
/// - Field 1: Type (enum 0x0F)
/// - Field 2: Operation (repeated KeyPageOperation)
pub fn marshal_update_key_page_body(operations: &[Vec<u8>]) -> Vec<u8> {
    let mut writer = BinaryWriter::new();

    // Field 1: Type (UpdateKeyPage = 0x0F)
    let _ = writer.write_uvarint(1);
    let _ = writer.write_uvarint(tx_types::UPDATE_KEY_PAGE);

    // Field 2: Operation (repeated, each as nested value)
    for op_bytes in operations {
        let _ = writer.write_uvarint(2);
        let _ = writer.write_uvarint(op_bytes.len() as u64);
        let _ = writer.write_bytes(op_bytes);
    }

    writer.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_metadata_hash() {
        let public_key = [1u8; 32];
        let signer = "acc://test.acme/book/1";
        let signer_version = 1;
        let timestamp = 1234567890000u64;

        let hash = compute_ed25519_signature_metadata_hash(
            &public_key,
            signer,
            signer_version,
            timestamp,
        );

        // Should produce a 32-byte hash
        assert_eq!(hash.len(), 32);

        // Same inputs should produce same hash
        let hash2 = compute_ed25519_signature_metadata_hash(
            &public_key,
            signer,
            signer_version,
            timestamp,
        );
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_transaction_hash() {
        let header = b"test header";
        let body = b"test body";

        let hash = compute_transaction_hash(header, body);
        assert_eq!(hash.len(), 32);

        // Deterministic
        let hash2 = compute_transaction_hash(header, body);
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_signing_preimage() {
        let sig_hash = [1u8; 32];
        let tx_hash = [2u8; 32];

        let preimage = create_signing_preimage(&sig_hash, &tx_hash);
        assert_eq!(preimage.len(), 32);
    }

    #[test]
    fn test_amount_encoding() {
        // Test various amounts
        assert_eq!(amount_to_bigint_bytes(0), vec![] as Vec<u8>);
        assert_eq!(amount_to_bigint_bytes(1), vec![1]);
        assert_eq!(amount_to_bigint_bytes(255), vec![255]);
        assert_eq!(amount_to_bigint_bytes(256), vec![1, 0]);
        assert_eq!(amount_to_bigint_bytes(0x123456), vec![0x12, 0x34, 0x56]);
    }
}
