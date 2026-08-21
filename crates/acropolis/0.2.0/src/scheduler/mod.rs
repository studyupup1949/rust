use std::fmt;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchedulerConfig {
    pub worker_pool_size: usize,
    pub task_queue_size: usize,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            worker_pool_size: 10,
            task_queue_size: 100,
        }
    }
}

pub struct ScheduledTask {
    interval: u64,
    ticks_since_last_run: u64,
    running: bool,
    task: Box<dyn FnMut() + Send>,
    on_busy: Option<Box<dyn FnMut() + Send>>,
}

pub struct ManualScheduler {
    interval: Duration,
    tasks: Vec<ScheduledTask>,
}

impl ManualScheduler {
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            tasks: Vec::new(),
        }
    }

    pub fn interval(&self) -> Duration {
        self.interval
    }

    pub fn change_interval(&mut self, interval: Duration) -> Result<(), SchedulerError> {
        if interval.is_zero() {
            return Err(SchedulerError::NonPositiveInterval);
        }
        self.interval = interval;
        Ok(())
    }

    pub fn register(
        &mut self,
        interval_ticks: u64,
        task: impl FnMut() + Send + 'static,
        on_busy: Option<impl FnMut() + Send + 'static>,
    ) -> Result<(), SchedulerError> {
        if interval_ticks == 0 {
            return Err(SchedulerError::NonPositiveTaskInterval);
        }
        self.tasks.push(ScheduledTask {
            interval: interval_ticks,
            ticks_since_last_run: 0,
            running: false,
            task: Box::new(task),
            on_busy: on_busy.map(|f| Box::new(f) as Box<dyn FnMut() + Send>),
        });
        Ok(())
    }

    pub fn tick(&mut self) -> usize {
        let mut ran = 0;
        for task in &mut self.tasks {
            task.ticks_since_last_run += 1;
            if task.ticks_since_last_run < task.interval {
                continue;
            }
            task.ticks_since_last_run = 0;
            if task.running {
                if let Some(on_busy) = &mut task.on_busy {
                    on_busy();
                }
                continue;
            }
            task.running = true;
            (task.task)();
            task.running = false;
            ran += 1;
        }
        ran
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchedulerError {
    NonPositiveInterval,
    NonPositiveTaskInterval,
}

impl fmt::Display for SchedulerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonPositiveInterval => f.write_str("scheduler interval must be positive"),
            Self::NonPositiveTaskInterval => {
                f.write_str("scheduled task interval must be positive")
            }
        }
    }
}

impl std::error::Error for SchedulerError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn scheduler_runs_tasks_at_tick_intervals() {
        let count = Arc::new(Mutex::new(0));
        let seen = Arc::clone(&count);
        let mut scheduler = ManualScheduler::new(Duration::from_secs(1));
        scheduler
            .register(2, move || *seen.lock().unwrap() += 1, None::<fn()>)
            .unwrap();

        assert_eq!(scheduler.tick(), 0);
        assert_eq!(scheduler.tick(), 1);
        assert_eq!(*count.lock().unwrap(), 1);
    }

    #[test]
    fn scheduler_rejects_zero_interval() {
        let mut scheduler = ManualScheduler::new(Duration::from_secs(1));
        assert_eq!(
            scheduler.change_interval(Duration::ZERO),
            Err(SchedulerError::NonPositiveInterval)
        );
    }
}
