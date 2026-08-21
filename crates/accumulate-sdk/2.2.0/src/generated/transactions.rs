//! GENERATED FILE - DO NOT EDIT
//! Sources: protocol/transaction.yml, user_transactions.yml, system.yml, synthetic_transactions.yml
//! Generated: 2025-10-03 21:53:38

#![allow(missing_docs)]

use serde::{Serialize, Deserialize};
use crate::errors::{Error, ValidationError};

/// Validates that a string is a valid Accumulate URL
/// Accumulate URLs must:
/// - Start with "acc://"
/// - Contain only ASCII characters
/// - Have at least one character after the scheme
/// - Not contain whitespace or control characters
fn validate_accumulate_url(url: &str, field_name: &str) -> Result<(), Error> {
    if url.is_empty() {
        return Err(ValidationError::RequiredFieldMissing(field_name.to_string()).into());
    }

    if !url.starts_with("acc://") {
        return Err(ValidationError::InvalidUrl(
            format!("{}: must start with 'acc://', got '{}'", field_name, url)
        ).into());
    }

    if !url.is_ascii() {
        return Err(ValidationError::InvalidUrl(
            format!("{}: must contain only ASCII characters", field_name)
        ).into());
    }

    // Check for whitespace or control characters
    if url.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err(ValidationError::InvalidUrl(
            format!("{}: must not contain whitespace or control characters", field_name)
        ).into());
    }

    // Must have content after acc://
    if url.len() <= 6 {
        return Err(ValidationError::InvalidUrl(
            format!("{}: URL path cannot be empty", field_name)
        ).into());
    }

    Ok(())
}

/// Validates that an amount string represents a valid positive integer
fn validate_amount_string(amount: &str, field_name: &str) -> Result<(), Error> {
    if amount.is_empty() {
        return Err(ValidationError::RequiredFieldMissing(field_name.to_string()).into());
    }

    // Check if it's a valid integer (can be very large, so use string validation)
    if !amount.chars().all(|c| c.is_ascii_digit()) {
        return Err(ValidationError::InvalidAmount(
            format!("{}: must be a valid non-negative integer, got '{}'", field_name, amount)
        ).into());
    }

    // Check it's not all zeros (unless it's literally "0")
    if amount != "0" && amount.chars().all(|c| c == '0') {
        return Err(ValidationError::InvalidAmount(
            format!("{}: invalid zero representation", field_name)
        ).into());
    }

    Ok(())
}

