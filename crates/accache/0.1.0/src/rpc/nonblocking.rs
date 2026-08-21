use solana_account::Account;
use solana_commitment_config::CommitmentConfig;
use solana_pubkey::Pubkey;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;

use crate::error::{AccacheError, Result};
use crate::rpc::MAX_GMA_BATCH;

/// Nonblocking (async) RPC fetcher.
pub struct RpcFetcher {
    client: RpcClient,
    commitment: CommitmentConfig,
}

impl RpcFetcher {
    pub fn new(url: impl Into<String>, commitment: CommitmentConfig) -> Self {
        Self {
            client: RpcClient::new_with_commitment(url.into(), commitment),
            commitment,
        }
    }

    pub async fn fetch(&self, key: &Pubkey) -> Result<Option<Account>> {
        let resp = self
            .client
            .get_account_with_commitment(key, self.commitment)
            .await
            .map_err(AccacheError::Rpc)?;
        Ok(resp.value)
    }

    pub async fn fetch_multiple(&self, keys: &[Pubkey]) -> Result<Vec<Option<Account>>> {
        let mut out = Vec::with_capacity(keys.len());
        for chunk in keys.chunks(MAX_GMA_BATCH) {
            let resp = self
                .client
                .get_multiple_accounts_with_commitment(chunk, self.commitment)
                .await
                .map_err(AccacheError::Rpc)?;
            out.extend(resp.value.into_iter());
        }
        Ok(out)
    }
}
