//! Storage abstraction for persisting CRDT state across process restarts.
//!
//! `abyo-crdt` uses an event-log model: every change is a [`crate::ListOp`]
//! (or `MapOp`, `CounterOp`, …). To persist a CRDT durably you have two
//! choices:
//!
//! - **Snapshot** the entire state via [`crate::List`]'s `serde` impl and
//!   write the bytes wholesale. Simple, but the file grows linearly with
//!   the doc and every save rewrites the world.
//! - **Append-only log** + occasional snapshots: write each new op as it
//!   happens, plus a snapshot every N ops. On startup, load the latest
//!   snapshot and replay any newer ops. Memory-bounded, fast restart.
//!
//! This module ships the second strategy as a generic [`Storage`] trait
//! and a [`FileStorage`] implementation. Callers decide which CRDT to
//! persist; we just give them a place to put bytes.
//!
//! ## Wire format
//!
//! Each persisted file is a sequence of length-prefixed records:
//!
//! ```text
//! [u32 LE: kind] [u32 LE: payload_len] [payload_len bytes]
//! ```
//!
//! `kind` is `0` for snapshot, `1` for op. Records are appended; readers
//! play them forward. Atomicity: `FileStorage::append_op` calls `flush`
//! and `sync_data`; `snapshot` writes to a tempfile and renames.
//!
//! ## Quick start
//!
//! ```no_run
//! use abyo_crdt::{List, ListOp};
//! use abyo_crdt::storage::{FileStorage, Storage};
//!
//! let mut store = FileStorage::open("/tmp/mydoc.crdt").unwrap();
//!
//! // Restore from disk.
//! let mut list: List<char> = store.load_snapshot::<List<char>>()
//!     .unwrap_or_else(|_| List::new(1));
//! for op in store.load_ops_after_snapshot::<ListOp<char>>().unwrap() {
//!     list.apply(op).unwrap();
//! }
//!
//! // Edit. Persist incrementally.
//! let op = list.insert(list.len(), 'X');
//! store.append_op(&op).unwrap();
//!
//! // Periodic snapshot to compact the log.
//! store.snapshot(&list).unwrap();
//! ```

#![cfg(feature = "storage")]

use serde::{de::DeserializeOwned, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{self, BufReader, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

const KIND_SNAPSHOT: u32 = 0;
const KIND_OP: u32 = 1;

/// Errors from the [`Storage`] layer.
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    /// Underlying I/O error.
    #[error("storage I/O: {0}")]
    Io(#[from] io::Error),
    /// Bincode serialization / deserialization error.
    #[error("storage codec: {0}")]
    Codec(#[from] bincode::Error),
    /// Backing file is corrupt.
    #[error("storage: corrupt record (kind {kind}, len {len})")]
    Corrupt {
        /// Kind byte read.
        kind: u32,
        /// Length read.
        len: u32,
    },
    /// No snapshot found.
    #[error("storage: no snapshot")]
    NoSnapshot,
}

/// Persistent storage for CRDT state.
///
/// Implementations must guarantee: after `append_op` returns, the op is
/// durable on stable storage. After `snapshot` returns, the snapshot is
/// durable AND any prior records (snapshots and ops) may be discarded.
pub trait Storage {
    /// Append a single op to the log.
    fn append_op<O: Serialize>(&mut self, op: &O) -> Result<(), StorageError>;

    /// Replace the on-disk state with a fresh snapshot. Implementations
    /// may compact older records (op log) at this point.
    fn snapshot<S: Serialize>(&mut self, state: &S) -> Result<(), StorageError>;

    /// Load the most recent snapshot, or `Err(StorageError::NoSnapshot)`
    /// if none.
    fn load_snapshot<S: DeserializeOwned>(&mut self) -> Result<S, StorageError>;

    /// Load every op recorded after the most recent snapshot, in order.
    fn load_ops_after_snapshot<O: DeserializeOwned>(&mut self) -> Result<Vec<O>, StorageError>;
}

/// Filesystem-backed [`Storage`].
///
/// Records are appended to a single file. `snapshot` writes a tempfile +
/// rename for atomicity; the old file is then truncated to just the new
/// snapshot record (acting as log compaction).
pub struct FileStorage {
    path: PathBuf,
    file: File,
}

impl FileStorage {
    /// Open or create a storage file at `path`.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        let path = path.as_ref().to_path_buf();
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)?;
        Ok(Self { path, file })
    }

    fn write_record(&mut self, kind: u32, payload: &[u8]) -> Result<(), StorageError> {
        self.file.seek(SeekFrom::End(0))?;
        let len = u32::try_from(payload.len()).map_err(|_| StorageError::Corrupt {
            kind,
            len: u32::MAX,
        })?;
        self.file.write_all(&kind.to_le_bytes())?;
        self.file.write_all(&len.to_le_bytes())?;
        self.file.write_all(payload)?;
        self.file.flush()?;
        self.file.sync_data()?;
        Ok(())
    }

    /// Walk the file from the start, returning the byte offset of the
    /// most recent snapshot record's HEADER and the snapshot payload bytes.
    /// `Err(NoSnapshot)` if none.
    fn find_latest_snapshot(&mut self) -> Result<(u64, Vec<u8>), StorageError> {
        self.file.seek(SeekFrom::Start(0))?;
        let mut reader = BufReader::new(&self.file);
        let mut latest: Option<(u64, Vec<u8>)> = None;
        let mut offset: u64 = 0;
        loop {
            let mut hdr = [0u8; 8];
            match reader.read_exact(&mut hdr) {
                Ok(()) => {}
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e.into()),
            }
            let kind = u32::from_le_bytes(hdr[0..4].try_into().unwrap());
            let len = u32::from_le_bytes(hdr[4..8].try_into().unwrap());
            let mut payload = vec![0u8; len as usize];
            reader.read_exact(&mut payload)?;
            if kind == KIND_SNAPSHOT {
                latest = Some((offset, payload));
            } else if kind != KIND_OP {
                return Err(StorageError::Corrupt { kind, len });
            }
            offset += 8 + u64::from(len);
        }
        latest.ok_or(StorageError::NoSnapshot)
    }
}

