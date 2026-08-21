//! `impl FileSystem for Ad1Vfs` — the forensic-vfs adapter (behind the `vfs`
//! feature) so an AD1 logical image's file tree composes as `Arc<dyn FileSystem>`
//! in the forensic-vfs engine, like the DAR / ISO 9660 / UDF adapters.
//!
//! AD1 is a *logical* archive — a flat list of full `/`-separated file paths, the
//! same shape as DAR — so the directory tree is derived: a synthetic root (node 0)
//! plus one node per catalogue entry, wired parent→children by splitting each full
//! path on `/`. Nodes are addressed by [`FileId::Opaque`] carrying an index into
//! an internal node vector built at [`Ad1Vfs::open`]; each file node keeps the
//! index of its backing [`Ad1Entry`] so [`FileSystem::read_at`] delegates straight
//! to [`Ad1Reader::read_at`] (positioned/streaming — no decompressed-content
//! cache).
//!
//! ## Mapping notes / known limits
//! - **Path-based open (known limit).** [`Ad1Reader::open`] opens by *path* because
//!   it discovers sibling `.ad2…` segments on disk, so [`Ad1Vfs::open`] is
//!   path-based too. This does **not** yet fit the engine's
//!   `FileSystemProbe::open(DynSource)` single-source model (one `Read + Seek`
//!   source). Reconciling multi-segment discovery with the probe contract is future
//!   work; the path-based `open` is the current entry point.
//! - **Interior mutability → `Mutex`.** The reader reaches its segment files through
//!   a `RefCell<File>` (interior mutability), which is `!Sync`; the [`FileSystem`]
//!   contract requires `Send + Sync` so one handle serves N workers. The reader is
//!   therefore wrapped in a `Mutex`, recovered-on-poison, so every `&self` read is
//!   serialized rather than racing. No content cache is needed — `read_at` is
//!   positioned and inflates only the chunks a range overlaps.
//! - **FsKind.** `forensic-vfs` re-exports `forensicnomicon_core`'s `FsKind`,
//!   which carries a dedicated [`FsKind::AD1`] identity, so
//!   [`FileSystem::kind`] reports [`FsKind::AD1`].
//! - **Sector sizes.** An AD1 image is a logical container with no media geometry;
//!   [`FileSystem::sector_sizes`] reports 512 for all three fields (a neutral
//!   default, not a real on-media block).
//! - **Times.** AD1 records timestamps only as zoneless display strings
//!   (`YYYYMMDDThhmmss`), not Unix epochs, so [`FsMeta::times`] is entirely `None`
//!   — honestly absent rather than a fabricated epoch-0. As the strings carry no
//!   zone, [`FileSystem::timestamp_zone`] is [`TimeZonePolicy::LocalUnknown`].
//! - **Ownership metadata.** AD1 stores no uid/gid/mode, so those `FsMeta` fields
//!   are `None`.
//! - **Single stream.** An AD1 entry has one data stream; a non-`Default`
//!   [`StreamId`] is refused loud.
//! - **Extents (first cut).** A logical archive exposes no on-media allocation
//!   runs, so [`FileSystem::extents`] yields a single logical run
//!   (`image_offset` = 0, `len` = the entry's uncompressed size) rather than true
//!   on-disk runs. Surfacing the stored chunk layout is future work.
//! - **Symlinks.** AD1 does not surface symlink targets, so [`FileSystem::read_link`]
//!   returns an empty target (matching the DAR / iso9660 / udf convention).
//! - **Deleted/unallocated (first cut).** AD1 carving of orphaned tree nodes and
//!   free-space enumeration are not yet surfaced, so [`FileSystem::deleted`] /
//!   [`FileSystem::unallocated`] are empty streams. Future work, not fabricated
//!   data.

use std::path::Path;
use std::sync::{Mutex, MutexGuard, PoisonError};

use forensic_vfs::{
    Allocation, ByteRun, DirEntry as VfsDirEntry, DirStream, ExtentStream, FileId, FileSystem,
    FsKind, FsMeta, MacbTimes, NodeKind, NodeStream, ResidencyKind, RunAlloc, RunFlags, RunInfo,
    SectorSizes, StreamId, TimeZonePolicy, VfsError, VfsResult,
};

