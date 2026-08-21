use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

use chrono::{DateTime, Utc};
use tokio::sync::{mpsc, oneshot, watch};
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

use crate::agent::session_actor::SessionCommand;
use crate::compositor::CronJobInfo;
use crate::compositor::state::CronJobState;
use crate::config::{CronJobConfig, MisfirePolicy};
use crate::cron::{describe_schedule, next_run_utc, parse_cron, parse_timezone};

/// Maximum duration to sleep between scheduler ticks. Capping the sleep keeps
/// the worker resilient to system sleep/hibernation: on platforms where the
/// monotonic clock pauses while the machine is suspended (e.g. macOS), an
/// unbounded `sleep_until(deadline)` would only fire after the *awake* time
/// elapsed, not the wall-clock deadline. By waking periodically and comparing
/// against `Utc::now()` we catch up as soon as the machine resumes.
#[allow(clippy::duration_suboptimal_units)]
const MAX_SLEEP_INTERVAL: Duration = Duration::from_secs(60);

/// Command sent to a per-session cron worker.
#[derive(Debug)]
pub enum CronCommand {
    /// Add or replace a cron job for this session.
    AddJob {
        job: CronJobConfig,
        respond_to: oneshot::Sender<anyhow::Result<CronJobInfo>>,
    },
    /// Remove a cron job from this session.
    RemoveJob {
        job_name: String,
        respond_to: oneshot::Sender<anyhow::Result<()>>,
    },
    /// List cron jobs for this session.
    ListJobs {
        respond_to: oneshot::Sender<anyhow::Result<Vec<CronJobInfo>>>,
    },
    /// Shut down the worker.
    Shutdown {
        done_tx: Option<oneshot::Sender<()>>,
    },
}

/// Wrapper around `DateTime<Utc>` so it can be used in a `BinaryHeap`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NaiveDateTimeWrapper(DateTime<Utc>);

impl PartialOrd for NaiveDateTimeWrapper {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NaiveDateTimeWrapper {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.timestamp().cmp(&other.0.timestamp()).then_with(|| {
            self.0
                .timestamp_subsec_nanos()
                .cmp(&other.0.timestamp_subsec_nanos())
        })
    }
}

/// Per-session cron worker that fires jobs back into the owning `SessionActor`.
pub struct CronWorker {
    name: String,
    rx: mpsc::UnboundedReceiver<CronCommand>,
    /// Callback into the owning session actor.
    session_tx: mpsc::UnboundedSender<SessionCommand>,
    /// job_name -> persisted job state.
    jobs: HashMap<String, CronJobState>,
    /// Priority queue ordered by `next_run_at`.
    queue: BinaryHeap<Reverse<(NaiveDateTimeWrapper, String)>>,
    /// Published whenever `jobs` changes.
    watch_tx: watch::Sender<HashMap<String, CronJobState>>,
}

impl CronWorker {
    /// Create a new cron worker for a single session.
    ///
    /// Returns the worker, a sender for commands, and a receiver for state
    /// snapshots. The snapshot is updated every time the job set changes.
    #[must_use]
    pub fn new(
        name: String,
        initial_jobs: HashMap<String, CronJobState>,
        session_tx: mpsc::UnboundedSender<SessionCommand>,
    ) -> (
        Self,
        mpsc::UnboundedSender<CronCommand>,
        watch::Receiver<HashMap<String, CronJobState>>,
    ) {
        let (tx, rx) = mpsc::unbounded_channel();
        let (watch_tx, watch_rx) = watch::channel(initial_jobs.clone());
        (
            Self {
                name,
                rx,
                session_tx,
                jobs: initial_jobs,
                queue: BinaryHeap::new(),
                watch_tx,
            },
            tx,
            watch_rx,
        )
    }

    fn publish_jobs(&self) {
        let _ = self.watch_tx.send(self.jobs.clone());
    }

