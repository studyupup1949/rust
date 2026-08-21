use crate::swarm::types::*;
use chrono::Utc;
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};

pub struct MemoryStore {
    conn: Arc<Mutex<Connection>>,
}

impl MemoryStore {
    pub async fn new(db_path: &Path) -> Result<Self, String> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create database directory: {}", e))?;
        }
        let conn = Connection::open(db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?;

        let store = MemoryStore {
            conn: Arc::new(Mutex::new(conn)),
        };
        store.initialize().await?;
        Ok(store)
    }

    async fn initialize(&self) -> Result<(), String> {
        let conn = self.conn.lock().await;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS issues (
                id TEXT PRIMARY KEY,
                domain TEXT NOT NULL,
                agent_name TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT NOT NULL,
                severity TEXT NOT NULL,
                source TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                metadata TEXT NOT NULL DEFAULT '{}',
                signature TEXT NOT NULL DEFAULT '',
                stage TEXT NOT NULL DEFAULT 'detected',
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS analyses (
                id TEXT PRIMARY KEY,
                issue_id TEXT NOT NULL,
                agent_name TEXT NOT NULL,
                root_cause TEXT NOT NULL,
                impact TEXT NOT NULL,
                approaches TEXT NOT NULL DEFAULT '[]',
                confidence REAL NOT NULL DEFAULT 0.0,
                reasoning TEXT NOT NULL DEFAULT '',
                timestamp TEXT NOT NULL,
                FOREIGN KEY (issue_id) REFERENCES issues(id)
            );

            CREATE TABLE IF NOT EXISTS actions (
                id TEXT PRIMARY KEY,
                issue_id TEXT NOT NULL,
                agent_name TEXT NOT NULL,
                description TEXT NOT NULL,
                commands TEXT NOT NULL DEFAULT '[]',
                rollback_commands TEXT NOT NULL DEFAULT '[]',
                files TEXT NOT NULL DEFAULT '[]',
                stage TEXT NOT NULL DEFAULT 'planned',
                confidence REAL NOT NULL DEFAULT 0.0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (issue_id) REFERENCES issues(id)
            );

            CREATE TABLE IF NOT EXISTS action_results (
                id TEXT PRIMARY KEY,
                action_id TEXT NOT NULL,
                success INTEGER NOT NULL DEFAULT 0,
                output TEXT NOT NULL DEFAULT '',
                error TEXT,
                duration_ms INTEGER NOT NULL DEFAULT 0,
                stage TEXT NOT NULL,
                verification_passed INTEGER NOT NULL DEFAULT 0,
                timestamp TEXT NOT NULL,
                FOREIGN KEY (action_id) REFERENCES actions(id)
            );

            CREATE TABLE IF NOT EXISTS decisions (
                id TEXT PRIMARY KEY,
                status TEXT NOT NULL DEFAULT 'detected',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                summary TEXT NOT NULL DEFAULT ''
            );

            CREATE TABLE IF NOT EXISTS patterns (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                domain TEXT NOT NULL,
                indicators TEXT NOT NULL DEFAULT '[]',
                solution_description TEXT NOT NULL DEFAULT '',
                confidence REAL NOT NULL DEFAULT 0.0,
                occurrences INTEGER NOT NULL DEFAULT 1,
                first_seen TEXT NOT NULL,
                last_seen TEXT NOT NULL,
                avg_execution_time_ms INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS predictions (
                id TEXT PRIMARY KEY,
                agent_name TEXT NOT NULL,
                predicted_issue TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                confidence REAL NOT NULL DEFAULT 0.0,
                time_until_expected TEXT NOT NULL DEFAULT '',
                suggested_action TEXT NOT NULL DEFAULT '',
                based_on_pattern TEXT,
                created_at TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'active'
            );

            CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_type TEXT NOT NULL,
                agent_name TEXT NOT NULL DEFAULT '',
                data TEXT NOT NULL DEFAULT '{}',
                timestamp TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS cycle_performance (
                id TEXT PRIMARY KEY,
                agent_name TEXT NOT NULL,
                cycle_duration_ms INTEGER NOT NULL,
                issues_found INTEGER NOT NULL,
                actions_attempted INTEGER NOT NULL,
                actions_succeeded INTEGER NOT NULL,
                confidence_threshold REAL NOT NULL,
                timestamp TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_issues_domain ON issues(domain);
            CREATE INDEX IF NOT EXISTS idx_issues_agent ON issues(agent_name);
            CREATE INDEX IF NOT EXISTS idx_issues_severity ON issues(severity);
            CREATE INDEX IF NOT EXISTS idx_actions_agent ON actions(agent_name);
            CREATE INDEX IF NOT EXISTS idx_actions_issue ON actions(issue_id);
            CREATE INDEX IF NOT EXISTS idx_patterns_domain ON patterns(domain);
            CREATE INDEX IF NOT EXISTS idx_predictions_agent ON predictions(agent_name);
            CREATE INDEX IF NOT EXISTS idx_events_type ON events(event_type);
            CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events(timestamp);
            ",
        )
        .map_err(|e| format!("Failed to initialize database: {}", e))?;

        info!("Database initialized successfully");
        Ok(())
    }

    pub async fn store_issue(&self, issue: &Issue) {
        let conn = self.conn.lock().await;
        let metadata = serde_json::to_string(&issue.metadata).unwrap_or_default();
        let result = conn.execute(
            "INSERT OR REPLACE INTO issues (id, domain, agent_name, title, description, severity, source, timestamp, metadata, signature, stage)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                issue.id,
                issue.domain.to_string(),
                issue.agent_name,
                issue.title,
                issue.description,
                issue.severity.to_string(),
                issue.source,
                issue.timestamp.to_rfc3339(),
                metadata,
                issue.signature,
                issue.stage.to_string(),
            ],
        );
        if let Err(e) = result {
            error!("Failed to store issue: {}", e);
        }
    }

    pub async fn store_analysis(&self, analysis: &crate::swarm::types::Analysis) {
        let conn = self.conn.lock().await;
        let approaches = serde_json::to_string(&analysis.suggested_approaches).unwrap_or_default();
        let result = conn.execute(
            "INSERT OR REPLACE INTO analyses (id, issue_id, agent_name, root_cause, impact, approaches, confidence, reasoning, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                uuid::Uuid::new_v4().to_string(),
                analysis.issue_id,
                analysis.agent_name,
                analysis.root_cause,
                analysis.impact,
                approaches,
                analysis.confidence,
                analysis.reasoning,
                chrono::Utc::now().to_rfc3339(),
            ],
        );
        if let Err(e) = result {
            error!("Failed to store analysis: {}", e);
        }
    }

    pub async fn store_action(&self, action: &Action) {
        let conn = self.conn.lock().await;
        let commands = serde_json::to_string(&action.commands).unwrap_or_default();
        let rollback = serde_json::to_string(&action.rollback_commands).unwrap_or_default();
        let files = serde_json::to_string(&action.files_to_modify).unwrap_or_default();
        let result = conn.execute(
            "INSERT OR REPLACE INTO actions (id, issue_id, agent_name, description, commands, rollback_commands, files, stage, confidence, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                action.id,
                action.issue_id,
                action.agent_name,
                action.description,
                commands,
                rollback,
                files,
                action.stage.to_string(),
                action.confidence,
                action.created_at.to_rfc3339(),
                action.updated_at.to_rfc3339(),
            ],
        );
        if let Err(e) = result {
            error!("Failed to store action: {}", e);
        }
    }

    pub async fn store_decision(&self, decision: &Decision) {
        let conn = self.conn.lock().await;
        let summary = format!(
            "[{}] {} - {}",
            decision.issue.domain,
            decision.issue.title,
            decision.status
        );
        let result = conn.execute(
            "INSERT OR REPLACE INTO decisions (id, status, created_at, updated_at, summary)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                decision.id,
                decision.status.to_string(),
                decision.created_at.to_rfc3339(),
                decision.updated_at.to_rfc3339(),
                summary,
            ],
        );
        if let Err(e) = result {
            error!("Failed to store decision: {}", e);
        }
    }

    pub async fn store_pattern(&self, pattern: &Pattern) {
        let conn = self.conn.lock().await;
        let indicators = serde_json::to_string(&pattern.indicators).unwrap_or_default();
        let result = conn.execute(
            "INSERT OR REPLACE INTO patterns (id, name, description, domain, indicators, solution_description, confidence, occurrences, first_seen, last_seen, avg_execution_time_ms)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                pattern.id,
                pattern.name,
                pattern.description,
                pattern.domain.to_string(),
                indicators,
                pattern.solution_description,
                pattern.confidence,
                pattern.occurrences,
                pattern.first_seen.to_rfc3339(),
                pattern.last_seen.to_rfc3339(),
                pattern.avg_execution_time_ms,
            ],
        );
        if let Err(e) = result {
            error!("Failed to store pattern: {}", e);
        }
    }

    pub async fn store_prediction(&self, prediction: &Prediction) {
        let conn = self.conn.lock().await;
        let result = conn.execute(
            "INSERT OR REPLACE INTO predictions (id, agent_name, predicted_issue, description, confidence, time_until_expected, suggested_action, based_on_pattern, created_at, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                prediction.id,
                prediction.agent_name,
                prediction.predicted_issue,
                prediction.description,
                prediction.confidence,
                prediction.time_until_expected,
                prediction.suggested_action,
                prediction.based_on_pattern,
                prediction.created_at.to_rfc3339(),
                prediction.status.to_string(),
            ],
        );
        if let Err(e) = result {
            error!("Failed to store prediction: {}", e);
        }
    }

    pub async fn log_event(&self, event_type: &str, agent: &str, data: &str) {
        let conn = self.conn.lock().await;
        let _ = conn.execute(
            "INSERT INTO events (event_type, agent_name, data, timestamp) VALUES (?1, ?2, ?3, ?4)",
            params![event_type, agent, data, Utc::now().to_rfc3339()],
        );
    }

    pub async fn get_recent_issues(&self, limit: usize) -> Vec<Issue> {
        let conn = self.conn.lock().await;
        let mut stmt = conn
            .prepare(
                "SELECT id, domain, agent_name, title, description, severity, source, timestamp, metadata, signature, stage
                 FROM issues ORDER BY timestamp DESC LIMIT ?1",
            )
            .unwrap();

        let issues = stmt
            .query_map(params![limit as i64], |row| {
                let metadata_str: String = row.get(8)?;
                let metadata: std::collections::HashMap<String, String> =
                    serde_json::from_str(&metadata_str).unwrap_or_default();
                Ok(Issue {
                    id: row.get(0)?,
                    domain: row.get::<_, String>(1)?.parse().unwrap_or(Domain::Repository),
                    agent_name: row.get(2)?,
                    title: row.get(3)?,
                    description: row.get(4)?,
                    severity: row.get::<_, String>(5)?.parse().unwrap_or(Severity::Info),
                    source: row.get(6)?,
                    timestamp: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                        .map(|d| d.to_utc())
                        .unwrap_or_else(|_| Utc::now()),
                    metadata,
                    signature: row.get(9)?,
                    stage: row.get::<_, String>(10)?.parse().unwrap_or(Stage::Detected),
                })
            })
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        issues
    }

    pub async fn get_recent_issues_for_agent(&self, agent_name: &str, limit: usize) -> Vec<Issue> {
        let conn = self.conn.lock().await;
        let mut stmt = conn
            .prepare(
                "SELECT id, domain, agent_name, title, description, severity, source, timestamp, metadata, signature, stage
                 FROM issues WHERE agent_name = ?1 ORDER BY timestamp DESC LIMIT ?2",
            )
            .unwrap();

        let issues = stmt
            .query_map(params![agent_name, limit as i64], |row| {
                let metadata_str: String = row.get(8)?;
                let metadata: std::collections::HashMap<String, String> =
                    serde_json::from_str(&metadata_str).unwrap_or_default();
                Ok(Issue {
                    id: row.get(0)?,
                    domain: row.get::<_, String>(1)?.parse().unwrap_or(Domain::Repository),
                    agent_name: row.get(2)?,
                    title: row.get(3)?,
                    description: row.get(4)?,
                    severity: row.get::<_, String>(5)?.parse().unwrap_or(Severity::Info),
                    source: row.get(6)?,
                    timestamp: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                        .map(|d| d.to_utc())
                        .unwrap_or_else(|_| Utc::now()),
                    metadata,
                    signature: row.get(9)?,
                    stage: row.get::<_, String>(10)?.parse().unwrap_or(Stage::Detected),
                })
            })
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        issues
    }

    pub async fn get_recent_actions_for_agent(
        &self,
        agent_name: &str,
        limit: usize,
    ) -> Vec<(Action, ActionResult)> {
        let conn = self.conn.lock().await;
        let mut stmt = conn
            .prepare(
                "SELECT a.id, a.issue_id, a.agent_name, a.description, a.commands, a.rollback_commands, a.files, a.stage, a.confidence, a.created_at, a.updated_at,
                        ar.success, ar.output, ar.error, ar.duration_ms, ar.stage, ar.verification_passed, ar.timestamp
                 FROM actions a
                 LEFT JOIN action_results ar ON ar.action_id = a.id
                 WHERE a.agent_name = ?1
                 ORDER BY a.created_at DESC LIMIT ?2",
            )
            .unwrap();

        let results = stmt
            .query_map(params![agent_name, limit as i64], |row| {
                let commands_str: String = row.get(4)?;
                let commands: Vec<String> =
                    serde_json::from_str(&commands_str).unwrap_or_default();
                let rollback_str: String = row.get(5)?;
                let rollback: Vec<String> =
                    serde_json::from_str(&rollback_str).unwrap_or_default();
                let files_str: String = row.get(6)?;
                let files: Vec<String> =
                    serde_json::from_str(&files_str).unwrap_or_default();

                let action = Action {
                    id: row.get(0)?,
                    issue_id: row.get(1)?,
                    agent_name: row.get(2)?,
                    approach_name: String::new(),
                    description: row.get(3)?,
                    commands,
                    rollback_commands: rollback,
                    files_to_modify: files,
                    stage: row.get::<_, String>(7)?.parse().unwrap_or(Stage::Detected),
                    confidence: row.get(8)?,
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(9)?)
                        .map(|d| d.to_utc())
                        .unwrap_or_else(|_| Utc::now()),
                    updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(10)?)
                        .map(|d| d.to_utc())
                        .unwrap_or_else(|_| Utc::now()),
                };

                let result = ActionResult {
                    action_id: row.get::<_, String>(0).unwrap_or_default(),
                    success: row.get::<_, i32>(11).unwrap_or(0) != 0,
                    output: row.get::<_, String>(12).unwrap_or_default(),
                    error: row.get(13)?,
                    duration_ms: row.get::<_, i64>(14).unwrap_or(0) as u64,
                    stage: row.get::<_, String>(15).unwrap_or_default().parse().unwrap_or(Stage::Detected),
                    verification_passed: row.get::<_, i32>(16).unwrap_or(0) != 0,
                    timestamp: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(17).unwrap_or_default())
                        .map(|d| d.to_utc())
                        .unwrap_or_else(|_| Utc::now()),
                };

                Ok((action, result))
            })
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        results
    }

    pub async fn find_similar_pattern(&self, signature: &str, domain: &Domain) -> Option<Pattern> {
        let conn = self.conn.lock().await;
        let domain_str = domain.to_string();
        let mut stmt = conn
            .prepare(
                "SELECT id, name, description, domain, indicators, solution_description, confidence, occurrences, first_seen, last_seen, avg_execution_time_ms
                 FROM patterns
                 WHERE domain = ?1 AND confidence > 0.3
                 ORDER BY
                   CASE WHEN indicators LIKE ?2 THEN confidence * 1.5
                        WHEN ?2 LIKE '%' || indicators || '%' THEN confidence * 1.3
                        ELSE confidence
                   END DESC
                 LIMIT 3",
            )
            .ok()?;

        let sig_pattern = format!("%{}%", signature.chars().take(50).collect::<String>());
        let results: Vec<Pattern> = stmt
            .query_map(params![domain_str, sig_pattern], |row| {
                let indicators_str: String = row.get(4)?;
                let indicators: Vec<String> =
                    serde_json::from_str(&indicators_str).unwrap_or_default();
                Ok(Pattern {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    domain: row.get::<_, String>(3)?.parse().unwrap_or(Domain::Repository),
                    indicators,
                    solution_description: row.get(5)?,
                    confidence: row.get(6)?,
                    occurrences: row.get::<_, i64>(7).unwrap_or(0) as u32,
                    first_seen: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                        .map(|d| d.to_utc())
                        .unwrap_or_else(|_| Utc::now()),
                    last_seen: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(9)?)
                        .map(|d| d.to_utc())
                        .unwrap_or_else(|_| Utc::now()),
                    avg_execution_time_ms: row.get::<_, i64>(10).unwrap_or(0) as u64,
                })
            })
            .ok()?
            .filter_map(|r| r.ok())
            .collect();

        results.into_iter().next()
    }

    pub async fn get_patterns(&self, domain: Option<&str>) -> Vec<Pattern> {
        let conn = self.conn.lock().await;

        let query = if let Some(_) = domain {
            "SELECT id, name, description, domain, indicators, solution_description, confidence, occurrences, first_seen, last_seen, avg_execution_time_ms
             FROM patterns WHERE domain = ?1 ORDER BY confidence DESC"
        } else {
            "SELECT id, name, description, domain, indicators, solution_description, confidence, occurrences, first_seen, last_seen, avg_execution_time_ms
             FROM patterns ORDER BY confidence DESC"
        };

        let mut stmt = conn.prepare(query).unwrap();

        let patterns: Vec<Pattern> = if let Some(d) = domain {
            stmt.query_map(params![d], |row| map_pattern(row))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect()
        } else {
            stmt.query_map([], |row| map_pattern(row))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect()
        };

        patterns
    }

    pub async fn get_predictions(&self, agent: Option<&str>, status: Option<&str>) -> Vec<Prediction> {
        let conn = self.conn.lock().await;

        let query = match (agent, status) {
            (Some(_a), Some(_s)) => {
                "SELECT id, agent_name, predicted_issue, description, confidence, time_until_expected, suggested_action, based_on_pattern, created_at, status
                 FROM predictions WHERE agent_name = ?1 AND status = ?2 ORDER BY confidence DESC"
            }
            (Some(_a), None) => {
                "SELECT id, agent_name, predicted_issue, description, confidence, time_until_expected, suggested_action, based_on_pattern, created_at, status
                 FROM predictions WHERE agent_name = ?1 ORDER BY confidence DESC"
            }
            (None, Some(_s)) => {
                "SELECT id, agent_name, predicted_issue, description, confidence, time_until_expected, suggested_action, based_on_pattern, created_at, status
                 FROM predictions WHERE status = ?1 ORDER BY confidence DESC"
            }
            (None, None) => {
                "SELECT id, agent_name, predicted_issue, description, confidence, time_until_expected, suggested_action, based_on_pattern, created_at, status
                 FROM predictions ORDER BY confidence DESC"
            }
        };

        let mut stmt = conn.prepare(query).unwrap();

        let predictions: Vec<Prediction> = match (agent, status) {
            (Some(a), Some(s)) => stmt
                .query_map(params![a, s], |row| map_prediction(row))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect(),
            (Some(a), None) => stmt
                .query_map(params![a], |row| map_prediction(row))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect(),
            (None, Some(s)) => stmt
                .query_map(params![s], |row| map_prediction(row))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect(),
            (None, None) => stmt
                .query_map([], |row| map_prediction(row))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect(),
        };

        predictions
    }

    pub async fn get_decisions(
        &self,
        agent: Option<&str>,
        limit: usize,
        status: Option<&str>,
    ) -> Vec<Decision> {
        let _conn = self.conn.lock().await;

        let recent_issues = match agent {
            Some(name) => self.get_recent_issues_for_agent(name, limit).await,
            None => self.get_recent_issues(limit).await,
        };

        let mut decisions = Vec::new();
        for issue in &recent_issues {
            let status_str = status.unwrap_or("");
            let decision_status = if status_str.is_empty() {
                DecisionStatus::Completed
            } else {
                match status_str {
                    "detected" => DecisionStatus::Detected,
                    "analyzing" => DecisionStatus::Analyzing,
                    "awaiting_approval" => DecisionStatus::AwaitingApproval,
                    "in_progress" => DecisionStatus::InProgress,
                    "completed" => DecisionStatus::Completed,
                    "failed" => DecisionStatus::Failed,
                    "rolled_back" => DecisionStatus::RolledBack,
                    "rejected" => DecisionStatus::Rejected,
                    _ => DecisionStatus::Completed,
                }
            };

            decisions.push(Decision {
                id: issue.id.clone(),
                issue: issue.clone(),
                analysis: None,
                action: None,
                result: None,
                status: decision_status,
                created_at: issue.timestamp,
                updated_at: issue.timestamp,
            });
        }

        decisions
    }

    pub async fn get_agent_stats(&self, agent_name: &str) -> (u64, u64, f64) {
        let issues = self.get_recent_issues_for_agent(agent_name, 1000).await;
        let actions = self.get_recent_actions_for_agent(agent_name, 1000).await;

        let total = actions.len() as u64;
        let successful = actions.iter().filter(|(_, r)| r.success).count() as u64;
        let rate = if total > 0 {
            successful as f64 / total as f64 * 100.0
        } else {
            100.0
        };

        (issues.len() as u64, total, rate)
    }

    pub async fn record_cycle(&self, perf: &crate::swarm::types::CyclePerformance) {
        let conn = self.conn.lock().await;
        let result = conn.execute(
            "INSERT INTO cycle_performance (id, agent_name, cycle_duration_ms, issues_found, actions_attempted, actions_succeeded, confidence_threshold, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                perf.id,
                perf.agent_name,
                perf.cycle_duration_ms,
                perf.issues_found,
                perf.actions_attempted,
                perf.actions_succeeded,
                perf.confidence_threshold,
                perf.timestamp.to_rfc3339(),
            ],
        );
        if let Err(e) = result {
            error!("Failed to record cycle performance: {}", e);
        }
    }

    pub async fn get_cycle_history(&self, agent: &str, limit: usize) -> Vec<crate::swarm::types::CyclePerformance> {
        let conn = self.conn.lock().await;
        let mut stmt = match conn.prepare(
            "SELECT id, agent_name, cycle_duration_ms, issues_found, actions_attempted, actions_succeeded, confidence_threshold, timestamp
             FROM cycle_performance
             WHERE agent_name = ?1
             ORDER BY timestamp DESC
             LIMIT ?2"
        ) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to prepare cycle history query: {}", e);
                return Vec::new();
            }
        };

        let cycles = match stmt.query_map(params![agent, limit as i64], |row| {
            Ok(crate::swarm::types::CyclePerformance {
                id: row.get(0)?,
                agent_name: row.get(1)?,
                cycle_duration_ms: row.get::<_, i64>(2).unwrap_or(0) as u64,
                issues_found: row.get::<_, i64>(3).unwrap_or(0) as u32,
                actions_attempted: row.get::<_, i64>(4).unwrap_or(0) as u32,
                actions_succeeded: row.get::<_, i64>(5).unwrap_or(0) as u32,
                confidence_threshold: row.get(6)?,
                timestamp: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                    .map(|d| d.to_utc())
                    .unwrap_or_else(|_| Utc::now()),
            })
        }) {
            Ok(i) => i,
            Err(e) => {
                error!("Failed to query cycle history: {}", e);
                return Vec::new();
            }
        };

        cycles.filter_map(|c| c.ok()).collect()
    }

    pub async fn get_agent_success_rate(&self, agent: &str, window: usize) -> f64 {
        let conn = self.conn.lock().await;
        let mut stmt = match conn.prepare(
            "SELECT actions_attempted, actions_succeeded
             FROM cycle_performance
             WHERE agent_name = ?1
             ORDER BY timestamp DESC
             LIMIT ?2"
        ) {
            Ok(s) => s,
            Err(_) => return 0.5, // default confidence
        };

        let rows = match stmt.query_map(params![agent, window as i64], |row| {
            Ok((row.get::<_, i64>(0).unwrap_or(0), row.get::<_, i64>(1).unwrap_or(0)))
        }) {
            Ok(r) => r,
            Err(_) => return 0.5,
        };

        let mut total_attempted = 0i64;
        let mut total_succeeded = 0i64;

        for row in rows {
            if let Ok((attempted, succeeded)) = row {
                total_attempted += attempted;
                total_succeeded += succeeded;
            }
        }

        if total_attempted == 0 {
            0.5 // default confidence when no history
        } else {
            (total_succeeded as f64 / total_attempted as f64).max(0.0).min(1.0)
        }
    }
}

