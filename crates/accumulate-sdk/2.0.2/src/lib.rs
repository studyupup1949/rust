//! Accumulate Rust SDK (V2/V3 unified) with DevNet-first flows
//!
//! This crate provides a unified client for interacting with Accumulate blockchain
//! networks, supporting both V2 and V3 protocol versions.

// Import the main V2/V3 client as the primary AccumulateClient
pub use crate::client::AccumulateClient;

/// Type alias for convenient access to AccumulateClient methods
pub type Accumulate = AccumulateClient;
pub use crate::codec::{
    TransactionCodec, TransactionEnvelope, TransactionSignature,
    TransactionBodyBuilder, TokenRecipient, KeySpec, BinaryReader, BinaryWriter,
    AccumulateHash, UrlHash, canonical_json, sha256_bytes, to_canonical_string
};
pub use crate::canonjson::{dumps_canonical, canonicalize};
pub use crate::crypto::ed25519::{Ed25519Signer, verify, verify_prehashed, verify_signature, sha256};
pub use crate::crypto::ed25519_helper::Ed25519Helper;
pub use crate::codec::hash_helper::HashHelper;
pub use crate::protocol::{EnvelopeBuilder, helpers as protocol_helpers};
pub use crate::generated::enums::{
    AccountAuthOperationType, AccountType, AllowedTransactionBit, BookType,
    DataEntryType, ExecutorVersion, KeyPageOperationType, NetworkMaintenanceOperationType,
    ObjectType, PartitionType, SignatureType, TransactionMax, TransactionType,
    VoteType, VoteTally,
};
pub use crate::generated::signatures::*;
pub use crate::generated::header::*;
pub use crate::generated::transactions::*;
// Import protocol types from generated module
pub use crate::generated::types as protocol_types;
// Import what's actually available from api_methods
pub use crate::generated::api_methods::{
    AccumulateRpc,
    // Parameter types used in tests
    StatusParams, QueryParams, ExecuteParams, QueryDirectoryParams, QueryTxParams,
    QueryTxLocalParams, ExecuteCreateIdentityParams, ExecuteSendTokensParams,
    FaucetParams, VersionParams, DescribeParams,
    // Response types used in tests
    StatusResponse, QueryResponse, ExecuteResponse, QueryDirectoryResponse, QueryTxResponse,
    QueryTxLocalResponse, ExecuteCreateIdentityResponse, ExecuteSendTokensResponse
};
// Export the generic client wrapper as a different name to avoid conflicts
pub use crate::generated::api_methods::AccumulateClient as GenericAccumulateClient;
pub use crate::runtime::signing::*;
pub use crate::runtime::rpc::*;
#[cfg(test)]
pub use crate::runtime::signing_test_shims;

/// Canonical JSON encoding utilities
pub mod canonjson;
/// Main Accumulate client implementation
pub mod client;
/// Binary encoding/decoding utilities
pub mod codec;
/// Cryptographic utilities (Ed25519, hashing)
pub mod crypto;
/// Error types and handling
pub mod errors;
/// Auto-generated protocol types
pub mod generated;
/// Helper utilities (SmartSigner, TxBody, KeyManager, QuickStart)
pub mod helpers;
/// JSON-RPC client implementation
pub mod json_rpc_client;
/// Protocol envelope and transaction building
pub mod protocol;
/// Runtime utilities (RPC, signing)
pub mod runtime;
/// V3 API type definitions
pub mod types;
/// Type matrix for testing
pub mod types_matrix;

// Re-export helper utilities for convenience
pub use crate::helpers::{
    // Constants
    KERMIT_V2, KERMIT_V3, DEVNET_V2, DEVNET_V3,
    // Transaction builders
    TxBody, TxResult,
    // Smart signing
    SmartSigner, KeyManager, KeyPageState, KeyEntry,
    // QuickStart API
    QuickStart, Wallet, AdiInfo, KeyPageInfo,
    // Polling utilities
    poll_for_balance, poll_for_credits, wait_for_tx,
    // URL derivation
    derive_lite_identity_url, derive_lite_token_account_url, sha256_hash,
};

