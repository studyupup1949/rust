//! Rent reclaim operations

use anyhow::{Context, Result};
use chrono::Utc;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::system_instruction;
use solana_sdk::transaction::Transaction;
use std::path::Path;
use tracing::{debug, info, warn};

use super::SolanaClient;
use crate::config::OperatorConfig;
use crate::types::{LogEntry, LogLevel, ReclaimOperation, ReclaimReason, SponsoredAccount};

/// Handles rent reclaim operations
pub struct RentReclaimer {
    client: SolanaClient,
    config: OperatorConfig,
    keypair: Option<Keypair>,
    operation_counter: u64,
}

impl RentReclaimer {
    pub fn new(client: SolanaClient, config: OperatorConfig) -> Result<Self> {
        let keypair = if let Some(path) = &config.keypair_path {
            Some(Self::load_keypair(path)?)
        } else {
            None
        };

        Ok(Self {
            client,
            config,
            keypair,
            operation_counter: 0,
        })
    }

    /// Load keypair from file
    fn load_keypair(path: &Path) -> Result<Keypair> {
        let data = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read keypair file: {}", path.display()))?;

        let bytes: &[u8] =
            serde_json::from_str::<&[u8]>(&data).with_context(|| "Failed to parse keypair JSON")?;

        Keypair::try_from(bytes).with_context(|| "Invalid keypair bytes")
    }

    /// Reclaim rent from a closed/empty token account
    pub async fn reclaim_token_account(
        &mut self,
        account: &SponsoredAccount,
        reason: ReclaimReason,
    ) -> Result<(ReclaimOperation, LogEntry)> {
        let now = Utc::now();
        self.operation_counter += 1;
        let op_id = self.operation_counter;

        let treasury = self
            .config
            .treasury_pubkey()
            .ok_or_else(|| anyhow::anyhow!("Treasury address not configured"))?;

        info!(
            "Reclaiming token account {} ({} lamports) -> {}",
            account.short_pubkey(),
            account.lamports,
            treasury
        );

        if self.config.dry_run {
            // Dry run mode - simulate success
            let log = LogEntry {
                timestamp: now,
                level: LogLevel::Info,
                message: format!(
                    "[DRY RUN] Would reclaim {} lamports from {}",
                    account.lamports,
                    account.short_pubkey()
                ),
                details: Some(format!("Reason: {}", reason)),
            };

            let operation = ReclaimOperation {
                id: op_id,
                account: account.pubkey,
                lamports_reclaimed: account.lamports,
                destination: treasury,
                signature: "DRY_RUN_NO_SIGNATURE".to_string(),
                timestamp: now,
                reason,
                success: true,
                error: None,
            };

            return Ok((operation, log));
        }

        // Real execution
        let keypair = self
            .keypair
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Keypair not loaded - cannot sign transactions"))?;

        // Create close account instruction for SPL token
        let close_ix = spl_token::instruction::close_account(
            &spl_token::id(),
            &account.pubkey,
            &treasury,
            &keypair.pubkey(),
            &[],
        )?;

        let blockhash = self.client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[close_ix],
            Some(&keypair.pubkey()),
            &[keypair],
            blockhash,
        );

