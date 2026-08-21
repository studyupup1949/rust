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
