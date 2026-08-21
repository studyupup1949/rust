use solana_account::Account;
use solana_commitment_config::CommitmentConfig;
use solana_pubkey::Pubkey;
use solana_rpc_client::rpc_client::RpcClient;

use crate::error::{AccacheError, Result};
use crate::rpc::MAX_GMA_BATCH;

/// Blocking RPC fetcher.
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

    pub fn url(&self) -> String {
        self.client.url()
    }

    /// Fetch one account. Returns `Ok(None)` if the account does not exist.
    pub fn fetch(&self, key: &Pubkey) -> Result<Option<Account>> {
        match self
            .client
            .get_account_with_commitment(key, self.commitment)
        {
            Ok(resp) => Ok(resp.value),
            Err(err) => Err(AccacheError::Rpc(err)),
        }
    }

    /// Fetch many accounts. Each result is `Some` if present, `None` if missing.
    /// Chunks pubkey lists into batches of [`MAX_GMA_BATCH`] for the RPC call.
    pub fn fetch_multiple(&self, keys: &[Pubkey]) -> Result<Vec<Option<Account>>> {
        let mut out = Vec::with_capacity(keys.len());
        for chunk in keys.chunks(MAX_GMA_BATCH) {
            let resp = self
                .client
                .get_multiple_accounts_with_commitment(chunk, self.commitment)
                .map_err(AccacheError::Rpc)?;
            out.extend(resp.value.into_iter());
        }
        Ok(out)
    }
}