        match self.client.send_and_confirm_transaction(&tx).await {
            Ok(sig) => {
                let log = LogEntry {
                    timestamp: now,
                    level: LogLevel::Success,
                    message: format!(
                        "Reclaimed {} lamports from {}",
                        account.lamports,
                        account.short_pubkey()
                    ),
                    details: Some(format!("Signature: {}", sig)),
                };

                let operation = ReclaimOperation {
                    id: op_id,
                    account: account.pubkey,
                    lamports_reclaimed: account.lamports,
                    destination: treasury,
                    signature: sig.to_string(),
                    timestamp: now,
                    reason,
                    success: true,
                    error: None,
                };

                Ok((operation, log))
            }
            Err(e) => {
                let log = LogEntry {
                    timestamp: now,
                    level: LogLevel::Error,
                    message: format!("Failed to reclaim from {}", account.short_pubkey()),
                    details: Some(e.to_string()),
                };

                let operation = ReclaimOperation {
                    id: op_id,
                    account: account.pubkey,
                    lamports_reclaimed: 0,
                    destination: treasury,
                    signature: String::new(),
                    timestamp: now,
                    reason,
                    success: false,
                    error: Some(e.to_string()),
                };

                Ok((operation, log))
            }
        }
    }

    /// Reclaim rent from a system account (transfer all lamports out)
    pub async fn reclaim_system_account(
        &mut self,
        account: &SponsoredAccount,
        reason: ReclaimReason,
    ) -> Result<(ReclaimOperation, LogEntry)> {
        let now = Utc::now();
        self.operation_counter += 1;
        let op_id = self.operation_counter;

        let treasury = self
            .config
            .treasury_pubkey()
            .ok_or_else(|| anyhow::anyhow!("Treasury address not configured"))?;

        info!(
            "Reclaiming system account {} ({} lamports) -> {}",
            account.short_pubkey(),
            account.lamports,
            treasury
        );

        if self.config.dry_run {
            let log = LogEntry {
                timestamp: now,
                level: LogLevel::Info,
                message: format!(
                    "[DRY RUN] Would transfer {} lamports from {} to treasury",
                    account.lamports,
                    account.short_pubkey()
                ),
                details: Some(format!("Reason: {}", reason)),
            };

            let operation = ReclaimOperation {
                id: op_id,
                account: account.pubkey,
                lamports_reclaimed: account.lamports,
                destination: treasury,
                signature: "DRY_RUN_NO_SIGNATURE".to_string(),
                timestamp: now,
                reason,
                success: true,
                error: None,
            };

            return Ok((operation, log));
        }

        let keypair = self
            .keypair
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Keypair not loaded - cannot sign transactions"))?;

        // For system accounts, we just transfer all lamports
        let transfer_ix =
            system_instruction::transfer(&account.pubkey, &treasury, account.lamports);

        let blockhash = self.client.get_latest_blockhash().await?;
        let tx = Transaction::new_signed_with_payer(
            &[transfer_ix],
            Some(&keypair.pubkey()),
            &[keypair],
            blockhash,
        );

        match self.client.send_and_confirm_transaction(&tx).await {
            Ok(sig) => {
                let log = LogEntry {
                    timestamp: now,
                    level: LogLevel::Success,
                    message: format!(
                        "Transferred {} lamports from {}",
                        account.lamports,
                        account.short_pubkey()
                    ),
                    details: Some(format!("Signature: {}", sig)),
                };

                let operation = ReclaimOperation {
                    id: op_id,
                    account: account.pubkey,
                    lamports_reclaimed: account.lamports,
                    destination: treasury,
                    signature: sig.to_string(),
                    timestamp: now,
                    reason,
                    success: true,
                    error: None,
                };

                Ok((operation, log))
            }
            Err(e) => {
                let log = LogEntry {
                    timestamp: now,
                    level: LogLevel::Error,
                    message: format!("Failed to transfer from {}", account.short_pubkey()),
                    details: Some(e.to_string()),
                };

                let operation = ReclaimOperation {
                    id: op_id,
                    account: account.pubkey,
                    lamports_reclaimed: 0,
                    destination: treasury,
                    signature: String::new(),
                    timestamp: now,
                    reason,
                    success: false,
                    error: Some(e.to_string()),
                };

                Ok((operation, log))
            }
        }
    }

    /// Estimate reclaim value for an account
    pub fn estimate_reclaim(&self, account: &SponsoredAccount) -> u64 {
        // For most accounts, we can reclaim the full lamports
        // Token accounts: full balance
        // System accounts: depends on if we control the account
        account.lamports
    }

    /// Check if reclaim is possible for an account
    pub fn can_reclaim(&self, account: &SponsoredAccount) -> bool {
        use crate::types::AccountStatus;

        match account.status {
            AccountStatus::Empty | AccountStatus::Reclaimable => true,
            AccountStatus::Inactive => true, // May need additional checks
            AccountStatus::Active | AccountStatus::Closed | AccountStatus::Unknown => false,
        }
    }

    /// Get configured treasury
    pub fn treasury(&self) -> Option<Pubkey> {
        self.config.treasury_pubkey()
    }

    /// Check if in dry run mode
    pub fn is_dry_run(&self) -> bool {
        self.config.dry_run
    }
}