impl Storage for FileStorage {
    fn append_op<O: Serialize>(&mut self, op: &O) -> Result<(), StorageError> {
        let payload = bincode::serialize(op)?;
        self.write_record(KIND_OP, &payload)
    }

    fn snapshot<S: Serialize>(&mut self, state: &S) -> Result<(), StorageError> {
        let payload = bincode::serialize(state)?;
        // Atomic rewrite: write a tempfile with just the snapshot record,
        // then rename over the original.
        let tmp_path = self.path.with_extension("tmp");
        {
            let mut tmp = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&tmp_path)?;
            let len = u32::try_from(payload.len()).map_err(|_| StorageError::Corrupt {
                kind: KIND_SNAPSHOT,
                len: u32::MAX,
            })?;
            tmp.write_all(&KIND_SNAPSHOT.to_le_bytes())?;
            tmp.write_all(&len.to_le_bytes())?;
            tmp.write_all(&payload)?;
            tmp.flush()?;
            tmp.sync_data()?;
        }
        std::fs::rename(&tmp_path, &self.path)?;
        // Reopen the file handle since the inode changed.
        self.file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&self.path)?;
        Ok(())
    }

    fn load_snapshot<S: DeserializeOwned>(&mut self) -> Result<S, StorageError> {
        let (_off, payload) = self.find_latest_snapshot()?;
        Ok(bincode::deserialize(&payload)?)
    }

    fn load_ops_after_snapshot<O: DeserializeOwned>(&mut self) -> Result<Vec<O>, StorageError> {
        // Find the snapshot's offset. Then read every record after it,
        // collecting the op-kind ones.
        let (snap_off, snap_payload) = match self.find_latest_snapshot() {
            Ok((off, p)) => (off, p),
            Err(StorageError::NoSnapshot) => (0, Vec::new()),
            Err(e) => return Err(e),
        };
        let mut after = snap_off + 8 + snap_payload.len() as u64;
        if snap_payload.is_empty() {
            after = 0;
        }
        self.file.seek(SeekFrom::Start(after))?;
        let mut reader = BufReader::new(&self.file);
        let mut ops = Vec::new();
        loop {
            let mut hdr = [0u8; 8];
            match reader.read_exact(&mut hdr) {
                Ok(()) => {}
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e.into()),
            }
            let kind = u32::from_le_bytes(hdr[0..4].try_into().unwrap());
            let len = u32::from_le_bytes(hdr[4..8].try_into().unwrap());
            let mut payload = vec![0u8; len as usize];
            reader.read_exact(&mut payload)?;
            if kind == KIND_OP {
                ops.push(bincode::deserialize(&payload)?);
            } else if kind != KIND_SNAPSHOT {
                return Err(StorageError::Corrupt { kind, len });
            }
        }
        Ok(ops)
    }
}