use crate::{Ad1Entry, Ad1Error, Ad1Reader};

/// A neutral logical block size for a logical archive (no media geometry).
const ARCHIVE_BLOCK: u32 = 512;

/// One node in the derived directory tree. The synthetic root is node 0
/// (`entry_idx` `None`); every catalogue entry becomes a node carrying the index
/// of its backing [`Ad1Entry`].
struct Node {
    /// Index into [`Ad1Reader::entries`]; `None` only for the synthetic root.
    entry_idx: Option<usize>,
    /// Last path component (raw bytes) — the name a parent lists this child under.
    name: Vec<u8>,
    kind: NodeKind,
    size: u64,
    /// Node ids of this node's directory children.
    children: Vec<u64>,
}

/// A mounted AD1 logical image exposed through the forensic-vfs `FileSystem`
/// contract. Reads are `&self` over an interior `Mutex`, so one handle serves N
/// workers.
pub struct Ad1Vfs {
    inner: Mutex<Ad1Reader>,
    nodes: Vec<Node>,
}

impl Ad1Vfs {
    /// Open an AD1 image given the path to its **first** segment (`*.ad1`);
    /// sibling `.ad2…` segments are discovered alongside it.
    ///
    /// Parses the AD1 tree, then derives the directory tree from the flat list of
    /// full paths: a synthetic root (node 0) plus one node per entry, wired
    /// parent→children by splitting each path on `/`.
    ///
    /// # Errors
    /// Any [`Ad1Error`] from the reader, mapped to the corresponding [`VfsError`]
    /// (a missing/​unrecognized signature becomes a loud [`VfsError::Bootstrap`]).
    pub fn open(path: &Path) -> VfsResult<Self> {
        let reader = Ad1Reader::open(path).map_err(map_err)?;
        let nodes = build_tree(reader.entries());
        Ok(Self {
            inner: Mutex::new(reader),
            nodes,
        })
    }

    /// Lock the interior reader, recovering from a poisoned mutex rather than
    /// panicking (Paranoid Gatekeeper).
    fn lock(&self) -> MutexGuard<'_, Ad1Reader> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Resolve a [`FileId`] to a node, or a loud error for any non-`Opaque` id or
    /// an index outside the node table.
    fn node_of(&self, id: FileId) -> VfsResult<&Node> {
        let idx = index_of(id)?;
        self.nodes
            .get(usize::try_from(idx).unwrap_or(usize::MAX))
            .ok_or(VfsError::Unsupported {
                layer: "ad1 file-id",
                scheme: format!("Opaque({idx}) out of range"),
            })
    }
}

/// The node index carried by a [`FileId`]; any other identity domain is a caller
/// error surfaced loud.
fn index_of(id: FileId) -> VfsResult<u64> {
    match id {
        FileId::Opaque(n) => Ok(n),
        other => Err(VfsError::Unsupported {
            layer: "ad1 file-id",
            scheme: format!("{other:?}"),
        }),
    }
}

/// An AD1 entry exposes a single unnamed data stream; a named-stream id is refused
/// loud.
fn require_default_stream(stream: StreamId) -> VfsResult<()> {
    match stream {
        StreamId::Default => Ok(()),
        other => Err(VfsError::Unsupported {
            layer: "ad1 stream",
            scheme: format!("{other:?}"),
        }),
    }
}

/// Map an [`Ad1Error`] to the VFS error type, keeping I/O distinct from a
/// structural decode failure and a not-AD1 signature from a bootstrap failure.
fn map_err(e: Ad1Error) -> VfsError {
    match e {
        Ad1Error::Io(source) => VfsError::Io {
            op: "ad1 read",
            source,
        },
        Ad1Error::NotAd1(detail) => VfsError::Bootstrap {
            stage: "ad1 mount",
            detail,
        },
        Ad1Error::Unsupported(scheme) => VfsError::Unsupported {
            layer: "ad1",
            scheme,
        },
        Ad1Error::Malformed(detail) => VfsError::Decode {
            layer: "ad1",
            offset: 0,
            detail,
            bytes: forensic_vfs::SmallHex::new(&[]),
        },
    }
}