fn map_pattern(row: &rusqlite::Row) -> rusqlite::Result<Pattern> {
    let indicators_str: String = row.get(4)?;
    let indicators: Vec<String> = serde_json::from_str(&indicators_str).unwrap_or_default();
    Ok(Pattern {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        domain: row.get::<_, String>(3)?.parse().unwrap_or(Domain::Repository),
        indicators,
        solution_description: row.get(5)?,
        confidence: row.get(6)?,
        occurrences: row.get::<_, i64>(7).unwrap_or(0) as u32,
        first_seen: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
            .map(|d| d.to_utc())
            .unwrap_or_else(|_| Utc::now()),
        last_seen: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(9)?)
            .map(|d| d.to_utc())
            .unwrap_or_else(|_| Utc::now()),
        avg_execution_time_ms: row.get::<_, i64>(10).unwrap_or(0) as u64,
    })
}

fn map_prediction(row: &rusqlite::Row) -> rusqlite::Result<Prediction> {
    let status_str: String = row.get(9)?;
    let status = match status_str.as_str() {
        "active" => PredictionStatus::Active,
        "averted" => PredictionStatus::Averted,
        "occurred" => PredictionStatus::Occurred,
        _ => PredictionStatus::Expired,
    };

    Ok(Prediction {
        id: row.get(0)?,
        agent_name: row.get(1)?,
        predicted_issue: row.get(2)?,
        description: row.get(3)?,
        confidence: row.get(4)?,
        time_until_expected: row.get(5)?,
        suggested_action: row.get(6)?,
        based_on_pattern: row.get(7)?,
        created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
            .map(|d| d.to_utc())
            .unwrap_or_else(|_| Utc::now()),
        status,
    })
}

