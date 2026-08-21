//! GENERATED FILE - DO NOT EDIT
//! Source: internal/api/v2/methods.yml
//! Generated: 2025-10-04 03:26:52

#![allow(missing_docs)]

use serde::{Serialize, Deserialize};
use crate::errors::Error;
use async_trait::async_trait;

// AccumulateRpc trait for transport abstraction
#[async_trait]
pub trait AccumulateRpc {
    async fn rpc_call<TParams: Serialize + Send + Sync, TResult: for<'de> Deserialize<'de>>(
        &self, method: &str, params: &TParams
    ) -> Result<TResult, Error>;
}

// Generic client wrapper
#[derive(Debug, Clone)]
pub struct AccumulateClient<C> {
    pub transport: C,
}

impl<C> AccumulateClient<C> {
    pub fn new(transport: C) -> Self {
        Self { transport }
    }
}

// Parameter structures
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusParams {
    // No parameters
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionParams {
    // No parameters
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DescribeParams {
    // No parameters
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricsParams {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub options: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FaucetParams {
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryParams {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub options: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryDirectoryParams {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub options: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryTxParams {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub options: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryTxLocalParams {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub options: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryTxHistoryParams {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub options: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryDataParams {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub options: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryDataSetParams {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub options: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryKeyPageIndexParams {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub options: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryMinorBlocksParams {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub options: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryMajorBlocksParams {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub options: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuerySynthParams {
    #[serde(flatten)]
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteParams {
    #[serde(flatten)]
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteDirectParams {
    #[serde(flatten)]
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteLocalParams {
    #[serde(flatten)]
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteCreateAdiParams {
    #[serde(flatten)]
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteCreateIdentityParams {
    #[serde(flatten)]
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteCreateDataAccountParams {
    #[serde(flatten)]
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteCreateKeyBookParams {
    #[serde(flatten)]
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteCreateKeyPageParams {
    #[serde(flatten)]
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteCreateTokenParams {
    #[serde(flatten)]
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteCreateTokenAccountParams {
    #[serde(flatten)]
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteSendTokensParams {
    #[serde(flatten)]
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteAddCreditsParams {
    #[serde(flatten)]
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteUpdateKeyPageParams {
    #[serde(flatten)]
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteUpdateKeyParams {
    #[serde(flatten)]
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteWriteDataParams {
    #[serde(flatten)]
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteIssueTokensParams {
    #[serde(flatten)]
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteWriteDataToParams {
    #[serde(flatten)]
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteBurnTokensParams {
    #[serde(flatten)]
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteUpdateAccountAuthParams {
    #[serde(flatten)]
    pub params: serde_json::Value,
}

// Result structures
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub last_block_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionResponse {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DescribeResponse {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricsResponse {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FaucetResponse {
    pub transaction_hash: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub simple_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub code: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryResponse {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryDirectoryResponse {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryTxResponse {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryTxLocalResponse {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryTxHistoryResponse {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryDataResponse {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryDataSetResponse {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryKeyPageIndexResponse {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryMinorBlocksResponse {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryMajorBlocksResponse {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuerySynthResponse {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteResponse {
    pub transaction_hash: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub simple_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub code: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteDirectResponse {
    pub transaction_hash: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub simple_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub code: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteLocalResponse {
    pub transaction_hash: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub simple_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub code: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteCreateAdiResponse {
    // No result data
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteCreateIdentityResponse {
    // No result data
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteCreateDataAccountResponse {
    // No result data
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteCreateKeyBookResponse {
    // No result data
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteCreateKeyPageResponse {
    // No result data
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteCreateTokenResponse {
    // No result data
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteCreateTokenAccountResponse {
    // No result data
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteSendTokensResponse {
    // No result data
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteAddCreditsResponse {
    // No result data
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteUpdateKeyPageResponse {
    // No result data
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteUpdateKeyResponse {
    // No result data
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteWriteDataResponse {
    // No result data
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteIssueTokensResponse {
    // No result data
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteWriteDataToResponse {
    // No result data
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteBurnTokensResponse {
    // No result data
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteUpdateAccountAuthResponse {
    // No result data
}

// Client implementation with strongly-typed methods
impl<C: AccumulateRpc + Send + Sync> AccumulateClient<C> {
    pub async fn status(&self, params: StatusParams) -> Result<StatusResponse, Error> {
        self.transport.rpc_call("status", &params).await
    }

    pub async fn version(&self, params: VersionParams) -> Result<VersionResponse, Error> {
        self.transport.rpc_call("version", &params).await
    }

    pub async fn describe(&self, params: DescribeParams) -> Result<DescribeResponse, Error> {
        self.transport.rpc_call("describe", &params).await
    }

    pub async fn metrics(&self, params: MetricsParams) -> Result<MetricsResponse, Error> {
        self.transport.rpc_call("metrics", &params).await
    }

    pub async fn faucet(&self, params: FaucetParams) -> Result<FaucetResponse, Error> {
        self.transport.rpc_call("faucet", &params).await
    }

    pub async fn query(&self, params: QueryParams) -> Result<QueryResponse, Error> {
        self.transport.rpc_call("query", &params).await
    }

    pub async fn query_directory(&self, params: QueryDirectoryParams) -> Result<QueryDirectoryResponse, Error> {
        self.transport.rpc_call("query-directory", &params).await
    }

    pub async fn query_tx(&self, params: QueryTxParams) -> Result<QueryTxResponse, Error> {
        self.transport.rpc_call("query-tx", &params).await
    }

    pub async fn query_tx_local(&self, params: QueryTxLocalParams) -> Result<QueryTxLocalResponse, Error> {
        self.transport.rpc_call("query-tx-local", &params).await
    }

    pub async fn query_tx_history(&self, params: QueryTxHistoryParams) -> Result<QueryTxHistoryResponse, Error> {
        self.transport.rpc_call("query-tx-history", &params).await
    }

    pub async fn query_data(&self, params: QueryDataParams) -> Result<QueryDataResponse, Error> {
        self.transport.rpc_call("query-data", &params).await
    }

    pub async fn query_data_set(&self, params: QueryDataSetParams) -> Result<QueryDataSetResponse, Error> {
        self.transport.rpc_call("query-data-set", &params).await
    }

    pub async fn query_key_page_index(&self, params: QueryKeyPageIndexParams) -> Result<QueryKeyPageIndexResponse, Error> {
        self.transport.rpc_call("query-key-index", &params).await
    }

    pub async fn query_minor_blocks(&self, params: QueryMinorBlocksParams) -> Result<QueryMinorBlocksResponse, Error> {
        self.transport.rpc_call("query-minor-blocks", &params).await
    }

    pub async fn query_major_blocks(&self, params: QueryMajorBlocksParams) -> Result<QueryMajorBlocksResponse, Error> {
        self.transport.rpc_call("query-major-blocks", &params).await
    }

    pub async fn query_synth(&self, params: QuerySynthParams) -> Result<QuerySynthResponse, Error> {
        self.transport.rpc_call("query-synth", &params).await
    }

    pub async fn execute(&self, params: ExecuteParams) -> Result<ExecuteResponse, Error> {
        self.transport.rpc_call("execute", &params).await
    }

    pub async fn execute_direct(&self, params: ExecuteDirectParams) -> Result<ExecuteDirectResponse, Error> {
        self.transport.rpc_call("execute-direct", &params).await
    }

    pub async fn execute_local(&self, params: ExecuteLocalParams) -> Result<ExecuteLocalResponse, Error> {
        self.transport.rpc_call("execute-local", &params).await
    }

    pub async fn execute_create_adi(&self, params: ExecuteCreateAdiParams) -> Result<ExecuteCreateAdiResponse, Error> {
        self.transport.rpc_call("create-adi", &params).await
    }

    pub async fn execute_create_identity(&self, params: ExecuteCreateIdentityParams) -> Result<ExecuteCreateIdentityResponse, Error> {
        self.transport.rpc_call("create-identity", &params).await
    }

    pub async fn execute_create_data_account(&self, params: ExecuteCreateDataAccountParams) -> Result<ExecuteCreateDataAccountResponse, Error> {
        self.transport.rpc_call("create-data-account", &params).await
    }

    pub async fn execute_create_key_book(&self, params: ExecuteCreateKeyBookParams) -> Result<ExecuteCreateKeyBookResponse, Error> {
        self.transport.rpc_call("create-key-book", &params).await
    }

    pub async fn execute_create_key_page(&self, params: ExecuteCreateKeyPageParams) -> Result<ExecuteCreateKeyPageResponse, Error> {
        self.transport.rpc_call("create-key-page", &params).await
    }

    pub async fn execute_create_token(&self, params: ExecuteCreateTokenParams) -> Result<ExecuteCreateTokenResponse, Error> {
        self.transport.rpc_call("create-token", &params).await
    }

    pub async fn execute_create_token_account(&self, params: ExecuteCreateTokenAccountParams) -> Result<ExecuteCreateTokenAccountResponse, Error> {
        self.transport.rpc_call("create-token-account", &params).await
    }

    pub async fn execute_send_tokens(&self, params: ExecuteSendTokensParams) -> Result<ExecuteSendTokensResponse, Error> {
        self.transport.rpc_call("send-tokens", &params).await
    }

    pub async fn execute_add_credits(&self, params: ExecuteAddCreditsParams) -> Result<ExecuteAddCreditsResponse, Error> {
        self.transport.rpc_call("add-credits", &params).await
    }

    pub async fn execute_update_key_page(&self, params: ExecuteUpdateKeyPageParams) -> Result<ExecuteUpdateKeyPageResponse, Error> {
        self.transport.rpc_call("update-key-page", &params).await
    }

    pub async fn execute_update_key(&self, params: ExecuteUpdateKeyParams) -> Result<ExecuteUpdateKeyResponse, Error> {
        self.transport.rpc_call("update-key", &params).await
    }

    pub async fn execute_write_data(&self, params: ExecuteWriteDataParams) -> Result<ExecuteWriteDataResponse, Error> {
        self.transport.rpc_call("write-data", &params).await
    }

    pub async fn execute_issue_tokens(&self, params: ExecuteIssueTokensParams) -> Result<ExecuteIssueTokensResponse, Error> {
        self.transport.rpc_call("issue-tokens", &params).await
    }

    pub async fn execute_write_data_to(&self, params: ExecuteWriteDataToParams) -> Result<ExecuteWriteDataToResponse, Error> {
        self.transport.rpc_call("write-data-to", &params).await
    }

    pub async fn execute_burn_tokens(&self, params: ExecuteBurnTokensParams) -> Result<ExecuteBurnTokensResponse, Error> {
        self.transport.rpc_call("burn-tokens", &params).await
    }

    pub async fn execute_update_account_auth(&self, params: ExecuteUpdateAccountAuthParams) -> Result<ExecuteUpdateAccountAuthResponse, Error> {
        self.transport.rpc_call("update-account-auth", &params).await
    }
}

pub fn __minimal_pair_for_test(method_name: &str) -> Option<(serde_json::Value, serde_json::Value)> {
    use serde_json::json;
    match method_name {
        "status" => Some((json!({}), json!({"ok": true}))),
        "version" => Some((json!({}), json!({"data": {}}))),
        "describe" => Some((json!({}), json!({"data": {}}))),
        "metrics" => Some((json!({"url": "acc://test.acme"}), json!({"data": {}}))),
        "faucet" => Some((json!({"url": "acc://test.acme"}), json!({"transactionHash": "deadbeef"}))),
        "query" => Some((json!({"url": "acc://test.acme"}), json!({"data": {}}))),
        "query-directory" => Some((json!({"url": "acc://test.acme"}), json!({"data": {}}))),
        "query-tx" => Some((json!({"url": "acc://test.acme"}), json!({"data": {}}))),
        "query-tx-local" => Some((json!({"url": "acc://test.acme"}), json!({"data": {}}))),
        "query-tx-history" => Some((json!({"url": "acc://test.acme"}), json!({"data": {}}))),
        "query-data" => Some((json!({"url": "acc://test.acme"}), json!({"data": {}}))),
        "query-data-set" => Some((json!({"url": "acc://test.acme"}), json!({"data": {}}))),
        "query-key-index" => Some((json!({"url": "acc://test.acme"}), json!({"data": {}}))),
        "query-minor-blocks" => Some((json!({"url": "acc://test.acme"}), json!({"data": {}}))),
        "query-major-blocks" => Some((json!({"url": "acc://test.acme"}), json!({"data": {}}))),
        "query-synth" => Some((json!({}), json!({"data": {}}))),
        "execute" => Some((json!({}), json!({"transactionHash": "deadbeef"}))),
        "execute-direct" => Some((json!({}), json!({"transactionHash": "deadbeef"}))),
        "execute-local" => Some((json!({}), json!({"transactionHash": "deadbeef"}))),
        "create-adi" => Some((json!({}), json!({"data": {}}))),
        "create-identity" => Some((json!({}), json!({"data": {}}))),
        "create-data-account" => Some((json!({}), json!({"data": {}}))),
        "create-key-book" => Some((json!({}), json!({"data": {}}))),
        "create-key-page" => Some((json!({}), json!({"data": {}}))),
        "create-token" => Some((json!({}), json!({"data": {}}))),
        "create-token-account" => Some((json!({}), json!({"data": {}}))),
        "send-tokens" => Some((json!({}), json!({"data": {}}))),
        "add-credits" => Some((json!({}), json!({"data": {}}))),
        "update-key-page" => Some((json!({}), json!({"data": {}}))),
        "update-key" => Some((json!({}), json!({"data": {}}))),
        "write-data" => Some((json!({}), json!({"data": {}}))),
        "issue-tokens" => Some((json!({}), json!({"data": {}}))),
        "write-data-to" => Some((json!({}), json!({"data": {}}))),
        "burn-tokens" => Some((json!({}), json!({"data": {}}))),
        "update-account-auth" => Some((json!({}), json!({"data": {}}))),
        _ => None,
    }
}