/// The last `/`-separated component of a path (the leaf name), as raw bytes. A path
/// with no separator is its own name.
fn leaf(path: &str) -> Vec<u8> {
    match path.rfind('/') {
        Some(pos) => path.get(pos + 1..).unwrap_or("").as_bytes().to_vec(),
        None => path.as_bytes().to_vec(),
    }
}

/// Derive the directory tree (node 0 = synthetic root) from the flat catalogue of
/// full `/`-separated paths. Every entry becomes a node; parent→children links are
/// resolved by full-path prefix. An entry whose parent directory was not itself
/// listed is attached to the root so it is never lost.
fn build_tree(entries: &[Ad1Entry]) -> Vec<Node> {
    let mut nodes: Vec<Node> = Vec::with_capacity(entries.len() + 1);
    // Node 0: synthetic root.
    nodes.push(Node {
        entry_idx: None,
        name: Vec::new(),
        kind: NodeKind::Dir,
        size: 0,
        children: Vec::new(),
    });

    // Map each full path to its node id as we create nodes.
    let mut by_path: std::collections::HashMap<&str, u64> = std::collections::HashMap::new();
    for (i, e) in entries.iter().enumerate() {
        let id = nodes.len() as u64;
        by_path.insert(e.path.as_str(), id);
        nodes.push(Node {
            entry_idx: Some(i),
            name: leaf(&e.path),
            kind: if e.is_dir {
                NodeKind::Dir
            } else {
                NodeKind::File
            },
            size: e.size,
            children: Vec::new(),
        });
    }

    // Wire parent→children by full-path prefix (the segment before the last '/').
    for e in entries {
        let Some(&child) = by_path.get(e.path.as_str()) else {
            continue; // cov:unreachable: every entry path was just inserted
        };
        let parent_id = match e.path.rfind('/') {
            Some(pos) => e
                .path
                .get(..pos)
                .and_then(|p| by_path.get(p))
                .copied()
                .unwrap_or(0),
            None => 0,
        };
        if let Some(parent) = nodes.get_mut(usize::try_from(parent_id).unwrap_or(usize::MAX)) {
            parent.children.push(child);
        }
    }
    nodes
}

impl FileSystem for Ad1Vfs {
    fn kind(&self) -> FsKind {
        // Dedicated AD1 identity from forensicnomicon-core (see the module note).
        FsKind::AD1
    }

    fn root(&self) -> FileId {
        FileId::Opaque(0)
    }

    fn sector_sizes(&self) -> SectorSizes {
        SectorSizes {
            logical: ARCHIVE_BLOCK,
            physical: ARCHIVE_BLOCK,
            cluster_or_block: ARCHIVE_BLOCK,
        }
    }

    fn timestamp_zone(&self) -> TimeZonePolicy {
        // AD1's timestamps are zoneless display strings (not surfaced as epochs).
        TimeZonePolicy::LocalUnknown
    }

    fn read_dir(&self, ino: FileId) -> VfsResult<DirStream> {
        let node = self.node_of(ino)?;
        if node.kind != NodeKind::Dir {
            return Err(VfsError::Decode {
                layer: "ad1",
                offset: 0,
                detail: format!("node {:?} is not a directory", index_of(ino)?),
                bytes: forensic_vfs::SmallHex::new(&[]),
            });
        }
        // Snapshot children into owned entries so the stream outlives the borrow.
        let mut out: Vec<VfsResult<VfsDirEntry>> = Vec::with_capacity(node.children.len());
        for &child in &node.children {
            let Some(c) = self.nodes.get(usize::try_from(child).unwrap_or(usize::MAX)) else {
                continue; // cov:unreachable: children hold in-range node ids by construction
            };
            out.push(Ok(VfsDirEntry {
                name: c.name.clone(),
                id: FileId::Opaque(child),
                kind: c.kind,
            }));
        }
        Ok(DirStream::new(out.into_iter()))
    }

