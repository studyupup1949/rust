//! Core types for Aang rent reclaim bot

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::fmt;

/// Status of a sponsored account
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountStatus {
    /// Account is active and in use
    Active,
    /// Account exists but has zero balance (potential for reclaim)
    Empty,
    /// Account has been closed, rent already reclaimed
    Closed,
    /// Account is inactive (no transactions for threshold period)
    Inactive,
    /// Account is eligible for rent reclaim
    Reclaimable,
    /// Unknown status (error fetching)
    Unknown,
}

impl fmt::Display for AccountStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Active => write!(f, "Active"),
            Self::Empty => write!(f, "Empty"),
            Self::Closed => write!(f, "Closed"),
            Self::Inactive => write!(f, "Inactive"),
            Self::Reclaimable => write!(f, "Reclaimable"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

impl AccountStatus {
    pub fn color(&self) -> ratatui::style::Color {
        use ratatui::style::Color;
        match self {
            Self::Active => Color::Green,
            Self::Empty => Color::Yellow,
            Self::Closed => Color::DarkGray,
            Self::Inactive => Color::Rgb(255, 165, 0), // Orange
            Self::Reclaimable => Color::Cyan,
            Self::Unknown => Color::Red,
        }
    }
}

/// A sponsored account tracked by Aang
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SponsoredAccount {
    /// The account's public key
    pub pubkey: Pubkey,
    /// Owner program of the account
    pub owner: Pubkey,
    /// Current lamports balance (rent deposit)
    pub lamports: u64,
    /// Account data size in bytes
    pub data_len: usize,
    /// Current status
    pub status: AccountStatus,
    /// When the account was first sponsored
    pub sponsored_at: DateTime<Utc>,
    /// Last time the account was checked
    pub last_checked: DateTime<Utc>,
    /// Last activity timestamp (if known)
    pub last_activity: Option<DateTime<Utc>>,
    /// Optional label/tag for the account
    pub label: Option<String>,
    /// The transaction signature that created this account (if known)
    pub creation_signature: Option<String>,
}

impl SponsoredAccount {
    pub fn new(pubkey: Pubkey, owner: Pubkey, lamports: u64, data_len: usize) -> Self {
        let now = Utc::now();
        Self {
            pubkey,
            owner,
            lamports,
            data_len,
            status: AccountStatus::Active,
            sponsored_at: now,
            last_checked: now,
            last_activity: Some(now),
            label: None,
            creation_signature: None,
        }
    }

    pub fn lamports_as_sol(&self) -> f64 {
        self.lamports as f64 / 1_000_000_000.0
    }

    pub fn short_pubkey(&self) -> String {
        let s = self.pubkey.to_string();
        if s.len() > 12 {
            format!("{}...{}", &s[..6], &s[s.len() - 4..])
        } else {
            s
        }
    }
}

/// A rent reclaim operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReclaimOperation {
    /// Unique ID for this operation
    pub id: u64,
    /// Account that was reclaimed
    pub account: Pubkey,
    /// Amount of lamports reclaimed
    pub lamports_reclaimed: u64,
    /// Destination treasury
    pub destination: Pubkey,
    /// Transaction signature
    pub signature: String,
    /// When the reclaim happened
    pub timestamp: DateTime<Utc>,
    /// Reason for reclaim
    pub reason: ReclaimReason,
    /// Whether the operation was successful
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

impl ReclaimOperation {
    pub fn sol_reclaimed(&self) -> f64 {
        self.lamports_reclaimed as f64 / 1_000_000_000.0
    }
}

/// Reason for reclaiming rent
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReclaimReason {
    /// Account was closed by user/program
    AccountClosed,
    /// Account has been inactive for threshold period
    Inactivity,
    /// Token account with zero balance
    EmptyTokenAccount,
    /// Manual reclaim triggered by operator
    ManualReclaim,
    /// Scheduled automatic reclaim
    ScheduledReclaim,
}

impl fmt::Display for ReclaimReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AccountClosed => write!(f, "Account Closed"),
            Self::Inactivity => write!(f, "Inactivity"),
            Self::EmptyTokenAccount => write!(f, "Empty Token Account"),
            Self::ManualReclaim => write!(f, "Manual Reclaim"),
            Self::ScheduledReclaim => write!(f, "Scheduled Reclaim"),
        }
    }
}

/// Log entry for the activity log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Info,
    Success,
    Warning,
    Error,
}

impl LogLevel {
    pub fn color(&self) -> ratatui::style::Color {
        use ratatui::style::Color;
        match self {
            Self::Info => Color::Blue,
            Self::Success => Color::Green,
            Self::Warning => Color::Yellow,
            Self::Error => Color::Red,
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Info => "i",
            Self::Success => "+",
            Self::Warning => "!",
            Self::Error => "x",
        }
    }
}

/// Alert for operator attention
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: u64,
    pub timestamp: DateTime<Utc>,
    pub severity: AlertSeverity,
    pub title: String,
    pub message: String,
    pub acknowledged: bool,
    pub related_accounts: Vec<Pubkey>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl AlertSeverity {
    pub fn color(&self) -> ratatui::style::Color {
        use ratatui::style::Color;
        match self {
            Self::Low => Color::Blue,
            Self::Medium => Color::Yellow,
            Self::High => Color::Rgb(255, 165, 0),
            Self::Critical => Color::Red,
        }
    }
}

/// Statistics for the dashboard
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RentStats {
    /// Total accounts being monitored
    pub total_accounts: usize,
    /// Number of active accounts
    pub active_accounts: usize,
    /// Number of reclaimable accounts
    pub reclaimable_accounts: usize,
    /// Number of closed accounts
    pub closed_accounts: usize,
    /// Total lamports locked as rent
    pub total_rent_locked: u64,
    /// Total lamports reclaimed
    pub total_rent_reclaimed: u64,
    /// Number of reclaim operations
    pub reclaim_operations: usize,
    /// Last scan timestamp
    pub last_scan: Option<DateTime<Utc>>,
}

impl RentStats {
    pub fn locked_sol(&self) -> f64 {
        self.total_rent_locked as f64 / 1_000_000_000.0
    }

    pub fn reclaimed_sol(&self) -> f64 {
        self.total_rent_reclaimed as f64 / 1_000_000_000.0
    }

    pub fn potential_reclaim_sol(&self) -> f64 {
        // Estimate: average rent per account * reclaimable accounts
        if self.total_accounts > 0 {
            let avg_rent = self.total_rent_locked as f64 / self.total_accounts as f64;
            (avg_rent * self.reclaimable_accounts as f64) / 1_000_000_000.0
        } else {
            0.0
        }
    }
}
