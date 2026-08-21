use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

pub const CIBOR_MARK_SIZE: usize = 52;
pub const BUNDLE_CIBOR_PARTS_SIZE: usize = 69;
pub const STORAGE_LAYOUT_VERSION: u16 = 1;

const MARK_MAGIC: [u8; 4] = *b"KOFF";
const PARTS_MAGIC: [u8; 4] = *b"KTXP";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BundleKey {
    pub slot: u64,
    pub hash: [u8; 32],
}

impl BundleKey {
    pub fn new(slot: u64, hash: [u8; 32]) -> Self {
        Self { slot, hash }
    }
}

impl From<&CiborMark> for BundleKey {
    fn from(mark: &CiborMark) -> Self {
        Self::new(mark.bundle_slot, mark.bundle_hash)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CiborCacheKey {
    pub bundle: BundleKey,
    pub byte_offset: u32,
    pub byte_length: u32,
}

impl CiborCacheKey {
    pub fn from_mark(mark: &CiborMark) -> Self {
        Self {
            bundle: BundleKey::from(mark),
            byte_offset: mark.byte_offset,
            byte_length: mark.byte_length,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CiborMark {
    pub bundle_slot: u64,
    pub bundle_hash: [u8; 32],
    pub byte_offset: u32,
    pub byte_length: u32,
}

impl CiborMark {
    pub fn encode(&self) -> [u8; CIBOR_MARK_SIZE] {
        let mut buf = [0; CIBOR_MARK_SIZE];
        buf[0..4].copy_from_slice(&MARK_MAGIC);
        buf[4..12].copy_from_slice(&self.bundle_slot.to_be_bytes());
        buf[12..44].copy_from_slice(&self.bundle_hash);
        buf[44..48].copy_from_slice(&self.byte_offset.to_be_bytes());
        buf[48..52].copy_from_slice(&self.byte_length.to_be_bytes());
        buf
    }

    pub fn decode(data: &[u8]) -> Result<Self, StorageError> {
        if data.len() != CIBOR_MARK_SIZE {
            return Err(StorageError::InvalidSize {
                expected: CIBOR_MARK_SIZE,
                actual: data.len(),
            });
        }
        if data[0..4] != MARK_MAGIC {
            return Err(StorageError::InvalidMagic("KOFF"));
        }
        let mut bundle_hash = [0; 32];
        bundle_hash.copy_from_slice(&data[12..44]);
        Ok(Self {
            bundle_slot: u64::from_be_bytes(data[4..12].try_into().expect("slice length checked")),
            bundle_hash,
            byte_offset: u32::from_be_bytes(data[44..48].try_into().expect("slice length checked")),
            byte_length: u32::from_be_bytes(data[48..52].try_into().expect("slice length checked")),
        })
    }
}

pub fn is_cibor_mark_storage(data: &[u8]) -> bool {
    data.len() == CIBOR_MARK_SIZE && data[0..4] == MARK_MAGIC
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleCiborParts {
    pub bundle_slot: u64,
    pub bundle_hash: [u8; 32],
    pub body_offset: u32,
    pub body_length: u32,
    pub witness_offset: u32,
    pub witness_length: u32,
    pub auxiliary_data_offset: u32,
    pub auxiliary_data_length: u32,
    pub is_valid: bool,
}

impl BundleCiborParts {
    pub fn encode(&self) -> [u8; BUNDLE_CIBOR_PARTS_SIZE] {
        let mut buf = [0; BUNDLE_CIBOR_PARTS_SIZE];
        buf[0..4].copy_from_slice(&PARTS_MAGIC);
        buf[4..12].copy_from_slice(&self.bundle_slot.to_be_bytes());
        buf[12..44].copy_from_slice(&self.bundle_hash);
        buf[44..48].copy_from_slice(&self.body_offset.to_be_bytes());
        buf[48..52].copy_from_slice(&self.body_length.to_be_bytes());
        buf[52..56].copy_from_slice(&self.witness_offset.to_be_bytes());
        buf[56..60].copy_from_slice(&self.witness_length.to_be_bytes());
        buf[60..64].copy_from_slice(&self.auxiliary_data_offset.to_be_bytes());
        buf[64..68].copy_from_slice(&self.auxiliary_data_length.to_be_bytes());
        buf[68] = u8::from(self.is_valid);
        buf
    }

    pub fn decode(data: &[u8]) -> Result<Self, StorageError> {
        if data.len() != BUNDLE_CIBOR_PARTS_SIZE {
            return Err(StorageError::InvalidSize {
                expected: BUNDLE_CIBOR_PARTS_SIZE,
                actual: data.len(),
            });
        }
        if data[0..4] != PARTS_MAGIC {
            return Err(StorageError::InvalidMagic("KTXP"));
        }
        let mut bundle_hash = [0; 32];
        bundle_hash.copy_from_slice(&data[12..44]);
        Ok(Self {
            bundle_slot: u64::from_be_bytes(data[4..12].try_into().expect("slice length checked")),
            bundle_hash,
            body_offset: u32::from_be_bytes(data[44..48].try_into().expect("slice length checked")),
            body_length: u32::from_be_bytes(data[48..52].try_into().expect("slice length checked")),
            witness_offset: u32::from_be_bytes(
                data[52..56].try_into().expect("slice length checked"),
            ),
            witness_length: u32::from_be_bytes(
                data[56..60].try_into().expect("slice length checked"),
            ),
            auxiliary_data_offset: u32::from_be_bytes(
                data[60..64].try_into().expect("slice length checked"),
            ),
            auxiliary_data_length: u32::from_be_bytes(
                data[64..68].try_into().expect("slice length checked"),
            ),
            is_valid: data[68] != 0,
        })
    }

    pub fn has_auxiliary_data(&self) -> bool {
        self.auxiliary_data_length > 0
    }

    pub fn reassemble_bundle_cibor(&self, bundle_cibor: &[u8]) -> Result<Vec<u8>, StorageError> {
        let body = slice_range(bundle_cibor, self.body_offset, self.body_length, "body")?;
        let witness = slice_range(
            bundle_cibor,
            self.witness_offset,
            self.witness_length,
            "witness",
        )?;
        let auxiliary_data = if self.has_auxiliary_data() {
            Some(slice_range(
                bundle_cibor,
                self.auxiliary_data_offset,
                self.auxiliary_data_length,
                "auxiliary_data",
            )?)
        } else {
            None
        };
        Ok(build_bundle_cibor_array(
            body,
            witness,
            self.is_valid,
            auxiliary_data,
        ))
    }
}

pub fn is_bundle_cibor_parts_storage(data: &[u8]) -> bool {
    data.len() == BUNDLE_CIBOR_PARTS_SIZE && data[0..4] == PARTS_MAGIC
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageBackendKind {
    Memory,
    LocalPath,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageLayout {
    pub root: PathBuf,
    pub bundles_dir: PathBuf,
    pub cibor_marks_dir: PathBuf,
    pub metadata_dir: PathBuf,
}

impl StorageLayout {
    pub fn under(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        Self {
            bundles_dir: root.join("bundles"),
            cibor_marks_dir: root.join("cibor-marks"),
            metadata_dir: root.join("metadata"),
            root,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoragePlan {
    pub backend: StorageBackendKind,
    pub layout: StorageLayout,
    pub layout_version: u16,
    pub open_paths: bool,
}

impl StoragePlan {
    pub fn local_closed(root: impl Into<PathBuf>) -> Self {
        Self {
            backend: StorageBackendKind::LocalPath,
            layout: StorageLayout::under(root),
            layout_version: STORAGE_LAYOUT_VERSION,
            open_paths: false,
        }
    }

    pub fn memory() -> Self {
        Self {
            backend: StorageBackendKind::Memory,
            layout: StorageLayout::under(Path::new(":memory:")),
            layout_version: STORAGE_LAYOUT_VERSION,
            open_paths: false,
        }
    }

    pub fn local_open(root: impl Into<PathBuf>) -> Self {
        Self {
            backend: StorageBackendKind::LocalPath,
            layout: StorageLayout::under(root),
            layout_version: STORAGE_LAYOUT_VERSION,
            open_paths: true,
        }
    }

    pub fn with_open_paths(mut self, open_paths: bool) -> Self {
        self.open_paths = open_paths;
        self
    }

    pub fn planned_paths(&self) -> Vec<&Path> {
        vec![
            self.layout.root.as_path(),
            self.layout.bundles_dir.as_path(),
            self.layout.cibor_marks_dir.as_path(),
            self.layout.metadata_dir.as_path(),
        ]
    }

    pub fn read_layout_version(&self) -> Result<Option<u16>, StorageError> {
        self.assert_local_paths_open("read layout version")?;
        let path = self.layout_version_path();
        if !path.exists() {
            return Ok(None);
        }
        let bytes = fs::read(&path).map_err(|err| StorageError::Io {
            action: "read layout version",
            path: path.clone(),
            message: err.to_string(),
        })?;
        if bytes.len() != 2 {
            return Err(StorageError::InvalidLayoutVersionBytes {
                path,
                actual: bytes.len(),
            });
        }
        Ok(Some(u16::from_be_bytes([bytes[0], bytes[1]])))
    }

    fn layout_version_path(&self) -> PathBuf {
        self.layout.metadata_dir.join("layout.version")
    }

    fn assert_local_paths_open(&self, action: &'static str) -> Result<(), StorageError> {
        if self.backend != StorageBackendKind::LocalPath {
            return Err(StorageError::UnsupportedBackend(self.backend));
        }
        if !self.open_paths {
            return Err(StorageError::PathsClosed { action });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageMigrationStep {
    CreateLayout(StorageLayout),
    WriteLayoutVersion(u16),
    RebuildCiborMarks,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageMigrationPlan {
    pub current_version: Option<u16>,
    pub target_version: u16,
    pub steps: Vec<StorageMigrationStep>,
}

impl StorageMigrationPlan {
    pub fn for_store(store: &StoragePlan, current_version: Option<u16>) -> Self {
        let steps = match current_version {
            Some(version) if version >= store.layout_version => Vec::new(),
            Some(_) => vec![
                StorageMigrationStep::RebuildCiborMarks,
                StorageMigrationStep::WriteLayoutVersion(store.layout_version),
            ],
            None => vec![
                StorageMigrationStep::CreateLayout(store.layout.clone()),
                StorageMigrationStep::WriteLayoutVersion(store.layout_version),
            ],
        };
        Self {
            current_version,
            target_version: store.layout_version,
            steps,
        }
    }

    pub fn is_noop(&self) -> bool {
        self.steps.is_empty()
    }

    pub fn execute(&self, store: &StoragePlan) -> Result<StorageMigrationReport, StorageError> {
        store.assert_local_paths_open("execute storage migration")?;
        let mut created_dirs = Vec::new();
        let mut wrote_layout_version = None;
        let mut rebuilt_cibor_marks = false;
        for step in &self.steps {
            match step {
                StorageMigrationStep::CreateLayout(layout) => {
                    for path in [
                        layout.root.as_path(),
                        layout.bundles_dir.as_path(),
                        layout.cibor_marks_dir.as_path(),
                        layout.metadata_dir.as_path(),
                    ] {
                        fs::create_dir_all(path).map_err(|err| StorageError::Io {
                            action: "create storage directory",
                            path: path.to_path_buf(),
                            message: err.to_string(),
                        })?;
                        created_dirs.push(path.to_path_buf());
                    }
                }
                StorageMigrationStep::WriteLayoutVersion(version) => {
                    fs::create_dir_all(&store.layout.metadata_dir).map_err(|err| {
                        StorageError::Io {
                            action: "create metadata directory",
                            path: store.layout.metadata_dir.clone(),
                            message: err.to_string(),
                        }
                    })?;
                    let path = store.layout_version_path();
                    fs::write(&path, version.to_be_bytes()).map_err(|err| StorageError::Io {
                        action: "write layout version",
                        path,
                        message: err.to_string(),
                    })?;
                    wrote_layout_version = Some(*version);
                }
                StorageMigrationStep::RebuildCiborMarks => {
                    fs::create_dir_all(&store.layout.cibor_marks_dir).map_err(|err| {
                        StorageError::Io {
                            action: "create byte-range mark directory",
                            path: store.layout.cibor_marks_dir.clone(),
                            message: err.to_string(),
                        }
                    })?;
                    rebuilt_cibor_marks = true;
                }
            }
        }
        Ok(StorageMigrationReport {
            created_dirs,
            wrote_layout_version,
            rebuilt_cibor_marks,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageMigrationReport {
    pub created_dirs: Vec<PathBuf>,
    pub wrote_layout_version: Option<u16>,
    pub rebuilt_cibor_marks: bool,
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryBlockStore {
    bundles: HashMap<BundleKey, Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageManifestEntry {
    pub key: BundleKey,
    pub byte_len: usize,
    pub has_byte_ranges: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StorageManifest {
    pub entries: Vec<StorageManifestEntry>,
}

impl StorageManifest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, entry: StorageManifestEntry) {
        self.entries.push(entry);
    }

    pub fn total_bytes(&self) -> usize {
        self.entries
            .iter()
            .fold(0_usize, |total, entry| total.saturating_add(entry.byte_len))
    }

    pub fn latest_key(&self) -> Option<BundleKey> {
        self.entries
            .iter()
            .max_by_key(|entry| entry.key.slot)
            .map(|entry| entry.key)
    }

    pub fn validate(&self) -> Result<(), StorageManifestError> {
        let mut seen = HashSet::new();
        for entry in &self.entries {
            if entry.byte_len == 0 {
                return Err(StorageManifestError::EmptyBlock(entry.key));
            }
            if !seen.insert(entry.key) {
                return Err(StorageManifestError::DuplicateBlock(entry.key));
            }
        }
        Ok(())
    }

    pub fn encode_text(&self) -> String {
        let mut out = String::from("# acropolis storage manifest v1\n");
        let mut entries = self.entries.clone();
        entries.sort_by_key(|entry| (entry.key.slot, entry.key.hash));
        for entry in entries {
            out.push_str(&format!(
                "bundle={} hash={} bytes={} ranges={}\n",
                entry.key.slot,
                hex32(entry.key.hash),
                entry.byte_len,
                entry.has_byte_ranges
            ));
        }
        out
    }

    pub fn parse_text(input: &str) -> Result<Self, StorageError> {
        let mut manifest = Self::new();
        for (line_no, raw_line) in input.lines().enumerate() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut slot = None;
            let mut hash = None;
            let mut byte_len = None;
            let mut has_byte_ranges = None;
            for part in line.split_whitespace() {
                let Some((key, value)) = part.split_once('=') else {
                    return Err(StorageError::InvalidManifestLine { line: line_no + 1 });
                };
                match key {
                    "bundle" => slot = Some(parse_manifest_u64(value, line_no + 1)?),
                    "hash" => hash = Some(parse_hex32(value)?),
                    "bytes" => byte_len = Some(parse_manifest_usize(value, line_no + 1)?),
                    "ranges" => {
                        has_byte_ranges = Some(match value {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(StorageError::InvalidManifestLine { line: line_no + 1 })
                            }
                        })
                    }
                    _ => return Err(StorageError::InvalidManifestLine { line: line_no + 1 }),
                }
            }
            manifest.add(StorageManifestEntry {
                key: BundleKey::new(
                    slot.ok_or(StorageError::InvalidManifestLine { line: line_no + 1 })?,
                    hash.ok_or(StorageError::InvalidManifestLine { line: line_no + 1 })?,
                ),
                byte_len: byte_len
                    .ok_or(StorageError::InvalidManifestLine { line: line_no + 1 })?,
                has_byte_ranges: has_byte_ranges
                    .ok_or(StorageError::InvalidManifestLine { line: line_no + 1 })?,
            });
        }
        manifest.validate()?;
        Ok(manifest)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageManifestError {
    EmptyBlock(BundleKey),
    DuplicateBlock(BundleKey),
}

impl fmt::Display for StorageManifestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyBlock(key) => write!(f, "stored block at slot {} has no bytes", key.slot),
            Self::DuplicateBlock(key) => {
                write!(f, "duplicate stored block at slot {}", key.slot)
            }
        }
    }
}

impl std::error::Error for StorageManifestError {}

impl InMemoryBlockStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn put_bundle(&mut self, key: BundleKey, bytes: Vec<u8>) {
        self.bundles.insert(key, bytes);
    }

    pub fn has_bundle(&self, key: BundleKey) -> bool {
        self.bundles.contains_key(&key)
    }

    pub fn bundle_count(&self) -> usize {
        self.bundles.len()
    }

    pub fn resolve_mark(&self, mark: &CiborMark) -> Result<Vec<u8>, StorageError> {
        let key = BundleKey::from(mark);
        let bundle = self
            .bundles
            .get(&key)
            .ok_or(StorageError::MissingBundle(key))?;
        Ok(slice_range(bundle, mark.byte_offset, mark.byte_length, "mark")?.to_vec())
    }

    pub fn remove_bundle(&mut self, key: BundleKey) -> Option<Vec<u8>> {
        self.bundles.remove(&key)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalBlockStore {
    layout: StorageLayout,
}

impl LocalBlockStore {
    pub fn open(plan: &StoragePlan) -> Result<Self, StorageError> {
        plan.assert_local_paths_open("open local block store")?;
        let current_version = plan.read_layout_version()?;
        let migration = StorageMigrationPlan::for_store(plan, current_version);
        migration.execute(plan)?;
        Ok(Self {
            layout: plan.layout.clone(),
        })
    }

    pub fn put_bundle(&self, key: BundleKey, bytes: &[u8]) -> Result<(), StorageError> {
        if bytes.is_empty() {
            return Err(StorageError::EmptyBundle(key));
        }
        let path = self.bundle_path(key);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|err| StorageError::Io {
                action: "create bundle directory",
                path: parent.to_path_buf(),
                message: err.to_string(),
            })?;
        }
        fs::write(&path, bytes).map_err(|err| StorageError::Io {
            action: "write bundle",
            path,
            message: err.to_string(),
        })
    }

    pub fn get_bundle(&self, key: BundleKey) -> Result<Vec<u8>, StorageError> {
        let path = self.bundle_path(key);
        match fs::read(&path) {
            Ok(bytes) => Ok(bytes),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                Err(StorageError::MissingBundle(key))
            }
            Err(err) => Err(StorageError::Io {
                action: "read bundle",
                path,
                message: err.to_string(),
            }),
        }
    }

    pub fn has_bundle(&self, key: BundleKey) -> bool {
        self.bundle_path(key).exists()
    }

    pub fn remove_bundle(&self, key: BundleKey) -> Result<bool, StorageError> {
        let path = self.bundle_path(key);
        match fs::remove_file(&path) {
            Ok(()) => Ok(true),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(err) => Err(StorageError::Io {
                action: "remove bundle",
                path,
                message: err.to_string(),
            }),
        }
    }

    pub fn write_manifest(&self, manifest: &StorageManifest) -> Result<PathBuf, StorageError> {
        manifest.validate()?;
        fs::create_dir_all(&self.layout.metadata_dir).map_err(|err| StorageError::Io {
            action: "create metadata directory",
            path: self.layout.metadata_dir.clone(),
            message: err.to_string(),
        })?;
        let path = self.manifest_path();
        fs::write(&path, manifest.encode_text()).map_err(|err| StorageError::Io {
            action: "write storage manifest",
            path: path.clone(),
            message: err.to_string(),
        })?;
        Ok(path)
    }

    pub fn read_manifest(&self) -> Result<Option<StorageManifest>, StorageError> {
        let path = self.manifest_path();
        if !path.exists() {
            return Ok(None);
        }
        let text = fs::read_to_string(&path).map_err(|err| StorageError::Io {
            action: "read storage manifest",
            path,
            message: err.to_string(),
        })?;
        StorageManifest::parse_text(&text).map(Some)
    }

    pub fn rebuild_manifest(&self) -> Result<StorageManifest, StorageError> {
        let mut manifest = StorageManifest::new();
        if !self.layout.bundles_dir.exists() {
            return Ok(manifest);
        }
        for entry in fs::read_dir(&self.layout.bundles_dir).map_err(|err| StorageError::Io {
            action: "read bundle directory",
            path: self.layout.bundles_dir.clone(),
            message: err.to_string(),
        })? {
            let entry = entry.map_err(|err| StorageError::Io {
                action: "read bundle directory entry",
                path: self.layout.bundles_dir.clone(),
                message: err.to_string(),
            })?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("cbor") {
                continue;
            }
            let key = parse_bundle_file_name(&path)?;
            let byte_len = entry
                .metadata()
                .map_err(|err| StorageError::Io {
                    action: "read bundle metadata",
                    path: path.clone(),
                    message: err.to_string(),
                })?
                .len() as usize;
            manifest.add(StorageManifestEntry {
                key,
                byte_len,
                has_byte_ranges: self.has_any_mark_for(key)?,
            });
        }
        manifest.validate()?;
        manifest
            .entries
            .sort_by_key(|entry| (entry.key.slot, entry.key.hash));
        Ok(manifest)
    }

    pub fn resolve_mark(&self, mark: &CiborMark) -> Result<Vec<u8>, StorageError> {
        let key = BundleKey::from(mark);
        let bundle = self.get_bundle(key)?;
        Ok(slice_range(&bundle, mark.byte_offset, mark.byte_length, "mark")?.to_vec())
    }

    pub fn write_mark(&self, mark: &CiborMark) -> Result<PathBuf, StorageError> {
        let path = self.mark_path(BundleKey::from(mark), mark.byte_offset, mark.byte_length);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|err| StorageError::Io {
                action: "create byte-range mark directory",
                path: parent.to_path_buf(),
                message: err.to_string(),
            })?;
        }
        fs::write(&path, mark.encode()).map_err(|err| StorageError::Io {
            action: "write byte-range mark",
            path: path.clone(),
            message: err.to_string(),
        })?;
        Ok(path)
    }

    pub fn rebuild_hot_cibor_cache(
        &self,
        capacity: usize,
    ) -> Result<HotCiborCacheRebuild, StorageError> {
        let mut cache = HotCiborCache::new(capacity);
        let mut scanned_marks = 0;
        let mut skipped_files = 0;
        if !self.layout.cibor_marks_dir.exists() {
            return Ok(HotCiborCacheRebuild {
                cache,
                scanned_marks,
                cached_marks: 0,
                skipped_files,
            });
        }
        let mut paths = Vec::new();
        for entry in fs::read_dir(&self.layout.cibor_marks_dir).map_err(|err| StorageError::Io {
            action: "read byte-range mark directory",
            path: self.layout.cibor_marks_dir.clone(),
            message: err.to_string(),
        })? {
            let entry = entry.map_err(|err| StorageError::Io {
                action: "read byte-range mark directory entry",
                path: self.layout.cibor_marks_dir.clone(),
                message: err.to_string(),
            })?;
            paths.push(entry.path());
        }
        paths.sort();
        for path in paths {
            if path.extension().and_then(|value| value.to_str()) != Some("mark") {
                skipped_files += 1;
                continue;
            }
            scanned_marks += 1;
            let bytes = fs::read(&path).map_err(|err| StorageError::Io {
                action: "read byte-range mark",
                path: path.clone(),
                message: err.to_string(),
            })?;
            let mark = CiborMark::decode(&bytes)?;
            let value = self.resolve_mark(&mark)?;
            cache.insert(CiborCacheKey::from_mark(&mark), value);
        }
        let cached_marks = cache.len();
        Ok(HotCiborCacheRebuild {
            cache,
            scanned_marks,
            cached_marks,
            skipped_files,
        })
    }

    pub fn rebuild_hot_cibor_cache_tolerant(
        &self,
        capacity: usize,
    ) -> Result<HotCiborCacheTolerantRebuild, StorageError> {
        let mut cache = HotCiborCache::new(capacity);
        let mut scanned_marks = 0;
        let mut skipped_files = 0;
        let mut invalid_marks = 0;
        let mut missing_bundles = 0;
        let mut skipped_marks = Vec::new();
        if !self.layout.cibor_marks_dir.exists() {
            return Ok(HotCiborCacheTolerantRebuild {
                cache,
                scanned_marks,
                cached_marks: 0,
                skipped_files,
                invalid_marks,
                missing_bundles,
                skipped_marks,
            });
        }
        let mut paths = Vec::new();
        for entry in fs::read_dir(&self.layout.cibor_marks_dir).map_err(|err| StorageError::Io {
            action: "read byte-range mark directory",
            path: self.layout.cibor_marks_dir.clone(),
            message: err.to_string(),
        })? {
            let entry = entry.map_err(|err| StorageError::Io {
                action: "read byte-range mark directory entry",
                path: self.layout.cibor_marks_dir.clone(),
                message: err.to_string(),
            })?;
            paths.push(entry.path());
        }
        paths.sort();
        for path in paths {
            if path.extension().and_then(|value| value.to_str()) != Some("mark") {
                skipped_files += 1;
                continue;
            }
            scanned_marks += 1;
            let bytes = fs::read(&path).map_err(|err| StorageError::Io {
                action: "read byte-range mark",
                path: path.clone(),
                message: err.to_string(),
            })?;
            let mark = match CiborMark::decode(&bytes) {
                Ok(mark) => mark,
                Err(error) => {
                    invalid_marks += 1;
                    skipped_marks.push(HotCiborCacheSkippedMark { path, error });
                    continue;
                }
            };
            match self.resolve_mark(&mark) {
                Ok(value) => cache.insert(CiborCacheKey::from_mark(&mark), value),
                Err(error @ StorageError::MissingBundle(_)) => {
                    missing_bundles += 1;
                    skipped_marks.push(HotCiborCacheSkippedMark { path, error });
                }
                Err(error @ StorageError::RangeOutOfBounds { .. }) => {
                    invalid_marks += 1;
                    skipped_marks.push(HotCiborCacheSkippedMark { path, error });
                }
                Err(error) => return Err(error),
            }
        }
        let cached_marks = cache.len();
        Ok(HotCiborCacheTolerantRebuild {
            cache,
            scanned_marks,
            cached_marks,
            skipped_files,
            invalid_marks,
            missing_bundles,
            skipped_marks,
        })
    }

    fn bundle_path(&self, key: BundleKey) -> PathBuf {
        self.layout
            .bundles_dir
            .join(format!("{:020}-{}.cbor", key.slot, hex32(key.hash)))
    }

    fn manifest_path(&self) -> PathBuf {
        self.layout.metadata_dir.join("manifest.txt")
    }

    fn has_any_mark_for(&self, key: BundleKey) -> Result<bool, StorageError> {
        if !self.layout.cibor_marks_dir.exists() {
            return Ok(false);
        }
        let prefix = format!("{:020}-{}-", key.slot, hex32(key.hash));
        for entry in fs::read_dir(&self.layout.cibor_marks_dir).map_err(|err| StorageError::Io {
            action: "read byte-range mark directory",
            path: self.layout.cibor_marks_dir.clone(),
            message: err.to_string(),
        })? {
            let entry = entry.map_err(|err| StorageError::Io {
                action: "read byte-range mark directory entry",
                path: self.layout.cibor_marks_dir.clone(),
                message: err.to_string(),
            })?;
            let Some(name) = entry.file_name().to_str().map(str::to_string) else {
                continue;
            };
            if name.starts_with(&prefix) && name.ends_with(".mark") {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn mark_path(&self, key: BundleKey, offset: u32, length: u32) -> PathBuf {
        self.layout.cibor_marks_dir.join(format!(
            "{:020}-{}-{:010}-{:010}.mark",
            key.slot,
            hex32(key.hash),
            offset,
            length
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HotCiborCache {
    capacity: usize,
    order: VecDeque<CiborCacheKey>,
    entries: HashMap<CiborCacheKey, Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HotCiborCacheRebuild {
    pub cache: HotCiborCache,
    pub scanned_marks: usize,
    pub cached_marks: usize,
    pub skipped_files: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HotCiborCacheTolerantRebuild {
    pub cache: HotCiborCache,
    pub scanned_marks: usize,
    pub cached_marks: usize,
    pub skipped_files: usize,
    pub invalid_marks: usize,
    pub missing_bundles: usize,
    pub skipped_marks: Vec<HotCiborCacheSkippedMark>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HotCiborCacheSkippedMark {
    pub path: PathBuf,
    pub error: StorageError,
}

impl HotCiborCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            order: VecDeque::new(),
            entries: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: CiborCacheKey, value: Vec<u8>) {
        if self.capacity == 0 {
            return;
        }
        if self.entries.contains_key(&key) {
            self.order.retain(|existing| existing != &key);
        }
        self.order.push_back(key.clone());
        self.entries.insert(key, value);
        while self.entries.len() > self.capacity {
            if let Some(oldest) = self.order.pop_front() {
                self.entries.remove(&oldest);
            }
        }
    }

    pub fn get(&mut self, key: &CiborCacheKey) -> Option<&[u8]> {
        if self.entries.contains_key(key) {
            self.order.retain(|existing| existing != key);
            self.order.push_back(key.clone());
        }
        self.entries.get(key).map(Vec::as_slice)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FadePlanner {
    pub stability_slots: u64,
}

impl FadePlanner {
    pub fn new(stability_slots: u64) -> Self {
        Self { stability_slots }
    }

    pub fn should_fade(&self, bundle_slot: u64, tip_slot: u64) -> bool {
        tip_slot >= bundle_slot.saturating_add(self.stability_slots)
    }

    pub fn fade_candidates<'a>(
        &self,
        keys: impl IntoIterator<Item = &'a BundleKey>,
        tip_slot: u64,
    ) -> Vec<BundleKey> {
        keys.into_iter()
            .copied()
            .filter(|key| self.should_fade(key.slot, tip_slot))
            .collect()
    }
}

fn slice_range<'a>(
    bundle: &'a [u8],
    offset: u32,
    length: u32,
    label: &'static str,
) -> Result<&'a [u8], StorageError> {
    let offset = offset as usize;
    let length = length as usize;
    let end = offset
        .checked_add(length)
        .ok_or(StorageError::RangeOutOfBounds {
            label,
            offset,
            length,
            bundle_size: bundle.len(),
        })?;
    if offset > bundle.len() || end > bundle.len() {
        return Err(StorageError::RangeOutOfBounds {
            label,
            offset,
            length,
            bundle_size: bundle.len(),
        });
    }
    Ok(&bundle[offset..end])
}

fn build_bundle_cibor_array(
    body: &[u8],
    witness: &[u8],
    is_valid: bool,
    auxiliary_data: Option<&[u8]>,
) -> Vec<u8> {
    let mut result = Vec::with_capacity(
        1 + body.len() + witness.len() + 1 + auxiliary_data.map_or(1, <[u8]>::len),
    );
    result.push(0x84);
    result.extend_from_slice(body);
    result.extend_from_slice(witness);
    result.push(if is_valid { 0xf5 } else { 0xf4 });
    if let Some(auxiliary_data) = auxiliary_data {
        result.extend_from_slice(auxiliary_data);
    } else {
        result.push(0xf6);
    }
    result
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageError {
    InvalidSize {
        expected: usize,
        actual: usize,
    },
    InvalidMagic(&'static str),
    MissingBundle(BundleKey),
    EmptyBundle(BundleKey),
    PathsClosed {
        action: &'static str,
    },
    UnsupportedBackend(StorageBackendKind),
    InvalidLayoutVersionBytes {
        path: PathBuf,
        actual: usize,
    },
    InvalidManifestLine {
        line: usize,
    },
    InvalidHash(String),
    Io {
        action: &'static str,
        path: PathBuf,
        message: String,
    },
    Manifest(StorageManifestError),
    RangeOutOfBounds {
        label: &'static str,
        offset: usize,
        length: usize,
        bundle_size: usize,
    },
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSize { expected, actual } => {
                write!(
                    f,
                    "invalid data size: expected {expected} bytes, got {actual}"
                )
            }
            Self::InvalidMagic(expected) => write!(f, "invalid magic: expected {expected}"),
            Self::MissingBundle(key) => write!(
                f,
                "missing bundle at slot {} with hash prefix {:02x}{:02x}{:02x}{:02x}",
                key.slot, key.hash[0], key.hash[1], key.hash[2], key.hash[3]
            ),
            Self::EmptyBundle(key) => write!(f, "empty bundle at slot {}", key.slot),
            Self::PathsClosed { action } => {
                write!(f, "storage paths are closed for action {action}")
            }
            Self::UnsupportedBackend(backend) => {
                write!(f, "unsupported storage backend {backend:?}")
            }
            Self::InvalidLayoutVersionBytes { path, actual } => write!(
                f,
                "invalid layout version bytes at {}: expected 2 got {actual}",
                path.display()
            ),
            Self::InvalidManifestLine { line } => write!(f, "invalid manifest line {line}"),
            Self::InvalidHash(value) => write!(f, "invalid hash {value}"),
            Self::Io {
                action,
                path,
                message,
            } => write!(
                f,
                "storage {action} failed at {}: {message}",
                path.display()
            ),
            Self::Manifest(err) => write!(f, "storage manifest error: {err}"),
            Self::RangeOutOfBounds {
                label,
                offset,
                length,
                bundle_size,
            } => write!(
                f,
                "{label} range [{offset}:{}] exceeds bundle size {bundle_size}",
                offset + length
            ),
        }
    }
}

impl std::error::Error for StorageError {}

impl From<StorageManifestError> for StorageError {
    fn from(value: StorageManifestError) -> Self {
        Self::Manifest(value)
    }
}

fn hex32(bytes: [u8; 32]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(64);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

fn parse_hex32(value: &str) -> Result<[u8; 32], StorageError> {
    if value.len() != 64 {
        return Err(StorageError::InvalidHash(value.to_string()));
    }
    let mut bytes = [0; 32];
    for (index, byte) in bytes.iter_mut().enumerate() {
        let start = index * 2;
        let part = &value[start..start + 2];
        *byte = u8::from_str_radix(part, 16)
            .map_err(|_| StorageError::InvalidHash(value.to_string()))?;
    }
    Ok(bytes)
}

fn parse_manifest_u64(value: &str, line: usize) -> Result<u64, StorageError> {
    value
        .parse()
        .map_err(|_| StorageError::InvalidManifestLine { line })
}

fn parse_manifest_usize(value: &str, line: usize) -> Result<usize, StorageError> {
    value
        .parse()
        .map_err(|_| StorageError::InvalidManifestLine { line })
}

fn parse_bundle_file_name(path: &Path) -> Result<BundleKey, StorageError> {
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or(StorageError::InvalidManifestLine { line: 0 })?;
    let stem = name
        .strip_suffix(".cbor")
        .ok_or(StorageError::InvalidManifestLine { line: 0 })?;
    let (slot, hash) = stem
        .split_once('-')
        .ok_or(StorageError::InvalidManifestLine { line: 0 })?;
    Ok(BundleKey::new(
        parse_manifest_u64(slot, 0)?,
        parse_hex32(hash)?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_storage_root(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "acropolis-storage-{label}-{}-{nanos}",
            std::process::id()
        ))
    }

    #[test]
    fn cibor_mark_round_trips_storage_layout() {
        let mark = CiborMark {
            bundle_slot: 42,
            bundle_hash: [7; 32],
            byte_offset: 10,
            byte_length: 20,
        };

        let encoded = mark.encode();
        assert_eq!(&encoded[0..4], b"KOFF");
        assert_eq!(encoded.len(), CIBOR_MARK_SIZE);
        assert_eq!(CiborMark::decode(&encoded).unwrap(), mark);
        assert!(is_cibor_mark_storage(&encoded));
    }

    #[test]
    fn bundle_cibor_parts_round_trip_and_reassemble() {
        let bundle = [0x01, 0x02, 0xa0, 0xa1, 0x99];
        let parts = BundleCiborParts {
            bundle_slot: 9,
            bundle_hash: [3; 32],
            body_offset: 0,
            body_length: 2,
            witness_offset: 2,
            witness_length: 1,
            auxiliary_data_offset: 3,
            auxiliary_data_length: 1,
            is_valid: true,
        };

        let encoded = parts.encode();
        assert_eq!(&encoded[0..4], b"KTXP");
        assert_eq!(BundleCiborParts::decode(&encoded).unwrap(), parts);
        assert_eq!(
            parts.reassemble_bundle_cibor(&bundle).unwrap(),
            vec![0x84, 0x01, 0x02, 0xa0, 0xf5, 0xa1]
        );
    }

    #[test]
    fn reassemble_checks_bounds() {
        let parts = BundleCiborParts {
            bundle_slot: 0,
            bundle_hash: [0; 32],
            body_offset: 5,
            body_length: 1,
            witness_offset: 0,
            witness_length: 0,
            auxiliary_data_offset: 0,
            auxiliary_data_length: 0,
            is_valid: false,
        };

        assert!(matches!(
            parts.reassemble_bundle_cibor(&[0; 2]),
            Err(StorageError::RangeOutOfBounds { label: "body", .. })
        ));
    }

    #[test]
    fn in_memory_block_store_resolves_marks_from_bundles() {
        let mut store = InMemoryBlockStore::new();
        let key = BundleKey::new(12, [4; 32]);
        store.put_bundle(key, vec![10, 11, 12, 13, 14]);
        let mark = CiborMark {
            bundle_slot: 12,
            bundle_hash: [4; 32],
            byte_offset: 1,
            byte_length: 3,
        };
        assert_eq!(store.resolve_mark(&mark).unwrap(), vec![11, 12, 13]);
    }

    #[test]
    fn storage_manifest_validates_local_block_metadata() {
        let mut manifest = StorageManifest::new();
        let first = BundleKey::new(1, [1; 32]);
        let second = BundleKey::new(2, [2; 32]);
        manifest.add(StorageManifestEntry {
            key: first,
            byte_len: 10,
            has_byte_ranges: true,
        });
        manifest.add(StorageManifestEntry {
            key: second,
            byte_len: 20,
            has_byte_ranges: false,
        });

        assert_eq!(manifest.validate(), Ok(()));
        assert_eq!(manifest.total_bytes(), 30);
        assert_eq!(manifest.latest_key(), Some(second));
    }

    #[test]
    fn storage_manifest_rejects_duplicate_blocks() {
        let key = BundleKey::new(1, [1; 32]);
        let mut manifest = StorageManifest::new();
        manifest.add(StorageManifestEntry {
            key,
            byte_len: 10,
            has_byte_ranges: true,
        });
        manifest.add(StorageManifestEntry {
            key,
            byte_len: 10,
            has_byte_ranges: true,
        });

        assert_eq!(
            manifest.validate(),
            Err(StorageManifestError::DuplicateBlock(key))
        );
    }

    #[test]
    fn storage_manifest_text_round_trips_deterministically() {
        let later = BundleKey::new(9, [9; 32]);
        let earlier = BundleKey::new(1, [1; 32]);
        let mut manifest = StorageManifest::new();
        manifest.add(StorageManifestEntry {
            key: later,
            byte_len: 90,
            has_byte_ranges: false,
        });
        manifest.add(StorageManifestEntry {
            key: earlier,
            byte_len: 10,
            has_byte_ranges: true,
        });

        let encoded = manifest.encode_text();
        assert!(encoded.starts_with("# acropolis storage manifest v1\n"));
        assert!(encoded.find("bundle=1").unwrap() < encoded.find("bundle=9").unwrap());
        let parsed = StorageManifest::parse_text(&encoded).unwrap();
        assert_eq!(parsed.encode_text(), encoded);
        assert_eq!(parsed.total_bytes(), manifest.total_bytes());
    }

    #[test]
    fn storage_plan_lists_paths_without_opening_them() {
        let plan = StoragePlan::local_closed(".acropolis");
        assert!(!plan.open_paths);
        assert_eq!(plan.backend, StorageBackendKind::LocalPath);
        assert_eq!(plan.planned_paths().len(), 4);
        assert_eq!(plan.layout.bundles_dir, PathBuf::from(".acropolis/bundles"));
    }

    #[test]
    fn closed_storage_plan_refuses_path_execution() {
        let plan = StoragePlan::local_closed(".acropolis");
        assert_eq!(
            plan.read_layout_version(),
            Err(StorageError::PathsClosed {
                action: "read layout version"
            })
        );
        assert_eq!(
            LocalBlockStore::open(&plan),
            Err(StorageError::PathsClosed {
                action: "open local block store"
            })
        );
    }

    #[test]
    fn storage_migration_plan_is_local_and_deterministic() {
        let store = StoragePlan::local_closed(".acropolis");
        let fresh = StorageMigrationPlan::for_store(&store, None);
        assert_eq!(fresh.target_version, STORAGE_LAYOUT_VERSION);
        assert!(matches!(
            fresh.steps.as_slice(),
            [
                StorageMigrationStep::CreateLayout(_),
                StorageMigrationStep::WriteLayoutVersion(STORAGE_LAYOUT_VERSION)
            ]
        ));
        assert!(StorageMigrationPlan::for_store(&store, Some(STORAGE_LAYOUT_VERSION)).is_noop());
    }

    #[test]
    fn storage_migration_executes_only_when_paths_are_open() {
        let root = temp_storage_root("migration");
        let plan = StoragePlan::local_open(&root);
        let migration = StorageMigrationPlan::for_store(&plan, None);
        let report = migration.execute(&plan).unwrap();
        assert_eq!(report.wrote_layout_version, Some(STORAGE_LAYOUT_VERSION));
        assert!(plan.layout.root.exists());
        assert!(plan.layout.bundles_dir.exists());
        assert!(plan.layout.cibor_marks_dir.exists());
        assert_eq!(
            plan.read_layout_version().unwrap(),
            Some(STORAGE_LAYOUT_VERSION)
        );
        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    fn local_block_store_round_trips_bundles_and_marks() {
        let root = temp_storage_root("blocks");
        let plan = StoragePlan::local_open(&root);
        let store = LocalBlockStore::open(&plan).unwrap();
        let key = BundleKey::new(44, [9; 32]);
        store.put_bundle(key, &[10, 11, 12, 13, 14]).unwrap();
        assert!(store.has_bundle(key));
        assert_eq!(store.get_bundle(key).unwrap(), vec![10, 11, 12, 13, 14]);

        let mark = CiborMark {
            bundle_slot: 44,
            bundle_hash: [9; 32],
            byte_offset: 1,
            byte_length: 3,
        };
        let mark_path = store.write_mark(&mark).unwrap();
        assert!(mark_path.exists());
        assert_eq!(store.resolve_mark(&mark).unwrap(), vec![11, 12, 13]);
        assert!(store.remove_bundle(key).unwrap());
        assert!(!store.remove_bundle(key).unwrap());
        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    fn local_block_store_writes_reads_and_rebuilds_manifest() {
        let root = temp_storage_root("manifest");
        let plan = StoragePlan::local_open(&root);
        let store = LocalBlockStore::open(&plan).unwrap();
        let key = BundleKey::new(44, [9; 32]);
        store.put_bundle(key, &[10, 11, 12, 13]).unwrap();
        store
            .write_mark(&CiborMark {
                bundle_slot: key.slot,
                bundle_hash: key.hash,
                byte_offset: 1,
                byte_length: 2,
            })
            .unwrap();

        let rebuilt = store.rebuild_manifest().unwrap();
        assert_eq!(rebuilt.total_bytes(), 4);
        assert_eq!(rebuilt.entries[0].key, key);
        assert!(rebuilt.entries[0].has_byte_ranges);

        let manifest_path = store.write_manifest(&rebuilt).unwrap();
        assert!(manifest_path.exists());
        assert_eq!(store.read_manifest().unwrap(), Some(rebuilt));
        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    fn local_block_store_rebuilds_hot_cache_from_marks() {
        let root = temp_storage_root("cache-rebuild");
        let plan = StoragePlan::local_open(&root);
        let store = LocalBlockStore::open(&plan).unwrap();
        let key = BundleKey::new(44, [9; 32]);
        store.put_bundle(key, &[10, 11, 12, 13, 14]).unwrap();
        let first = CiborMark {
            bundle_slot: key.slot,
            bundle_hash: key.hash,
            byte_offset: 1,
            byte_length: 2,
        };
        let second = CiborMark {
            bundle_slot: key.slot,
            bundle_hash: key.hash,
            byte_offset: 3,
            byte_length: 2,
        };
        store.write_mark(&first).unwrap();
        store.write_mark(&second).unwrap();
        std::fs::write(plan.layout.cibor_marks_dir.join("ignore.tmp"), b"skip").unwrap();

        let mut report = store.rebuild_hot_cibor_cache(4).unwrap();
        assert_eq!(report.scanned_marks, 2);
        assert_eq!(report.cached_marks, 2);
        assert_eq!(report.skipped_files, 1);
        assert_eq!(
            report.cache.get(&CiborCacheKey::from_mark(&first)),
            Some(&[11, 12][..])
        );
        assert_eq!(
            report.cache.get(&CiborCacheKey::from_mark(&second)),
            Some(&[13, 14][..])
        );
        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    fn local_block_store_tolerant_cache_rebuild_reports_skipped_marks() {
        let root = temp_storage_root("cache-rebuild-tolerant");
        let plan = StoragePlan::local_open(&root);
        let store = LocalBlockStore::open(&plan).unwrap();
        let key = BundleKey::new(44, [9; 32]);
        store.put_bundle(key, &[10, 11, 12, 13, 14]).unwrap();
        let valid = CiborMark {
            bundle_slot: key.slot,
            bundle_hash: key.hash,
            byte_offset: 1,
            byte_length: 2,
        };
        let missing = CiborMark {
            bundle_slot: 45,
            bundle_hash: [8; 32],
            byte_offset: 0,
            byte_length: 1,
        };
        store.write_mark(&valid).unwrap();
        store.write_mark(&missing).unwrap();
        std::fs::write(plan.layout.cibor_marks_dir.join("bad.mark"), b"bad").unwrap();
        std::fs::write(plan.layout.cibor_marks_dir.join("ignore.tmp"), b"skip").unwrap();

        let mut report = store.rebuild_hot_cibor_cache_tolerant(4).unwrap();

        assert_eq!(report.scanned_marks, 3);
        assert_eq!(report.cached_marks, 1);
        assert_eq!(report.skipped_files, 1);
        assert_eq!(report.invalid_marks, 1);
        assert_eq!(report.missing_bundles, 1);
        assert_eq!(report.skipped_marks.len(), 2);
        assert!(report
            .skipped_marks
            .iter()
            .any(|skipped| matches!(skipped.error, StorageError::InvalidSize { .. })));
        assert!(report
            .skipped_marks
            .iter()
            .any(|skipped| matches!(skipped.error, StorageError::MissingBundle(_))));
        assert_eq!(
            report.cache.get(&CiborCacheKey::from_mark(&valid)),
            Some(&[11, 12][..])
        );
        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    fn local_block_store_rejects_empty_bundles() {
        let root = temp_storage_root("empty");
        let plan = StoragePlan::local_open(&root);
        let store = LocalBlockStore::open(&plan).unwrap();
        let key = BundleKey::new(1, [1; 32]);
        assert_eq!(
            store.put_bundle(key, &[]),
            Err(StorageError::EmptyBundle(key))
        );
        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    fn hot_cache_evicts_oldest_entry() {
        let mut cache = HotCiborCache::new(1);
        let first = CiborCacheKey {
            bundle: BundleKey::new(1, [1; 32]),
            byte_offset: 0,
            byte_length: 1,
        };
        let second = CiborCacheKey {
            bundle: BundleKey::new(2, [2; 32]),
            byte_offset: 0,
            byte_length: 1,
        };
        cache.insert(first.clone(), vec![1]);
        cache.insert(second.clone(), vec![2]);
        assert!(cache.get(&first).is_none());
        assert_eq!(cache.get(&second), Some(&[2][..]));
    }

    #[test]
    fn fade_planner_selects_stable_old_bundles() {
        let planner = FadePlanner::new(5);
        let young = BundleKey::new(8, [1; 32]);
        let old = BundleKey::new(4, [2; 32]);
        assert_eq!(planner.fade_candidates([&young, &old], 9), vec![old]);
    }
}