    /// Run the scheduler loop until a shutdown command is received.
    pub async fn run(
        mut self,
        initial_jobs: HashMap<String, CronJobState>,
        cancel: CancellationToken,
    ) {
        // Merge any additional persisted jobs supplied at start time.
        for (name, state) in initial_jobs {
            self.jobs.entry(name).or_insert(state);
        }
        // Prime the scheduling queue from the initial job set.
        let mut initial: Vec<CronJobState> = self.jobs.values().cloned().collect();
        initial.sort_by_key(|s| s.config.name.clone());
        for state in initial {
            if let Some(next_run_at) = state.next_run_at {
                self.queue.push(Reverse((
                    NaiveDateTimeWrapper(next_run_at),
                    state.config.name,
                )));
            }
        }
        self.publish_jobs();
        info!(name = %self.name, jobs = self.jobs.len(), "cron worker started");

        loop {
            let next_deadline = self.queue.peek().map(|Reverse((ts, _))| *ts);

            if let Some(deadline) = next_deadline {
                let sleep_dur = sleep_duration(deadline.0);
                tokio::select! {
                    Some(cmd) = self.rx.recv() => {
                        if self.handle_command(cmd).await {
                            return;
                        }
                    }
                    () = tokio::time::sleep(sleep_dur) => {
                        self.fire_due_jobs().await;
                    }
                    () = cancel.cancelled() => {
                        info!(name = %self.name, "cron worker cancelled");
                        return;
                    }
                }
            } else {
                tokio::select! {
                    Some(cmd) = self.rx.recv() => {
                        if self.handle_command(cmd).await {
                            return;
                        }
                    }
                    () = cancel.cancelled() => {
                        info!(name = %self.name, "cron worker cancelled");
                        return;
                    }
                }
            }
        }
    }

    async fn handle_command(&mut self, cmd: CronCommand) -> bool {
        match cmd {
            CronCommand::AddJob { job, respond_to } => {
                let result = self.handle_add_job(job).await;
                if result.is_ok() {
                    self.publish_jobs();
                }
                let _ = respond_to.send(result);
                false
            }
            CronCommand::RemoveJob {
                job_name,
                respond_to,
            } => {
                self.remove_job(&job_name);
                self.publish_jobs();
                let _ = respond_to.send(Ok(()));
                false
            }
            CronCommand::ListJobs { respond_to } => {
                let jobs = self.build_job_infos();
                let _ = respond_to.send(Ok(jobs));
                false
            }
            CronCommand::Shutdown { done_tx } => {
                if let Some(tx) = done_tx {
                    let _ = tx.send(());
                }
                true
            }
        }
    }

    async fn handle_add_job(&mut self, job: CronJobConfig) -> anyhow::Result<CronJobInfo> {
        validate_cron_job_name(&job.name)?;

        let saved = self.jobs.get(&job.name).cloned();

        let (next_run_at, description) = match (&job.run_at, &job.schedule) {
            (Some(run_at), _) => {
                let run_at = run_at
                    .parse::<DateTime<Utc>>()
                    .map_err(|e| anyhow::anyhow!("invalid run_at timestamp: {}", e))?;
                if run_at <= Utc::now() && saved.is_none() {
                    anyhow::bail!("run_at must be in the future");
                }
                if saved.as_ref().and_then(|s| s.last_run_at).is_some() {
                    self.remove_job(&job.name);
                    anyhow::bail!(
                        "one-time cron job '{}' has already fired and was removed",
                        job.name
                    );
                }
                (run_at, format!("One-time run at {}", run_at.to_rfc3339()))
            }
            (None, Some(schedule)) => {
                let cron = parse_cron(schedule)?;
                let tz = parse_timezone(&job.timezone)?;
                let description = describe_schedule(&cron);
                let now = Utc::now();
                let after = saved.as_ref().and_then(|s| s.last_run_at).unwrap_or(now);
                let mut next = next_run_utc(&cron, tz, after)?;
                if next < now {
                    match job.misfire_policy {
                        MisfirePolicy::Skip => {
                            next = next_run_utc(&cron, tz, now)?;
                        }
                        MisfirePolicy::FireOnce => {}
                    }
                }
                (next, description)
            }
            (None, None) => anyhow::bail!("either schedule or run_at must be provided"),
        };

        self.remove_job(&job.name);

        self.jobs.insert(
            job.name.clone(),
            CronJobState {
                config: job.clone(),
                last_run_at: saved.as_ref().and_then(|s| s.last_run_at),
                next_run_at: Some(next_run_at),
            },
        );
        self.queue.push(Reverse((
            NaiveDateTimeWrapper(next_run_at),
            job.name.clone(),
        )));

        info!(
            job_name = job.name,
            next_run = %next_run_at,
            description = %description,
            "started cron job"
        );
        Ok(CronJobInfo {
            config: job,
            last_run_at: saved.as_ref().and_then(|s| s.last_run_at),
            next_run_at: Some(next_run_at),
            description,
        })
    }

    fn remove_job(&mut self, job_name: &str) {
        self.jobs.remove(job_name);
        self.remove_from_queue(job_name);
    }

    fn remove_from_queue(&mut self, job_name: &str) {
        let mut kept = Vec::new();
        while let Some(item) = self.queue.pop() {
            if item.0.1 != job_name {
                kept.push(item);
            }
        }
        for item in kept {
            self.queue.push(item);
        }
    }

