//! Account monitoring for sponsored accounts

use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use tracing::{debug, info, warn};

use super::SolanaClient;
use crate::config::MonitoringConfig;
use crate::types::{AccountStatus, LogEntry, LogLevel, RentStats, SponsoredAccount};

/// Monitors sponsored accounts for rent reclaim opportunities
pub struct AccountMonitor {
    client: SolanaClient,
    config: MonitoringConfig,
}

impl AccountMonitor {
    pub fn new(client: SolanaClient, config: MonitoringConfig) -> Self {
        Self { client, config }
    }

    /// Scan a list of accounts and update their status
    pub async fn scan_accounts(
        &self,
        accounts: &mut [SponsoredAccount],
    ) -> Result<(RentStats, Vec<LogEntry>)> {
        let mut stats = RentStats::default();
        let mut logs = Vec::new();
        let now = Utc::now();

        // Collect pubkeys for batch fetch
        let pubkeys: Vec<Pubkey> = accounts.iter().map(|a| a.pubkey).collect();

        if pubkeys.is_empty() {
            logs.push(LogEntry {
                timestamp: now,
                level: LogLevel::Info,
                message: "No accounts to scan".to_string(),
                details: None,
            });
            return Ok((stats, logs));
        }

        info!("Scanning {} accounts", pubkeys.len());
        logs.push(LogEntry {
            timestamp: now,
            level: LogLevel::Info,
            message: format!("Starting scan of {} accounts", pubkeys.len()),
            details: None,
        });

        // Fetch all accounts in batches
        let on_chain_accounts = self
            .client
            .get_multiple_accounts(&pubkeys)
            .await
            .context("Failed to fetch accounts")?;

        // Process each account
        for (i, sponsored) in accounts.iter_mut().enumerate() {
            let on_chain = &on_chain_accounts[i];
            sponsored.last_checked = now;

            match on_chain {
                Some(account) => {
                    // Account exists on chain
                    sponsored.lamports = account.lamports;
                    sponsored.data_len = account.data.len();
                    sponsored.owner = account.owner;

                    // Determine status
                    let new_status = self.determine_status(sponsored, account).await;

                    if new_status != sponsored.status {
                        debug!(
                            "Account {} status changed: {:?} -> {:?}",
                            sponsored.short_pubkey(),
                            sponsored.status,
                            new_status
                        );
                        logs.push(LogEntry {
                            timestamp: now,
                            level: LogLevel::Info,
                            message: format!(
                                "Account {} status: {:?} -> {:?}",
                                sponsored.short_pubkey(),
                                sponsored.status,
                                new_status
                            ),
                            details: Some(format!("Lamports: {}", account.lamports)),
                        });
                    }
                    sponsored.status = new_status;

                    // Update stats
                    stats.total_rent_locked += account.lamports;
                    match sponsored.status {
                        AccountStatus::Active => stats.active_accounts += 1,
                        AccountStatus::Reclaimable
                        | AccountStatus::Empty
                        | AccountStatus::Inactive => {
                            stats.reclaimable_accounts += 1;
                        }
                        _ => {}
                    }
                }
                None => {
                    // Account no longer exists - was closed
                    if sponsored.status != AccountStatus::Closed {
                        logs.push(LogEntry {
                            timestamp: now,
                            level: LogLevel::Success,
                            message: format!(
                                "Account {} has been closed",
                                sponsored.short_pubkey()
                            ),
                            details: Some(format!(
                                "Previously had {} lamports",
                                sponsored.lamports
                            )),
                        });
                    }
                    sponsored.status = AccountStatus::Closed;
                    sponsored.lamports = 0;
                    stats.closed_accounts += 1;
                }
            }

            stats.total_accounts += 1;
        }

        stats.last_scan = Some(now);

        logs.push(LogEntry {
            timestamp: now,
            level: LogLevel::Success,
            message: format!(
                "Scan complete: {} active, {} reclaimable, {} closed",
                stats.active_accounts, stats.reclaimable_accounts, stats.closed_accounts
            ),
            details: Some(format!("Total rent locked: {:.6} SOL", stats.locked_sol())),
        });

        Ok((stats, logs))
    }

    /// Determine the status of an account
    async fn determine_status(
        &self,
        sponsored: &SponsoredAccount,
        account: &solana_sdk::account::Account,
    ) -> AccountStatus {
        // Check if whitelisted - always active
        if self.config.is_whitelisted(&sponsored.pubkey) {
            return AccountStatus::Active;
        }

        // Check for token accounts with zero balance
        if account.owner == spl_token::id() && account.data.len() == 165 {
            // Parse token account to check balance
            if let Ok(token_account) = self.parse_token_account(&account.data) {
                if token_account.amount == 0 {
                    return AccountStatus::Empty;
                }
            }
        }

        // Check for general empty accounts (no data, just lamports)
        if account.data.is_empty() || account.data.iter().all(|&b| b == 0) {
            return AccountStatus::Empty;
        }

        // Check for inactivity
        if let Some(last_activity) = sponsored.last_activity {
            let inactivity_threshold = Duration::days(self.config.inactivity_threshold_days as i64);
            if Utc::now() - last_activity > inactivity_threshold {
                return AccountStatus::Inactive;
            }
        }

        // Account is active
        AccountStatus::Active
    }

    /// Parse a token account from raw data
    fn parse_token_account(&self, data: &[u8]) -> Result<TokenAccountInfo> {
        if data.len() < 165 {
            anyhow::bail!("Invalid token account data length");
        }

        // Token account layout:
        // 0-32: mint
        // 32-64: owner
        // 64-72: amount (u64 little endian)
        // ... rest of fields

        let amount = u64::from_le_bytes(data[64..72].try_into()?);

        Ok(TokenAccountInfo { amount })
    }

    /// Find sponsored accounts by querying recent transactions
    pub async fn discover_sponsored_accounts(
        &self,
        fee_payer: &Pubkey,
        limit: usize,
    ) -> Result<Vec<SponsoredAccount>> {
        info!("Discovering accounts sponsored by {}", fee_payer);

        // Get recent signatures for the fee payer
        let signatures = self
            .client
            .get_signatures_for_address(fee_payer, limit)
            .await?;

        info!("Found {} recent transactions", signatures.len());

        // For now, return empty - full implementation would parse transactions
        // to find account creations where fee_payer paid rent
        Ok(vec![])
    }

    /// Check activity for a specific account
    pub async fn check_account_activity(
        &self,
        pubkey: &Pubkey,
    ) -> Result<Option<chrono::DateTime<Utc>>> {
        let sigs = self.client.get_signatures_for_address(pubkey, 1).await?;

        if sigs.is_empty() {
            Ok(None)
        } else {
            // In a full implementation, we'd get the actual timestamp
            // For now, if there are signatures, consider it recently active
            Ok(Some(Utc::now()))
        }
    }

    /// Get rent exemption amount for a data size
    pub async fn get_rent_exemption(&self, data_len: usize) -> Result<u64> {
        self.client
            .get_minimum_balance_for_rent_exemption(data_len)
            .await
    }
}

/// Simplified token account info
struct TokenAccountInfo {
    amount: u64,
}

/// Result of scanning accounts
#[derive(Debug)]
pub struct ScanResult {
    pub stats: RentStats,
    pub logs: Vec<LogEntry>,
    pub reclaimable: Vec<Pubkey>,
}
