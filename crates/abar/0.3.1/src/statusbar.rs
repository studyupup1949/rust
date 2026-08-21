use std::time::Duration;
use std::{fmt, thread};

use crate::statusblock::StatusBlock;

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

    /// Tells all blocks in the StatusBar to update themselves if needed.
    pub fn update(&mut self) {
        for block in &mut self.blocks {
            block.update()
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
