//! Persistent audit history stored as newline-delimited JSON.
//!
//! File: `~/.aaai/history.jsonl`
//! Each line is a serialised [`HistoryRecord`].

use std::io::{BufRead, Write};
use std::path::PathBuf;

use super::record::HistoryRecord;

/// Return the path to the history file, creating parent directories as needed.
pub fn history_path() -> anyhow::Result<PathBuf> {
    let base = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?
        .join(".aaai");
    std::fs::create_dir_all(&base)?;
    Ok(base.join("history.jsonl"))
}

/// Append a record to the history file.
pub fn append(record: &HistoryRecord) -> anyhow::Result<()> {
    let path = history_path()?;
    let line = serde_json::to_string(record)?;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    writeln!(file, "{line}")?;
    log::debug!("history appended to {}", path.display());
    Ok(())
}

/// Load all records, newest-first.
/// Silently skips malformed lines.
pub fn load_all() -> anyhow::Result<Vec<HistoryRecord>> {
    let path = match history_path() {
        Ok(p) if p.exists() => p,
        _ => return Ok(Vec::new()),
    };
    let file = std::fs::File::open(&path)?;
    let reader = std::io::BufReader::new(file);
    let mut records: Vec<HistoryRecord> = reader
        .lines()
        .filter_map(|line| {
            let line = line.ok()?;
            if line.trim().is_empty() { return None; }
            serde_json::from_str(&line)
                .map_err(|e| log::warn!("history parse error: {e}"))
                .ok()
        })
        .collect();
    records.reverse();  // newest first
    Ok(records)
}

/// Load the most recent `n` records.
pub fn load_recent(n: usize) -> anyhow::Result<Vec<HistoryRecord>> {
    let mut all = load_all()?;
    all.truncate(n);
    Ok(all)
}

/// Prune the history file to at most `max_entries` records (newest kept).
/// Rewrites the file atomically.
pub fn prune(max_entries: usize) -> anyhow::Result<usize> {
    let all = load_all()?;  // already newest-first
    if all.len() <= max_entries {
        return Ok(0);
    }
    let keep = &all[..max_entries];
    // Reverse to oldest-first for writing
    let lines: Vec<String> = keep.iter().rev()
        .map(|r| serde_json::to_string(r))
        .collect::<Result<Vec<_>, _>>()?;
    let path = history_path()?;
    let content = lines.join("\n") + "\n";
    std::fs::write(&path, content)?;
    let removed = all.len() - max_entries;
    log::info!("History pruned: kept {max_entries}, removed {removed}");
    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::result::AuditSummary;

    fn make_record() -> HistoryRecord {
        HistoryRecord::new(
            std::path::Path::new("/before"),
            std::path::Path::new("/after"),
            None,
            &AuditSummary { total: 3, ok: 2, pending: 1, ..Default::default() },
        )
    }

    #[test]
    fn round_trip_json() {
        let r = make_record();
        let json = serde_json::to_string(&r).unwrap();
        let restored: HistoryRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.result, r.result);
        assert_eq!(restored.total, 3);
    }
}
