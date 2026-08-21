//! Application state and logic for Aang TUI

use anyhow::Result;
use chrono::Utc;
use solana_sdk::pubkey::Pubkey;
use std::collections::VecDeque;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::config::Config;
use crate::solana::{AccountMonitor, RentReclaimer, SolanaClient};
use crate::types::{
    AccountStatus, Alert, AlertSeverity, LogEntry, LogLevel, ReclaimOperation, ReclaimReason,
    RentStats, SponsoredAccount,
};

/// Active tab in the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Dashboard,
    Accounts,
    Logs,
    Alerts,
    Settings,
}

impl Tab {
    pub fn next(&self) -> Self {
        match self {
            Self::Dashboard => Self::Accounts,
            Self::Accounts => Self::Logs,
            Self::Logs => Self::Alerts,
            Self::Alerts => Self::Settings,
            Self::Settings => Self::Dashboard,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Self::Dashboard => Self::Settings,
            Self::Accounts => Self::Dashboard,
            Self::Logs => Self::Accounts,
            Self::Alerts => Self::Logs,
            Self::Settings => Self::Alerts,
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            Self::Dashboard => "Dashboard",
            Self::Accounts => "Accounts",
            Self::Logs => "Activity Log",
            Self::Alerts => "Alerts",
            Self::Settings => "Settings",
        }
    }

    pub fn all() -> [Tab; 5] {
        [
            Tab::Dashboard,
            Tab::Accounts,
            Tab::Logs,
            Tab::Alerts,
            Tab::Settings,
        ]
    }
}

/// Input mode for the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMode {
    #[default]
    Normal,
    /// Adding a new account
    AddAccount,
    /// Confirming reclaim action
    ConfirmReclaim,
    /// Editing settings
    EditSettings,
    /// Command input
    Command,
}

/// Main application state
pub struct App {
    pub config: Config,
    pub tab: Tab,
    pub input_mode: InputMode,
    pub input_buffer: String,

    // Data
    pub accounts: Vec<SponsoredAccount>,
    pub reclaim_history: Vec<ReclaimOperation>,
    pub logs: VecDeque<LogEntry>,
    pub alerts: Vec<Alert>,
    pub stats: RentStats,

    // Selection state
    pub account_selected: usize,
    pub log_scroll: usize,
    pub alert_selected: usize,

    // Status
    pub is_scanning: bool,
    pub last_error: Option<String>,
    pub status_message: Option<String>,

    // Background task flag
    pub auto_scan_enabled: bool,

    // Solana clients (wrapped for async)
    client: Option<SolanaClient>,
    monitor: Option<AccountMonitor>,
    reclaimer: Option<RentReclaimer>,

    // Alert counter
    alert_counter: u64,
}

impl Default for App {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

impl App {
    pub fn new(config: Config) -> Self {
        Self {
            auto_scan_enabled: config.monitoring.auto_scan,
            config,
            tab: Tab::default(),
            input_mode: InputMode::default(),
            input_buffer: String::new(),
            accounts: Vec::new(),
            reclaim_history: Vec::new(),
            logs: VecDeque::with_capacity(1000),
            alerts: Vec::new(),
            stats: RentStats::default(),
            account_selected: 0,
            log_scroll: 0,
            alert_selected: 0,
            is_scanning: false,
            last_error: None,
            status_message: None,
            client: None,
            monitor: None,
            reclaimer: None,
            alert_counter: 0,
        }
    }

    /// Initialize Solana connections
    pub fn init_solana(&mut self) -> Result<()> {
        let client = SolanaClient::new(&self.config.network)?;
        let monitor = AccountMonitor::new(
            SolanaClient::new(&self.config.network)?,
            self.config.monitoring.clone(),
        );
        let reclaimer = RentReclaimer::new(
            SolanaClient::new(&self.config.network)?,
            self.config.operator.clone(),
        )?;

        self.client = Some(client);
        self.monitor = Some(monitor);
        self.reclaimer = Some(reclaimer);

        self.log(LogLevel::Success, "Connected to Solana", None);
        Ok(())
    }

    /// Add a log entry
    pub fn log(&mut self, level: LogLevel, message: impl Into<String>, details: Option<String>) {
        let entry = LogEntry {
            timestamp: Utc::now(),
            level,
            message: message.into(),
            details,
        };
        self.logs.push_front(entry);

        // Trim old logs
        while self.logs.len() > self.config.storage.max_log_entries {
            self.logs.pop_back();
        }
    }