impl std::str::FromStr for Domain {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "repository" => Ok(Domain::Repository),
            "logs" => Ok(Domain::Logs),
            "metrics" => Ok(Domain::Metrics),
            "health" => Ok(Domain::Health),
            "task" => Ok(Domain::Task),
            "trace" => Ok(Domain::Trace),
            _ => Err(format!("Unknown domain: {}", s)),
        }
    }
}

impl std::str::FromStr for Severity {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "critical" => Ok(Severity::Critical),
            "high" => Ok(Severity::High),
            "medium" => Ok(Severity::Medium),
            "low" => Ok(Severity::Low),
            "info" => Ok(Severity::Info),
            _ => Ok(Severity::Info),
        }
    }
}

impl std::str::FromStr for Stage {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "detected" => Ok(Stage::Detected),
            "analyzing" => Ok(Stage::Analyzing),
            "planned" => Ok(Stage::Planned),
            "testing" => Ok(Stage::Testing),
            "validating" => Ok(Stage::Validating),
            "executing" => Ok(Stage::Executing),
            "verifying" => Ok(Stage::Verifying),
            "completed" => Ok(Stage::Completed),
            "failed" => Ok(Stage::Failed),
            "rolled_back" => Ok(Stage::RolledBack),
            _ => Ok(Stage::Detected),
        }
    }
}
