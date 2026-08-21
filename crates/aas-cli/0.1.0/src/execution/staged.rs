use crate::config::settings::ExecutionConfig;
use crate::swarm::types::*;
use chrono::Utc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{error, info, warn};

pub struct ExecutionEngine {
    config: ExecutionConfig,
    semaphore: Arc<Semaphore>,
    rollback_manager: Arc<RollbackManager>,
}

impl ExecutionEngine {
    pub fn new(config: &ExecutionConfig) -> Self {
        let max = if config.max_concurrent_actions > 0 {
            config.max_concurrent_actions as usize
        } else {
            3
        };

        ExecutionEngine {
            config: config.clone(),
            semaphore: Arc::new(Semaphore::new(max)),
            rollback_manager: Arc::new(RollbackManager::new(config.rollback_enabled)),
        }
    }

    pub async fn execute_staged(&self, action: &Action, ctx: &crate::swarm::agent::AgentContext) -> ActionResult {
        let mut action = action.clone();
        let _permit = self.semaphore.acquire().await;
        let start = std::time::Instant::now();

        action.stage = Stage::Testing;
        let test_result = self.run_stage("test", &action, ctx).await;
        if !test_result.success {
            return test_result;
        }

        action.stage = Stage::Validating;
        let validate_result = self.run_stage("validate", &action, ctx).await;
        if !validate_result.success {
            return validate_result;
        }

        action.stage = Stage::Executing;
        let exec_result = self.run_stage("execute", &action, ctx).await;
        if !exec_result.success {
            self.rollback_manager.record_failure(&action, &exec_result).await;
            return exec_result;
        }

        action.stage = Stage::Verifying;
        let verify_result = self.run_stage("verify", &action, ctx).await;
        let total_duration_ms = start.elapsed().as_millis() as u64;

        if !verify_result.success {
            warn!(
                "Verification failed for action {}, attempting rollback",
                action.id
            );
            self.rollback_manager.execute_rollback(&action, ctx).await;
            return ActionResult {
                action_id: action.id.clone(),
                success: false,
                output: "Action executed but verification failed".to_string(),
                error: Some(verify_result.error.unwrap_or_else(|| "Verification failed".to_string())),
                duration_ms: total_duration_ms,
                stage: Stage::RolledBack,
                verification_passed: false,
                timestamp: Utc::now(),
            };
        }

        info!("Action {} completed successfully through all stages", action.id);
        self.rollback_manager.record_success(&action).await;
        ActionResult {
            action_id: action.id.clone(),
            success: true,
            output: "All stages passed successfully".to_string(),
            error: None,
            duration_ms: total_duration_ms,
            stage: Stage::Completed,
            verification_passed: true,
            timestamp: Utc::now(),
        }
    }

    async fn run_stage(&self, stage_name: &str, action: &Action, ctx: &crate::swarm::agent::AgentContext) -> ActionResult {
        info!("Stage '{}' for action {}: {}", stage_name, action.id, action.description);

        ctx.event_bus
            .emit(AgentEvent::ActionStarted {
                agent: action.agent_name.clone(),
                action_id: action.id.clone(),
                stage: action.stage.clone(),
                timestamp: Utc::now(),
            })
            .await;

        match stage_name {
            "test" => self.run_test_stage(action).await,
            "validate" => self.run_validate_stage(action).await,
            "execute" => self.run_execute_stage(action).await,
            "verify" => self.run_verify_stage(action).await,
            _ => ActionResult {
                action_id: action.id.clone(),
                success: false,
                output: String::new(),
                error: Some(format!("Unknown stage: {}", stage_name)),
                duration_ms: 0,
                stage: Stage::Failed,
                verification_passed: false,
                timestamp: Utc::now(),
            },
        }
    }

    async fn run_test_stage(&self, action: &Action) -> ActionResult {
        let start = std::time::Instant::now();
        let mut output = String::new();
        let mut errors = Vec::new();

        for cmd in &action.commands {
            let test_cmd = format!("set -e; {}. If the exit code is 0, the command succeeded. If it fails, report the failure.", cmd);
            let result = tokio::process::Command::new("sh")
                .arg("-c")
                .arg("set -e; echo '--- DRY RUN ---'; echo 'Command would execute:'")
                .arg(&test_cmd)
                .output()
                .await;

            match result {
                Ok(out) => {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    output.push_str(&format!("[test] {}", stdout));
                    if !out.status.success() {
                        errors.push(format!("Test simulation failed for: {}", cmd));
                    }
                }
                Err(e) => {
                    errors.push(format!("Test runner error: {}", e));
                }
            }
        }

        for file in &action.files_to_modify {
            let path = std::path::Path::new(file);
            if path.exists() {
                output.push_str(&format!("[test] File exists (will modify): {}\n", file));
                if let Ok(meta) = path.metadata() {
                    if meta.len() > 10_000_000 {
                        errors.push(format!("File too large to modify safely: {} ({}MB)", file, meta.len() / 1_000_000));
                    }
                }
            } else {
                output.push_str(&format!("[test] New file (will create): {}\n", file));
            }
        }

        let success = errors.is_empty();
        let duration = start.elapsed().as_millis() as u64;

        info!("Test stage {} for action {}", if success { "PASSED" } else { "FAILED" }, action.id);
        ActionResult {
            action_id: action.id.clone(),
            success,
            output,
            error: if errors.is_empty() { None } else { Some(errors.join("; ")) },
            duration_ms: duration,
            stage: Stage::Testing,
            verification_passed: success,
            timestamp: Utc::now(),
        }
    }

