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

use crate::types::{BulletSection, PlaybookBullet, StructuredPlaybook};

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

        let db_path = dir.join(format!("{}__{}.db", org_id, project_id));
        let conn = Connection::open(db_path)?;

        // Initialize schema
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS playbook_bullets (
                id TEXT PRIMARY KEY,
                section TEXT NOT NULL,
                content TEXT NOT NULL,
                helpful REAL DEFAULT 0,
                harmful REAL DEFAULT 0,
                confidence REAL DEFAULT 0.5,
                observations REAL DEFAULT 0,
                evidence TEXT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                last_used TEXT,
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
                    observations: row.get::<_, f64>(6)?,
                    evidence,
                    created_at: row.get(8)?,
                    last_used: row.get(9)?,
                    root_cause: String::new(),
                    error_context: String::new(),
                })
            })
            .ok()?;

        let mut playbook = StructuredPlaybook::default();

        for bullet in rows.flatten() {
            match bullet.section {
                BulletSection::StrategiesAndHardRules => {
                    playbook.strategies_and_hard_rules.push(bullet)
                }
                BulletSection::UsefulCodeSnippets => playbook.useful_code_snippets.push(bullet),
                BulletSection::TroubleshootingAndPitfalls => {
                    playbook.troubleshooting_and_pitfalls.push(bullet)
                }
                BulletSection::ApisToUse => playbook.apis_to_use.push(bullet),
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
    ///
    /// The delete + all inserts run inside an explicit transaction so a crash
    /// between the DELETE and the last INSERT cannot leave a partial playbook.
    pub fn save_playbook(&self, playbook: &StructuredPlaybook) {
        let db = match self.db.lock() {
            Ok(db) => db,
            Err(_) => return,
        };

        let now = chrono::Utc::now().to_rfc3339();

        // Begin explicit transaction
        if db.execute_batch("BEGIN;").is_err() {
            return;
        }

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

        // Commit — if this fails the DELETE + inserts are rolled back atomically.
        let _ = db.execute_batch("COMMIT;");
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
            let _ = db.execute_batch("DELETE FROM playbook_bullets; DELETE FROM sync_state;");
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
        let dir = config.and_then(|c| c.cache_dir).unwrap_or_else(|| {
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
    pub fn recall_session(
        &self,
        session_id: &str,
    ) -> Result<Option<SessionPinResult>, rusqlite::Error> {
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
            db.execute(
                "DELETE FROM sessions WHERE session_id = ?1",
                params![session_id],
            )?;
            return Ok(None);
        }

        // Get patterns
        let mut stmt =
            db.prepare("SELECT pattern_data FROM session_patterns WHERE session_id = ?1")?;
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
        let changes = db.execute("DELETE FROM sessions WHERE expires_at < ?1", params![now])?;

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
             ORDER BY s.created_at DESC, s.rowid DESC",
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

        let db_path = dir.join(format!("{}__{}_index.db", config.org_id, config.project_id));
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
        let mut stmt = match self
            .db
            .prepare("SELECT path FROM files WHERE is_hub = 1 ORDER BY imported_by DESC LIMIT ?1")
        {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        stmt.query_map(params![limit], |row| row.get::<_, String>(0))
            .map(|rows| rows.filter_map(|r| r.ok()).collect())
            .unwrap_or_default()
    }

    /// Get entry point files.
    pub fn get_entry_points(&self) -> Vec<String> {
        let mut stmt = match self
            .db
            .prepare("SELECT path FROM files WHERE is_entry_point = 1")
        {
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
            .query_row("SELECT COUNT(*) FROM files WHERE is_hub = 1", [], |row| {
                row.get::<_, u32>(0)
            })
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
        self.db
            .execute_batch("DELETE FROM files; DELETE FROM project_meta;")?;
        Ok(())
    }
}

// =============================================================================
// ACE 1.5 Graph Cache (#104 — Stage 2)
// =============================================================================

/// SQLite-backed graph cache for ACE 1.5 pattern co-application graph.
///
/// Schema (byte-identical across all language SDKs — CONTRACT §5a):
///
/// ```sql
/// CREATE TABLE IF NOT EXISTS patterns (
///   pattern_id        TEXT PRIMARY KEY,
///   payload_json      TEXT    NOT NULL,
///   cumulative_reward REAL    NOT NULL DEFAULT 0,
///   fetched_at_ms     INTEGER NOT NULL,
///   expires_at_ms     INTEGER NOT NULL
/// );
/// CREATE TABLE IF NOT EXISTS edges (
///   src    TEXT    NOT NULL,
///   dst    TEXT    NOT NULL,
///   weight INTEGER NOT NULL,
///   PRIMARY KEY (src, dst)
/// );
/// CREATE INDEX IF NOT EXISTS idx_edges_src_weight_dst ON edges(src, weight, dst);
/// CREATE INDEX IF NOT EXISTS idx_patterns_expires     ON patterns(expires_at_ms);
/// ```
///
/// TTL: `expires_at_ms = fetched_at_ms + 604_800_000` (7 days).
/// Isolation: one DB file per `(org, project)` → `~/.ace-cache/<org>__<project>.db`.
/// WAL mode is set at connection open.
pub struct GraphCache {
    db: Mutex<Connection>,
    ttl_ms: i64,
    /// Organization identifier — injected as `X-ACE-Org` on graph refresh requests.
    org_id: String,
    /// Project identifier — injected as `X-ACE-Project` on graph refresh requests.
    project_id: String,
}

/// 7 days in milliseconds (F-046 mode B fixed TTL / default).
pub const GRAPH_TTL_MS: i64 = 604_800_000;

/// Result returned by [`GraphCache::refresh_from_server`].
///
/// The method is **best-effort** — it never throws. Callers can inspect
/// `edges_upserted` to know how many edges were written and `truncated` to
/// detect whether the server capped the response at 50 000 edges.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphRefreshResult {
    /// Number of edges successfully upserted into the local `edges` table.
    pub edges_upserted: usize,
    /// Whether the server indicated the response was truncated (`"truncated": true`).
    pub truncated: bool,
}

impl GraphCache {
    /// Open (or create) the graph cache for `(org_id, project_id)`.
    ///
    /// File: `<cache_dir>/<org_id>__<project_id>.db` (double underscore).
    /// If an old 5-min-KV schema already exists, `CREATE TABLE IF NOT EXISTS`
    /// adds the new tables without destroying existing data (migration path).
    ///
    /// # Arguments
    /// * `org_id` - Organization identifier.
    /// * `project_id` - Project identifier.
    /// * `cache_dir` - Optional custom cache directory.
    /// * `ttl_ms` - Optional TTL in milliseconds. Defaults to `GRAPH_TTL_MS` (7 days = 604_800_000).
    pub fn new(
        org_id: &str,
        project_id: &str,
        cache_dir: Option<std::path::PathBuf>,
    ) -> Result<Self, rusqlite::Error> {
        Self::new_with_ttl(org_id, project_id, cache_dir, None)
    }

    /// Open (or create) the graph cache with an explicit TTL.
    ///
    /// Identical to [`GraphCache::new`] but accepts an optional `ttl_ms`
    /// override. When `ttl_ms` is `None` the default `GRAPH_TTL_MS` (7 days)
    /// is used, preserving backward compatibility.
    pub fn new_with_ttl(
        org_id: &str,
        project_id: &str,
        cache_dir: Option<std::path::PathBuf>,
        ttl_ms: Option<i64>,
    ) -> Result<Self, rusqlite::Error> {
        let dir = cache_dir.unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".ace-cache")
        });

        std::fs::create_dir_all(&dir).ok();

        // Sanitise org_id / project_id before embedding in a filesystem path.
        // Removes `/`, `\`, and null bytes that could escape `~/.ace-cache/`.
        let safe = |s: &str| -> String {
            s.chars()
                .map(|c| {
                    if matches!(c, '/' | '\\' | '\0') {
                        '_'
                    } else {
                        c
                    }
                })
                .collect()
        };
        let safe_org = safe(org_id);
        let safe_project = safe(project_id);

        // CONTRACT §5c: one DB file per (org, project) → <org>__<project>.db (double underscore)
        let db_path = dir.join(format!("{safe_org}__{safe_project}.db"));

        let conn = Connection::open(&db_path)?;

        // WAL mode first (must be set before schema creation for full effect)
        conn.pragma_update(None, "journal_mode", "WAL")?;

        // Canonical DDL from CONTRACT §5a — verbatim, adapted only for rusqlite
        // param-binding syntax (positional `?1` is identical to the contract).
        // `sync_state` is added via `CREATE TABLE IF NOT EXISTS` so old DBs
        // that already have the patterns/edges tables are migrated transparently.
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS patterns (
              pattern_id        TEXT PRIMARY KEY,
              payload_json      TEXT    NOT NULL,
              cumulative_reward REAL    NOT NULL DEFAULT 0,
              fetched_at_ms     INTEGER NOT NULL,
              expires_at_ms     INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS edges (
              src    TEXT    NOT NULL,
              dst    TEXT    NOT NULL,
              weight INTEGER NOT NULL,
              PRIMARY KEY (src, dst)
            );
            CREATE INDEX IF NOT EXISTS idx_edges_src_weight_dst ON edges(src, weight, dst);
            CREATE INDEX IF NOT EXISTS idx_patterns_expires     ON patterns(expires_at_ms);
            CREATE TABLE IF NOT EXISTS sync_state (
              key        TEXT PRIMARY KEY,
              value      TEXT NOT NULL,
              updated_at TEXT NOT NULL
            );
            ",
        )?;

        Ok(Self {
            db: Mutex::new(conn),
            ttl_ms: ttl_ms.unwrap_or(GRAPH_TTL_MS),
            org_id: org_id.to_string(),
            project_id: project_id.to_string(),
        })
    }

    // ── Facade API (identical intent across all 5 language SDKs) ─────────────

    /// Retrieve a non-expired pattern by id. Returns `(payload_json, cumulative_reward)`.
    /// Lazily prunes the row if it is expired (CONTRACT §5c).
    pub fn get_pattern(&self, pattern_id: &str) -> Result<Option<(String, f64)>, rusqlite::Error> {
        let db = self.lock()?;
        let now_ms = chrono::Utc::now().timestamp_millis();

        let row = db.query_row(
            "SELECT payload_json, cumulative_reward, expires_at_ms \
             FROM patterns WHERE pattern_id = ?1",
            params![pattern_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, f64>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            },
        );

        match row {
            Ok((payload, reward, expires_at)) => {
                if expires_at <= now_ms {
                    // Lazy prune: remove expired row on read
                    let _ = db.execute(
                        "DELETE FROM patterns WHERE pattern_id = ?1",
                        params![pattern_id],
                    );
                    Ok(None)
                } else {
                    Ok(Some((payload, reward)))
                }
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Return 2-hop flat-join neighbors of `id` with `weight >= min_weight`
    /// that are not expired at `now`.
    ///
    /// CONTRACT §5b flat 2-hop SQL (NOT a recursive CTE — the load-bearing perf fix).
    /// Returns `Vec<(pattern_id, payload_json, cumulative_reward)>`.
    pub fn neighbors(
        &self,
        id: &str,
        _hops: u32, // always 2 — flat SQL hard-codes two hops
        min_weight: i64,
    ) -> Result<Vec<(String, String, f64)>, rusqlite::Error> {
        let db = self.lock()?;
        let now_ms = chrono::Utc::now().timestamp_millis();

        // Canonical flat-2-hop SQL from CONTRACT §5b — verbatim.
        let sql = "
            SELECT DISTINCT n.pattern_id, n.payload_json, n.cumulative_reward
            FROM (
              SELECT e1.dst AS id
              FROM edges e1
              WHERE e1.src = ?1 AND e1.weight >= ?2
              UNION
              SELECT e2.dst AS id
              FROM edges e1
              JOIN edges e2 ON e2.src = e1.dst
              WHERE e1.src = ?1 AND e1.weight >= ?2
                AND e2.weight >= ?2 AND e2.dst <> ?1
            ) hop
            JOIN patterns n ON n.pattern_id = hop.id
            WHERE n.expires_at_ms > ?3;
        ";

        let mut stmt = db.prepare(sql)?;
        let rows = stmt.query_map(params![id, min_weight, now_ms], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, f64>(2)?,
            ))
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Upsert a pattern with an explicit `fetched_at_ms` timestamp.
    ///
    /// `expires_at_ms` is computed as `fetched_at_ms + self.ttl_ms` (configurable TTL,
    /// default 604_800_000 ms = 7 days).
    pub fn upsert_pattern_at(
        &self,
        pattern_id: &str,
        payload_json: &str,
        cumulative_reward: f64,
        fetched_at_ms: i64,
    ) -> Result<(), rusqlite::Error> {
        let db = self.lock()?;
        let expires_at_ms = fetched_at_ms + self.ttl_ms;

        db.execute(
            "INSERT OR REPLACE INTO patterns
             (pattern_id, payload_json, cumulative_reward, fetched_at_ms, expires_at_ms)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                pattern_id,
                payload_json,
                cumulative_reward,
                fetched_at_ms,
                expires_at_ms
            ],
        )?;
        Ok(())
    }

    /// Upsert a pattern using the current wall-clock time as `fetched_at_ms`.
    pub fn upsert_pattern(
        &self,
        pattern_id: &str,
        payload_json: &str,
        cumulative_reward: f64,
    ) -> Result<(), rusqlite::Error> {
        let now_ms = chrono::Utc::now().timestamp_millis();
        self.upsert_pattern_at(pattern_id, payload_json, cumulative_reward, now_ms)
    }

    /// Upsert a directed edge `(src, dst, weight)`.
    pub fn upsert_edge(&self, src: &str, dst: &str, weight: i64) -> Result<(), rusqlite::Error> {
        let db = self.lock()?;
        db.execute(
            "INSERT OR REPLACE INTO edges (src, dst, weight) VALUES (?1, ?2, ?3)",
            params![src, dst, weight],
        )?;
        Ok(())
    }

    /// Explicitly prune all expired patterns and their orphan edges.
    /// Returns the number of pattern rows deleted.
    ///
    /// Orphan edges (src or dst referencing a pruned pattern) are removed in
    /// the same transaction to prevent unbounded edge-table growth and stale
    /// rows returning from the 2-hop neighbour query.
    pub fn prune(&self) -> Result<usize, rusqlite::Error> {
        let db = self.lock()?;
        let now_ms = chrono::Utc::now().timestamp_millis();
        db.execute_batch("BEGIN;")?;
        let n = db.execute(
            "DELETE FROM patterns WHERE expires_at_ms <= ?1",
            params![now_ms],
        );
        let n = match n {
            Ok(count) => count,
            Err(e) => {
                let _ = db.execute_batch("ROLLBACK;");
                return Err(e);
            }
        };
        // Remove edges whose src or dst no longer exists in patterns.
        let edge_result = db.execute_batch(
            "DELETE FROM edges WHERE src NOT IN (SELECT pattern_id FROM patterns) \
             OR dst NOT IN (SELECT pattern_id FROM patterns);",
        );
        if let Err(e) = edge_result {
            let _ = db.execute_batch("ROLLBACK;");
            return Err(e);
        }
        db.execute_batch("COMMIT;")?;
        Ok(n)
    }

    // ── Server refresh hook (CO_APPLIED graph endpoint — #104) ──────────────

    /// Refresh local graph-edge topology from the server.
    ///
    /// Issues `GET /patterns/graph?min_weight=<n>[&since=<ms>]` using the
    /// supplied authenticated `reqwest::Client` and `HeaderMap`.  On success
    /// every returned edge is upserted into the local `edges` table via the
    /// existing `upsert_edge` surface.  No schema changes are made.
    ///
    /// # Arguments
    /// * `http`       – Shared `reqwest::Client` (same instance as `AceClient`).
    /// * `base_url`   – Server base URL (e.g. `"https://ace-api.code-engine.app"`).
    /// * `headers`    – Pre-built auth headers (Bearer + X-ACE-Project, etc.).
    /// * `min_weight` – Minimum edge weight filter.  Defaults to **5** when
    ///   `None` (matches the `neighbors()` default and stays
    ///   safely under the 50 000-edge server cap).
    /// * `since_ms`   – Optional millisecond-epoch lower-bound forwarded as
    ///   `&since=<ms>` query param.  Omitted when `None`.
    ///
    /// # Best-effort semantics
    /// Any network failure, non-200 status, JSON parse error, or empty edge
    /// list returns `Ok(GraphRefreshResult { edges_upserted: 0, truncated: false })`
    /// — the method **never panics or propagates errors** to the caller.
    /// The local cache is left intact on every error path.
    ///
    /// When the server returns `"truncated": true` a warning is logged to
    /// stderr and `GraphRefreshResult::truncated` is set to `true`.
    pub async fn refresh_from_server(
        &self,
        http: &reqwest::Client,
        base_url: &str,
        headers: &reqwest::header::HeaderMap,
        min_weight: Option<i64>,
        since_ms: Option<i64>,
    ) -> Result<GraphRefreshResult, String> {
        let mw = min_weight.unwrap_or(5);

        let mut url = format!("{}/patterns/graph?min_weight={}", base_url, mw);
        if let Some(since) = since_ms {
            url.push_str(&format!("&since={}", since));
        }

        // Inject X-ACE-Project and X-ACE-Org from the cache's stored identity.
        // This matches exactly how AceClient::build_headers() sets these headers
        // on every authenticated request (search, traces, etc.).
        // Multi-project users get HTTP 400 without X-ACE-Project — this is the
        // fix for that production bug.
        let mut merged_headers = headers.clone();
        if let Ok(v) = reqwest::header::HeaderValue::from_str(&self.project_id) {
            merged_headers.insert("X-ACE-Project", v);
        }
        if !self.org_id.is_empty() && self.org_id != "default" {
            if let Ok(v) = reqwest::header::HeaderValue::from_str(&self.org_id) {
                merged_headers.insert("X-ACE-Org", v);
            }
        }

        // Fire the request — treat any error as best-effort (no throw).
        let response = match http.get(&url).headers(merged_headers).send().await {
            Ok(r) => r,
            Err(e) => {
                // Network failure: leave cache intact, return Ok(0).
                eprintln!("[ace-sdk] refresh_from_server: network error: {}", e);
                return Ok(GraphRefreshResult {
                    edges_upserted: 0,
                    truncated: false,
                });
            }
        };

        let status = response.status().as_u16();
        if status >= 400 {
            eprintln!(
                "[ace-sdk] refresh_from_server: server returned HTTP {}",
                status
            );
            return Ok(GraphRefreshResult {
                edges_upserted: 0,
                truncated: false,
            });
        }

        // Parse the response body.
        let body: serde_json::Value = match response.json().await {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[ace-sdk] refresh_from_server: JSON parse error: {}", e);
                return Ok(GraphRefreshResult {
                    edges_upserted: 0,
                    truncated: false,
                });
            }
        };

        let truncated = body
            .get("truncated")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if truncated {
            eprintln!(
                "[ace-sdk] refresh_from_server: WARNING — server response was truncated \
                 (>50 000 edges). Only the highest-weight edges were returned. \
                 Consider raising min_weight or using since_ms to narrow the window."
            );
        }

        let edges = match body.get("edges").and_then(|v| v.as_array()) {
            Some(arr) => arr,
            None => {
                return Ok(GraphRefreshResult {
                    edges_upserted: 0,
                    truncated,
                });
            }
        };

        if edges.is_empty() {
            return Ok(GraphRefreshResult {
                edges_upserted: 0,
                truncated,
            });
        }

        let mut count: usize = 0;
        for edge in edges {
            let src = match edge.get("src").and_then(|v| v.as_str()) {
                Some(s) => s,
                None => continue,
            };
            let dst = match edge.get("dst").and_then(|v| v.as_str()) {
                Some(s) => s,
                None => continue,
            };
            let weight = match edge.get("weight").and_then(|v| v.as_i64()) {
                Some(w) => w,
                None => continue,
            };

            if self.upsert_edge(src, dst, weight).is_ok() {
                count += 1;
            }
        }

        Ok(GraphRefreshResult {
            edges_upserted: count,
            truncated,
        })
    }

    // ── Arc-level fire-and-forget refresh (no MutexGuard across await) ───────

    /// Fire-and-forget graph-edge refresh suitable for `tokio::spawn`.
    ///
    /// Unlike [`GraphCache::refresh_from_server`] (which borrows `&self` and
    /// therefore requires the `MutexGuard` to live across the HTTP await), this
    /// function takes an `Arc<Mutex<GraphCache>>` and locks ONLY for the brief
    /// synchronous phase: building the URL, reading the DB handle is not needed
    /// up front. The HTTP request runs without any lock held; edges are written
    /// one-at-a-time via the existing `upsert_edge` surface (each write
    /// re-acquires the lock briefly).
    ///
    /// Returns `Ok(GraphRefreshResult)` or `Ok` with zero edges on any error
    /// path (best-effort semantics).
    pub async fn refresh_from_server_arc(
        gc_arc: std::sync::Arc<std::sync::Mutex<GraphCache>>,
        http: reqwest::Client,
        base_url: String,
        headers: reqwest::header::HeaderMap,
        min_weight: Option<i64>,
        since_ms: Option<i64>,
    ) -> Result<GraphRefreshResult, String> {
        let mw = min_weight.unwrap_or(5);

        let mut url = format!("{}/patterns/graph?min_weight={}", base_url, mw);
        if let Some(since) = since_ms {
            url.push_str(&format!("&since={}", since));
        }

        // Inject X-ACE-Project and X-ACE-Org from the cache's stored identity
        // (same logic as the non-Arc variant and AceClient::build_headers()).
        let mut merged_headers = headers;
        {
            // Brief lock — only to read org_id/project_id, not across await.
            if let Ok(gc) = gc_arc.lock() {
                if let Ok(v) = reqwest::header::HeaderValue::from_str(&gc.project_id) {
                    merged_headers.insert("X-ACE-Project", v);
                }
                if !gc.org_id.is_empty() && gc.org_id != "default" {
                    if let Ok(v) = reqwest::header::HeaderValue::from_str(&gc.org_id) {
                        merged_headers.insert("X-ACE-Org", v);
                    }
                }
            }
        }

        // Fire the HTTP request — no lock held here.
        let response = match http.get(&url).headers(merged_headers).send().await {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[ace-sdk] refresh_from_server_arc: network error: {}", e);
                return Ok(GraphRefreshResult {
                    edges_upserted: 0,
                    truncated: false,
                });
            }
        };

        let status = response.status().as_u16();
        if status >= 400 {
            eprintln!(
                "[ace-sdk] refresh_from_server_arc: server returned HTTP {}",
                status
            );
            return Ok(GraphRefreshResult {
                edges_upserted: 0,
                truncated: false,
            });
        }

        let body: serde_json::Value = match response.json().await {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[ace-sdk] refresh_from_server_arc: JSON parse error: {}", e);
                return Ok(GraphRefreshResult {
                    edges_upserted: 0,
                    truncated: false,
                });
            }
        };

        let truncated = body
            .get("truncated")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if truncated {
            eprintln!(
                "[ace-sdk] refresh_from_server_arc: WARNING — response truncated (>50 000 edges)"
            );
        }

        let edges = match body.get("edges").and_then(|v| v.as_array()) {
            Some(arr) => arr.clone(), // clone to release the body borrow
            None => {
                return Ok(GraphRefreshResult {
                    edges_upserted: 0,
                    truncated,
                });
            }
        };

        if edges.is_empty() {
            return Ok(GraphRefreshResult {
                edges_upserted: 0,
                truncated,
            });
        }

        // Write edges — each upsert briefly acquires and releases the lock.
        let mut count: usize = 0;
        for edge in &edges {
            let src = match edge.get("src").and_then(|v| v.as_str()) {
                Some(s) => s.to_string(),
                None => continue,
            };
            let dst = match edge.get("dst").and_then(|v| v.as_str()) {
                Some(s) => s.to_string(),
                None => continue,
            };
            let weight = match edge.get("weight").and_then(|v| v.as_i64()) {
                Some(w) => w,
                None => continue,
            };

            if let Ok(gc) = gc_arc.lock() {
                if gc.upsert_edge(&src, &dst, weight).is_ok() {
                    count += 1;
                }
            }
        }

        Ok(GraphRefreshResult {
            edges_upserted: count,
            truncated,
        })
    }

    // ── Sync-state helpers (throttle gate for graph edge refresh) ────────────

    /// Read a sync-state value by key. Returns `None` when the key is absent.
    ///
    /// Used by `AceClient::search_patterns15` to check whether the throttle
    /// window for `graph_edges_synced_at` has elapsed before calling
    /// `refresh_from_server`. Best-effort: returns `Ok(None)` on DB error.
    pub fn get_sync_state(&self, key: &str) -> Result<Option<String>, rusqlite::Error> {
        let db = self.lock()?;
        match db.query_row(
            "SELECT value FROM sync_state WHERE key = ?1",
            params![key],
            |row| row.get::<_, String>(0),
        ) {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Write a sync-state value for the given key.
    ///
    /// Used by `AceClient::search_patterns15` to stamp `graph_edges_synced_at`
    /// after a successful (fire-and-forget) `refresh_from_server` call.
    /// Best-effort: swallows all errors (consistent with cache-miss semantics).
    pub fn set_sync_state(&self, key: &str, value: &str) {
        let now = chrono::Utc::now().to_rfc3339();
        if let Ok(db) = self.lock() {
            let _ = db.execute(
                "INSERT OR REPLACE INTO sync_state (key, value, updated_at) VALUES (?1, ?2, ?3)",
                params![key, value, now],
            );
        }
    }

    // ── Test-only inspection helpers (pub for integration tests) ─────────────

    /// Count all patterns regardless of expiry (test helper).
    pub fn count_all_patterns(&self) -> Result<u64, rusqlite::Error> {
        let db = self.lock()?;
        db.query_row("SELECT COUNT(*) FROM patterns", [], |row| row.get(0))
    }

    /// Return `expires_at_ms` for a given pattern id (test helper).
    pub fn get_expires_at_ms(&self, pattern_id: &str) -> Result<Option<i64>, rusqlite::Error> {
        let db = self.lock()?;
        match db.query_row(
            "SELECT expires_at_ms FROM patterns WHERE pattern_id = ?1",
            params![pattern_id],
            |row| row.get(0),
        ) {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Return the current journal_mode (test helper for WAL verification).
    pub fn get_journal_mode(&self) -> Result<String, rusqlite::Error> {
        let db = self.lock()?;
        db.query_row("PRAGMA journal_mode", [], |row| row.get(0))
    }

    /// Return all table names in the DB (test helper for schema verification).
    pub fn list_table_names(&self) -> Result<Vec<String>, rusqlite::Error> {
        let db = self.lock()?;
        let mut stmt =
            db.prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")?;
        let names: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(names)
    }

    /// Return all index names in the DB (test helper for schema verification).
    pub fn list_index_names(&self) -> Result<Vec<String>, rusqlite::Error> {
        let db = self.lock()?;
        let mut stmt =
            db.prepare("SELECT name FROM sqlite_master WHERE type='index' ORDER BY name")?;
        let names: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(names)
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, rusqlite::Error> {
        self.db.lock().map_err(|_| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_ERROR),
                Some("GraphCache mutex poisoned".to_string()),
            )
        })
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
        let cache = LocalCacheService::new(
            "test-org",
            "test-project",
            5,
            Some(tmp.path().to_path_buf()),
        );
        assert!(cache.is_ok());
    }

    #[test]
    fn test_cache_needs_sync_when_empty() {
        let tmp = TempDir::new().unwrap();
        let cache = LocalCacheService::new(
            "test-org",
            "test-project",
            5,
            Some(tmp.path().to_path_buf()),
        )
        .unwrap();
        assert!(cache.needs_sync());
    }

    #[test]
    fn test_cache_get_playbook_empty() {
        let tmp = TempDir::new().unwrap();
        let cache = LocalCacheService::new(
            "test-org",
            "test-project",
            5,
            Some(tmp.path().to_path_buf()),
        )
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
                helpful: 5.0,
                harmful: 0.0,
                confidence: 0.9,
                observations: 10.0,
                evidence: vec!["src/main.rs".to_string()],
                created_at: "2025-01-01T00:00:00Z".to_string(),
                last_used: Some("2025-01-02T00:00:00Z".to_string()),
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
                helpful: 3.0,
                harmful: 0.0,
                confidence: 0.8,
                observations: 5.0,
                evidence: vec![],
                created_at: "2025-01-01T00:00:00Z".to_string(),
                last_used: Some("2025-01-01T00:00:00Z".to_string()),
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
            helpful: 1.0,
            harmful: 0.0,
            confidence: 0.8,
            observations: 3.0,
            evidence: vec![],
            created_at: "2025-01-01T00:00:00Z".to_string(),
            last_used: Some("2025-01-01T00:00:00Z".to_string()),
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

        storage
            .pin_session("sess-1", "error handling", &patterns, 0.7, 10)
            .unwrap();

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
        storage
            .pin_session("sess-del", "query", &patterns, 0.5, 5)
            .unwrap();

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
        storage
            .pin_session("expired-sess", "q", &patterns, 0.5, 5)
            .unwrap();

        // Manually expire it
        {
            let db = storage.db.lock().unwrap();
            db.execute(
                "UPDATE sessions SET expires_at = 0 WHERE session_id = ?1",
                params!["expired-sess"],
            )
            .unwrap();
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
