// GENERATED FILE - DO NOT EDIT
// Source: protocol/enums.yml | Generated: 2025-10-03 19:49:45

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AccountAuthOperationType {
    #[serde(rename = "unknown")]
    Unknown,
    #[serde(rename = "enable")]
    Enable,
    #[serde(rename = "disable")]
    Disable,
    #[serde(rename = "addauthority")]
    AddAuthority,
    #[serde(rename = "removeauthority")]
    RemoveAuthority,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AccountType {
    #[serde(rename = "unknown")]
    Unknown,
    #[serde(rename = "anchorLedger", alias = "anchorledger")]
    AnchorLedger,
    #[serde(rename = "identity")]
    Identity,
    #[serde(rename = "tokenIssuer")]
    TokenIssuer,
    #[serde(rename = "tokenAccount", alias = "tokenaccount")]
    TokenAccount,
    #[serde(rename = "liteTokenAccount", alias = "litetokenaccount")]
    LiteTokenAccount,
    #[serde(rename = "blockLedger")]
    BlockLedger,
    #[serde(rename = "keyPage")]
    KeyPage,
    #[serde(rename = "keyBook")]
    KeyBook,
    #[serde(rename = "dataAccount", alias = "dataaccount")]
    DataAccount,
    #[serde(rename = "liteDataAccount")]
    LiteDataAccount,
    #[serde(rename = "unknownSigner")]
    UnknownSigner,
    #[serde(rename = "systemLedger")]
    SystemLedger,
    #[serde(rename = "liteIdentity")]
    LiteIdentity,
    #[serde(rename = "syntheticLedger")]
    SyntheticLedger,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AllowedTransactionBit {
    #[serde(rename = "updatekeypage")]
    UpdateKeyPage,
    #[serde(rename = "updateaccountauth")]
    UpdateAccountAuth,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BookType {
    #[serde(rename = "normal")]
    Normal,
    #[serde(rename = "validator")]
    Validator,
    #[serde(rename = "operator")]
    Operator,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataEntryType {
    #[serde(rename = "unknown")]
    Unknown,
    #[serde(rename = "factom")]
    Factom,
    #[serde(rename = "accumulate")]
    Accumulate,
    #[serde(rename = "doublehash")]
    DoubleHash,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExecutorVersion {
    #[serde(rename = "v1")]
    V1,
    #[serde(rename = "v1SignatureAnchoring", alias = "v1-signatureAnchoring")]
    V1SignatureAnchoring,
    #[serde(rename = "v1DoubleHashEntries", alias = "v1-doubleHashEntries")]
    V1DoubleHashEntries,
    #[serde(rename = "v1Halt", alias = "v1-halt")]
    V1Halt,
    #[serde(rename = "v2")]
    V2,
    #[serde(rename = "v2Baikonur", alias = "v2-baikonur")]
    V2Baikonur,
    #[serde(rename = "v2Vandenberg", alias = "v2-vandenberg")]
    V2Vandenberg,
    #[serde(rename = "v2Jiuquan", alias = "v2-jiuquan")]
    V2Jiuquan,
    #[serde(rename = "vNext", alias = "vnext")]
    VNext,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyPageOperationType {
    #[serde(rename = "unknown")]
    Unknown,
    #[serde(rename = "update")]
    Update,
    #[serde(rename = "remove")]
    Remove,
    #[serde(rename = "add")]
    Add,
    #[serde(rename = "setthreshold")]
    SetThreshold,
    #[serde(rename = "updateallowed")]
    UpdateAllowed,
    #[serde(rename = "setrejectthreshold")]
    SetRejectThreshold,
    #[serde(rename = "setresponsethreshold")]
    SetResponseThreshold,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NetworkMaintenanceOperationType {
    #[serde(rename = "unknown")]
    Unknown,
    #[serde(rename = "pendingtransactiongc")]
    PendingTransactionGC,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ObjectType {
    #[serde(rename = "unknown")]
    Unknown,
    #[serde(rename = "account")]
    Account,
    #[serde(rename = "transaction")]
    Transaction,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PartitionType {
    #[serde(rename = "directory")]
    Directory,
    #[serde(rename = "blockValidator", alias = "block-validator")]
    BlockValidator,
    #[serde(rename = "blockSummary", alias = "block-summary")]
    BlockSummary,
    #[serde(rename = "bootstrap")]
    Bootstrap,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SignatureType {
    #[serde(rename = "unknown")]
    Unknown,
    #[serde(rename = "legacyED25519")]
    LegacyED25519,
    #[serde(rename = "ed25519")]
    ED25519,
    #[serde(rename = "rcd1")]
    RCD1,
    #[serde(rename = "receipt")]
    Receipt,
    #[serde(rename = "synthetic")]
    Partition,
    #[serde(rename = "set")]
    Set,
    #[serde(rename = "remote")]
    Remote,
    #[serde(rename = "btc")]
    BTC,
    #[serde(rename = "btclegacy")]
    BTCLegacy,
    #[serde(rename = "eth")]
    ETH,
    #[serde(rename = "delegated")]
    Delegated,
    #[serde(rename = "internal")]
    Internal,
    #[serde(rename = "authority")]
    Authority,
    #[serde(rename = "rsasha256")]
    RsaSha256,
    #[serde(rename = "ecdsasha256")]
    EcdsaSha256,
    #[serde(rename = "typeddata")]
    TypedData,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransactionMax {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "synthetic")]
    Synthetic,
    #[serde(rename = "system")]
    System,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransactionType {
    #[serde(rename = "unknown")]
    Unknown,
    #[serde(rename = "createIdentity")]
    CreateIdentity,
    #[serde(rename = "createTokenAccount")]
    CreateTokenAccount,
    #[serde(rename = "sendTokens")]
    SendTokens,
    #[serde(rename = "createDataAccount")]
    CreateDataAccount,
    #[serde(rename = "writeData")]
    WriteData,
    #[serde(rename = "writeDataTo")]
    WriteDataTo,
    #[serde(rename = "acmeFaucet")]
    AcmeFaucet,
    #[serde(rename = "createToken")]
    CreateToken,
    #[serde(rename = "issueTokens")]
    IssueTokens,
    #[serde(rename = "burnTokens")]
    BurnTokens,
    #[serde(rename = "createLiteTokenAccount")]
    CreateLiteTokenAccount,
    #[serde(rename = "createKeyPage")]
    CreateKeyPage,
    #[serde(rename = "createKeyBook")]
    CreateKeyBook,
    #[serde(rename = "addCredits")]
    AddCredits,
    #[serde(rename = "updateKeyPage")]
    UpdateKeyPage,
    #[serde(rename = "lockAccount")]
    LockAccount,
    #[serde(rename = "burnCredits")]
    BurnCredits,
    #[serde(rename = "transferCredits")]
    TransferCredits,
    #[serde(rename = "updateAccountAuth")]
    UpdateAccountAuth,
    #[serde(rename = "updateKey")]
    UpdateKey,
    #[serde(rename = "networkMaintenance")]
    NetworkMaintenance,
    #[serde(rename = "activateProtocolVersion")]
    ActivateProtocolVersion,
    #[serde(rename = "signPending")]
    Remote,
    #[serde(rename = "syntheticCreateIdentity")]
    SyntheticCreateIdentity,
    #[serde(rename = "syntheticWriteData")]
    SyntheticWriteData,
    #[serde(rename = "syntheticDepositTokens")]
    SyntheticDepositTokens,
    #[serde(rename = "syntheticDepositCredits")]
    SyntheticDepositCredits,
    #[serde(rename = "syntheticBurnTokens")]
    SyntheticBurnTokens,
    #[serde(rename = "syntheticForwardTransaction")]
    SyntheticForwardTransaction,
    #[serde(rename = "systemGenesis")]
    SystemGenesis,
    #[serde(rename = "directoryAnchor")]
    DirectoryAnchor,
    #[serde(rename = "blockValidatorAnchor")]
    BlockValidatorAnchor,
    #[serde(rename = "systemWriteData")]
    SystemWriteData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VoteType {
    #[serde(rename = "accept")]
    Accept,
    #[serde(rename = "reject")]
    Reject,
    #[serde(rename = "abstain")]
    Abstain,
    #[serde(rename = "suggest")]
    Suggest,
}

impl VoteType {
    /// Get the numeric value of the vote type (matches Go protocol)
    pub fn value(&self) -> u64 {
        match self {
            VoteType::Accept => 0,
            VoteType::Reject => 1,
            VoteType::Abstain => 2,
            VoteType::Suggest => 3,
        }
    }

    /// Create a VoteType from its numeric value
    pub fn from_u64(value: u64) -> Option<Self> {
        match value {
            0 => Some(VoteType::Accept),
            1 => Some(VoteType::Reject),
            2 => Some(VoteType::Abstain),
            3 => Some(VoteType::Suggest),
            _ => None,
        }
    }

    /// Get all possible vote types
    pub fn all() -> &'static [VoteType] {
        &[VoteType::Accept, VoteType::Reject, VoteType::Abstain, VoteType::Suggest]
    }

    /// Check if this vote type approves (accepts) the proposal
    pub fn is_approval(&self) -> bool {
        matches!(self, VoteType::Accept)
    }

    /// Check if this vote type rejects the proposal
    pub fn is_rejection(&self) -> bool {
        matches!(self, VoteType::Reject)
    }

    /// Check if this vote type is an abstention
    pub fn is_abstention(&self) -> bool {
        matches!(self, VoteType::Abstain)
    }

    /// Check if this vote type is a suggestion (proposal)
    pub fn is_suggestion(&self) -> bool {
        matches!(self, VoteType::Suggest)
    }

    /// Get string representation matching Go protocol
    pub fn as_str(&self) -> &'static str {
        match self {
            VoteType::Accept => "accept",
            VoteType::Reject => "reject",
            VoteType::Abstain => "abstain",
            VoteType::Suggest => "suggest",
        }
    }

    /// Parse from string (case-insensitive)
    pub fn from_str_case_insensitive(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "accept" => Some(VoteType::Accept),
            "reject" => Some(VoteType::Reject),
            "abstain" => Some(VoteType::Abstain),
            "suggest" => Some(VoteType::Suggest),
            _ => None,
        }
    }
}

impl Default for VoteType {
    /// Default vote type is Accept (matches Go protocol)
    fn default() -> Self {
        VoteType::Accept
    }
}

impl std::fmt::Display for VoteType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Vote tallying result for counting signature votes
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VoteTally {
    /// Number of accept votes
    pub accept: u64,
    /// Number of reject votes
    pub reject: u64,
    /// Number of abstain votes
    pub abstain: u64,
    /// Number of suggest votes
    pub suggest: u64,
}

impl VoteTally {
    /// Create an empty tally
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a vote to the tally
    pub fn add_vote(&mut self, vote: VoteType) {
        match vote {
            VoteType::Accept => self.accept += 1,
            VoteType::Reject => self.reject += 1,
            VoteType::Abstain => self.abstain += 1,
            VoteType::Suggest => self.suggest += 1,
        }
    }

    /// Add multiple votes of the same type
    pub fn add_votes(&mut self, vote: VoteType, count: u64) {
        match vote {
            VoteType::Accept => self.accept += count,
            VoteType::Reject => self.reject += count,
            VoteType::Abstain => self.abstain += count,
            VoteType::Suggest => self.suggest += count,
        }
    }

    /// Get the total number of votes (excluding abstentions)
    pub fn total_active(&self) -> u64 {
        self.accept + self.reject + self.suggest
    }

    /// Get the total number of all votes (including abstentions)
    pub fn total(&self) -> u64 {
        self.accept + self.reject + self.abstain + self.suggest
    }

    /// Check if the vote passes with simple majority (more accepts than rejects)
    pub fn passes_simple_majority(&self) -> bool {
        self.accept > self.reject
    }

    /// Check if the vote passes with strict majority (>50% accepts of active votes)
    pub fn passes_strict_majority(&self) -> bool {
        let total_active = self.total_active();
        if total_active == 0 {
            return false;
        }
        self.accept > total_active / 2
    }

    /// Check if the vote passes with supermajority (>=2/3 accepts of active votes)
    pub fn passes_supermajority(&self) -> bool {
        let total_active = self.total_active();
        if total_active == 0 {
            return false;
        }
        // Using >= 2/3 threshold: accept * 3 >= total * 2
        self.accept * 3 >= total_active * 2
    }

    /// Check if the vote passes with given threshold (accepts as percentage of active votes)
    pub fn passes_threshold(&self, threshold_percent: u64) -> bool {
        if threshold_percent > 100 {
            return false;
        }
        let total_active = self.total_active();
        if total_active == 0 {
            return threshold_percent == 0;
        }
        // accept * 100 >= total * threshold
        self.accept * 100 >= total_active * threshold_percent
    }

    /// Get the acceptance percentage (of active votes)
    pub fn acceptance_percentage(&self) -> f64 {
        let total_active = self.total_active();
        if total_active == 0 {
            return 0.0;
        }
        (self.accept as f64 / total_active as f64) * 100.0
    }

    /// Merge another tally into this one
    pub fn merge(&mut self, other: &VoteTally) {
        self.accept += other.accept;
        self.reject += other.reject;
        self.abstain += other.abstain;
        self.suggest += other.suggest;
    }
}


pub fn __roundtrip_one(enum_name: &str, tag: &str) -> Result<(), Box<dyn std::error::Error>> {
    match enum_name {
        "AccountAuthOperationType" => {
            let v = serde_json::Value::String(tag.to_string());
            let val: AccountAuthOperationType = serde_json::from_value(v.clone())?;
            let back = serde_json::to_value(&val)?;
            if back != v {
                return Err(format!("Roundtrip failed for {}::{}: expected {}, got {}",
                    enum_name, tag, v, back).into());
            }
        }
        "AccountType" => {
            let v = serde_json::Value::String(tag.to_string());
            let val: AccountType = serde_json::from_value(v.clone())?;
            let back = serde_json::to_value(&val)?;
            if back != v {
                return Err(format!("Roundtrip failed for {}::{}: expected {}, got {}",
                    enum_name, tag, v, back).into());
            }
        }
        "AllowedTransactionBit" => {
            let v = serde_json::Value::String(tag.to_string());
            let val: AllowedTransactionBit = serde_json::from_value(v.clone())?;
            let back = serde_json::to_value(&val)?;
            if back != v {
                return Err(format!("Roundtrip failed for {}::{}: expected {}, got {}",
                    enum_name, tag, v, back).into());
            }
        }
        "BookType" => {
            let v = serde_json::Value::String(tag.to_string());
            let val: BookType = serde_json::from_value(v.clone())?;
            let back = serde_json::to_value(&val)?;
            if back != v {
                return Err(format!("Roundtrip failed for {}::{}: expected {}, got {}",
                    enum_name, tag, v, back).into());
            }
        }
        "DataEntryType" => {
            let v = serde_json::Value::String(tag.to_string());
            let val: DataEntryType = serde_json::from_value(v.clone())?;
            let back = serde_json::to_value(&val)?;
            if back != v {
                return Err(format!("Roundtrip failed for {}::{}: expected {}, got {}",
                    enum_name, tag, v, back).into());
            }
        }
        "ExecutorVersion" => {
            let v = serde_json::Value::String(tag.to_string());
            let val: ExecutorVersion = serde_json::from_value(v.clone())?;
            let back = serde_json::to_value(&val)?;
            if back != v {
                return Err(format!("Roundtrip failed for {}::{}: expected {}, got {}",
                    enum_name, tag, v, back).into());
            }
        }
        "KeyPageOperationType" => {
            let v = serde_json::Value::String(tag.to_string());
            let val: KeyPageOperationType = serde_json::from_value(v.clone())?;
            let back = serde_json::to_value(&val)?;
            if back != v {
                return Err(format!("Roundtrip failed for {}::{}: expected {}, got {}",
                    enum_name, tag, v, back).into());
            }
        }
        "NetworkMaintenanceOperationType" => {
            let v = serde_json::Value::String(tag.to_string());
            let val: NetworkMaintenanceOperationType = serde_json::from_value(v.clone())?;
            let back = serde_json::to_value(&val)?;
            if back != v {
                return Err(format!("Roundtrip failed for {}::{}: expected {}, got {}",
                    enum_name, tag, v, back).into());
            }
        }
        "ObjectType" => {
            let v = serde_json::Value::String(tag.to_string());
            let val: ObjectType = serde_json::from_value(v.clone())?;
            let back = serde_json::to_value(&val)?;
            if back != v {
                return Err(format!("Roundtrip failed for {}::{}: expected {}, got {}",
                    enum_name, tag, v, back).into());
            }
        }
        "PartitionType" => {
            let v = serde_json::Value::String(tag.to_string());
            let val: PartitionType = serde_json::from_value(v.clone())?;
            let back = serde_json::to_value(&val)?;
            if back != v {
                return Err(format!("Roundtrip failed for {}::{}: expected {}, got {}",
                    enum_name, tag, v, back).into());
            }
        }
        "SignatureType" => {
            let v = serde_json::Value::String(tag.to_string());
            let val: SignatureType = serde_json::from_value(v.clone())?;
            let back = serde_json::to_value(&val)?;
            if back != v {
                return Err(format!("Roundtrip failed for {}::{}: expected {}, got {}",
                    enum_name, tag, v, back).into());
            }
        }
        "TransactionMax" => {
            let v = serde_json::Value::String(tag.to_string());
            let val: TransactionMax = serde_json::from_value(v.clone())?;
            let back = serde_json::to_value(&val)?;
            if back != v {
                return Err(format!("Roundtrip failed for {}::{}: expected {}, got {}",
                    enum_name, tag, v, back).into());
            }
        }
        "TransactionType" => {
            let v = serde_json::Value::String(tag.to_string());
            let val: TransactionType = serde_json::from_value(v.clone())?;
            let back = serde_json::to_value(&val)?;
            if back != v {
                return Err(format!("Roundtrip failed for {}::{}: expected {}, got {}",
                    enum_name, tag, v, back).into());
            }
        }
        "VoteType" => {
            let v = serde_json::Value::String(tag.to_string());
            let val: VoteType = serde_json::from_value(v.clone())?;
            let back = serde_json::to_value(&val)?;
            if back != v {
                return Err(format!("Roundtrip failed for {}::{}: expected {}, got {}",
                    enum_name, tag, v, back).into());
            }
        }
        _ => return Err(format!("Unknown enum: {}", enum_name).into()),
    }
    Ok(())
}

pub fn __get_all_enum_variants() -> std::collections::HashMap<String, Vec<String>> {
    let mut map = std::collections::HashMap::new();
    map.insert("AccountAuthOperationType".to_string(), vec!["unknown".to_string(), "enable".to_string(), "disable".to_string(), "addauthority".to_string(), "removeauthority".to_string()]);
    map.insert("AccountType".to_string(), vec!["unknown".to_string(), "anchorledger".to_string(), "identity".to_string(), "token".to_string(), "tokenaccount".to_string(), "litetokenaccount".to_string(), "blockledger".to_string(), "keypage".to_string(), "keybook".to_string(), "dataaccount".to_string(), "litedataaccount".to_string(), "unknownsigner".to_string(), "systemledger".to_string(), "liteidentity".to_string(), "syntheticledger".to_string()]);
    map.insert("AllowedTransactionBit".to_string(), vec!["updatekeypage".to_string(), "updateaccountauth".to_string()]);
    map.insert("BookType".to_string(), vec!["normal".to_string(), "validator".to_string(), "operator".to_string()]);
    map.insert("DataEntryType".to_string(), vec!["unknown".to_string(), "factom".to_string(), "accumulate".to_string(), "doublehash".to_string()]);
    map.insert("ExecutorVersion".to_string(), vec!["v1".to_string(), "v1-signatureAnchoring".to_string(), "v1-doubleHashEntries".to_string(), "v1-halt".to_string(), "v2".to_string(), "v2-baikonur".to_string(), "v2-vandenberg".to_string(), "v2-jiuquan".to_string(), "vnext".to_string()]);
    map.insert("KeyPageOperationType".to_string(), vec!["unknown".to_string(), "update".to_string(), "remove".to_string(), "add".to_string(), "setthreshold".to_string(), "updateallowed".to_string(), "setrejectthreshold".to_string(), "setresponsethreshold".to_string()]);
    map.insert("NetworkMaintenanceOperationType".to_string(), vec!["unknown".to_string(), "pendingtransactiongc".to_string()]);
    map.insert("ObjectType".to_string(), vec!["unknown".to_string(), "account".to_string(), "transaction".to_string()]);
    map.insert("PartitionType".to_string(), vec!["directory".to_string(), "block-validator".to_string(), "block-summary".to_string(), "bootstrap".to_string()]);
    map.insert("SignatureType".to_string(), vec!["unknown".to_string(), "legacyed25519".to_string(), "ed25519".to_string(), "rcd1".to_string(), "receipt".to_string(), "synthetic".to_string(), "set".to_string(), "remote".to_string(), "btc".to_string(), "btclegacy".to_string(), "eth".to_string(), "delegated".to_string(), "internal".to_string(), "authority".to_string(), "rsasha256".to_string(), "ecdsasha256".to_string(), "typeddata".to_string()]);
    map.insert("TransactionMax".to_string(), vec!["user".to_string(), "synthetic".to_string(), "system".to_string()]);
    map.insert("TransactionType".to_string(), vec!["unknown".to_string(), "createIdentity".to_string(), "createTokenAccount".to_string(), "sendTokens".to_string(), "createDataAccount".to_string(), "writeData".to_string(), "writeDataTo".to_string(), "acmeFaucet".to_string(), "createToken".to_string(), "issueTokens".to_string(), "burnTokens".to_string(), "createLiteTokenAccount".to_string(), "createKeyPage".to_string(), "createKeyBook".to_string(), "addCredits".to_string(), "updateKeyPage".to_string(), "lockAccount".to_string(), "burnCredits".to_string(), "transferCredits".to_string(), "updateAccountAuth".to_string(), "updateKey".to_string(), "networkMaintenance".to_string(), "activateProtocolVersion".to_string(), "signPending".to_string(), "syntheticCreateIdentity".to_string(), "syntheticWriteData".to_string(), "syntheticDepositTokens".to_string(), "syntheticDepositCredits".to_string(), "syntheticBurnTokens".to_string(), "syntheticForwardTransaction".to_string(), "systemGenesis".to_string(), "directoryAnchor".to_string(), "blockValidatorAnchor".to_string(), "systemWriteData".to_string()]);
    map.insert("VoteType".to_string(), vec!["accept".to_string(), "reject".to_string(), "abstain".to_string(), "suggest".to_string()]);
    map
}