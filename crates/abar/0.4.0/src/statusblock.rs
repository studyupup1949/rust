use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Encapsulates a Fn() -> String closure.
///
/// Each StatusBlock has a unique name, some command that returns a string, and
/// a polling interval. The result of the command will be cached, and will be
/// updated iff the update() method is called *and* the time since the last
/// update is > the polling interval.
///
/// # Building
///
/// StatusBlocks follow the builder pattern for instantiation, so to make a new
/// one you might use something like this:
///
/// ```
/// let block = StatusBlock::new()
///     .name("example")
///     .command(&|| "hello".to_string())
///     .poll_interval(Duration::from_secs(5));
///     .concurrent(true);
/// ```
pub struct StatusBlock {
    #[allow(dead_code)]
    name:                 String,
    command:              Arc<dyn Fn() -> String + Send + Sync>,
    poll_interval:        Option<Duration>,
    update_in_background: bool,
    min_size:             Option<usize>,
    max_size:             Option<usize>,
    cache:                String,
    last_update:          Option<Instant>,
    promised_result:      bool,
}

pub type Command = Arc<dyn Fn() -> String + Send + Sync>;

impl StatusBlock {
    /// Returns a new StatusBlock with default values. The defaults are:
    ///
    /// ```
    /// StatusBlock {
    ///     name:                 String::new(),
    ///     command:              Box::new(|| String::new()),
    ///     poll_interval:        None,
    ///     update_in_background: false,
    ///     min_size:             None,
    ///     max_size:             None,
    /// }
    /// ```
    pub fn new() -> Self {
        Self {
            name:                 String::new(),
            command:              Arc::new(String::new),
            poll_interval:        None,
            update_in_background: false,
            min_size:             None,
            max_size:             None,
            cache:                String::new(),
            last_update:          None,
            promised_result:      false,
        }
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn command(
        mut self,
        command: Arc<dyn Fn() -> String + Send + Sync>,
    ) -> Self {
        self.command = command;
        self
    }

    pub fn poll_interval(mut self, poll_interval: Duration) -> Self {
        self.poll_interval = Some(poll_interval);
        self
    }

    pub fn update_in_background(mut self, is_concurrent: bool) -> Self {
        self.update_in_background = is_concurrent;
        self
    }

    pub fn min_size(mut self, min_size: usize) -> Self {
        self.min_size = Some(min_size);
        self
    }

    pub fn max_size(mut self, max_size: usize) -> Self {
        self.max_size = Some(max_size);
        self
    }

    pub fn size(mut self, size: usize) -> Self {
        self.min_size = Some(size);
        self.max_size = Some(size);
        self
    }

    pub fn needs_update(&self) -> bool {
        if let Some(last_update) = self.last_update {
            match self.poll_interval {
                Some(interval) =>
                    Instant::now().duration_since(last_update) >= interval
                        && !self.promised_result,
                None => false,
            }
        } else {
            !self.promised_result
        }
    }

    /// Returns a reference to the name of the StatusBlock.
    #[allow(dead_code)]
    pub fn get_name(&self) -> &str {
        &self.name.as_str()
    }

    /// Returns a reference to the StatusBlocks's cache.
    pub fn get_cache(&self) -> &String {
        &self.cache
    }

    /// Returns a clone of the StatusBlock's command reference.
    pub fn get_command(&self) -> Command {
        Arc::clone(&self.command)
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    pub fn is_concurrent(&self) -> bool {
        self.update_in_background
    }

    /// Iff the StatusBlock needs to be updated, update it.
    pub fn update(&mut self) {
        if self.needs_update() {
            self.cache = (self.command)();
            self.last_update = Some(Instant::now());
        }
    }

    /// Force the StatusBlock to update itself.
    pub fn update_unchecked(&mut self) {
        self.cache = (self.command)();
        self.last_update = Some(Instant::now());
    }

    /// Manually give the StatusBlock a String to update itself with.
    pub fn manual_update(&mut self, val: String) {
        self.cache = val;
        self.promised_result = false;
    }

    /// Override the update cycle, preventing further updates until the next
    /// manual_update() call.
    pub fn promise_result(&mut self) {
        self.last_update = Some(Instant::now());
        self.promised_result = true;
    }
}


impl fmt::Display for StatusBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = self.cache.to_string();

        if let Some(max) = self.max_size {
            out.truncate(max);
        }

        if let Some(min) = self.min_size {
            let diff = min - out.len();
            if diff > 0 {
                out.push_str(&" ".repeat(diff));
            }
        }

        write!(f, "{}", out)
    }
}

impl Default for StatusBlock {
    fn default() -> Self {
        Self::new()
    }
}
