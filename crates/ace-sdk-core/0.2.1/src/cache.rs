//! SQLite-based local cache for playbook data.
//!
//! Architecture:
//! - Location: `~/.ace-cache/{org_id}_{project_id}.db`
//! - Purpose: Fast local cache, survives restarts
//! - TTL: configurable (default: 5 minutes)
//! - Source of truth: Remote server

use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::sync::Mutex;

use crate::types::{PlaybookBullet, BulletSection, StructuredPlaybook};

/// Configuration for the local cache.
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub org_id: String,
    pub project_id: String,
    pub ttl_minutes: u32,
    pub cache_dir: Option<PathBuf>,
}

/// SQLite-backed local cache service.
pub struct LocalCacheService {
    db: Mutex<Connection>,
    ttl_ms: u64,
}

impl LocalCacheService {
    /// Create a new local cache service.
    ///
    /// # Arguments
    /// * `org_id` - Organization identifier
    /// * `project_id` - Project identifier
    /// * `ttl_minutes` - Cache time-to-live in minutes
    /// * `cache_dir` - Optional custom cache directory
    pub fn new(
        org_id: &str,
        project_id: &str,
        ttl_minutes: u32,
        cache_dir: Option<PathBuf>,
    ) -> Result<Self, rusqlite::Error> {
        let dir = cache_dir.unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".ace-cache")
        });

        std::fs::create_dir_all(&dir).ok();

        let db_path = dir.join(format!("{}_{}.db", org_id, project_id));
        let conn = Connection::open(db_path)?;

        // Initialize schema
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS playbook_bullets (
                id TEXT PRIMARY KEY,
                section TEXT NOT NULL,
                content TEXT NOT NULL,
                helpful INTEGER DEFAULT 0,
                harmful INTEGER DEFAULT 0,
                confidence REAL DEFAULT 0.5,
                observations INTEGER DEFAULT 0,
                evidence TEXT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                last_used TEXT DEFAULT CURRENT_TIMESTAMP,
                synced_at TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_section ON playbook_bullets(section);
            CREATE INDEX IF NOT EXISTS idx_confidence ON playbook_bullets(confidence);
            CREATE INDEX IF NOT EXISTS idx_helpful ON playbook_bullets(helpful DESC);

            CREATE TABLE IF NOT EXISTS sync_state (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            PRAGMA journal_mode = WAL;
            ",
        )?;

        Ok(Self {
            db: Mutex::new(conn),
            ttl_ms: ttl_minutes as u64 * 60 * 1000,
        })
    }

    /// Get cached playbook (if fresh).
    pub fn get_playbook(&self) -> Option<StructuredPlaybook> {
        if self.needs_sync() {
            return None;
        }

        let db = self.db.lock().ok()?;
        let mut stmt = db
            .prepare("SELECT id, section, content, helpful, harmful, confidence, observations, evidence, created_at, last_used FROM playbook_bullets")
            .ok()?;

        let rows = stmt
            .query_map([], |row| {
                let section_str: String = row.get(1)?;
                let evidence_str: Option<String> = row.get(7)?;
                let evidence: Vec<String> = evidence_str
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_default();

                let section = match section_str.as_str() {
                    "strategies_and_hard_rules" => BulletSection::StrategiesAndHardRules,
                    "useful_code_snippets" => BulletSection::UsefulCodeSnippets,
                    "troubleshooting_and_pitfalls" => BulletSection::TroubleshootingAndPitfalls,
                    "apis_to_use" => BulletSection::ApisToUse,
                    _ => BulletSection::StrategiesAndHardRules,
                };

                Ok(PlaybookBullet {
                    id: row.get(0)?,
                    section,
                    content: row.get(2)?,
                    domain: None,
                    concrete_domain: None,
                    helpful: row.get(3)?,
                    harmful: row.get(4)?,
                    confidence: row.get(5)?,
                    observations: row.get::<_, u32>(6)?,
                    evidence,
                    created_at: row.get(8)?,
                    last_used: row.get(9)?,
                    root_cause: String::new(),
                    error_context: String::new(),
                })
            })
            .ok()?;

        let mut playbook = StructuredPlaybook::default();

        for row in rows {
            if let Ok(bullet) = row {
                match bullet.section {
                    BulletSection::StrategiesAndHardRules => {
                        playbook.strategies_and_hard_rules.push(bullet)
                    }
                    BulletSection::UsefulCodeSnippets => {
                        playbook.useful_code_snippets.push(bullet)
                    }
                    BulletSection::TroubleshootingAndPitfalls => {
                        playbook.troubleshooting_and_pitfalls.push(bullet)
                    }
                    BulletSection::ApisToUse => playbook.apis_to_use.push(bullet),
                }
            }
        }

        let total = playbook.strategies_and_hard_rules.len()
            + playbook.useful_code_snippets.len()
            + playbook.troubleshooting_and_pitfalls.len()
            + playbook.apis_to_use.len();

        if total == 0 {
            return None;
        }

        Some(playbook)
    }

    /// Save playbook to cache.
    pub fn save_playbook(&self, playbook: &StructuredPlaybook) {
        let db = match self.db.lock() {
            Ok(db) => db,
            Err(_) => return,
        };

        let now = chrono::Utc::now().to_rfc3339();

        // Clear old data
        let _ = db.execute("DELETE FROM playbook_bullets", []);

        let insert_bullets = |bullets: &[PlaybookBullet], section: &str| {
            for bullet in bullets {
                let evidence_json = serde_json::to_string(&bullet.evidence).unwrap_or_default();
                let _ = db.execute(
                    "INSERT INTO playbook_bullets (id, section, content, helpful, harmful, confidence, observations, evidence, created_at, last_used, synced_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                    params![
                        bullet.id,
                        section,
                        bullet.content,
                        bullet.helpful,
                        bullet.harmful,
                        bullet.confidence,
                        bullet.observations,
                        evidence_json,
                        bullet.created_at,
                        bullet.last_used,
                        now
                    ],
                );
            }
        };

        insert_bullets(
            &playbook.strategies_and_hard_rules,
            "strategies_and_hard_rules",
        );
        insert_bullets(&playbook.useful_code_snippets, "useful_code_snippets");
        insert_bullets(
            &playbook.troubleshooting_and_pitfalls,
            "troubleshooting_and_pitfalls",
        );
        insert_bullets(&playbook.apis_to_use, "apis_to_use");

        // Update sync state
        self.set_sync_state_inner(&db, "last_sync", &now);
    }

    /// Check if cache needs sync (>TTL).
    pub fn needs_sync(&self) -> bool {
        let db = match self.db.lock() {
            Ok(db) => db,
            Err(_) => return true,
        };

        let last_sync = self.get_sync_state_inner(&db, "last_sync");
        match last_sync {
            Some(ts) => {
                let parsed = chrono::DateTime::parse_from_rfc3339(&ts);
                match parsed {
                    Ok(last) => {
                        let elapsed = chrono::Utc::now()
                            .signed_duration_since(last)
                            .num_milliseconds() as u64;
                        elapsed > self.ttl_ms
                    }
                    Err(_) => true,
                }
            }
            None => true,
        }
    }

    /// Clear entire cache.
    pub fn clear(&self) {
        if let Ok(db) = self.db.lock() {
            let _ = db.execute_batch(
                "DELETE FROM playbook_bullets; DELETE FROM sync_state;",
            );
        }
    }

    /// Get sync state value.
    fn get_sync_state_inner(&self, db: &Connection, key: &str) -> Option<String> {
        db.query_row(
            "SELECT value FROM sync_state WHERE key = ?1",
            params![key],
            |row| row.get(0),
        )
        .ok()
    }

    /// Set sync state value.
    fn set_sync_state_inner(&self, db: &Connection, key: &str, value: &str) {
        let now = chrono::Utc::now().to_rfc3339();
        let _ = db.execute(
            "INSERT OR REPLACE INTO sync_state (key, value, updated_at) VALUES (?1, ?2, ?3)",
            params![key, value, now],
        );
    }
}