    async fn run_validate_stage(&self, action: &Action) -> ActionResult {
        let start = std::time::Instant::now();
        let mut output = String::new();
        let mut errors = Vec::new();

        let rollback_available = !action.rollback_commands.is_empty();
        if !rollback_available {
            warn!("Action {} has no rollback commands — irreversible if it fails", action.id);
            output.push_str("[validate] No rollback commands (action is irreversible)\n");
        } else {
            output.push_str(&format!("[validate] Rollback available: {} command(s)\n", action.rollback_commands.len()));
        }

        for file in &action.files_to_modify {
            if let Ok(meta) = std::path::Path::new(file).metadata() {
                let modified = chrono::DateTime::from(meta.modified().unwrap_or(std::time::SystemTime::now()));
                let age_hours = (chrono::Utc::now() - modified).num_hours();
                output.push_str(&format!("[validate] {} (last modified {} hours ago)\n", file, age_hours));
            }
        }

        let _disk = match tokio::process::Command::new("df")
            .arg("-k")
            .arg(".")
            .output()
            .await
        {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let line = stdout.lines().nth(1).unwrap_or("");
                let fields: Vec<&str> = line.split_whitespace().collect();
                if fields.len() >= 4 {
                    let available_kb: f64 = fields[3].parse().unwrap_or(0.0);
                    output.push_str(&format!("[validate] Available disk: {:.0} MB\n", available_kb / 1024.0));
                    if available_kb < 102_400.0 {
                        errors.push(format!("Low disk space: {:.0} MB available", available_kb / 1024.0));
                    }
                    available_kb
                } else { 0.0 }
            }
            Err(_) => 0.0,
        };

        let success = errors.is_empty();
        let duration = start.elapsed().as_millis() as u64;

