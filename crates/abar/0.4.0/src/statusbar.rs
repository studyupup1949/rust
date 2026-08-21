use std::sync::mpsc;
use std::thread::JoinHandle;
use std::time::Duration;
use std::{fmt, thread};

use crate::statusblock::{Command, StatusBlock};

type JobSender = spmc::Sender<(usize, Command)>;
type JobReceiver = spmc::Receiver<(usize, Command)>;
type ResultSender = mpsc::Sender<(usize, String)>;
type ResultReceiver = mpsc::Receiver<(usize, String)>;

/// Encapsulates a number of StatusBlocks.
///
/// Contains information re. how StatusBlocks should be formatted, delimited,
/// rendered, etc. as well as methods that operate across all blocks at once.
///
/// # Building
///
/// StatusBars follow the builder pattern for instantiation, so to make a new
/// one you might use something like this:
///
/// ```
/// let blocks: Vec<StatusBlock> = vec![];
///
/// let status = StatusBar::new()
///     .blocks(blocks)
///     .refresh_rate(Duration::from_millis(500))
///     .delimiter(" | ")
///     .left_buffer(" >>> ")
///     .left_buffer(" <<< ");
/// ```
pub struct StatusBar {
    delimiter:          String,
    blocks:             Vec<StatusBlock>,
    refresh_rate:       Duration,
    left_buffer:        String,
    right_buffer:       String,
    hide_empty_modules: bool,
    jobs_channel:       (JobSender, JobReceiver),
    results_channel:    (ResultSender, ResultReceiver),
    threads:            Vec<JoinHandle<()>>,
}

impl StatusBar {
    /// Returns a new StatusBar with default values. The defaults are:
    ///
    /// ```
    /// StatusBar {
    ///     blocks:             Vec::new(),
    ///     refresh_rate:       Duration::from_secs(1),
    ///     delimiter:          String::new(),
    ///     left_buffer:        String::new(),
    ///     right_buffer:       String::new(),
    ///     hide_empty_modules: false,
    /// }
    /// ```
    pub fn new() -> Self {
        StatusBar {
            blocks:             Vec::new(),
            refresh_rate:       Duration::from_secs(1),
            delimiter:          String::new(),
            left_buffer:        String::new(),
            right_buffer:       String::new(),
            hide_empty_modules: false,
            jobs_channel:       spmc::channel(),
            results_channel:    mpsc::channel(),
            threads:            Vec::new(),
        }
    }

    pub fn blocks(mut self, blocks: Vec<StatusBlock>) -> Self {
        self.blocks = blocks;
        self
    }

    pub fn refresh_rate(mut self, refresh_rate: Duration) -> Self {
        self.refresh_rate = refresh_rate;
        self
    }

    pub fn delimiter(mut self, delimiter: &str) -> Self {
        self.delimiter = delimiter.to_string();
        self
    }

    pub fn left_buffer(mut self, left_buffer: &str) -> Self {
        self.left_buffer = left_buffer.to_string();
        self
    }

    pub fn right_buffer(mut self, right_buffer: &str) -> Self {
        self.right_buffer = right_buffer.to_string();
        self
    }

    pub fn hide_empty_modules(mut self, hide_empty_modules: bool) -> Self {
        self.hide_empty_modules = hide_empty_modules;
        self
    }

    /// Puts the current thread to sleep for an amount of time defined by the
    /// StatusBar's refresh_rate.
    pub fn sleep(&self) {
        thread::sleep(self.refresh_rate)
    }

    pub fn get_refresh_rate(&self) -> Duration {
        self.refresh_rate
    }

    pub fn get_channels(&self) -> (JobReceiver, ResultSender) {
        let (_, jobs_rx) = &self.jobs_channel;
        let (results_tx, _) = &self.results_channel;

        (jobs_rx.clone(), results_tx.clone())
    }

    /// Spawns a default worker thread to handle asyncronous blocks.
    pub fn spawn_worker(&mut self) {
        let (jobs_rx, results_tx) = self.get_channels();

        self.threads.push(thread::spawn(move || loop {
            let (i, job) = jobs_rx.recv().unwrap();
            results_tx.send((i, (job)())).unwrap();
        }));
    }

    /// Updates all blocks that need to be updated. Concurrent blocks create a
    /// job (passed into jobs_tx), while non-concurrent blocks are updated
    /// immediately.
    pub fn update(&mut self) {
        let (jobs_tx, jobs_rx) = &mut self.jobs_channel;
        let (results_tx, results_rx) = &mut self.results_channel;

        // if there are no workers, clear the async queue
        if self.threads.is_empty() {
            while let Ok((i, job)) = jobs_rx.try_recv() {
                results_tx.send((i, (job)())).unwrap();
            }
        }

        for i in 0..self.blocks.len() {
            let block = &mut self.blocks[i];
            if block.needs_update() {
                // println!("\"{}\" needs update", block.get_name());
                if block.is_concurrent() {
                    block.promise_result();
                    jobs_tx.send((i, block.get_command())).unwrap();
                } else {
                    block.update_unchecked();
                }
            }
        }

        while let Ok((index, value)) = results_rx.try_recv() {
            // println!("\"{}\" updated", self.blocks[index].get_name());
            self.blocks[index].manual_update(value);
        }
    }

    fn get_delimiter_at_index(&self, i: usize) -> String {
        match i >= 1 {
            true => self.delimiter.clone(),
            false => String::new(),
        }
    }
}

impl fmt::Display for StatusBar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = String::new();

        for (i, block) in self.blocks.iter().enumerate() {
            if !self.hide_empty_modules || !block.is_empty() {
                out.push_str(&format!(
                    "{}{}",
                    self.get_delimiter_at_index(i),
                    block,
                ));
            }
        }

        write!(f, "{}{}{}", self.left_buffer, out, self.right_buffer)
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}