    /// Add an alert
    pub fn add_alert(
        &mut self,
        severity: AlertSeverity,
        title: impl Into<String>,
        message: impl Into<String>,
        related_accounts: Vec<Pubkey>,
    ) {
        self.alert_counter += 1;
        let alert = Alert {
            id: self.alert_counter,
            timestamp: Utc::now(),
            severity,
            title: title.into(),
            message: message.into(),
            acknowledged: false,
            related_accounts,
        };
        self.alerts.insert(0, alert);
    }

    /// Add a sponsored account to track
    pub fn add_account(&mut self, pubkey_str: &str) -> Result<()> {
        let pubkey = Pubkey::from_str(pubkey_str)?;

        // Check if already tracking
        if self.accounts.iter().any(|a| a.pubkey == pubkey) {
            anyhow::bail!("Account already being tracked");
        }

        let account = SponsoredAccount::new(pubkey, solana_sdk::system_program::id(), 0, 0);

        self.accounts.push(account);
        self.log(
            LogLevel::Info,
            format!("Added account to tracking: {}", pubkey_str),
            None,
        );

        Ok(())
    }

    /// Remove an account from tracking
    pub fn remove_account(&mut self, index: usize) -> Option<SponsoredAccount> {
        if index < self.accounts.len() {
            let account = self.accounts.remove(index);
            self.log(
                LogLevel::Info,
                format!("Removed account: {}", account.short_pubkey()),
                None,
            );
            Some(account)
        } else {
            None
        }
    }

    /// Get the currently selected account
    pub fn selected_account(&self) -> Option<&SponsoredAccount> {
        self.accounts.get(self.account_selected)
    }

    /// Get mutable reference to selected account
    pub fn selected_account_mut(&mut self) -> Option<&mut SponsoredAccount> {
        self.accounts.get_mut(self.account_selected)
    }

    /// Scan all accounts (sync wrapper for display)
    pub async fn scan_accounts(&mut self) -> Result<()> {
        if self.monitor.is_none() {
            anyhow::bail!("Solana client not initialized");
        }

        self.is_scanning = true;
        self.log(LogLevel::Info, "Starting account scan...", None);

        let monitor = self.monitor.as_ref().unwrap();
        let (stats, logs) = monitor.scan_accounts(&mut self.accounts).await?;

        self.stats = stats;
        for log in logs {
            self.logs.push_front(log);
        }

        // Check for alerts
        self.check_alerts();

        self.is_scanning = false;
        Ok(())
    }

    /// Check and generate alerts based on current state
    fn check_alerts(&mut self) {
        // Alert for large idle rent
        let idle_lamports: u64 = self
            .accounts
            .iter()
            .filter(|a| {
                matches!(
                    a.status,
                    AccountStatus::Empty | AccountStatus::Inactive | AccountStatus::Reclaimable
                )
            })
            .map(|a| a.lamports)
            .sum();

        let idle_sol = idle_lamports as f64 / 1_000_000_000.0;
        if idle_sol >= self.config.alerts.large_idle_threshold_sol {
            self.add_alert(
                AlertSeverity::High,
                "Large Idle Rent Detected",
                format!(
                    "{:.4} SOL is sitting idle in reclaimable accounts",
                    idle_sol
                ),
                self.accounts
                    .iter()
                    .filter(|a| {
                        matches!(
                            a.status,
                            AccountStatus::Empty
                                | AccountStatus::Inactive
                                | AccountStatus::Reclaimable
                        )
                    })
                    .map(|a| a.pubkey)
                    .collect(),
            );
        }

        // Alert for many idle accounts
        let idle_count = self
            .accounts
            .iter()
            .filter(|a| {
                matches!(
                    a.status,
                    AccountStatus::Empty | AccountStatus::Inactive | AccountStatus::Reclaimable
                )
            })
            .count();

        if idle_count >= self.config.alerts.idle_count_threshold {
            self.add_alert(
                AlertSeverity::Medium,
                "Many Idle Accounts",
                format!(
                    "{} accounts are idle and may be candidates for rent reclaim",
                    idle_count
                ),
                vec![],
            );
        }
    }

    /// Reclaim rent from selected account
    pub async fn reclaim_selected(&mut self) -> Result<()> {
        let account = self.selected_account().cloned();
        if account.is_none() {
            anyhow::bail!("No account selected");
        }
        let account = account.unwrap();

        let reclaimer = self
            .reclaimer
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Reclaimer not initialized"))?;

        if !reclaimer.can_reclaim(&account) {
            anyhow::bail!("Account is not eligible for reclaim");
        }

        let reason = match account.status {
            AccountStatus::Empty => ReclaimReason::EmptyTokenAccount,
            AccountStatus::Inactive => ReclaimReason::Inactivity,
            AccountStatus::Reclaimable => ReclaimReason::ScheduledReclaim,
            _ => ReclaimReason::ManualReclaim,
        };

        // Determine reclaim method based on owner
        let (operation, log) = if account.owner == spl_token::id() {
            reclaimer.reclaim_token_account(&account, reason).await?
        } else {
            reclaimer.reclaim_system_account(&account, reason).await?
        };

        self.logs.push_front(log);
        self.reclaim_history.push(operation.clone());

        if operation.success {
            self.stats.total_rent_reclaimed += operation.lamports_reclaimed;
            self.stats.reclaim_operations += 1;

            // Update account status
            if let Some(acc) = self
                .accounts
                .iter_mut()
                .find(|a| a.pubkey == account.pubkey)
            {
                acc.status = AccountStatus::Closed;
                acc.lamports = 0;
            }
        }

        Ok(())
    }