/// In-memory storage — useful for tests.
#[derive(Default)]
pub struct MemoryStorage {
    snapshot: Option<Vec<u8>>,
    ops: Vec<Vec<u8>>,
}

impl MemoryStorage {
    /// Empty storage.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Storage for MemoryStorage {
    fn append_op<O: Serialize>(&mut self, op: &O) -> Result<(), StorageError> {
        self.ops.push(bincode::serialize(op)?);
        Ok(())
    }

    fn snapshot<S: Serialize>(&mut self, state: &S) -> Result<(), StorageError> {
        self.snapshot = Some(bincode::serialize(state)?);
        self.ops.clear();
        Ok(())
    }

    fn load_snapshot<S: DeserializeOwned>(&mut self) -> Result<S, StorageError> {
        let bytes = self.snapshot.as_ref().ok_or(StorageError::NoSnapshot)?;
        Ok(bincode::deserialize(bytes)?)
    }

    fn load_ops_after_snapshot<O: DeserializeOwned>(&mut self) -> Result<Vec<O>, StorageError> {
        self.ops
            .iter()
            .map(|bytes| bincode::deserialize(bytes).map_err(StorageError::from))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{List, ListOp};

    #[test]
    fn memory_storage_round_trip() {
        let mut store = MemoryStorage::new();
        let mut list = List::<char>::new(1);
        list.insert(0, 'a');
        list.insert(1, 'b');
        store.snapshot(&list).unwrap();

        let op = list.insert(2, 'c');
        store.append_op(&op).unwrap();

        // Restore.
        let restored: List<char> = store.load_snapshot().unwrap();
        let ops: Vec<ListOp<char>> = store.load_ops_after_snapshot().unwrap();
        assert_eq!(restored.to_vec(), vec!['a', 'b']);
        assert_eq!(ops.len(), 1);

        let mut replayed = restored;
        for op in ops {
            replayed.apply(op).unwrap();
        }
        assert_eq!(replayed.to_vec(), vec!['a', 'b', 'c']);
    }

    #[test]
    fn file_storage_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("doc.crdt");
        {
            let mut store = FileStorage::open(&path).unwrap();
            let mut list = List::<u32>::new(1);
            for i in 0..50u32 {
                let op = list.insert(i as usize, i);
                store.append_op(&op).unwrap();
            }
            store.snapshot(&list).unwrap();
            // Add a few ops after the snapshot.
            let op = list.insert(50, 999);
            store.append_op(&op).unwrap();
        }
        // Reopen.
        let mut store = FileStorage::open(&path).unwrap();
        let snap: List<u32> = store.load_snapshot().unwrap();
        assert_eq!(snap.len(), 50);
        let ops: Vec<ListOp<u32>> = store.load_ops_after_snapshot().unwrap();
        assert_eq!(ops.len(), 1);
        let mut replayed = snap;
        for op in ops {
            replayed.apply(op).unwrap();
        }
        assert_eq!(replayed.len(), 51);
        assert_eq!(replayed.get(50), Some(&999));
    }

    #[test]
    fn file_storage_snapshot_replaces_file() {
        // After `snapshot`, the file should contain exactly one snapshot
        // record (no leftover op records from before).
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("doc.crdt");
        let mut store = FileStorage::open(&path).unwrap();
        let mut list = List::<u32>::new(1);
        for i in 0..100u32 {
            let op = list.insert(i as usize, i);
            store.append_op(&op).unwrap();
        }
        store.snapshot(&list).unwrap();
        // Now there should be ZERO ops after the snapshot in the file.
        let ops: Vec<ListOp<u32>> = store.load_ops_after_snapshot().unwrap();
        assert!(ops.is_empty(), "snapshot didn't replace prior ops");
        // And the loaded snapshot equals the in-memory list.
        let restored: List<u32> = store.load_snapshot().unwrap();
        assert_eq!(restored.to_vec(), list.to_vec());
    }
}