// =============================================================================
// Session Storage
// =============================================================================

/// Configuration for session storage.
#[derive(Debug, Clone)]
pub struct SessionStorageConfig {
    pub cache_dir: Option<PathBuf>,
}

/// A pinned session with its patterns.
#[derive(Debug, Clone)]
pub struct SessionPin {
    pub session_id: String,
    pub query: String,
    pub threshold: f64,
    pub top_k: u32,
    pub patterns: Vec<PlaybookBullet>,
    pub created_at: i64,
    pub expires_at: i64,
}

/// Result of recalling a session.
#[derive(Debug, Clone)]
pub struct SessionPinResult {
    pub similar_patterns: Vec<PlaybookBullet>,
    pub count: u32,
    pub threshold: f64,
    pub top_k: u32,
    pub session_id: String,
    pub pinned_at: i64,
    pub expires_at: i64,
}

/// Session listing entry.
#[derive(Debug, Clone)]
pub struct SessionListEntry {
    pub session_id: String,
    pub query: String,
    pub pattern_count: u32,
    pub created_at: i64,
    pub expires_at: i64,
}

/// Persistent session storage for pattern pinning.
///
/// Allows patterns retrieved during a hook to be pinned to a session
/// and later recalled without redundant server calls.
///
/// Storage: `~/.ace-cache/sessions.db` (SQLite)
/// TTL: 24 hours
pub struct SessionStorage {
    db: Mutex<Connection>,
}