    /// Reclaim all eligible accounts
    pub async fn reclaim_all(&mut self) -> Result<usize> {
        let reclaimable: Vec<_> = self
            .accounts
            .iter()
            .enumerate()
            .filter(|(_, a)| {
                matches!(
                    a.status,
                    AccountStatus::Empty | AccountStatus::Inactive | AccountStatus::Reclaimable
                )
            })
            .map(|(i, _)| i)
            .collect();

        let count = reclaimable.len();
        self.log(
            LogLevel::Info,
            format!("Reclaiming {} accounts...", count),
            None,
        );

        for idx in reclaimable.into_iter().rev() {
            self.account_selected = idx;
            if let Err(e) = self.reclaim_selected().await {
                self.log(LogLevel::Error, format!("Reclaim failed: {}", e), None);
            }
        }

        Ok(count)
    }

    /// Acknowledge an alert
    pub fn acknowledge_alert(&mut self, index: usize) {
        if let Some(alert) = self.alerts.get_mut(index) {
            alert.acknowledged = true;
        }
    }

    /// Clear all acknowledged alerts
    pub fn clear_acknowledged_alerts(&mut self) {
        self.alerts.retain(|a| !a.acknowledged);
    }

    /// Get unacknowledged alert count
    pub fn unacknowledged_alerts(&self) -> usize {
        self.alerts.iter().filter(|a| !a.acknowledged).count()
    }

    /// Navigate accounts list
    pub fn next_account(&mut self) {
        if !self.accounts.is_empty() {
            self.account_selected = (self.account_selected + 1) % self.accounts.len();
        }
    }

    pub fn prev_account(&mut self) {
        if !self.accounts.is_empty() {
            self.account_selected = self
                .account_selected
                .checked_sub(1)
                .unwrap_or(self.accounts.len() - 1);
        }
    }

    /// Navigate alerts
    pub fn next_alert(&mut self) {
        if !self.alerts.is_empty() {
            self.alert_selected = (self.alert_selected + 1) % self.alerts.len();
        }
    }

    pub fn prev_alert(&mut self) {
        if !self.alerts.is_empty() {
            self.alert_selected = self
                .alert_selected
                .checked_sub(1)
                .unwrap_or(self.alerts.len() - 1);
        }
    }

    /// Scroll logs
    pub fn scroll_logs_up(&mut self) {
        self.log_scroll = self.log_scroll.saturating_sub(1);
    }

    pub fn scroll_logs_down(&mut self) {
        if self.log_scroll < self.logs.len().saturating_sub(1) {
            self.log_scroll += 1;
        }
    }

    /// Get treasury balance (would need async, placeholder)
    pub fn treasury_balance(&self) -> Option<u64> {
        // Would need async call
        None
    }

    /// Export accounts to JSON
    pub fn export_accounts(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(&self.accounts)?)
    }

    /// Import accounts from JSON
    pub fn import_accounts(&mut self, json: &str) -> Result<usize> {
        let accounts: Vec<SponsoredAccount> = serde_json::from_str(json)?;
        let count = accounts.len();
        self.accounts.extend(accounts);
        self.log(LogLevel::Info, format!("Imported {} accounts", count), None);
        Ok(count)
    }

    /// Get help text for current tab
    pub fn help_text(&self) -> &'static str {
        match self.tab {
            Tab::Dashboard => {
                "Dashboard: View rent statistics | [S]can | [R]eclaim all | [Tab] Switch tab"
            }
            Tab::Accounts => {
                "Accounts: [A]dd | [D]elete | [R]eclaim | [S]can | [j/k] Navigate | [Tab] Switch"
            }
            Tab::Logs => "Logs: [j/k] Scroll | [c] Clear | [Tab] Switch tab",
            Tab::Alerts => "Alerts: [Enter] Acknowledge | [c] Clear acknowledged | [Tab] Switch",
            Tab::Settings => "Settings: [e] Edit | [s] Save | [Tab] Switch tab",
        }
    }
}
