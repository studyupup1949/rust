//! Cron task scheduler
//!
//! Provides background task scheduling and execution management.

use crate::parser::CronExpression;
use crate::store::{CronStore, FileCronStore};
use crate::telemetry;
use crate::types::{
    AgentExecutor, AgentJobConfig, CronError, CronJob, JobExecution, JobStatus, JobType, Result,
};
use chrono::Utc;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use tokio::process::Command;
use tokio::sync::{broadcast, RwLock};
use tokio::time::{interval, Duration};

/// Scheduler events for monitoring
#[derive(Debug, Clone)]
pub enum SchedulerEvent {
    /// Scheduler started
    Started,
    /// Scheduler stopped
    Stopped,
    /// Job started execution
    JobStarted {
        job_id: String,
        execution_id: String,
    },
    /// Job completed successfully
    JobCompleted {
        job_id: String,
        execution_id: String,
    },
    /// Job failed
    JobFailed {
        job_id: String,
        execution_id: String,
        error: String,
    },
    /// Job timed out
    JobTimeout {
        job_id: String,
        execution_id: String,
    },
}

/// Cron manager for job scheduling and execution
pub struct CronManager {
    /// Storage backend
    store: Arc<dyn CronStore>,
    /// Event broadcaster
    event_tx: broadcast::Sender<SchedulerEvent>,
    /// Scheduler running flag
    running: Arc<RwLock<bool>>,
    /// Workspace directory
    workspace: String,
    /// Optional agent executor for agent-mode jobs
    agent_executor: Option<Arc<dyn AgentExecutor>>,
}

impl CronManager {
    /// Create a new cron manager with file-based storage
    pub async fn new<P: AsRef<Path>>(workspace: P) -> Result<Self> {
        let workspace_str = workspace.as_ref().to_string_lossy().to_string();
        let store = Arc::new(FileCronStore::new(&workspace_str).await?);
        let (event_tx, _) = broadcast::channel(100);

        Ok(Self {
            store,
            event_tx,
            running: Arc::new(RwLock::new(false)),
            workspace: workspace_str,
            agent_executor: None,
        })
    }

    /// Create a cron manager with a custom store
    pub fn with_store(store: Arc<dyn CronStore>, workspace: String) -> Self {
        let (event_tx, _) = broadcast::channel(100);
        Self {
            store,
            event_tx,
            running: Arc::new(RwLock::new(false)),
            workspace,
            agent_executor: None,
        }
    }

    /// Set the agent executor for agent-mode cron jobs.
    pub fn set_agent_executor(&mut self, executor: Arc<dyn AgentExecutor>) {
        self.agent_executor = Some(executor);
    }

    /// Subscribe to scheduler events
    pub fn subscribe(&self) -> broadcast::Receiver<SchedulerEvent> {
        self.event_tx.subscribe()
    }

    /// Add a new cron job
    pub async fn add_job(&self, name: &str, schedule: &str, command: &str) -> Result<CronJob> {
        // Validate schedule
        let expr = CronExpression::parse(schedule)?;

        // Check for duplicate name
        if self.store.find_job_by_name(name).await?.is_some() {
            return Err(CronError::JobExists(name.to_string()));
        }

        // Create job
        let mut job = CronJob::new(name, schedule, command);
        job.next_run = expr.next_after(Utc::now());
        job.working_dir = Some(self.workspace.clone());

        // Save
        self.store.save_job(&job).await?;

        tracing::info!("Added cron job: {} ({})", job.name, job.id);
        Ok(job)
    }

    /// Add a new agent-mode cron job.
    ///
    /// The `command` field is used as the agent prompt. When the job fires,
    /// it creates a temporary Agent with the given config and sends the prompt.
    pub async fn add_agent_job(
        &self,
        name: &str,
        schedule: &str,
        prompt: &str,
        config: AgentJobConfig,
    ) -> Result<CronJob> {
        let expr = CronExpression::parse(schedule)?;

        if self.store.find_job_by_name(name).await?.is_some() {
            return Err(CronError::JobExists(name.to_string()));
        }

        let mut job = CronJob::new(name, schedule, prompt);
        job.job_type = JobType::Agent;
        job.agent_config = Some(config);
        job.next_run = expr.next_after(Utc::now());
        job.working_dir = Some(self.workspace.clone());

        self.store.save_job(&job).await?;

        tracing::info!("Added agent cron job: {} ({})", job.name, job.id);
        Ok(job)
    }