    fn build_job_infos(&self) -> Vec<CronJobInfo> {
        self.jobs
            .values()
            .map(|state| {
                let description = match (&state.config.run_at, &state.config.schedule) {
                    (Some(run_at), _) => format!("One-time run at {}", run_at),
                    (None, Some(schedule)) => parse_cron(schedule).map_or_else(
                        |_| "invalid schedule".to_string(),
                        |c| describe_schedule(&c),
                    ),
                    (None, None) => "unknown schedule".to_string(),
                };
                CronJobInfo {
                    config: state.config.clone(),
                    last_run_at: state.last_run_at,
                    next_run_at: state.next_run_at,
                    description,
                }
            })
            .collect()
    }

    async fn fire_due_jobs(&mut self) {
        let now = Utc::now();
        while let Some(Reverse((ts, _job_name))) = self.queue.peek() {
            if ts.0 > now {
                break;
            }
            let Some(Reverse((ts, job_name))) = self.queue.pop() else {
                break;
            };
            let Some(mut state) = self.jobs.remove(&job_name) else {
                continue;
            };
            let config = state.config.clone();

            if config.run_at.is_some() {
                info!(
                    name = %self.name,
                    %job_name,
                    scheduled_at = %ts.0,
                    fired_at = %now,
                    "cron job firing (one-time)"
                );
                self.send_prompt(&job_name, &config.prompt);
                self.publish_jobs();
                continue;
            }

            let Some(schedule) = config.schedule else {
                error!(job_name, "cron job has neither schedule nor run_at");
                continue;
            };

            let cron = match parse_cron(&schedule) {
                Ok(c) => c,
                Err(e) => {
                    error!(job_name, error = %e, "cron worker failed to parse schedule");
                    continue;
                }
            };
            let tz = match parse_timezone(&config.timezone) {
                Ok(t) => t,
                Err(e) => {
                    error!(job_name, error = %e, "cron worker failed to parse timezone");
                    continue;
                }
            };

            if ts.0 < now - chrono::Duration::minutes(1)
                && config.misfire_policy == MisfirePolicy::Skip
            {
                match next_run_utc(&cron, tz, now) {
                    Ok(next) => {
                        info!(
                            name = %self.name,
                            %job_name,
                            scheduled_at = %ts.0,
                            skipped_at = %now,
                            %next,
                            "cron job misfired, skipping to next run"
                        );
                        state.next_run_at = Some(next);
                        self.jobs.insert(job_name.clone(), state.clone());
                        self.queue
                            .push(Reverse((NaiveDateTimeWrapper(next), job_name)));
                        self.publish_jobs();
                        continue;
                    }
                    Err(e) => {
                        error!(job_name, error = %e, "failed to compute next run after misfire");
                        continue;
                    }
                }
            }

            let next = match next_run_utc(&cron, tz, now) {
                Ok(n) => n,
                Err(e) => {
                    error!(job_name, error = %e, "failed to compute next run");
                    continue;
                }
            };
            info!(
                name = %self.name,
                %job_name,
                scheduled_at = %ts.0,
                fired_at = %now,
                %next,
                "cron job firing"
            );
            state.next_run_at = Some(next);
            state.last_run_at = Some(now);
            self.jobs.insert(job_name.clone(), state.clone());
            self.queue
                .push(Reverse((NaiveDateTimeWrapper(next), job_name.clone())));

            self.send_prompt(&job_name, &config.prompt);
            self.publish_jobs();
        }
    }

    fn send_prompt(&self, job_name: &str, prompt: &str) {
        let prompt_id = format!("cron-{}-{}", self.name, uuid::Uuid::new_v4());
        if self
            .session_tx
            .send(SessionCommand::Prompt {
                prompt_id,
                content: prompt.to_string(),
                cron_job_name: Some(job_name.to_string()),
                send_result_to: None,
            })
            .is_err()
        {
            error!("cron worker session command channel closed");
        }
    }
}

fn sleep_duration(deadline: DateTime<Utc>) -> Duration {
    let now = Utc::now();
    if deadline <= now {
        Duration::ZERO
    } else {
        #[allow(clippy::duration_suboptimal_units)]
        let until = (deadline - now).to_std().unwrap_or(MAX_SLEEP_INTERVAL);
        until.min(MAX_SLEEP_INTERVAL)
    }
}

fn validate_cron_job_name(name: &str) -> anyhow::Result<()> {
    if name.is_empty() {
        anyhow::bail!("cron job name cannot be empty");
    }
    if name.len() > 128 {
        anyhow::bail!("cron job name too long (max 128 characters)");
    }
    if name
        .chars()
        .any(|c| c.is_whitespace() || c == '/' || c == '\\' || c == '\0')
    {
        anyhow::bail!("cron job name contains invalid characters");
    }
    Ok(())
}