        info!("Validation stage {} for action {}", if success { "PASSED" } else { "FAILED" }, action.id);
        ActionResult {
            action_id: action.id.clone(),
            success,
            output,
            error: if errors.is_empty() { None } else { Some(errors.join("; ")) },
            duration_ms: duration,
            stage: Stage::Validating,
            verification_passed: success,
            timestamp: Utc::now(),
        }
    }

    async fn run_execute_stage(&self, action: &Action) -> ActionResult {
        let start = std::time::Instant::now();
        let mut output = String::new();
        let mut errors = Vec::new();
        let mut all_success = true;

        // Validate commands before execution
        for cmd in &action.commands {
            if let Some(err) = Self::validate_command(cmd) {
                errors.push(format!("Command validation failed: {}", err));
                all_success = false;
            }
        }

        if !all_success {
            return ActionResult {
                action_id: action.id.clone(),
                success: false,
                output: errors.join("\n"),
                error: Some("Command validation failed".to_string()),
                duration_ms: 0,
                stage: Stage::Executing,
                verification_passed: false,
                timestamp: Utc::now(),
            };
        }

        for cmd in &action.commands {
            info!("Executing: {}", cmd);
            output.push_str(&format!("$ {}\n", cmd));

            match tokio::process::Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .output()
                .await
            {
                Ok(out) => {
                    if !out.stdout.is_empty() {
                        output.push_str(&format!("{}\n", String::from_utf8_lossy(&out.stdout)));
                    }
                    if !out.stderr.is_empty() {
                        let stderr = String::from_utf8_lossy(&out.stderr);
                        output.push_str(&format!("stderr: {}\n", stderr));
                        if !out.status.success() {
                            errors.push(stderr.trim().to_string());
                            all_success = false;
                        }
                    }
                    if !out.status.success() && out.stdout.is_empty() {
                        errors.push(format!("Exit code: {}", out.status));
                        all_success = false;
                    }
                }
                Err(e) => {
                    errors.push(format!("Execution error: {}", e));
                    all_success = false;
                }
            }
        }

        let duration = start.elapsed().as_millis() as u64;
        info!("Execution stage {} for action {}", if all_success { "COMPLETED" } else { "FAILED" }, action.id);

        ActionResult {
            action_id: action.id.clone(),
            success: all_success,
            output,
            error: if errors.is_empty() { None } else { Some(errors.join("\n")) },
            duration_ms: duration,
            stage: Stage::Executing,
            verification_passed: all_success,
            timestamp: Utc::now(),
        }
    }

    async fn run_verify_stage(&self, action: &Action) -> ActionResult {
        let start = std::time::Instant::now();
        let mut output = String::new();
        let mut errors = Vec::new();

        for cmd in &action.commands {
            let verify_cmd = format!("{} 2>&1; echo \"EXIT:$?\"", cmd);
            match tokio::process::Command::new("sh")
                .arg("-c")
                .arg(&verify_cmd)
                .output()
                .await
            {
                Ok(out) => {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    let exit_code_line = stdout.lines().last().unwrap_or("");
                    let exit_code = exit_code_line.strip_prefix("EXIT:").and_then(|s| s.parse::<i32>().ok()).unwrap_or(-1);
                    output.push_str(&format!("[verify] Command exit code: {}\n", exit_code));
                    if exit_code != 0 {
                        errors.push(format!("Verification failed for command (exit {}): {}", exit_code, cmd));
                    }
                }
                Err(e) => {
                    errors.push(format!("Verification error: {}", e));
                }
            }
        }

        for file in &action.files_to_modify {
            let path = std::path::Path::new(file);
            output.push_str(&format!("[verify] File {}: {}\n", file, if path.exists() { "exists" } else { "missing" }));
        }

        if errors.is_empty() {
            output.push_str("[verify] All verifications passed\n");
        }

        let success = errors.is_empty();
        let duration = start.elapsed().as_millis() as u64;

        info!("Verification stage {} for action {}", if success { "PASSED" } else { "FAILED" }, action.id);
        ActionResult {
            action_id: action.id.clone(),
            success,
            output,
            error: if errors.is_empty() { None } else { Some(errors.join("; ")) },
            duration_ms: duration,
            stage: Stage::Verifying,
            verification_passed: success,
            timestamp: Utc::now(),
        }
    }

    pub fn rollback_manager(&self) -> Arc<RollbackManager> {
        self.rollback_manager.clone()
    }

    pub fn requires_approval(&self, action_type: &str) -> bool {
        self.config.approval_required_for.contains(&action_type.to_string())
    }

    fn validate_command(cmd: &str) -> Option<String> {
        let danger_patterns = [
            "rm -rf /",
            "dd if=",
            ":(){:|:&};:",
            "DROP TABLE",
            "DELETE FROM",
            "TRUNCATE TABLE",
            "mkfs",
            "shred",
        ];

        for pattern in &danger_patterns {
            if cmd.contains(pattern) {
                return Some(format!("Dangerous pattern detected: {}", pattern));
            }
        }

        // Warn on potentially risky but allowed operations
        if cmd.contains("rm -rf") || cmd.contains("sudo") {
            warn!("Risky command allowed (with validation): {}", cmd);
        }

        None
    }
}

pub struct RollbackManager {
    enabled: bool,
    rollback_in_progress: Arc<AtomicBool>,
}

impl RollbackManager {
    pub fn new(enabled: bool) -> Self {
        RollbackManager {
            enabled,
            rollback_in_progress: Arc::new(AtomicBool::new(false)),
        }
    }

    pub async fn record_success(&self, _action: &Action) {
        info!("Action recorded as successful, rollback available");
    }

    pub async fn record_failure(&self, _action: &Action, _result: &ActionResult) {
        warn!(
            "Action failed at stage {:?}",
            _result.stage
        );
    }

    pub async fn execute_rollback(&self, action: &Action, _ctx: &crate::swarm::agent::AgentContext) {
        if !self.enabled {
            warn!("Rollback disabled, cannot rollback action {}", action.id);
            return;
        }

        self.rollback_in_progress.store(true, Ordering::SeqCst);
        info!("Executing rollback for action {}: {}", action.id, action.description);

        for cmd in &action.rollback_commands {
            info!("Rollback: executing '{}'", cmd);
            match tokio::process::Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .output()
                .await
            {
                Ok(out) => {
                    if !out.status.success() {
                        error!(
                            "Rollback command failed: {} - {}",
                            cmd,
                            String::from_utf8_lossy(&out.stderr)
                        );
                    }
                }
                Err(e) => {
                    error!("Failed to execute rollback command '{}': {}", cmd, e);
                }
            }
        }

        _ctx.event_bus
            .emit(AgentEvent::ActionRolledBack {
                agent: action.agent_name.clone(),
                action_id: action.id.clone(),
                timestamp: Utc::now(),
            })
            .await;

        info!("Rollback completed for action {}", action.id);
        self.rollback_in_progress.store(false, Ordering::SeqCst);
    }

    pub fn is_rolling_back(&self) -> bool {
        self.rollback_in_progress.load(Ordering::SeqCst)
    }
}