impl SessionStorage {
    /// 24 hours in milliseconds.
    const SESSION_TTL_MS: i64 = 24 * 60 * 60 * 1000;

    /// Create and initialize session storage.
    pub fn new(config: Option<SessionStorageConfig>) -> Result<Self, rusqlite::Error> {
        let dir = config
            .and_then(|c| c.cache_dir)
            .unwrap_or_else(|| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".ace-cache")
            });

        std::fs::create_dir_all(&dir).ok();

        let db_path = dir.join("sessions.db");
        let conn = Connection::open(db_path)?;

        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS sessions (
                session_id TEXT PRIMARY KEY,
                query TEXT NOT NULL,
                threshold REAL NOT NULL,
                top_k INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                expires_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS session_patterns (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                pattern_data TEXT NOT NULL,
                FOREIGN KEY (session_id) REFERENCES sessions(session_id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_session_expires
            ON sessions(expires_at);

            PRAGMA journal_mode = WAL;
            PRAGMA foreign_keys = ON;
            ",
        )?;

        let storage = Self {
            db: Mutex::new(conn),
        };

        // Cleanup expired sessions on init
        let _ = storage.cleanup_expired();

        Ok(storage)
    }

    /// Pin patterns to a session.
    pub fn pin_session(
        &self,
        session_id: &str,
        query: &str,
        patterns: &[PlaybookBullet],
        threshold: f64,
        top_k: u32,
    ) -> Result<(), rusqlite::Error> {
        let db = self.db.lock().map_err(|_| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_ERROR),
                Some("Lock poisoned".to_string()),
            )
        })?;

        let now = chrono::Utc::now().timestamp_millis();
        let expires_at = now + Self::SESSION_TTL_MS;

        // Delete existing patterns for this session
        db.execute(
            "DELETE FROM session_patterns WHERE session_id = ?1",
            params![session_id],
        )?;

        // Insert or replace session metadata
        db.execute(
            "INSERT OR REPLACE INTO sessions (session_id, query, threshold, top_k, created_at, expires_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![session_id, query, threshold, top_k, now, expires_at],
        )?;

        // Insert patterns
        for pattern in patterns {
            let json = serde_json::to_string(pattern).unwrap_or_default();
            db.execute(
                "INSERT INTO session_patterns (session_id, pattern_data) VALUES (?1, ?2)",
                params![session_id, json],
            )?;
        }

        Ok(())
    }

    /// Recall patterns for a session.
    pub fn recall_session(&self, session_id: &str) -> Result<Option<SessionPinResult>, rusqlite::Error> {
        let db = self.db.lock().map_err(|_| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_ERROR),
                Some("Lock poisoned".to_string()),
            )
        })?;

        let session = db.query_row(
            "SELECT session_id, query, threshold, top_k, created_at, expires_at FROM sessions WHERE session_id = ?1",
            params![session_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, f64>(2)?,
                    row.get::<_, u32>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                ))
            },
        );

        let (sid, _query, threshold, top_k, created_at, expires_at) = match session {
            Ok(s) => s,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
            Err(e) => return Err(e),
        };

        // Check expiry
        let now = chrono::Utc::now().timestamp_millis();
        if now > expires_at {
            db.execute("DELETE FROM sessions WHERE session_id = ?1", params![session_id])?;
            return Ok(None);
        }

        // Get patterns
        let mut stmt = db.prepare(
            "SELECT pattern_data FROM session_patterns WHERE session_id = ?1",
        )?;
        let patterns: Vec<PlaybookBullet> = stmt
            .query_map(params![session_id], |row| {
                let json: String = row.get(0)?;
                Ok(json)
            })?
            .filter_map(|r| r.ok())
            .filter_map(|json| serde_json::from_str(&json).ok())
            .collect();

        let count = patterns.len() as u32;

        Ok(Some(SessionPinResult {
            similar_patterns: patterns,
            count,
            threshold,
            top_k,
            session_id: sid,
            pinned_at: created_at,
            expires_at,
        }))
    }

    /// Delete a specific session.
    pub fn delete_session(&self, session_id: &str) -> Result<bool, rusqlite::Error> {
        let db = self.db.lock().map_err(|_| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_ERROR),
                Some("Lock poisoned".to_string()),
            )
        })?;

        let changes = db.execute(
            "DELETE FROM sessions WHERE session_id = ?1",
            params![session_id],
        )?;

        Ok(changes > 0)
    }

    /// Cleanup expired sessions.
    pub fn cleanup_expired(&self) -> Result<usize, rusqlite::Error> {
        let db = self.db.lock().map_err(|_| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_ERROR),
                Some("Lock poisoned".to_string()),
            )
        })?;

        let now = chrono::Utc::now().timestamp_millis();
        let changes = db.execute(
            "DELETE FROM sessions WHERE expires_at < ?1",
            params![now],
        )?;

        Ok(changes)
    }

    /// List all active (non-expired) sessions.
    pub fn list_sessions(&self) -> Result<Vec<SessionListEntry>, rusqlite::Error> {
        let db = self.db.lock().map_err(|_| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_ERROR),
                Some("Lock poisoned".to_string()),
            )
        })?;

        let now = chrono::Utc::now().timestamp_millis();
        let mut stmt = db.prepare(
            "SELECT s.session_id, s.query, s.created_at, s.expires_at, COUNT(sp.id) as pattern_count
             FROM sessions s
             LEFT JOIN session_patterns sp ON s.session_id = sp.session_id
             WHERE s.expires_at > ?1
             GROUP BY s.session_id
             ORDER BY s.created_at DESC",
        )?;

        let entries = stmt
            .query_map(params![now], |row| {
                Ok(SessionListEntry {
                    session_id: row.get(0)?,
                    query: row.get(1)?,
                    created_at: row.get(2)?,
                    expires_at: row.get(3)?,
                    pattern_count: row.get(4)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(entries)
    }
}

// =============================================================================
// Project Index
// =============================================================================

/// Entry for a file in the project index.
#[derive(Debug, Clone)]
pub struct ProjectIndexEntry {
    pub path: String,
    pub language: Option<String>,
    pub imports: u32,
    pub imported_by: u32,
    pub is_hub: bool,
    pub is_entry_point: bool,
    pub last_modified: Option<i64>,
}

/// Project metadata stored in the index.
#[derive(Debug, Clone)]
pub struct ProjectMetadata {
    pub last_commit: Option<String>,
    pub last_indexed: i64,
    pub primary_language: Option<String>,
    pub total_files: u32,
}

/// Configuration for project index.
#[derive(Debug, Clone)]
pub struct ProjectIndexConfig {
    pub org_id: String,
    pub project_id: String,
    pub cache_dir: Option<PathBuf>,
}

/// Statistics from project index.
#[derive(Debug, Clone)]
pub struct ProjectIndexStats {
    pub total_files: u32,
    pub hub_files: u32,
    pub entry_points: u32,
    pub languages: std::collections::HashMap<String, u32>,
}

/// SQLite-based project file index for smart file selection.
///
/// Stores file metadata from import graph analysis.
/// Uses git commits for cache invalidation.
pub struct ProjectIndex {
    db: Connection,
    #[allow(dead_code)]
    org_id: String,
    #[allow(dead_code)]
    project_id: String,
}

impl ProjectIndex {
    /// Create a new project index.
    pub fn new(config: ProjectIndexConfig) -> Result<Self, rusqlite::Error> {
        let dir = config.cache_dir.unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".ace-cache")
        });

        std::fs::create_dir_all(&dir).ok();

        let db_path = dir.join(format!("{}_{}_index.db", config.org_id, config.project_id));
        let conn = Connection::open(db_path)?;

        conn.pragma_update(None, "journal_mode", "WAL")?;

        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS files (
                path TEXT PRIMARY KEY,
                language TEXT,
                imports INTEGER DEFAULT 0,
                imported_by INTEGER DEFAULT 0,
                is_hub INTEGER DEFAULT 0,
                is_entry_point INTEGER DEFAULT 0,
                last_modified INTEGER,
                indexed_at INTEGER
            );

            CREATE TABLE IF NOT EXISTS project_meta (
                key TEXT PRIMARY KEY,
                value TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_hub ON files(is_hub) WHERE is_hub = 1;
            CREATE INDEX IF NOT EXISTS idx_entry ON files(is_entry_point) WHERE is_entry_point = 1;
            CREATE INDEX IF NOT EXISTS idx_language ON files(language);
            CREATE INDEX IF NOT EXISTS idx_imported_by ON files(imported_by DESC);
            ",
        )?;

        Ok(Self {
            db: conn,
            org_id: config.org_id,
            project_id: config.project_id,
        })
    }

    /// Get hub files (most imported), limited to `limit` results.
    pub fn get_hub_files(&self, limit: u32) -> Vec<String> {
        let mut stmt = match self.db.prepare(
            "SELECT path FROM files WHERE is_hub = 1 ORDER BY imported_by DESC LIMIT ?1",
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        stmt.query_map(params![limit], |row| row.get::<_, String>(0))
            .map(|rows| rows.filter_map(|r| r.ok()).collect())
            .unwrap_or_default()
    }

    /// Get entry point files.
    pub fn get_entry_points(&self) -> Vec<String> {
        let mut stmt = match self.db.prepare(
            "SELECT path FROM files WHERE is_entry_point = 1",
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        stmt.query_map([], |row| row.get::<_, String>(0))
            .map(|rows| rows.filter_map(|r| r.ok()).collect())
            .unwrap_or_default()
    }

    /// Get most imported files.
    pub fn get_most_imported(&self, limit: u32) -> Vec<ProjectIndexEntry> {
        let mut stmt = match self.db.prepare(
            "SELECT path, language, imports, imported_by, is_hub, is_entry_point, last_modified
             FROM files ORDER BY imported_by DESC LIMIT ?1",
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        stmt.query_map(params![limit], |row| {
            Ok(ProjectIndexEntry {
                path: row.get(0)?,
                language: row.get(1)?,
                imports: row.get(2)?,
                imported_by: row.get(3)?,
                is_hub: row.get::<_, i32>(4)? != 0,
                is_entry_point: row.get::<_, i32>(5)? != 0,
                last_modified: row.get(6)?,
            })
        })
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default()
    }

    /// Get index statistics.
    pub fn get_stats(&self) -> ProjectIndexStats {
        let total_files = self
            .db
            .query_row("SELECT COUNT(*) FROM files", [], |row| row.get::<_, u32>(0))
            .unwrap_or(0);

        let hub_files = self
            .db
            .query_row(
                "SELECT COUNT(*) FROM files WHERE is_hub = 1",
                [],
                |row| row.get::<_, u32>(0),
            )
            .unwrap_or(0);

        let entry_points = self
            .db
            .query_row(
                "SELECT COUNT(*) FROM files WHERE is_entry_point = 1",
                [],
                |row| row.get::<_, u32>(0),
            )
            .unwrap_or(0);

        let mut languages = std::collections::HashMap::new();
        if let Ok(mut stmt) = self.db.prepare(
            "SELECT language, COUNT(*) FROM files WHERE language IS NOT NULL GROUP BY language",
        ) {
            if let Ok(rows) = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?))
            }) {
                for row in rows.flatten() {
                    languages.insert(row.0, row.1);
                }
            }
        }

        ProjectIndexStats {
            total_files,
            hub_files,
            entry_points,
            languages,
        }
    }

    /// Get metadata value by key.
    pub fn get_metadata(&self, key: &str) -> Option<String> {
        self.db
            .query_row(
                "SELECT value FROM project_meta WHERE key = ?1",
                params![key],
                |row| row.get(0),
            )
            .ok()
    }

    /// Set metadata value.
    pub fn set_metadata(&self, key: &str, value: &str) -> Result<(), rusqlite::Error> {
        self.db.execute(
            "INSERT OR REPLACE INTO project_meta (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    /// Insert a file entry into the index.
    pub fn insert_file(
        &self,
        path: &str,
        imports: u32,
        imported_by: u32,
        is_hub: bool,
        is_entry_point: bool,
    ) -> Result<(), rusqlite::Error> {
        let now = chrono::Utc::now().timestamp_millis();
        self.db.execute(
            "INSERT OR REPLACE INTO files (path, imports, imported_by, is_hub, is_entry_point, indexed_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![path, imports, imported_by, is_hub as i32, is_entry_point as i32, now],
        )?;
        Ok(())
    }

    /// Clear all data from the index.
    pub fn clear(&self) -> Result<(), rusqlite::Error> {
        self.db.execute_batch("DELETE FROM files; DELETE FROM project_meta;")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // =========================================================================
    // LocalCacheService tests
    // =========================================================================

    #[test]
    fn test_cache_creation() {
        let tmp = TempDir::new().unwrap();
        let cache =
            LocalCacheService::new("test-org", "test-project", 5, Some(tmp.path().to_path_buf()));
        assert!(cache.is_ok());
    }

    #[test]
    fn test_cache_needs_sync_when_empty() {
        let tmp = TempDir::new().unwrap();
        let cache =
            LocalCacheService::new("test-org", "test-project", 5, Some(tmp.path().to_path_buf()))
                .unwrap();
        assert!(cache.needs_sync());
    }

    #[test]
    fn test_cache_get_playbook_empty() {
        let tmp = TempDir::new().unwrap();
        let cache =
            LocalCacheService::new("test-org", "test-project", 5, Some(tmp.path().to_path_buf()))
                .unwrap();
        assert!(cache.get_playbook().is_none());
    }

    #[test]
    fn test_cache_save_and_retrieve() {
        let tmp = TempDir::new().unwrap();
        let cache = LocalCacheService::new(
            "test-org",
            "test-project",
            60, // Long TTL so it doesn't expire during test
            Some(tmp.path().to_path_buf()),
        )
        .unwrap();

        let playbook = StructuredPlaybook {
            strategies_and_hard_rules: vec![PlaybookBullet {
                id: "test-1".to_string(),
                section: BulletSection::StrategiesAndHardRules,
                content: "Always use Result<T, E>".to_string(),
                domain: None,
                concrete_domain: None,
                helpful: 5,
                harmful: 0,
                confidence: 0.9,
                observations: 10,
                evidence: vec!["src/main.rs".to_string()],
                created_at: "2025-01-01T00:00:00Z".to_string(),
                last_used: "2025-01-02T00:00:00Z".to_string(),
            root_cause: String::new(),
            error_context: String::new(),
            }],
            ..Default::default()
        };

        cache.save_playbook(&playbook);

        // Should now be available
        let retrieved = cache.get_playbook();
        assert!(retrieved.is_some());
        let pb = retrieved.unwrap();
        assert_eq!(pb.strategies_and_hard_rules.len(), 1);
        assert_eq!(
            pb.strategies_and_hard_rules[0].content,
            "Always use Result<T, E>"
        );
    }

    #[test]
    fn test_cache_clear() {
        let tmp = TempDir::new().unwrap();
        let cache = LocalCacheService::new(
            "test-org",
            "test-project",
            60,
            Some(tmp.path().to_path_buf()),
        )
        .unwrap();

        let playbook = StructuredPlaybook {
            apis_to_use: vec![PlaybookBullet {
                id: "test-2".to_string(),
                section: BulletSection::ApisToUse,
                content: "Use reqwest for HTTP".to_string(),
                domain: None,
                concrete_domain: None,
                helpful: 3,
                harmful: 0,
                confidence: 0.8,
                observations: 5,
                evidence: vec![],
                created_at: "2025-01-01T00:00:00Z".to_string(),
                last_used: "2025-01-01T00:00:00Z".to_string(),
            root_cause: String::new(),
            error_context: String::new(),
            }],
            ..Default::default()
        };

        cache.save_playbook(&playbook);
        cache.clear();
        assert!(cache.get_playbook().is_none());
    }

    // =========================================================================
    // SessionStorage tests
    // =========================================================================

    fn make_test_bullet(id: &str, content: &str) -> PlaybookBullet {
        PlaybookBullet {
            id: id.to_string(),
            section: BulletSection::StrategiesAndHardRules,
            content: content.to_string(),
            domain: None,
            concrete_domain: None,
            helpful: 1,
            harmful: 0,
            confidence: 0.8,
            observations: 3,
            evidence: vec![],
            created_at: "2025-01-01T00:00:00Z".to_string(),
            last_used: "2025-01-01T00:00:00Z".to_string(),
        root_cause: String::new(),
        error_context: String::new(),
        }
    }

    #[test]
    fn test_session_storage_creation() {
        let tmp = TempDir::new().unwrap();
        let config = SessionStorageConfig {
            cache_dir: Some(tmp.path().to_path_buf()),
        };
        let storage = SessionStorage::new(Some(config));
        assert!(storage.is_ok());
    }

    #[test]
    fn test_session_pin_and_recall() {
        let tmp = TempDir::new().unwrap();
        let config = SessionStorageConfig {
            cache_dir: Some(tmp.path().to_path_buf()),
        };
        let storage = SessionStorage::new(Some(config)).unwrap();

        let patterns = vec![
            make_test_bullet("b1", "Use Result for errors"),
            make_test_bullet("b2", "Prefer &str over String"),
        ];

        storage.pin_session("sess-1", "error handling", &patterns, 0.7, 10).unwrap();

        let result = storage.recall_session("sess-1").unwrap();
        assert!(result.is_some());
        let pin = result.unwrap();
        assert_eq!(pin.count, 2);
        assert_eq!(pin.threshold, 0.7);
        assert_eq!(pin.top_k, 10);
        assert_eq!(pin.similar_patterns.len(), 2);
        assert_eq!(pin.similar_patterns[0].content, "Use Result for errors");
    }

    #[test]
    fn test_session_recall_not_found() {
        let tmp = TempDir::new().unwrap();
        let config = SessionStorageConfig {
            cache_dir: Some(tmp.path().to_path_buf()),
        };
        let storage = SessionStorage::new(Some(config)).unwrap();

        let result = storage.recall_session("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_session_delete() {
        let tmp = TempDir::new().unwrap();
        let config = SessionStorageConfig {
            cache_dir: Some(tmp.path().to_path_buf()),
        };
        let storage = SessionStorage::new(Some(config)).unwrap();

        let patterns = vec![make_test_bullet("b1", "test")];
        storage.pin_session("sess-del", "query", &patterns, 0.5, 5).unwrap();

        assert!(storage.delete_session("sess-del").unwrap());
        assert!(!storage.delete_session("sess-del").unwrap());
        assert!(storage.recall_session("sess-del").unwrap().is_none());
    }

    #[test]
    fn test_session_list() {
        let tmp = TempDir::new().unwrap();
        let config = SessionStorageConfig {
            cache_dir: Some(tmp.path().to_path_buf()),
        };
        let storage = SessionStorage::new(Some(config)).unwrap();

        let p1 = vec![make_test_bullet("b1", "p1")];
        let p2 = vec![make_test_bullet("b2", "p2"), make_test_bullet("b3", "p3")];

        storage.pin_session("s1", "query1", &p1, 0.5, 5).unwrap();
        storage.pin_session("s2", "query2", &p2, 0.6, 10).unwrap();

        let sessions = storage.list_sessions().unwrap();
        assert_eq!(sessions.len(), 2);
        // Most recent first
        assert_eq!(sessions[0].session_id, "s2");
        assert_eq!(sessions[0].pattern_count, 2);
        assert_eq!(sessions[1].session_id, "s1");
        assert_eq!(sessions[1].pattern_count, 1);
    }

    #[test]
    fn test_session_cleanup_expired() {
        let tmp = TempDir::new().unwrap();
        let config = SessionStorageConfig {
            cache_dir: Some(tmp.path().to_path_buf()),
        };
        let storage = SessionStorage::new(Some(config)).unwrap();

        // Pin a session, then manually set it expired
        let patterns = vec![make_test_bullet("b1", "test")];
        storage.pin_session("expired-sess", "q", &patterns, 0.5, 5).unwrap();

        // Manually expire it
        {
            let db = storage.db.lock().unwrap();
            db.execute(
                "UPDATE sessions SET expires_at = 0 WHERE session_id = ?1",
                params!["expired-sess"],
            ).unwrap();
        }

        let cleaned = storage.cleanup_expired().unwrap();
        assert_eq!(cleaned, 1);

        assert!(storage.recall_session("expired-sess").unwrap().is_none());
    }

    // =========================================================================
    // ProjectIndex tests
    // =========================================================================

    #[test]
    fn test_project_index_creation() {
        let tmp = TempDir::new().unwrap();
        let idx = ProjectIndex::new(ProjectIndexConfig {
            org_id: "org".to_string(),
            project_id: "proj".to_string(),
            cache_dir: Some(tmp.path().to_path_buf()),
        });
        assert!(idx.is_ok());
    }

    #[test]
    fn test_project_index_insert_and_query() {
        let tmp = TempDir::new().unwrap();
        let idx = ProjectIndex::new(ProjectIndexConfig {
            org_id: "org".to_string(),
            project_id: "proj".to_string(),
            cache_dir: Some(tmp.path().to_path_buf()),
        })
        .unwrap();

        idx.insert_file("src/utils.rs", 2, 8, true, false).unwrap();
        idx.insert_file("src/main.rs", 5, 0, false, true).unwrap();
        idx.insert_file("src/lib.rs", 3, 3, false, false).unwrap();

        let hubs = idx.get_hub_files(10);
        assert_eq!(hubs.len(), 1);
        assert_eq!(hubs[0], "src/utils.rs");

        let entries = idx.get_entry_points();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0], "src/main.rs");

        let most = idx.get_most_imported(2);
        assert_eq!(most.len(), 2);
        assert_eq!(most[0].path, "src/utils.rs");
        assert_eq!(most[0].imported_by, 8);
    }

    #[test]
    fn test_project_index_stats() {
        let tmp = TempDir::new().unwrap();
        let idx = ProjectIndex::new(ProjectIndexConfig {
            org_id: "org".to_string(),
            project_id: "proj".to_string(),
            cache_dir: Some(tmp.path().to_path_buf()),
        })
        .unwrap();

        idx.insert_file("a.rs", 1, 6, true, false).unwrap();
        idx.insert_file("b.rs", 0, 0, false, true).unwrap();
        idx.insert_file("c.rs", 2, 2, false, false).unwrap();

        let stats = idx.get_stats();
        assert_eq!(stats.total_files, 3);
        assert_eq!(stats.hub_files, 1);
        assert_eq!(stats.entry_points, 1);
    }

    #[test]
    fn test_project_index_metadata() {
        let tmp = TempDir::new().unwrap();
        let idx = ProjectIndex::new(ProjectIndexConfig {
            org_id: "org".to_string(),
            project_id: "proj".to_string(),
            cache_dir: Some(tmp.path().to_path_buf()),
        })
        .unwrap();

        assert!(idx.get_metadata("last_commit").is_none());

        idx.set_metadata("last_commit", "abc123").unwrap();
        assert_eq!(idx.get_metadata("last_commit"), Some("abc123".to_string()));
    }

    #[test]
    fn test_project_index_clear() {
        let tmp = TempDir::new().unwrap();
        let idx = ProjectIndex::new(ProjectIndexConfig {
            org_id: "org".to_string(),
            project_id: "proj".to_string(),
            cache_dir: Some(tmp.path().to_path_buf()),
        })
        .unwrap();

        idx.insert_file("x.rs", 0, 0, false, false).unwrap();
        idx.set_metadata("key", "val").unwrap();

        idx.clear().unwrap();
        assert_eq!(idx.get_stats().total_files, 0);
        assert!(idx.get_metadata("key").is_none());
    }
}