    fn extents(&self, ino: FileId, stream: StreamId) -> VfsResult<ExtentStream> {
        let node = self.node_of(ino)?;
        require_default_stream(stream)?;
        // First cut: a logical archive exposes no on-media runs, so a non-empty
        // file yields one logical run (image_offset 0). See the module note.
        if node.size == 0 {
            return Ok(ExtentStream::empty());
        }
        let run = RunInfo {
            run: ByteRun {
                image_offset: 0,
                len: node.size,
                flags: RunFlags::default(),
            },
            alloc: RunAlloc::Allocated,
        };
        Ok(ExtentStream::new(std::iter::once(Ok(run))))
    }

    fn lookup(&self, parent: FileId, name: &[u8]) -> VfsResult<Option<FileId>> {
        let node = self.node_of(parent)?;
        if node.kind != NodeKind::Dir {
            return Err(VfsError::Decode {
                layer: "ad1",
                offset: 0,
                detail: format!("node {:?} is not a directory", index_of(parent)?),
                bytes: forensic_vfs::SmallHex::new(&[]),
            });
        }
        for &child in &node.children {
            if let Some(c) = self.nodes.get(usize::try_from(child).unwrap_or(usize::MAX)) {
                if c.name == name {
                    return Ok(Some(FileId::Opaque(child)));
                }
            }
        }
        Ok(None)
    }

    fn meta(&self, ino: FileId) -> VfsResult<FsMeta> {
        let idx = index_of(ino)?;
        let node = self.node_of(ino)?;
        Ok(FsMeta {
            ino: idx,
            kind: node.kind,
            allocated: Allocation::Allocated,
            size: node.size,
            nlink: 1,
            // AD1 records no uid/gid/mode.
            uid: None,
            gid: None,
            mode: None,
            // AD1's timestamps are zoneless display strings, not epochs; honestly
            // absent rather than fabricated epoch-0 (see the module note).
            times: MacbTimes::default(),
            streams: Vec::new(),
            residency: ResidencyKind::NonResident,
            link_target: None,
        })
    }

    fn read_at(&self, ino: FileId, stream: StreamId, off: u64, buf: &mut [u8]) -> VfsResult<usize> {
        require_default_stream(stream)?;
        // Validate the node exists / is a file; a directory (or the root) has no
        // extractable data and reads as 0.
        let (kind, entry_idx) = {
            let node = self.node_of(ino)?;
            (node.kind, node.entry_idx)
        };
        if kind != NodeKind::File {
            return Ok(0);
        }
        let Some(entry_idx) = entry_idx else {
            return Ok(0);
        };
        let guard = self.lock();
        // Clone the entry to drop the `entries()` borrow so `buf` and `read_at`
        // (both borrowing the guard) can coexist across the fill loop.
        let Some(entry) = guard.entries().get(entry_idx).cloned() else {
            return Ok(0); // cov:unreachable: entry_idx came from build_tree, in range
        };
        // Ad1Reader::read_at is positioned/streaming and may short-read per call;
        // loop until the buffer is filled or a read yields 0 (EOF / short chunk).
        let mut filled = 0usize;
        while filled < buf.len() {
            let cur = off.saturating_add(filled as u64);
            let Some(dst) = buf.get_mut(filled..) else {
                break; // cov:unreachable: filled < buf.len() holds by the while guard
            };
            let n = guard.read_at(&entry, cur, dst).map_err(map_err)?;
            if n == 0 {
                break;
            }
            filled += n;
        }
        Ok(filled)
    }

    fn read_link(&self, ino: FileId, _cap: usize) -> VfsResult<Vec<u8>> {
        // Validate the id is loud on a bad FileId, then report no target: AD1 does
        // not surface symlink targets (matching the DAR / iso9660 / udf adapters).
        self.node_of(ino)?;
        Ok(Vec::new())
    }

    fn deleted(&self) -> VfsResult<NodeStream> {
        Ok(NodeStream::empty())
    }

    fn unallocated(&self) -> VfsResult<ExtentStream> {
        Ok(ExtentStream::empty())
    }
}

#[cfg(all(test, feature = "testfix"))]
mod tests {
    use super::*;
    use crate::testfix;
    use forensic_vfs::{
        Allocation, FileId, FileSystem, FsKind, NodeKind, RunAlloc, StreamId, TimeZonePolicy,
    };