    /// Get a job by ID
    pub async fn get_job(&self, id: &str) -> Result<Option<CronJob>> {
        self.store.load_job(id).await
    }

    /// Get a job by name
    pub async fn get_job_by_name(&self, name: &str) -> Result<Option<CronJob>> {
        self.store.find_job_by_name(name).await
    }

    /// List all jobs
    pub async fn list_jobs(&self) -> Result<Vec<CronJob>> {
        self.store.list_jobs().await
    }

    /// Update a job
    pub async fn update_job(
        &self,
        id: &str,
        schedule: Option<&str>,
        command: Option<&str>,
        timeout_ms: Option<u64>,
    ) -> Result<CronJob> {
        let mut job = self
            .store
            .load_job(id)
            .await?
            .ok_or_else(|| CronError::JobNotFound(id.to_string()))?;

        if let Some(schedule) = schedule {
            let expr = CronExpression::parse(schedule)?;
            job.schedule = schedule.to_string();
            job.next_run = expr.next_after(Utc::now());
        }

        if let Some(command) = command {
            job.command = command.to_string();
        }

        if let Some(timeout) = timeout_ms {
            job.timeout_ms = timeout;
        }

        job.updated_at = Utc::now();
        self.store.save_job(&job).await?;

        tracing::info!("Updated cron job: {} ({})", job.name, job.id);
        Ok(job)
    }

    /// Pause a job
    pub async fn pause_job(&self, id: &str) -> Result<CronJob> {
        let mut job = self
            .store
            .load_job(id)
            .await?
            .ok_or_else(|| CronError::JobNotFound(id.to_string()))?;

        job.status = JobStatus::Paused;
        job.updated_at = Utc::now();
        self.store.save_job(&job).await?;

        tracing::info!("Paused cron job: {} ({})", job.name, job.id);
        Ok(job)
    }

    /// Resume a paused job
    pub async fn resume_job(&self, id: &str) -> Result<CronJob> {
        let mut job = self
            .store
            .load_job(id)
            .await?
            .ok_or_else(|| CronError::JobNotFound(id.to_string()))?;

        job.status = JobStatus::Active;
        job.updated_at = Utc::now();

        // Recalculate next run
        if let Ok(expr) = CronExpression::parse(&job.schedule) {
            job.next_run = expr.next_after(Utc::now());
        }

        self.store.save_job(&job).await?;

        tracing::info!("Resumed cron job: {} ({})", job.name, job.id);
        Ok(job)
    }

    /// Remove a job
    pub async fn remove_job(&self, id: &str) -> Result<()> {
        let job = self
            .store
            .load_job(id)
            .await?
            .ok_or_else(|| CronError::JobNotFound(id.to_string()))?;

        self.store.delete_job(id).await?;

        tracing::info!("Removed cron job: {} ({})", job.name, job.id);
        Ok(())
    }

    /// Get execution history for a job
    pub async fn get_history(&self, job_id: &str, limit: usize) -> Result<Vec<JobExecution>> {
        self.store.load_executions(job_id, limit).await
    }

    /// Manually run a job
    pub async fn run_job(&self, id: &str) -> Result<JobExecution> {
        let job = self
            .store
            .load_job(id)
            .await?
            .ok_or_else(|| CronError::JobNotFound(id.to_string()))?;

        self.execute_job(&job).await
    }

