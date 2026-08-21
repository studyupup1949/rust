//! Logger trait and implementations.
//!
//! Provides an abstraction layer so the SDK can be used in CLI, MCP, or
//! library contexts without hardcoding output behavior.

/// Logger interface for ACE operations.
pub trait ILogger: Send + Sync {
    /// Log debug message (only shown in verbose mode).
    fn debug(&self, message: &str);
    /// Log informational message.
    fn info(&self, message: &str);
    /// Log success message.
    fn success(&self, message: &str);
    /// Log warning message.
    fn warn(&self, message: &str);
    /// Log error message.
    fn error(&self, message: &str);
    /// Log trace message (very detailed).
    fn trace(&self, message: &str);
    /// Check if in verbose mode.
    fn is_verbose(&self) -> bool;
    /// Check if in trace mode.
    fn is_trace(&self) -> bool;
}

/// No-op logger that discards all output.
///
/// Used as default when no logger is provided.
#[derive(Debug, Clone, Default)]
pub struct NoopLogger;

impl ILogger for NoopLogger {
    fn debug(&self, _message: &str) {}
    fn info(&self, _message: &str) {}
    fn success(&self, _message: &str) {}
    fn warn(&self, _message: &str) {}
    fn error(&self, _message: &str) {}
    fn trace(&self, _message: &str) {}
    fn is_verbose(&self) -> bool {
        false
    }
    fn is_trace(&self) -> bool {
        false
    }
}

/// Simple stderr logger for debugging.
#[derive(Debug, Clone, Default)]
pub struct StderrLogger {
    pub verbose: bool,
    pub trace: bool,
}

impl ILogger for StderrLogger {
    fn debug(&self, message: &str) {
        if self.verbose {
            eprintln!("[DEBUG] {}", message);
        }
    }

    fn info(&self, message: &str) {
        eprintln!("[INFO] {}", message);
    }

    fn success(&self, message: &str) {
        eprintln!("[OK] {}", message);
    }

    fn warn(&self, message: &str) {
        eprintln!("[WARN] {}", message);
    }

    fn error(&self, message: &str) {
        eprintln!("[ERROR] {}", message);
    }

    fn trace(&self, message: &str) {
        if self.trace {
            eprintln!("[TRACE] {}", message);
        }
    }

    fn is_verbose(&self) -> bool {
        self.verbose
    }

    fn is_trace(&self) -> bool {
        self.trace
    }
}