// Re-export V3 API types for convenience
pub use crate::types::{
    // V3 Service Types
    V3NodeInfo, ServiceAddress, NodeInfoOptions,
    V3ConsensusStatus, LastBlock, ConsensusPeerInfo, ConsensusStatusOptions,
    V3NetworkStatus, AcmeOracle, PartitionExecutorVersion, NetworkStatusOptions,
    V3Metrics, MetricsOptions,
    V3Submission, SubmitOptions, ValidateOptions,
    V3FaucetOptions, V3SnapshotInfo, ListSnapshotsOptions,
    FindServiceOptions, FindServiceResult, SubscribeOptions,
    // V3 Query Types
    RangeOptions, ReceiptOptions, DefaultQuery,
    ChainQuery, DataQuery, DirectoryQuery, PendingQuery, BlockQuery,
    AnchorSearchQuery, PublicKeySearchQuery, PublicKeyHashSearchQuery,
    DelegateSearchQuery, MessageHashSearchQuery, V3Query,
};

use anyhow::Result;
use std::time::Duration;
use url::Url;

/// Configuration options for the Accumulate client
#[derive(Debug, Clone)]
pub struct AccOptions {
    /// Request timeout duration
    pub timeout: Duration,
    /// Default headers to include with requests
    pub headers: std::collections::HashMap<String, String>,
}

impl Default for AccOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            headers: std::collections::HashMap::new(),
        }
    }
}

impl AccumulateClient {
    /// Create a new client from explicit V2 and V3 endpoints
    pub async fn from_endpoints(v2: Url, v3: Url, opts: AccOptions) -> Result<Self> {
        Self::new_with_options(v2, v3, opts)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Create a new client from environment variables
    ///
    /// Reads the following environment variables:
    /// - `ACCUMULATE_V2_URL`: V2 endpoint URL
    /// - `ACCUMULATE_V3_URL`: V3 endpoint URL
    /// - `ACCUMULATE_TIMEOUT_MS`: Request timeout in milliseconds (optional, defaults to 30000)
    pub async fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok(); // Load .env file if present, ignore errors

        let v2_url = std::env::var("ACCUMULATE_V2_URL")
            .map_err(|_| anyhow::anyhow!("ACCUMULATE_V2_URL environment variable not set"))?;
        let v3_url = std::env::var("ACCUMULATE_V3_URL")
            .map_err(|_| anyhow::anyhow!("ACCUMULATE_V3_URL environment variable not set"))?;

        let v2 = Url::parse(&v2_url)?;
        let v3 = Url::parse(&v3_url)?;

        let timeout_ms = std::env::var("ACCUMULATE_TIMEOUT_MS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(30_000);

        let opts = AccOptions {
            timeout: Duration::from_millis(timeout_ms),
            ..Default::default()
        };

        Self::from_endpoints(v2, v3, opts).await
    }

    /// Create a client configured for DevNet
    pub async fn devnet(opts: AccOptions) -> Result<Self> {
        let v2 = Url::parse("http://localhost:26660/v2")?;
        let v3 = Url::parse("http://localhost:26661/v3")?;
        Self::from_endpoints(v2, v3, opts).await
    }

    /// Create a client configured for TestNet
    pub async fn testnet(opts: AccOptions) -> Result<Self> {
        let v2 = Url::parse("https://testnet.accumulatenetwork.io/v2")?;
        let v3 = Url::parse("https://testnet.accumulatenetwork.io/v3")?;
        Self::from_endpoints(v2, v3, opts).await
    }

    /// Create a client configured for MainNet
    pub async fn mainnet(opts: AccOptions) -> Result<Self> {
        let v2 = Url::parse("https://mainnet.accumulatenetwork.io/v2")?;
        let v3 = Url::parse("https://mainnet.accumulatenetwork.io/v3")?;
        Self::from_endpoints(v2, v3, opts).await
    }

    /// Create a client configured for a custom endpoint
    pub async fn custom(base_url: &str, opts: AccOptions) -> Result<Self> {
        let base = Url::parse(base_url)?;
        let v2 = base.join("/v2")?;
        let v3 = base.join("/v3")?;
        Self::from_endpoints(v2, v3, opts).await
    }
}
