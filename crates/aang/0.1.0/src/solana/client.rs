//! Solana RPC client wrapper

use anyhow::{Context, Result};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig};
use solana_client::rpc_filter::{Memcmp, RpcFilterType};
use solana_sdk::account::Account;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use std::str::FromStr;
use std::time::Duration;

use crate::config::NetworkConfig;

/// Wrapper around Solana RPC client
pub struct SolanaClient {
    client: RpcClient,
    commitment: CommitmentConfig,
}

impl SolanaClient {
    /// Create a new Solana client from network config
    pub fn new(config: &NetworkConfig) -> Result<Self> {
        let commitment = match config.commitment.as_str() {
            "processed" => CommitmentConfig::processed(),
            "confirmed" => CommitmentConfig::confirmed(),
            "finalized" => CommitmentConfig::finalized(),
            _ => CommitmentConfig::confirmed(),
        };

        let client = RpcClient::new_with_timeout_and_commitment(
            config.rpc_url.clone(),
            Duration::from_secs(config.timeout_secs),
            commitment,
        );

        Ok(Self { client, commitment })
    }

    /// Get account info for a pubkey
    pub async fn get_account(&self, pubkey: &Pubkey) -> Result<Option<Account>> {
        match self.client.get_account(pubkey).await {
            Ok(account) => Ok(Some(account)),
            Err(e) => {
                // Check if account doesn't exist
                let err_str = e.to_string();
                if err_str.contains("AccountNotFound") || err_str.contains("could not find account")
                {
                    Ok(None)
                } else {
                    Err(e).context("Failed to get account")
                }
            }
        }
    }

    /// Get multiple accounts at once
    pub async fn get_multiple_accounts(&self, pubkeys: &[Pubkey]) -> Result<Vec<Option<Account>>> {
        if pubkeys.is_empty() {
            return Ok(vec![]);
        }

        // Solana limits batch requests, so chunk if needed
        const CHUNK_SIZE: usize = 100;
        let mut results = Vec::with_capacity(pubkeys.len());

        for chunk in pubkeys.chunks(CHUNK_SIZE) {
            let accounts = self
                .client
                .get_multiple_accounts(chunk)
                .await
                .context("Failed to get multiple accounts")?;
            results.extend(accounts);
        }

        Ok(results)
    }

    /// Get all accounts owned by a program
    pub async fn get_program_accounts(
        &self,
        program_id: &Pubkey,
    ) -> Result<Vec<(Pubkey, Account)>> {
        let config = RpcProgramAccountsConfig {
            filters: None,
            account_config: RpcAccountInfoConfig {
                encoding: Some(solana_account_decoder::UiAccountEncoding::Base64),
                commitment: Some(self.commitment),
                ..Default::default()
            },
            ..Default::default()
        };

        let accounts = self
            .client
            .get_program_accounts_with_config(program_id, config)
            .await
            .context("Failed to get program accounts")?;

        Ok(accounts)
    }

    /// Get token accounts owned by a specific owner
    pub async fn get_token_accounts_by_owner(
        &self,
        owner: &Pubkey,
    ) -> Result<Vec<(Pubkey, Account)>> {
        let token_program = spl_token::id();

        let config = RpcProgramAccountsConfig {
            filters: Some(vec![
                // Token account size is 165 bytes
                RpcFilterType::DataSize(165),
                // Filter by owner (offset 32, the owner field in token account)
                RpcFilterType::Memcmp(Memcmp::new_raw_bytes(32, owner.to_bytes().to_vec())),
            ]),
            account_config: RpcAccountInfoConfig {
                encoding: Some(solana_account_decoder::UiAccountEncoding::Base64),
                commitment: Some(self.commitment),
                ..Default::default()
            },
            ..Default::default()
        };

        let accounts = self
            .client
            .get_program_accounts_with_config(&token_program, config)
            .await
            .context("Failed to get token accounts")?;

        Ok(accounts)
    }

    /// Get signatures for an account (to check activity)
    pub async fn get_signatures_for_address(
        &self,
        address: &Pubkey,
        limit: usize,
    ) -> Result<Vec<Signature>> {
        let sigs = self
            .client
            .get_signatures_for_address(address)
            .await
            .context("Failed to get signatures")?;

        Ok(sigs
            .into_iter()
            .take(limit)
            .filter_map(|s| Signature::from_str(&s.signature).ok())
            .collect())
    }

    /// Get minimum balance for rent exemption
    pub async fn get_minimum_balance_for_rent_exemption(&self, data_len: usize) -> Result<u64> {
        let balance = self
            .client
            .get_minimum_balance_for_rent_exemption(data_len)
            .await
            .context("Failed to get minimum balance for rent exemption")?;
        Ok(balance)
    }

    /// Get current slot
    pub async fn get_slot(&self) -> Result<u64> {
        let slot = self.client.get_slot().await.context("Failed to get slot")?;
        Ok(slot)
    }

    /// Get balance of an account
    pub async fn get_balance(&self, pubkey: &Pubkey) -> Result<u64> {
        let balance = self
            .client
            .get_balance(pubkey)
            .await
            .context("Failed to get balance")?;
        Ok(balance)
    }

    /// Check if an account exists
    pub async fn account_exists(&self, pubkey: &Pubkey) -> Result<bool> {
        Ok(self.get_account(pubkey).await?.is_some())
    }

    /// Send and confirm transaction
    pub async fn send_and_confirm_transaction(
        &self,
        transaction: &solana_sdk::transaction::Transaction,
    ) -> Result<Signature> {
        let sig = self
            .client
            .send_and_confirm_transaction(transaction)
            .await
            .context("Failed to send and confirm transaction")?;
        Ok(sig)
    }

    /// Get recent blockhash
    pub async fn get_latest_blockhash(&self) -> Result<solana_sdk::hash::Hash> {
        let blockhash = self
            .client
            .get_latest_blockhash()
            .await
            .context("Failed to get latest blockhash")?;
        Ok(blockhash)
    }

    /// Get the underlying RPC client
    pub fn inner(&self) -> &RpcClient {
        &self.client
    }
}
