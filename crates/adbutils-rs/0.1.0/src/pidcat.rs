//! Compact logcat colorizer. Inspired by `adbutils/pidcat.py` (a port of Jake
//! Wharton's pidcat). This is a focused subset: it streams `logcat -v brief`,
//! filters by minimum priority and optional tag substrings, and colorizes the
//! priority column with ANSI codes.

use std::sync::LazyLock;

use regex::Regex;

use crate::device::AdbDevice;
use crate::errors::Result;

/// Log priority levels in ascending severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Verbose,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

impl Priority {
    /// Parse a priority from its logcat letter (`V`/`D`/`I`/`W`/`E`/`F`).
    pub fn from_letter(c: char) -> Option<Priority> {
        Some(match c {
            'V' => Priority::Verbose,
            'D' => Priority::Debug,
            'I' => Priority::Info,
            'W' => Priority::Warn,
            'E' => Priority::Error,
            'F' => Priority::Fatal,
            _ => return None,
        })
    }

    /// ANSI background/foreground color for the priority tag.
    fn ansi(self) -> &'static str {
        match self {
            Priority::Verbose | Priority::Debug => "\x1b[37m", // white
            Priority::Info => "\x1b[32m",                      // green
            Priority::Warn => "\x1b[33m",                      // yellow
            Priority::Error | Priority::Fatal => "\x1b[31m",   // red
        }
    }
}

/// Options controlling the pidcat stream.
#[derive(Debug, Default, Clone)]
pub struct PidcatOptions {
    /// Minimum priority to show (default: Verbose).
    pub min_priority: Option<Priority>,
    /// Only show lines whose tag contains one of these substrings.
    pub tag_filters: Vec<String>,
    /// Clear the log buffer before streaming.
    pub clear: bool,
}

impl AdbDevice {
    /// Stream colorized logcat to stdout until the connection closes or the task
    /// is aborted. `sink` receives each formatted line (use `print!` in a CLI).
    pub async fn pidcat<F>(&self, opts: PidcatOptions, mut sink: F) -> Result<()>
    where
        F: FnMut(&str),
    {
        static LINE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"^([A-Z])/(.+?)\(\s*(\d+)\): (.*)$").unwrap());

        if opts.clear {
            self.shell("logcat --clear").await?;
        }
        let min = opts.min_priority.unwrap_or(Priority::Verbose);
        let mut stream = self.shell_stream("logcat -v brief").await?;

        let mut buf = Vec::new();
        loop {
            let chunk = stream.read(4096).await?;
            if chunk.is_empty() {
                break;
            }
            buf.extend_from_slice(&chunk);
            // Emit complete lines.
            while let Some(nl) = buf.iter().position(|&b| b == b'\n') {
                let line: Vec<u8> = buf.drain(..=nl).collect();
                let line = String::from_utf8_lossy(&line);
                let line = line.trim_end();
                if let Some(c) = LINE.captures(line) {
                    let prio = Priority::from_letter(c[1].chars().next().unwrap());
                    let tag = &c[2];
                    let msg = &c[4];
                    if let Some(p) = prio {
                        if p < min {
                            continue;
                        }
                        if !opts.tag_filters.is_empty()
                            && !opts.tag_filters.iter().any(|f| tag.contains(f.as_str()))
                        {
                            continue;
                        }
                        sink(&format!(
                            "{}{} {:<23}\x1b[0m {}",
                            p.ansi(),
                            &c[1],
                            tag.trim(),
                            msg
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}