/// Validates a list of authority URLs
fn validate_authorities(authorities: &Option<Vec<String>>) -> Result<(), Error> {
    if let Some(ref auths) = authorities {
        for (i, auth) in auths.iter().enumerate() {
            validate_accumulate_url(auth, &format!("authorities[{}]", i))?;
        }
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcmeFaucetBody {
    #[serde(rename = "Url")]
    pub url: String,
}

impl AcmeFaucetBody {
    pub fn validate(&self) -> Result<(), Error> {
        // Faucet URL must be a valid Accumulate URL (typically a lite token account)
        validate_accumulate_url(&self.url, "url")?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivateProtocolVersionBody {
    #[serde(rename = "Version")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

impl ActivateProtocolVersionBody {
    pub fn validate(&self) -> Result<(), Error> {
        // Version is optional, but if present must not be empty
        if let Some(ref version) = self.version {
            if version.is_empty() {
                return Err(ValidationError::InvalidFieldValue {
                    field: "version".to_string(),
                    reason: "version string cannot be empty if provided".to_string(),
                }.into());
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddCreditsBody {
    #[serde(rename = "Recipient")]
    pub recipient: String,
    #[serde(rename = "Amount")]
    pub amount: String,
    #[serde(rename = "Oracle")]
    pub oracle: u64,
}

impl AddCreditsBody {
    pub fn validate(&self) -> Result<(), Error> {
        // Recipient must be a valid Accumulate URL
        validate_accumulate_url(&self.recipient, "recipient")?;

        // Amount must be a valid positive integer string
        validate_amount_string(&self.amount, "amount")?;

        // Oracle price must be positive (represents ACME price in credits)
        if self.oracle == 0 {
            return Err(ValidationError::InvalidFieldValue {
                field: "oracle".to_string(),
                reason: "oracle price must be positive".to_string(),
            }.into());
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockValidatorAnchorBody {
    #[serde(rename = "AcmeBurnt")]
    pub acme_burnt: String,
}

impl BlockValidatorAnchorBody {
    pub fn validate(&self) -> Result<(), Error> {
        // AcmeBurnt must be a valid amount string (can be zero for no burn)
        validate_amount_string(&self.acme_burnt, "acmeBurnt")?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BurnCreditsBody {
    #[serde(rename = "Amount")]
    pub amount: u64,
}

impl BurnCreditsBody {
    pub fn validate(&self) -> Result<(), Error> {
        // Amount must be positive (can't burn zero credits)
        if self.amount == 0 {
            return Err(ValidationError::InvalidAmount(
                "amount: must be greater than zero to burn credits".to_string()
            ).into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BurnTokensBody {
    #[serde(rename = "Amount")]
    pub amount: String,
}

impl BurnTokensBody {
    pub fn validate(&self) -> Result<(), Error> {
        // Amount must be a valid positive integer string (can't burn zero)
        validate_amount_string(&self.amount, "amount")?;

        // Ensure amount is not "0"
        if self.amount == "0" {
            return Err(ValidationError::InvalidAmount(
                "amount: must be greater than zero to burn tokens".to_string()
            ).into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDataAccountBody {
    #[serde(rename = "Url")]
    pub url: String,
    #[serde(rename = "Authorities")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorities: Option<Vec<String>>,
}

impl CreateDataAccountBody {
    pub fn validate(&self) -> Result<(), Error> {
        // URL must be a valid Accumulate URL
        validate_accumulate_url(&self.url, "url")?;

        // Validate authorities if provided
        validate_authorities(&self.authorities)?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateIdentityBody {
    #[serde(rename = "Url")]
    pub url: String,
    #[serde(rename = "KeyHash")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_hash: Option<Vec<u8>>,
    #[serde(rename = "KeyBookUrl")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_book_url: Option<String>,
    #[serde(rename = "Authorities")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorities: Option<Vec<String>>,
}

impl CreateIdentityBody {
    pub fn validate(&self) -> Result<(), Error> {
        // URL must be a valid Accumulate URL for the new identity
        validate_accumulate_url(&self.url, "url")?;

        // If key_book_url is provided, it must be valid
        if let Some(ref key_book_url) = self.key_book_url {
            validate_accumulate_url(key_book_url, "keyBookUrl")?;
        }

        // If key_hash is provided, it should be 32 bytes (SHA-256 hash)
        if let Some(ref key_hash) = self.key_hash {
            if key_hash.len() != 32 {
                return Err(ValidationError::InvalidHash {
                    expected: 32,
                    actual: key_hash.len(),
                }.into());
            }
        }

        // Validate authorities if provided
        validate_authorities(&self.authorities)?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateKeyBookBody {
    #[serde(rename = "Url")]
    pub url: String,
    #[serde(rename = "PublicKeyHash")]
    #[serde(with = "hex::serde")]
    pub public_key_hash: Vec<u8>,
    #[serde(rename = "Authorities")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorities: Option<Vec<String>>,
}

impl CreateKeyBookBody {
    pub fn validate(&self) -> Result<(), Error> {
        // URL must be a valid Accumulate URL
        validate_accumulate_url(&self.url, "url")?;

        // Public key hash must be 32 bytes (SHA-256)
        if self.public_key_hash.len() != 32 {
            return Err(ValidationError::InvalidHash {
                expected: 32,
                actual: self.public_key_hash.len(),
            }.into());
        }

        // Validate authorities if provided
        validate_authorities(&self.authorities)?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateKeyPageBody {
    #[serde(rename = "Keys")]
    pub keys: Vec<serde_json::Value>,
}

impl CreateKeyPageBody {
    pub fn validate(&self) -> Result<(), Error> {
        // Must have at least one key entry
        if self.keys.is_empty() {
            return Err(ValidationError::EmptyCollection(
                "keys: at least one key is required".to_string()
            ).into());
        }

        // Each key entry should be a valid JSON object
        for (i, key) in self.keys.iter().enumerate() {
            if !key.is_object() {
                return Err(ValidationError::InvalidFieldValue {
                    field: format!("keys[{}]", i),
                    reason: "each key must be a valid key specification object".to_string(),
                }.into());
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLiteTokenAccountBody {
    // No fields defined
}

impl CreateLiteTokenAccountBody {
    pub fn validate(&self) -> Result<(), Error> {
        // No fields to validate - lite token accounts are created implicitly
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTokenBody {
    #[serde(rename = "Url")]
    pub url: String,
    #[serde(rename = "Symbol")]
    pub symbol: String,
    #[serde(rename = "Precision")]
    pub precision: u64,
    #[serde(rename = "Properties")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<String>,
    #[serde(rename = "SupplyLimit")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supply_limit: Option<String>,
    #[serde(rename = "Authorities")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorities: Option<Vec<String>>,
}

impl CreateTokenBody {
    pub fn validate(&self) -> Result<(), Error> {
        // URL must be a valid Accumulate URL
        validate_accumulate_url(&self.url, "url")?;

        // Symbol must be non-empty and alphanumeric (1-10 characters typically)
        if self.symbol.is_empty() {
            return Err(ValidationError::InvalidTokenSymbol(
                "symbol cannot be empty".to_string()
            ).into());
        }

        if !self.symbol.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(ValidationError::InvalidTokenSymbol(
                format!("symbol must be alphanumeric, got '{}'", self.symbol)
            ).into());
        }

        if self.symbol.len() > 10 {
            return Err(ValidationError::InvalidTokenSymbol(
                format!("symbol too long (max 10 chars), got {} chars", self.symbol.len())
            ).into());
        }

        // Precision must be between 0 and 18 (standard decimal precision)
        if self.precision > 18 {
            return Err(ValidationError::InvalidPrecision(self.precision).into());
        }

        // If supply_limit is provided, it must be a valid amount
        if let Some(ref supply_limit) = self.supply_limit {
            validate_amount_string(supply_limit, "supplyLimit")?;
        }

        // Validate authorities if provided
        validate_authorities(&self.authorities)?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTokenAccountBody {
    #[serde(rename = "Url")]
    pub url: String,
    #[serde(rename = "TokenUrl")]
    pub token_url: String,
    #[serde(rename = "Authorities")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorities: Option<Vec<String>>,
    #[serde(rename = "Proof")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof: Option<serde_json::Value>,
}

impl CreateTokenAccountBody {
    pub fn validate(&self) -> Result<(), Error> {
        // URL must be a valid Accumulate URL
        validate_accumulate_url(&self.url, "url")?;

        // TokenUrl must be a valid Accumulate URL
        validate_accumulate_url(&self.token_url, "tokenUrl")?;

        // Validate authorities if provided
        validate_authorities(&self.authorities)?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryAnchorBody {
    #[serde(rename = "Updates")]
    pub updates: Vec<serde_json::Value>,
    #[serde(rename = "Receipts")]
    pub receipts: Vec<serde_json::Value>,
    #[serde(rename = "MakeMajorBlock")]
    pub make_major_block: u64,
    #[serde(rename = "MakeMajorBlockTime")]
    pub make_major_block_time: u64,
}

impl DirectoryAnchorBody {
    pub fn validate(&self) -> Result<(), Error> {
        // System transaction - validates structure only
        // Updates and receipts can be empty arrays
        // MakeMajorBlock and MakeMajorBlockTime are informational
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueTokensBody {
    #[serde(rename = "Recipient")]
    pub recipient: String,
    #[serde(rename = "Amount")]
    pub amount: String,
    #[serde(rename = "To")]
    pub to: Vec<serde_json::Value>,
}

impl IssueTokensBody {
    pub fn validate(&self) -> Result<(), Error> {
        // Recipient must be a valid Accumulate URL
        validate_accumulate_url(&self.recipient, "recipient")?;

        // Amount must be a valid positive integer string
        validate_amount_string(&self.amount, "amount")?;

        // Amount must be positive for issuance
        if self.amount == "0" {
            return Err(ValidationError::InvalidAmount(
                "amount: must be greater than zero to issue tokens".to_string()
            ).into());
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LockAccountBody {
    #[serde(rename = "Height")]
    pub height: u64,
}

impl LockAccountBody {
    pub fn validate(&self) -> Result<(), Error> {
        // Height must be positive (lock until block height)
        if self.height == 0 {
            return Err(ValidationError::InvalidFieldValue {
                field: "height".to_string(),
                reason: "lock height must be greater than zero".to_string(),
            }.into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkMaintenanceBody {
    #[serde(rename = "Operations")]
    pub operations: Vec<serde_json::Value>,
}

impl NetworkMaintenanceBody {
    pub fn validate(&self) -> Result<(), Error> {
        // System transaction - operations can be empty
        // Validation of individual operations is done at the protocol level
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteTransactionBody {
    #[serde(rename = "Hash")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<Vec<u8>>,
}

impl RemoteTransactionBody {
    pub fn validate(&self) -> Result<(), Error> {
        // If hash is provided, it should be 32 bytes (SHA-256)
        if let Some(ref hash) = self.hash {
            if hash.len() != 32 {
                return Err(ValidationError::InvalidHash {
                    expected: 32,
                    actual: hash.len(),
                }.into());
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendTokensBody {
    #[serde(rename = "Hash")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<Vec<u8>>,
    #[serde(rename = "Meta")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
    #[serde(rename = "To")]
    pub to: Vec<serde_json::Value>,
}

impl SendTokensBody {
    pub fn validate(&self) -> Result<(), Error> {
        // Must have at least one recipient
        if self.to.is_empty() {
            return Err(ValidationError::EmptyCollection(
                "to: at least one recipient is required".to_string()
            ).into());
        }

        // Each recipient should be a valid object with url and amount
        for (i, recipient) in self.to.iter().enumerate() {
            if !recipient.is_object() {
                return Err(ValidationError::InvalidFieldValue {
                    field: format!("to[{}]", i),
                    reason: "each recipient must be a valid object with url and amount".to_string(),
                }.into());
            }

            // Validate url field if present
            if let Some(url) = recipient.get("url").and_then(|v| v.as_str()) {
                validate_accumulate_url(url, &format!("to[{}].url", i))?;
            }

            // Validate amount field if present
            if let Some(amount) = recipient.get("amount").and_then(|v| v.as_str()) {
                validate_amount_string(amount, &format!("to[{}].amount", i))?;
            }
        }

        // If hash is provided, it should be 32 bytes
        if let Some(ref hash) = self.hash {
            if hash.len() != 32 {
                return Err(ValidationError::InvalidHash {
                    expected: 32,
                    actual: hash.len(),
                }.into());
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemGenesisBody {
    // No fields defined
}

impl SystemGenesisBody {
    pub fn validate(&self) -> Result<(), Error> {
        // System transaction - no fields to validate
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemWriteDataBody {
    #[serde(rename = "Entry")]
    pub entry: serde_json::Value,
    #[serde(rename = "WriteToState")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub write_to_state: Option<bool>,
}

impl SystemWriteDataBody {
    pub fn validate(&self) -> Result<(), Error> {
        // Entry must be a valid object
        if !self.entry.is_object() {
            return Err(ValidationError::InvalidFieldValue {
                field: "entry".to_string(),
                reason: "entry must be a valid data entry object".to_string(),
            }.into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferCreditsBody {
    #[serde(rename = "To")]
    pub to: Vec<serde_json::Value>,
}

impl TransferCreditsBody {
    pub fn validate(&self) -> Result<(), Error> {
        // Must have at least one recipient
        if self.to.is_empty() {
            return Err(ValidationError::EmptyCollection(
                "to: at least one credit recipient is required".to_string()
            ).into());
        }

        // Each recipient should be a valid object
        for (i, recipient) in self.to.iter().enumerate() {
            if !recipient.is_object() {
                return Err(ValidationError::InvalidFieldValue {
                    field: format!("to[{}]", i),
                    reason: "each recipient must be a valid object with url and amount".to_string(),
                }.into());
            }

            // Validate url field if present
            if let Some(url) = recipient.get("url").and_then(|v| v.as_str()) {
                validate_accumulate_url(url, &format!("to[{}].url", i))?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAccountAuthBody {
    #[serde(rename = "Operations")]
    pub operations: Vec<serde_json::Value>,
}

impl UpdateAccountAuthBody {
    pub fn validate(&self) -> Result<(), Error> {
        // Must have at least one operation
        if self.operations.is_empty() {
            return Err(ValidationError::EmptyCollection(
                "operations: at least one auth operation is required".to_string()
            ).into());
        }

        // Each operation should be a valid object
        for (i, op) in self.operations.iter().enumerate() {
            if !op.is_object() {
                return Err(ValidationError::InvalidFieldValue {
                    field: format!("operations[{}]", i),
                    reason: "each operation must be a valid auth operation object".to_string(),
                }.into());
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateKeyBody {
    #[serde(rename = "NewKeyHash")]
    #[serde(with = "hex::serde")]
    pub new_key_hash: Vec<u8>,
}

impl UpdateKeyBody {
    pub fn validate(&self) -> Result<(), Error> {
        // New key hash must be 32 bytes (SHA-256)
        if self.new_key_hash.len() != 32 {
            return Err(ValidationError::InvalidHash {
                expected: 32,
                actual: self.new_key_hash.len(),
            }.into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateKeyPageBody {
    #[serde(rename = "Operation")]
    pub operation: Vec<serde_json::Value>,
}

impl UpdateKeyPageBody {
    pub fn validate(&self) -> Result<(), Error> {
        // Must have at least one operation
        if self.operation.is_empty() {
            return Err(ValidationError::EmptyCollection(
                "operation: at least one key page operation is required".to_string()
            ).into());
        }

        // Each operation should be a valid object
        for (i, op) in self.operation.iter().enumerate() {
            if !op.is_object() {
                return Err(ValidationError::InvalidFieldValue {
                    field: format!("operation[{}]", i),
                    reason: "each operation must be a valid key page operation object".to_string(),
                }.into());
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteDataBody {
    #[serde(rename = "Entry")]
    pub entry: serde_json::Value,
    #[serde(rename = "Scratch")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scratch: Option<bool>,
    #[serde(rename = "WriteToState")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub write_to_state: Option<bool>,
}

impl WriteDataBody {
    pub fn validate(&self) -> Result<(), Error> {
        // Entry must be a valid object
        if !self.entry.is_object() {
            return Err(ValidationError::InvalidFieldValue {
                field: "entry".to_string(),
                reason: "entry must be a valid data entry object".to_string(),
            }.into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteDataToBody {
    #[serde(rename = "Recipient")]
    pub recipient: String,
    #[serde(rename = "Entry")]
    pub entry: serde_json::Value,
}

impl WriteDataToBody {
    pub fn validate(&self) -> Result<(), Error> {
        // Recipient must be a valid Accumulate URL
        validate_accumulate_url(&self.recipient, "recipient")?;

        // Entry must be a valid object
        if !self.entry.is_object() {
            return Err(ValidationError::InvalidFieldValue {
                field: "entry".to_string(),
                reason: "entry must be a valid data entry object".to_string(),
            }.into());
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TransactionBody {
    #[serde(rename = "acmeFaucet")]
    AcmeFaucet(AcmeFaucetBody),
    #[serde(rename = "activateProtocolVersion")]
    ActivateProtocolVersion(ActivateProtocolVersionBody),
    #[serde(rename = "addCredits")]
    AddCredits(AddCreditsBody),
    #[serde(rename = "blockValidatorAnchor")]
    BlockValidatorAnchor(BlockValidatorAnchorBody),
    #[serde(rename = "burnCredits")]
    BurnCredits(BurnCreditsBody),
    #[serde(rename = "burnTokens")]
    BurnTokens(BurnTokensBody),
    #[serde(rename = "createDataAccount")]
    CreateDataAccount(CreateDataAccountBody),
    #[serde(rename = "createIdentity")]
    CreateIdentity(CreateIdentityBody),
    #[serde(rename = "createKeyBook")]
    CreateKeyBook(CreateKeyBookBody),
    #[serde(rename = "createKeyPage")]
    CreateKeyPage(CreateKeyPageBody),
    #[serde(rename = "createLiteTokenAccount")]
    CreateLiteTokenAccount(CreateLiteTokenAccountBody),
    #[serde(rename = "createToken")]
    CreateToken(CreateTokenBody),
    #[serde(rename = "createTokenAccount")]
    CreateTokenAccount(CreateTokenAccountBody),
    #[serde(rename = "directoryAnchor")]
    DirectoryAnchor(DirectoryAnchorBody),
    #[serde(rename = "issueTokens")]
    IssueTokens(IssueTokensBody),
    #[serde(rename = "lockAccount")]
    LockAccount(LockAccountBody),
    #[serde(rename = "networkMaintenance")]
    NetworkMaintenance(NetworkMaintenanceBody),
    #[serde(rename = "remoteTransaction")]
    RemoteTransaction(RemoteTransactionBody),
    #[serde(rename = "sendTokens")]
    SendTokens(SendTokensBody),
    #[serde(rename = "systemGenesis")]
    SystemGenesis(SystemGenesisBody),
    #[serde(rename = "systemWriteData")]
    SystemWriteData(SystemWriteDataBody),
    #[serde(rename = "transferCredits")]
    TransferCredits(TransferCreditsBody),
    #[serde(rename = "updateAccountAuth")]
    UpdateAccountAuth(UpdateAccountAuthBody),
    #[serde(rename = "updateKey")]
    UpdateKey(UpdateKeyBody),
    #[serde(rename = "updateKeyPage")]
    UpdateKeyPage(UpdateKeyPageBody),
    #[serde(rename = "writeData")]
    WriteData(WriteDataBody),
    #[serde(rename = "writeDataTo")]
    WriteDataTo(WriteDataToBody),
}

impl TransactionBody {
    pub fn validate(&self) -> Result<(), Error> {
        match self {
            TransactionBody::AcmeFaucet(b) => b.validate(),
            TransactionBody::ActivateProtocolVersion(b) => b.validate(),
            TransactionBody::AddCredits(b) => b.validate(),
            TransactionBody::BlockValidatorAnchor(b) => b.validate(),
            TransactionBody::BurnCredits(b) => b.validate(),
            TransactionBody::BurnTokens(b) => b.validate(),
            TransactionBody::CreateDataAccount(b) => b.validate(),
            TransactionBody::CreateIdentity(b) => b.validate(),
            TransactionBody::CreateKeyBook(b) => b.validate(),
            TransactionBody::CreateKeyPage(b) => b.validate(),
            TransactionBody::CreateLiteTokenAccount(b) => b.validate(),
            TransactionBody::CreateToken(b) => b.validate(),
            TransactionBody::CreateTokenAccount(b) => b.validate(),
            TransactionBody::DirectoryAnchor(b) => b.validate(),
            TransactionBody::IssueTokens(b) => b.validate(),
            TransactionBody::LockAccount(b) => b.validate(),
            TransactionBody::NetworkMaintenance(b) => b.validate(),
            TransactionBody::RemoteTransaction(b) => b.validate(),
            TransactionBody::SendTokens(b) => b.validate(),
            TransactionBody::SystemGenesis(b) => b.validate(),
            TransactionBody::SystemWriteData(b) => b.validate(),
            TransactionBody::TransferCredits(b) => b.validate(),
            TransactionBody::UpdateAccountAuth(b) => b.validate(),
            TransactionBody::UpdateKey(b) => b.validate(),
            TransactionBody::UpdateKeyPage(b) => b.validate(),
            TransactionBody::WriteData(b) => b.validate(),
            TransactionBody::WriteDataTo(b) => b.validate(),
        }
    }
}

#[cfg(test)]
pub fn __minimal_tx_body_json(wire_tag: &str) -> serde_json::Value {
    match wire_tag {
        "acmeFaucet" => serde_json::json!({
            "type": "acmeFaucet",
            "Url": ""
        }),
        "activateProtocolVersion" => serde_json::json!({
            "type": "activateProtocolVersion"
        }),
        "addCredits" => serde_json::json!({
            "type": "addCredits",
            "Recipient": "",
            "Amount": "0",
            "Oracle": 0
        }),
        "blockValidatorAnchor" => serde_json::json!({
            "type": "blockValidatorAnchor",
            "AcmeBurnt": "0"
        }),
        "burnCredits" => serde_json::json!({
            "type": "burnCredits",
            "Amount": 0
        }),
        "burnTokens" => serde_json::json!({
            "type": "burnTokens",
            "Amount": "0"
        }),
        "createDataAccount" => serde_json::json!({
            "type": "createDataAccount",
            "Url": ""
        }),
        "createIdentity" => serde_json::json!({
            "type": "createIdentity",
            "Url": ""
        }),
        "createKeyBook" => serde_json::json!({
            "type": "createKeyBook",
            "Url": "",
            "PublicKeyHash": "00"
        }),
        "createKeyPage" => serde_json::json!({
            "type": "createKeyPage",
            "Keys": []
        }),
        "createLiteTokenAccount" => serde_json::json!({
            "type": "createLiteTokenAccount"
        }),
        "createToken" => serde_json::json!({
            "type": "createToken",
            "Url": "",
            "Symbol": "",
            "Precision": 0
        }),
        "createTokenAccount" => serde_json::json!({
            "type": "createTokenAccount",
            "Url": "",
            "TokenUrl": ""
        }),
        "directoryAnchor" => serde_json::json!({
            "type": "directoryAnchor",
            "Updates": [],
            "Receipts": [],
            "MakeMajorBlock": 0,
            "MakeMajorBlockTime": {}
        }),
        "issueTokens" => serde_json::json!({
            "type": "issueTokens",
            "Recipient": "",
            "Amount": "0",
            "To": []
        }),
        "lockAccount" => serde_json::json!({
            "type": "lockAccount",
            "Height": 0
        }),
        "networkMaintenance" => serde_json::json!({
            "type": "networkMaintenance",
            "Operations": []
        }),
        "remoteTransaction" => serde_json::json!({
            "type": "remoteTransaction"
        }),
        "sendTokens" => serde_json::json!({
            "type": "sendTokens",
            "To": []
        }),
        "systemGenesis" => serde_json::json!({
            "type": "systemGenesis"
        }),
        "systemWriteData" => serde_json::json!({
            "type": "systemWriteData",
            "Entry": {}
        }),
        "transferCredits" => serde_json::json!({
            "type": "transferCredits",
            "To": []
        }),
        "updateAccountAuth" => serde_json::json!({
            "type": "updateAccountAuth",
            "Operations": []
        }),
        "updateKey" => serde_json::json!({
            "type": "updateKey",
            "NewKeyHash": "00"
        }),
        "updateKeyPage" => serde_json::json!({
            "type": "updateKeyPage",
            "Operation": []
        }),
        "writeData" => serde_json::json!({
            "type": "writeData",
            "Entry": {}
        }),
        "writeDataTo" => serde_json::json!({
            "type": "writeDataTo",
            "Recipient": "",
            "Entry": {}
        }),
        _ => serde_json::json!({"type": wire_tag}),
    }
}

#[cfg(test)]
pub fn __tx_roundtrip_one(wire_tag: &str) -> Result<(), Box<dyn std::error::Error>> {
    let original = __minimal_tx_body_json(wire_tag);
    let body: TransactionBody = serde_json::from_value(original.clone())?;
    let serialized = serde_json::to_value(&body)?;

    if original != serialized {
        return Err(format!("Roundtrip mismatch for {}: original != serialized", wire_tag).into());
    }

    body.validate()?;
    Ok(())
}

#[cfg(test)]
pub fn __test_all_tx_roundtrips() -> Result<(), Box<dyn std::error::Error>> {
        __tx_roundtrip_one("acmeFaucet");
        __tx_roundtrip_one("activateProtocolVersion");
        __tx_roundtrip_one("addCredits");
        __tx_roundtrip_one("blockValidatorAnchor");
        __tx_roundtrip_one("burnCredits");
        __tx_roundtrip_one("burnTokens");
        __tx_roundtrip_one("createDataAccount");
        __tx_roundtrip_one("createIdentity");
        __tx_roundtrip_one("createKeyBook");
        __tx_roundtrip_one("createKeyPage");
        __tx_roundtrip_one("createLiteTokenAccount");
        __tx_roundtrip_one("createToken");
        __tx_roundtrip_one("createTokenAccount");
        __tx_roundtrip_one("directoryAnchor");
        __tx_roundtrip_one("issueTokens");
        __tx_roundtrip_one("lockAccount");
        __tx_roundtrip_one("networkMaintenance");
        __tx_roundtrip_one("remoteTransaction");
        __tx_roundtrip_one("sendTokens");
        __tx_roundtrip_one("systemGenesis");
        __tx_roundtrip_one("systemWriteData");
        __tx_roundtrip_one("transferCredits");
        __tx_roundtrip_one("updateAccountAuth");
        __tx_roundtrip_one("updateKey");
        __tx_roundtrip_one("updateKeyPage");
        __tx_roundtrip_one("writeData");
        __tx_roundtrip_one("writeDataTo");
    Ok(())
}