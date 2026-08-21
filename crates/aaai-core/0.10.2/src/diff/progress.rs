//! Progress reporting for the diff engine.
//!
//! The diff engine emits [`DiffProgress`] events through any type that
//! implements [`ProgressSink`].  The CLI uses this to drive an `indicatif`
//! progress bar; the GUI can route events to an iced channel.

/// A single progress event emitted during a diff walk.
#[derive(Debug, Clone)]
pub enum DiffProgress {
    /// Directory walk started; `total` is the approximate number of unique paths.
    Started { total: usize },
    /// One path has been processed.
    File { path: String, processed: usize, total: usize },
    /// All files have been processed; engine is sorting.
    Sorting,
    /// The diff is complete.
    Done { total_files: usize },
}

/// Anything that can receive progress events.
pub trait ProgressSink: Send + Sync {
    fn emit(&self, event: DiffProgress);
}

/// A sink that discards all events (default / no-op).
#[derive(Default)]
pub struct NullProgress;

impl ProgressSink for NullProgress {
    fn emit(&self, _: DiffProgress) {}
}

/// A sink backed by a `std::sync::mpsc` channel.
pub struct ChannelProgress {
    tx: std::sync::mpsc::Sender<DiffProgress>,
}

impl ChannelProgress {
    pub fn new(tx: std::sync::mpsc::Sender<DiffProgress>) -> Self {
        Self { tx }
    }
}

impl ProgressSink for ChannelProgress {
    fn emit(&self, event: DiffProgress) {
        let _ = self.tx.send(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    #[test]
    fn channel_progress_sends_events() {
        let (tx, rx) = mpsc::channel();
        let sink = ChannelProgress::new(tx);
        sink.emit(DiffProgress::Started { total: 10 });
        sink.emit(DiffProgress::Done { total_files: 10 });
        let events: Vec<_> = rx.try_iter().collect();
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn null_progress_does_not_panic() {
        let sink = NullProgress;
        sink.emit(DiffProgress::Started { total: 0 });
        sink.emit(DiffProgress::Sorting);
    }
}