    /// Build the canonical sample tree, write it to a tempdir as `image.ad1`, and
    /// open it through the adapter. Returns the tempdir (kept alive), the mounted
    /// filesystem, and the builder's expected per-entry facts (ground truth).
    fn open_sample() -> (tempfile::TempDir, Ad1Vfs, Vec<testfix::Expected>) {
        let built = testfix::build(testfix::sample_tree());
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("image.ad1");
        std::fs::write(&path, &built.bytes).unwrap();
        let fs = Ad1Vfs::open(&path).unwrap();
        (dir, fs, built.expected)
    }

    fn expected_of<'a>(exp: &'a [testfix::Expected], path: &str) -> &'a testfix::Expected {
        exp.iter().find(|e| e.path == path).expect("expected entry")
    }

    /// Resolve a `/`-separated path from the synthetic root via `lookup`.
    fn resolve(fs: &Ad1Vfs, parts: &[&[u8]]) -> FileId {
        let mut id = fs.root();
        for p in parts {
            id = fs.lookup(id, p).unwrap().unwrap();
        }
        id
    }

    /// Drain a file to EOF by looping `read_at`.
    fn read_all(fs: &Ad1Vfs, id: FileId) -> Vec<u8> {
        let mut out = Vec::new();
        let mut off = 0u64;
        loop {
            let mut buf = [0u8; 4096];
            let n = fs.read_at(id, StreamId::Default, off, &mut buf).unwrap();
            if n == 0 {
                break;
            }
            out.extend_from_slice(&buf[..n]);
            off += n as u64;
        }
        out
    }

    #[test]
    fn kind_root_zone_and_sectors() {
        let (_d, fs, _e) = open_sample();
        assert_eq!(fs.kind(), FsKind::AD1);
        assert!(matches!(fs.root(), FileId::Opaque(0)));
        // AD1's timestamps are zoneless display strings, not epochs.
        assert_eq!(fs.timestamp_zone(), TimeZonePolicy::LocalUnknown);
        let ss = fs.sector_sizes();
        assert_eq!(ss.logical, 512);
        assert_eq!(ss.cluster_or_block, 512);
        assert!(ss.physical >= 512);
        assert_eq!(fs.meta(fs.root()).unwrap().kind, NodeKind::Dir);
    }

    #[test]
    fn lists_root_and_reaches_root_dir() {
        let (_d, fs, _e) = open_sample();
        let names: Vec<Vec<u8>> = fs
            .read_dir(fs.root())
            .unwrap()
            .map(|e| e.unwrap().name)
            .collect();
        assert!(
            names.iter().any(|n| n == b"root"),
            "synthetic root should list the 'root' dir, got {names:?}"
        );
        let root_dir = fs.lookup(fs.root(), b"root").unwrap().unwrap();
        assert_eq!(fs.meta(root_dir).unwrap().kind, NodeKind::Dir);
    }

    #[test]
    fn reads_hello_meta_and_content() {
        let (_d, fs, exp) = open_sample();
        let e = expected_of(&exp, "root/hello.txt");
        let id = resolve(&fs, &[b"root", b"hello.txt"]);
        let m = fs.meta(id).unwrap();
        assert_eq!(m.kind, NodeKind::File);
        assert_eq!(m.size, e.size);
        assert_eq!(m.allocated, Allocation::Allocated);
        // AD1 stores no epoch timestamps; honestly absent, never epoch-0.
        assert!(m.times.modified.is_none());
        assert!(m.times.accessed.is_none());
        assert!(m.times.changed.is_none());
        assert!(m.times.born.is_none());
        assert_eq!(m.uid, None);
        assert_eq!(m.gid, None);
        assert_eq!(m.mode, None);
        assert_eq!(read_all(&fs, id), *e.data.as_ref().unwrap());
    }

    #[test]
    fn reads_large_file_spanning_chunks() {
        let (_d, fs, exp) = open_sample();
        let e = expected_of(&exp, "root/sub/a.bin");
        let id = resolve(&fs, &[b"root", b"sub", b"a.bin"]);
        let m = fs.meta(id).unwrap();
        assert_eq!(m.size, e.size);
        assert!(m.size > u64::from(testfix::CHUNK_SIZE), "spans >1 chunk");
        assert_eq!(&read_all(&fs, id), e.data.as_ref().unwrap());
    }

    #[test]
    fn directory_reports_dir_kind() {
        let (_d, fs, _e) = open_sample();
        let id = resolve(&fs, &[b"root", b"sub"]);
        assert_eq!(fs.meta(id).unwrap().kind, NodeKind::Dir);
        assert!(fs.read_dir(id).is_ok());
    }

    #[test]
    fn empty_file_reads_zero_and_no_extents() {
        let (_d, fs, _e) = open_sample();
        let id = resolve(&fs, &[b"root", b"sub", b"empty.dat"]);
        let m = fs.meta(id).unwrap();
        assert_eq!(m.size, 0);
        assert_eq!(m.kind, NodeKind::File);
        let mut buf = [0u8; 8];
        assert_eq!(fs.read_at(id, StreamId::Default, 0, &mut buf).unwrap(), 0);
        assert_eq!(fs.extents(id, StreamId::Default).unwrap().count(), 0);
    }

    #[test]
    fn extents_hello_single_run_and_root() {
        let (_d, fs, exp) = open_sample();
        let e = expected_of(&exp, "root/hello.txt");
        let id = resolve(&fs, &[b"root", b"hello.txt"]);
        let runs: Vec<_> = fs
            .extents(id, StreamId::Default)
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].run.len, e.size);
        assert_eq!(runs[0].alloc, RunAlloc::Allocated);
        let root_runs: Vec<_> = fs
            .extents(fs.root(), StreamId::Default)
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        assert!(root_runs.len() <= 1);
    }

    #[test]
    fn read_at_offset_and_past_eof() {
        let (_d, fs, _e) = open_sample();
        let id = resolve(&fs, &[b"root", b"hello.txt"]);
        let mut buf = [0u8; 8];
        // "Hello, AD1!\n" — offset 7 is "AD1!\n".
        let n = fs.read_at(id, StreamId::Default, 7, &mut buf).unwrap();
        assert_eq!(&buf[..n], b"AD1!\n");
        assert_eq!(
            fs.read_at(id, StreamId::Default, 9999, &mut buf).unwrap(),
            0
        );
    }

    #[test]
    fn wrong_file_id_and_stream_are_loud() {
        let (_d, fs, _e) = open_sample();
        let bad = FileId::NtfsRef { entry: 5, seq: 1 };
        assert!(fs.meta(bad).is_err());
        assert!(fs.read_dir(bad).is_err());
        assert!(fs.lookup(bad, b"x").is_err());
        assert!(fs.read_link(bad, 8).is_err());
        // An out-of-range node index is refused.
        assert!(fs.meta(FileId::Opaque(9_999_999)).is_err());
        // A named stream is refused.
        let id = resolve(&fs, &[b"root", b"hello.txt"]);
        assert!(fs
            .read_at(id, StreamId::Named(1), 0, &mut [0u8; 4])
            .is_err());
        assert!(fs.extents(id, StreamId::Named(1)).is_err());
        // read_dir on a file is loud.
        assert!(fs.read_dir(id).is_err());
    }

    #[test]
    fn lookup_missing_is_none() {
        let (_d, fs, _e) = open_sample();
        assert!(fs.lookup(fs.root(), b"NOPE.NOTPRESENT").unwrap().is_none());
    }

    #[test]
    fn empty_forensic_surfaces() {
        let (_d, fs, _e) = open_sample();
        assert_eq!(fs.deleted().unwrap().count(), 0);
        assert_eq!(fs.unallocated().unwrap().count(), 0);
        let id = resolve(&fs, &[b"root", b"hello.txt"]);
        assert!(fs.read_link(id, 4096).unwrap().is_empty());
    }

    #[test]
    fn index_of_rejects_non_opaque() {
        assert!(super::index_of(FileId::Opaque(42)).is_ok());
        assert!(super::index_of(FileId::NtfsRef { entry: 1, seq: 1 }).is_err());
    }

    #[test]
    fn leaf_splits_on_last_separator() {
        assert_eq!(super::leaf("root/sub/a.bin"), b"a.bin");
        assert_eq!(super::leaf("toplevel"), b"toplevel");
        assert_eq!(super::leaf("a/b/c"), b"c");
    }
}