    /// Execute a job
    async fn execute_job(&self, job: &CronJob) -> Result<JobExecution> {
        let span = tracing::info_span!(
            "a3s.cron.execute_job",
            a3s.cron.job_id = %job.id,
            a3s.cron.job_name = %job.name,
            a3s.cron.job_status = tracing::field::Empty,
            a3s.cron.job_duration_ms = tracing::field::Empty,
        );
        let _guard = span.enter();
        let exec_start = Instant::now();

        let mut execution = JobExecution::new(&job.id);

        // Emit start event
        let _ = self.event_tx.send(SchedulerEvent::JobStarted {
            job_id: job.id.clone(),
            execution_id: execution.id.clone(),
        });

        // Update job status to running
        let mut running_job = job.clone();
        running_job.status = JobStatus::Running;
        self.store.save_job(&running_job).await?;

        // Execute command with timeout
        let timeout = Duration::from_millis(job.timeout_ms);
        let working_dir = job.working_dir.as_deref().unwrap_or(&self.workspace);

        // Result type: Ok(Ok((exit_code, stdout, stderr))) or Ok(Err(io_err)) or Err(timeout)
        let result: std::result::Result<
            std::result::Result<(i32, String, String), std::io::Error>,
            tokio::time::error::Elapsed,
        > = match job.job_type {
            JobType::Agent => {
                let agent_executor = self.agent_executor.clone();
                let agent_config = job.agent_config.clone();
                let prompt = job.command.clone();
                let wd = working_dir.to_string();

                tokio::time::timeout(timeout, async move {
                    let executor = agent_executor.ok_or_else(|| {
                        std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "No agent executor configured for agent-mode cron job",
                        )
                    })?;
                    let config = agent_config.ok_or_else(|| {
                        std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "Agent job missing agent_config",
                        )
                    })?;
                    match executor.execute(&config, &prompt, &wd).await {
                        Ok(text) => Ok((0, text, String::new())),
                        Err(e) => Ok((1, String::new(), e)),
                    }
                })
                .await
            }
            JobType::Shell => {
                tokio::time::timeout(timeout, async {
                    let output = Command::new("sh")
                        .arg("-c")
                        .arg(&job.command)
                        .current_dir(working_dir)
                        .envs(job.env.iter().map(|(k, v)| (k.as_str(), v.as_str())))
                        .output()
                        .await?;
                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    let exit_code = output.status.code().unwrap_or(-1);
                    Ok((exit_code, stdout, stderr))
                })
                .await
            }
        };

        // Process result
        execution = match result {
            Ok(Ok((exit_code, stdout, stderr))) => execution.complete(exit_code, stdout, stderr),
            Ok(Err(e)) => execution.fail(format!("Failed to execute command: {}", e)),
            Err(_) => {
                let _ = self.event_tx.send(SchedulerEvent::JobTimeout {
                    job_id: job.id.clone(),
                    execution_id: execution.id.clone(),
                });
                execution.timeout()
            }
        };

        // Save execution
        self.store.save_execution(&execution).await?;

        // Update job statistics
        let mut updated_job = job.clone();
        updated_job.status = JobStatus::Active;
        updated_job.last_run = Some(execution.started_at);
        updated_job.updated_at = Utc::now();

        if execution.status == crate::types::ExecutionStatus::Success {
            updated_job.run_count += 1;
            let _ = self.event_tx.send(SchedulerEvent::JobCompleted {
                job_id: job.id.clone(),
                execution_id: execution.id.clone(),
            });
        } else {
            updated_job.fail_count += 1;
            let _ = self.event_tx.send(SchedulerEvent::JobFailed {
                job_id: job.id.clone(),
                execution_id: execution.id.clone(),
                error: execution.error.clone().unwrap_or_default(),
            });
        }

        // Calculate next run
        if let Ok(expr) = CronExpression::parse(&updated_job.schedule) {
            updated_job.next_run = expr.next_after(Utc::now());
        }

        self.store.save_job(&updated_job).await?;

        // Record telemetry
        let duration = exec_start.elapsed();
        let status_str = if execution.status == crate::types::ExecutionStatus::Success {
            "success"
        } else if execution.status == crate::types::ExecutionStatus::Timeout {
            "timeout"
        } else {
            "failed"
        };
        span.record(telemetry::ATTR_JOB_STATUS, status_str);
        span.record(telemetry::ATTR_JOB_DURATION_MS, duration.as_millis() as i64);
        telemetry::record_job_execution(&job.name, status_str, duration.as_secs_f64());

        Ok(execution)
    }

    /// Start the scheduler background task
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(());
        }
        *running = true;
        drop(running);

        let _ = self.event_tx.send(SchedulerEvent::Started);
        tracing::info!("Cron scheduler started");

        let store = self.store.clone();
        let event_tx = self.event_tx.clone();
        let running = self.running.clone();
        let workspace = self.workspace.clone();
        let agent_executor = self.agent_executor.clone();

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(60));

            loop {
                ticker.tick().await;
                telemetry::record_scheduler_tick();

                // Check if still running
                if !*running.read().await {
                    break;
                }

                // Get all active jobs
                let jobs = match store.list_jobs().await {
                    Ok(jobs) => jobs,
                    Err(e) => {
                        tracing::error!("Failed to list jobs: {}", e);
                        continue;
                    }
                };

                let now = Utc::now();

                for job in jobs {
                    // Skip non-active jobs
                    if job.status != JobStatus::Active {
                        continue;
                    }

                    // Check if job should run
                    if let Some(next_run) = job.next_run {
                        if next_run <= now {
                            // Create a temporary manager for execution
                            let manager = CronManager {
                                store: store.clone(),
                                event_tx: event_tx.clone(),
                                running: running.clone(),
                                workspace: workspace.clone(),
                                agent_executor: agent_executor.clone(),
                            };

                            if let Err(e) = manager.execute_job(&job).await {
                                tracing::error!("Failed to execute job {}: {}", job.id, e);
                            }
                        }
                    }
                }
            }

            let _ = event_tx.send(SchedulerEvent::Stopped);
            tracing::info!("Cron scheduler stopped");
        });

        Ok(())
    }

    /// Stop the scheduler
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
    }

    /// Check if scheduler is running
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::MemoryCronStore;

    fn create_test_manager() -> CronManager {
        let store = Arc::new(MemoryCronStore::new());
        CronManager::with_store(store, "/tmp".to_string())
    }

    #[tokio::test]
    async fn test_add_job() {
        let manager = create_test_manager();

        let job = manager
            .add_job("test-job", "*/5 * * * *", "echo hello")
            .await
            .unwrap();

        assert_eq!(job.name, "test-job");
        assert_eq!(job.schedule, "*/5 * * * *");
        assert_eq!(job.command, "echo hello");
        assert!(job.next_run.is_some());
    }

    #[tokio::test]
    async fn test_add_duplicate_name() {
        let manager = create_test_manager();

        manager
            .add_job("unique", "* * * * *", "echo 1")
            .await
            .unwrap();

        let result = manager.add_job("unique", "* * * * *", "echo 2").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_add_invalid_schedule() {
        let manager = create_test_manager();

        let result = manager.add_job("bad", "invalid", "echo").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_job() {
        let manager = create_test_manager();

        let job = manager
            .add_job("findme", "* * * * *", "echo")
            .await
            .unwrap();

        let found = manager.get_job(&job.id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "findme");
    }

    #[tokio::test]
    async fn test_list_jobs() {
        let manager = create_test_manager();

        for i in 1..=3 {
            manager
                .add_job(&format!("job-{}", i), "* * * * *", "echo")
                .await
                .unwrap();
        }

        let jobs = manager.list_jobs().await.unwrap();
        assert_eq!(jobs.len(), 3);
    }

    #[tokio::test]
    async fn test_update_job() {
        let manager = create_test_manager();

        let job = manager
            .add_job("updatable", "* * * * *", "echo v1")
            .await
            .unwrap();

        let updated = manager
            .update_job(&job.id, Some("0 * * * *"), Some("echo v2"), Some(30000))
            .await
            .unwrap();

        assert_eq!(updated.schedule, "0 * * * *");
        assert_eq!(updated.command, "echo v2");
        assert_eq!(updated.timeout_ms, 30000);
    }

    #[tokio::test]
    async fn test_pause_resume() {
        let manager = create_test_manager();

        let job = manager
            .add_job("pausable", "* * * * *", "echo")
            .await
            .unwrap();

        // Pause
        let paused = manager.pause_job(&job.id).await.unwrap();
        assert_eq!(paused.status, JobStatus::Paused);

        // Resume
        let resumed = manager.resume_job(&job.id).await.unwrap();
        assert_eq!(resumed.status, JobStatus::Active);
    }

    #[tokio::test]
    async fn test_remove_job() {
        let manager = create_test_manager();

        let job = manager
            .add_job("removable", "* * * * *", "echo")
            .await
            .unwrap();

        manager.remove_job(&job.id).await.unwrap();

        let found = manager.get_job(&job.id).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_run_job() {
        let manager = create_test_manager();

        let job = manager
            .add_job("runnable", "* * * * *", "echo hello")
            .await
            .unwrap();

        let execution = manager.run_job(&job.id).await.unwrap();
        assert!(execution.stdout.contains("hello"));
    }

    #[tokio::test]
    async fn test_run_job_failure() {
        let manager = create_test_manager();

        let job = manager
            .add_job("failing", "* * * * *", "exit 1")
            .await
            .unwrap();

        let execution = manager.run_job(&job.id).await.unwrap();
        assert_eq!(execution.status, crate::types::ExecutionStatus::Failed);
    }

    #[tokio::test]
    async fn test_get_history() {
        let manager = create_test_manager();

        let job = manager
            .add_job("historical", "* * * * *", "echo test")
            .await
            .unwrap();

        // Run job multiple times
        for _ in 0..3 {
            manager.run_job(&job.id).await.unwrap();
        }

        let history = manager.get_history(&job.id, 10).await.unwrap();
        assert_eq!(history.len(), 3);
    }

    #[tokio::test]
    async fn test_event_subscription() {
        let manager = create_test_manager();
        let mut rx = manager.subscribe();

        let job = manager
            .add_job("evented", "* * * * *", "echo test")
            .await
            .unwrap();

        // Run job
        manager.run_job(&job.id).await.unwrap();

        // Check events
        let event = rx.try_recv().unwrap();
        match event {
            SchedulerEvent::JobStarted { job_id, .. } => {
                assert_eq!(job_id, job.id);
            }
            _ => panic!("Expected JobStarted event"),
        }
    }

    // --- Agent-mode tests ---

    /// Mock agent executor for testing
    struct MockAgentExecutor {
        response: String,
        should_fail: bool,
    }

    #[async_trait::async_trait]
    impl AgentExecutor for MockAgentExecutor {
        async fn execute(
            &self,
            _config: &AgentJobConfig,
            _prompt: &str,
            _working_dir: &str,
        ) -> std::result::Result<String, String> {
            if self.should_fail {
                Err("Mock agent error".to_string())
            } else {
                Ok(self.response.clone())
            }
        }
    }

    fn create_agent_config() -> AgentJobConfig {
        AgentJobConfig {
            model: "test-model".to_string(),
            api_key: "test-key".to_string(),
            workspace: None,
            system_prompt: None,
            base_url: None,
        }
    }

    #[tokio::test]
    async fn test_add_agent_job() {
        let manager = create_test_manager();

        let job = manager
            .add_agent_job(
                "agent-task",
                "*/5 * * * *",
                "Refactor auth module",
                create_agent_config(),
            )
            .await
            .unwrap();

        assert_eq!(job.name, "agent-task");
        assert_eq!(job.job_type, JobType::Agent);
        assert_eq!(job.command, "Refactor auth module");
        assert!(job.agent_config.is_some());
        assert!(job.next_run.is_some());
    }

    #[tokio::test]
    async fn test_add_agent_job_duplicate_name() {
        let manager = create_test_manager();

        manager
            .add_agent_job("unique-agent", "* * * * *", "prompt", create_agent_config())
            .await
            .unwrap();

        let result = manager
            .add_agent_job(
                "unique-agent",
                "* * * * *",
                "prompt2",
                create_agent_config(),
            )
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_run_agent_job_success() {
        let store = Arc::new(MemoryCronStore::new());
        let mut manager = CronManager::with_store(store, "/tmp".to_string());
        manager.set_agent_executor(Arc::new(MockAgentExecutor {
            response: "Refactored 3 files".to_string(),
            should_fail: false,
        }));

        let job = manager
            .add_agent_job(
                "agent-run",
                "* * * * *",
                "Refactor auth",
                create_agent_config(),
            )
            .await
            .unwrap();

        let execution = manager.run_job(&job.id).await.unwrap();
        assert_eq!(execution.status, crate::types::ExecutionStatus::Success);
        assert!(execution.stdout.contains("Refactored 3 files"));
    }

    #[tokio::test]
    async fn test_run_agent_job_failure() {
        let store = Arc::new(MemoryCronStore::new());
        let mut manager = CronManager::with_store(store, "/tmp".to_string());
        manager.set_agent_executor(Arc::new(MockAgentExecutor {
            response: String::new(),
            should_fail: true,
        }));

        let job = manager
            .add_agent_job(
                "agent-fail",
                "* * * * *",
                "Bad prompt",
                create_agent_config(),
            )
            .await
            .unwrap();

        let execution = manager.run_job(&job.id).await.unwrap();
        assert_eq!(execution.status, crate::types::ExecutionStatus::Failed);
    }

    #[tokio::test]
    async fn test_run_agent_job_no_executor() {
        let manager = create_test_manager();

        let job = manager
            .add_agent_job("no-executor", "* * * * *", "prompt", create_agent_config())
            .await
            .unwrap();

        let execution = manager.run_job(&job.id).await.unwrap();
        assert_eq!(execution.status, crate::types::ExecutionStatus::Failed);
        assert!(execution
            .error
            .as_deref()
            .unwrap_or("")
            .contains("No agent executor"));
    }

    #[tokio::test]
    async fn test_shell_job_type_default() {
        let manager = create_test_manager();

        let job = manager
            .add_job("shell-default", "* * * * *", "echo hello")
            .await
            .unwrap();

        assert_eq!(job.job_type, JobType::Shell);
        assert!(job.agent_config.is_none());
    }
}
