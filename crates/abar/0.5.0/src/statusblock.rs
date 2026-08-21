use std::fmt;
use std::time::{Duration, Instant};

pub type Command = fn() -> String;

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
/// # use std::time::Duration;
/// # use std::sync::Arc;
/// # use abar::StatusBlock;
///
/// let block = StatusBlock::default()
///     .name("example")
///     .command(String::new)
///     .poll_interval(Duration::from_secs(5))
///     .update_in_background(true);
/// ```
pub struct StatusBlock
{
    pub name:          String,
    pub command:       Command,
    pub poll_interval: Option<Duration>,
    pub is_concurrent: bool,
    pub min_size:      Option<usize>,
    pub max_size:      Option<usize>,
    cache:             String,
    last_update:       Option<Instant>,
    promised_result:   bool,
}

impl StatusBlock
{
    pub fn is_empty(&self) -> bool { self.cache.is_empty() }

    /// Returns whether or not the StatusBlock needs to be updated.
    pub fn needs_update(&self) -> bool
    {
        if self.last_update.is_none() {
            true
        }
        else if self.promised_result || self.poll_interval.is_none() {
            false
        }
        else {
            let last_update = self.last_update.unwrap();
            let poll_interval = self.poll_interval.unwrap();
            let now = Instant::now();

            now.duration_since(last_update) >= poll_interval
        }
    }

    /// Iff the StatusBlock needs to be updated, update it.
    pub fn update(&mut self)
    {
        if self.needs_update() {
            self.cache = (self.command)();
            self.last_update = Some(Instant::now());
        }
    }

    /// Force the StatusBlock to update itself.
    pub fn update_now(&mut self)
    {
        self.cache = (self.command)();
        self.last_update = Some(Instant::now());
    }

    /// Manually give the StatusBlock a String to update itself with.
    pub fn manual_update(&mut self, val: String)
    {
        self.cache = val;
        self.promised_result = false;
    }

    /// Override the update cycle, preventing further updates until the next
    /// manual_update() call.
    pub fn promise_result(&mut self)
    {
        self.last_update = Some(Instant::now());
        self.promised_result = true;
    }


    // ------ builder methods ------ //

    pub fn name(mut self, name: &str) -> Self
    {
        self.name = name.to_string();
        self
    }

    pub fn command(mut self, command: Command) -> Self
    {
        self.command = command;
        self
    }

    pub fn poll_interval(mut self, poll_interval: Duration) -> Self
    {
        self.poll_interval = Some(poll_interval);
        self
    }

    pub fn update_in_background(mut self, is_concurrent: bool) -> Self
    {
        self.is_concurrent = is_concurrent;
        self
    }

    pub fn min_size(mut self, min_size: usize) -> Self
    {
        self.min_size = Some(min_size);
        self
    }

    pub fn max_size(mut self, max_size: usize) -> Self
    {
        self.max_size = Some(max_size);
        self
    }

    pub fn size(mut self, size: usize) -> Self
    {
        self.min_size = Some(size);
        self.max_size = Some(size);
        self
    }
}


impl fmt::Display for StatusBlock
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
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

impl Default for StatusBlock
{
    fn default() -> Self
    {
        Self {
            name:            String::new(),
            command:         String::new,
            poll_interval:   None,
            is_concurrent:   false,
            min_size:        None,
            max_size:        None,
            cache:           String::new(),
            last_update:     None,
            promised_result: false,
        }
    }
}
