use crate::aaml::parsing::{strip_comment, unwrap_quotes};
use crate::error::AamlError;
use crate::pipeline::Pipeline;
use memmap2::{Mmap, MmapOptions};
use rustc_hash::FxHasher;
use std::fs;
use std::fs::File;
use std::hash::Hasher;
use std::path::{Path, PathBuf};
use tinyvec::TinyVec;

// AOT mode-specific code paths are cfg-gated below and can coexist in all-features builds.

const INVALID_INDEX: u32 = u32::MAX;
const CURRENT_AOT_VERSION: u32 = 1;
type FrontendErrors = TinyVec<[Option<AamlError>; 4]>;

#[inline]
fn push_frontend_error(errors: &mut FrontendErrors, err: AamlError) {
    errors.push(Some(err));
}

#[inline]
fn finalize_frontend_errors(errors: FrontendErrors) -> Option<Vec<AamlError>> {
    if errors.is_empty() {
        return None;
    }

    Some(errors.into_iter().flatten().collect())
}

#[repr(u32)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug, Clone, Copy)]
pub enum NodeKind {
    Root = 0,
    Assignment = 1,
}

/// Byte-range into `string_blob`.
#[repr(C)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug, Clone, Copy, Default)]
pub struct Span {
    pub start: u32,
    pub len: u32,
}

/// Fixed-size (16-byte) node used by the cooked SoA layout.
#[repr(C)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug, Clone, Copy)]
pub struct FlatNode {
    pub kind: u32,
    pub parent: u32,
    pub first_child: u32,
    pub next_sibling: u32,
}

#[repr(C)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug, Clone, Copy)]
pub struct SymbolEntry {
    pub hash: u64,
    pub node_index: u32,
}

/// Fully cooked DOD payload used by runtime mmap loading.
#[repr(C)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug, Clone)]
pub struct CookedAotBlob {
    pub version: u32,
    pub string_blob: Vec<u8>,
    pub nodes: Vec<FlatNode>,
    pub key_spans: Vec<Span>,
    pub value_spans: Vec<Span>,
    pub hash_table: Vec<SymbolEntry>,
}

impl CookedAotBlob {
    fn empty() -> Self {
        Self {
            version: CURRENT_AOT_VERSION,
            string_blob: Vec::new(),
            nodes: vec![FlatNode {
                kind: NodeKind::Root as u32,
                parent: INVALID_INDEX,
                first_child: INVALID_INDEX,
                next_sibling: INVALID_INDEX,
            }],
            key_spans: vec![Span::default()],
            value_spans: vec![Span::default()],
            hash_table: Vec::new(),
        }
    }
}

struct SoaBuilder {
    string_blob: Vec<u8>,
    nodes: Vec<FlatNode>,
    key_spans: Vec<Span>,
    value_spans: Vec<Span>,
    symbols: Vec<SymbolEntry>,
    last_root_child: Option<u32>,
    #[cfg(feature = "release")]
    queue_type_checks: Vec<u32>,
    #[cfg(feature = "release")]
    queue_ref_resolve: Vec<u32>,
}

impl SoaBuilder {
    fn with_capacity(n: usize) -> Self {
        let base = CookedAotBlob::empty();
        Self {
            string_blob: Vec::with_capacity(n.saturating_mul(32)),
            nodes: {
                let mut v = base.nodes;
                v.reserve(n);
                v
            },
            key_spans: {
                let mut v = base.key_spans;
                v.reserve(n);
                v
            },
            value_spans: {
                let mut v = base.value_spans;
                v.reserve(n);
                v
            },
            symbols: Vec::with_capacity(n),
            last_root_child: None,
            #[cfg(feature = "release")]
            queue_type_checks: Vec::with_capacity(n),
            #[cfg(feature = "release")]
            queue_ref_resolve: Vec::new(),
        }
    }

    fn push_string(&mut self, value: &str) -> Result<Span, AamlError> {
        let start = u32::try_from(self.string_blob.len()).map_err(|_| AamlError::InvalidValue {
            details: "string blob exceeds u32 address space".to_string(),
            expected: "input <= 4GB".to_string(),
            diagnostics: None,
        })?;
        let len = u32::try_from(value.len()).map_err(|_| AamlError::InvalidValue {
            details: "string length exceeds u32".to_string(),
            expected: "single token <= 4GB".to_string(),
            diagnostics: None,
        })?;
        self.string_blob.extend_from_slice(value.as_bytes());
        Ok(Span { start, len })
    }

    fn push_assignment(&mut self, key: &str, value: &str) -> Result<u32, AamlError> {
        let key_span = self.push_string(key)?;
        let value_span = self.push_string(value)?;
        let idx = u32::try_from(self.nodes.len()).map_err(|_| AamlError::InvalidValue {
            details: "node count exceeds u32".to_string(),
            expected: "<= 4,294,967,295 nodes".to_string(),
            diagnostics: None,
        })?;

        self.nodes.push(FlatNode {
            kind: NodeKind::Assignment as u32,
            parent: 0,
            first_child: INVALID_INDEX,
            next_sibling: INVALID_INDEX,
        });
        self.key_spans.push(key_span);
        self.value_spans.push(value_span);

        match self.last_root_child {
            Some(last) => {
                self.nodes[last as usize].next_sibling = idx;
            }
            None => {
                self.nodes[0].first_child = idx;
            }
        }
        self.last_root_child = Some(idx);

        #[cfg(feature = "release")]
        {
            self.queue_type_checks.push(idx);
            if value.starts_with('$') {
                self.queue_ref_resolve.push(idx);
            }
        }

        Ok(idx)
    }
    #[cfg(feature = "release")]
    fn validate_release_type_checks(&self) -> Result<(), AamlError> {
        for &node_index in &self.queue_type_checks {
            let i = node_index as usize;
            if self.key_spans[i].len == 0 || self.value_spans[i].len == 0 {
                return Err(AamlError::InvalidValue {
                    details: format!("empty key/value at node {node_index}"),
                    expected: "non-empty key and value".to_string(),
                    diagnostics: None,
                });
            }
        }

        Ok(())
    }

    #[cfg(feature = "release")]
    fn validate_release_references(&self) -> Result<(), AamlError> {
        for &node_index in &self.queue_ref_resolve {
            let value = self.span_bytes(self.value_spans[node_index as usize])?;
            if !value.starts_with(b"$") {
                continue;
            }

            let reference =
                std::str::from_utf8(&value[1..]).map_err(|_| AamlError::InvalidValue {
                    details: "reference value is not utf8".to_string(),
                    expected: "references like $some.key".to_string(),
                    diagnostics: None,
                })?;

            if self.find_symbol(reference).is_none() {
                return Err(AamlError::NotFound {
                    key: reference.to_string(),
                    context: "release reference validation".to_string(),
                    diagnostics: None,
                });
            }
        }

        Ok(())
    }

    #[cfg(feature = "release")]
    fn run_release_validation(&self) -> Result<(), AamlError> {
        self.validate_release_type_checks()?;
        self.validate_release_references()
    }

    #[cfg(feature = "release")]
    fn span_bytes(&self, span: Span) -> Result<&[u8], AamlError> {
        let start = span.start as usize;
        let end = start + span.len as usize;
        self.string_blob
            .get(start..end)
            .ok_or_else(|| AamlError::InvalidValue {
                details: "span points outside string blob".to_string(),
                expected: "valid span range".to_string(),
                diagnostics: None,
            })
    }
    #[cfg(feature = "release")]
    fn lower_bound_symbol_hash(&self, target_hash: u64) -> usize {
        let mut left = 0usize;
        let mut right = self.symbols.len();

        while left < right {
            let mid = left + ((right - left) / 2);
            let entry = self.symbols[mid];
            if entry.hash < target_hash {
                left = mid + 1;
            } else {
                right = mid;
            }
        }

        left
    }

    #[cfg(feature = "release")]
    fn find_symbol(&self, key: &str) -> Option<u32> {
        let target_hash = hash64(key.as_bytes());
        let mut i = self.lower_bound_symbol_hash(target_hash);
        while i < self.symbols.len() {
            let entry = self.symbols[i];
            if entry.hash != target_hash {
                break;
            }
            let span = self.key_spans[entry.node_index as usize];
            if self.span_bytes(span).ok()? == key.as_bytes() {
                return Some(entry.node_index);
            }
            i += 1;
        }

        None
    }

    fn finish(self) -> CookedAotBlob {
        let capacity = (self.symbols.len() * 2).next_power_of_two();
        let capacity = std::cmp::max(capacity, 16);

        let mut hash_table = vec![
            SymbolEntry {
                hash: 0,
                node_index: INVALID_INDEX
            };
            capacity
        ];

        let mask = (capacity - 1) as u64;

        for entry in self.symbols {
            let mut idx = (entry.hash & mask) as usize;
            loop {
                if hash_table[idx].node_index == INVALID_INDEX {
                    hash_table[idx] = entry;
                    break;
                }
                idx = (idx + 1) & (mask as usize);
            }
        }

        CookedAotBlob {
            version: CURRENT_AOT_VERSION,
            string_blob: self.string_blob,
            nodes: self.nodes,
            key_spans: self.key_spans,
            value_spans: self.value_spans,
            hash_table,
        }
    }
}

#[inline(always)]
fn hash64(bytes: &[u8]) -> u64 {
    let mut hasher = FxHasher::default();
    hasher.write(bytes);
    let h = hasher.finish();
    h ^ h.rotate_right(32)
}

fn split_assignment(line: &str) -> Option<(&str, &str)> {
    let pos = line.find('=')?;
    let key = line[..pos].trim();
    let raw_value = line[pos + 1..].trim();
    if key.is_empty() || raw_value.is_empty() {
        return None;
    }

    let value = if raw_value.starts_with('{') || raw_value.starts_with('[') {
        raw_value
    } else {
        unwrap_quotes(raw_value)
    };
    Some((key, value))
}

fn should_use_pipeline_compile(text: &str) -> bool {
    text.lines()
        .any(|line| strip_comment(line).trim_start().starts_with('@'))
}

fn parse_assignment_line(
    builder: &mut SoaBuilder,
    errors: &mut FrontendErrors,
    line_idx: usize,
    raw_line: &str,
    line: &str,
    #[cfg(feature = "dev")] dev_symbols: &mut rustc_hash::FxHashMap<u64, TinyVec<[u32; 4]>>,
    #[cfg(feature = "dev")] dev_root_children: &mut tinyvec::ArrayVec<[u32; 16]>,
) {
    let Some((key, value)) = split_assignment(line) else {
        push_frontend_error(
            errors,
            AamlError::ParseError {
                line: line_idx + 1,
                content: raw_line.to_string(),
                details: "expected 'key = value' assignment".to_string(),
                diagnostics: None,
            },
        );
        return;
    };

    match builder.push_assignment(key, value) {
        Ok(node_index) => {
            let hash = hash64(key.as_bytes());
            #[cfg(feature = "dev")]
            {
                dev_symbols.entry(hash).or_default().push(node_index);
                if dev_root_children.len() < dev_root_children.capacity() {
                    dev_root_children.push(node_index);
                }
            }
            #[cfg(not(feature = "dev"))]
            {
                builder.symbols.push(SymbolEntry { hash, node_index });
            }
        }
        Err(err) => push_frontend_error(errors, err),
    }
}

fn compile_text(text: &str) -> Result<CookedAotBlob, Vec<AamlError>> {
    if should_use_pipeline_compile(text) {
        return compile_via_pipeline(text);
    }

    let estimated = text.lines().count();
    let mut builder = SoaBuilder::with_capacity(estimated);
    let mut errors: FrontendErrors = TinyVec::new();

    #[cfg(feature = "dev")]
    let mut dev_symbols: rustc_hash::FxHashMap<u64, TinyVec<[u32; 4]>> =
        rustc_hash::FxHashMap::default();
    #[cfg(feature = "dev")]
    let mut dev_root_children: tinyvec::ArrayVec<[u32; 16]> = tinyvec::ArrayVec::new();

    for (line_idx, raw_line) in text.lines().enumerate() {
        let line = strip_comment(raw_line).trim();
        if line.is_empty() {
            continue;
        }

        if line.starts_with('@') {
            continue;
        }

        parse_assignment_line(
            &mut builder,
            &mut errors,
            line_idx,
            raw_line,
            line,
            #[cfg(feature = "dev")]
            &mut dev_symbols,
            #[cfg(feature = "dev")]
            &mut dev_root_children,
        );
    }

    if let Some(errors) = finalize_frontend_errors(errors) {
        return Err(errors);
    }

    #[cfg(feature = "dev")]
    {
        for (hash, indices) in dev_symbols {
            for node_index in indices {
                builder.symbols.push(SymbolEntry { hash, node_index });
            }
        }
        let _ = dev_root_children;
    }

    #[cfg(feature = "release")]
    if let Err(err) = builder.run_release_validation() {
        return Err(vec![err]);
    }

    Ok(builder.finish())
}

fn compile_via_pipeline(text: &str) -> Result<CookedAotBlob, Vec<AamlError>> {
    let pipeline = Pipeline::new();
    let output = pipeline.process(text)?;

    let estimated = output.map.len();
    let mut builder = SoaBuilder::with_capacity(estimated);
    let mut errors: FrontendErrors = TinyVec::new();

    for (key, value) in output.map {
        let hash = hash64(key.as_bytes());
        match builder.push_assignment(&key, &value) {
            Ok(node_index) => {
                builder.symbols.push(SymbolEntry { hash, node_index });
            }
            Err(err) => push_frontend_error(&mut errors, err),
        }
    }

    if let Some(errors) = finalize_frontend_errors(errors) {
        return Err(errors);
    }

    #[cfg(feature = "release")]
    if let Err(err) = builder.run_release_validation() {
        return Err(vec![err]);
    }

    Ok(builder.finish())
}

fn io_err(details: impl Into<String>) -> AamlError {
    AamlError::IoError {
        details: details.into(),
        diagnostics: None,
    }
}

fn cache_path(source: &Path) -> PathBuf {
    source.with_extension("aam.bin")
}

fn cache_needs_rebuild(source: &Path, cache: &Path) -> Result<bool, AamlError> {
    let cache_meta = match fs::metadata(cache) {
        Ok(meta) => meta,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(true),
        Err(e) => {
            return Err(io_err(format!(
                "failed to stat cache '{}': {e}",
                cache.display()
            )));
        }
    };

    let source_meta = fs::metadata(source)
        .map_err(|e| io_err(format!("failed to stat source '{}': {e}", source.display())))?;

    let cache_mtime = cache_meta
        .modified()
        .map_err(|e| io_err(format!("failed to read cache mtime: {e}")))?;
    let source_mtime = source_meta
        .modified()
        .map_err(|e| io_err(format!("failed to read source mtime: {e}")))?;

    Ok(cache_mtime < source_mtime)
}

pub struct AamCompiler;

impl AamCompiler {
    /// Compiles a text `.aam` asset into a cooked SoA `.aam.bin` blob.
    pub fn cook(source_path: impl AsRef<Path>) -> Result<PathBuf, Vec<AamlError>> {
        let source_path = source_path.as_ref();
        let cache = cache_path(source_path);

        let content = fs::read_to_string(source_path).map_err(|e| {
            vec![io_err(format!(
                "failed to read source asset '{}': {e}",
                source_path.display()
            ))]
        })?;

        let cooked = compile_text(&content)?;
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&cooked).map_err(|e| {
            vec![AamlError::DirectiveError {
                directive: "cook".to_string(),
                message: format!("failed to archive cooked output: {e}"),
                diagnostics: None,
            }]
        })?;

        let tmp_cache = cache.with_extension("aam.bin.tmp");

        let mut temp_file = File::create(&tmp_cache)
            .map_err(|e| vec![io_err(format!("failed to create temp file: {e}"))])?;

        std::io::Write::write_all(&mut temp_file, bytes.as_slice())
            .map_err(|e| vec![io_err(format!("failed to write cooked cache: {e}"))])?;

        temp_file
            .sync_all()
            .map_err(|e| vec![io_err(format!("failed to sync cooked cache: {e}"))])?;

        drop(temp_file);

        fs::rename(&tmp_cache, &cache)
            .map_err(|e| vec![io_err(format!("failed to rename cooked cache: {e}"))])?;

        Ok(cache)
    }
}

/// Zero-copy runtime view over a cooked `.aam.bin` cache.
#[derive(Debug)]
pub struct MappedAam {
    mmap: Mmap,
}

impl MappedAam {
    pub fn archived(&self) -> &rkyv::Archived<CookedAotBlob> {
        // SAFETY: `AamLoader::load_fast` validates archive format in checked modes.
        unsafe { rkyv::access_unchecked::<rkyv::Archived<CookedAotBlob>>(&self.mmap) }
    }

    fn span_bounds(span: &rkyv::Archived<Span>) -> (usize, usize) {
        let start = span.start.to_native() as usize;
        let end = start + span.len.to_native() as usize;
        (start, end)
    }

    #[cfg(feature = "unsafe_fast_path")]
    fn key_equals(&self, node_index: usize, key: &[u8]) -> bool {
        let archived = self.archived();
        let span = &archived.key_spans.as_slice()[node_index];
        let (start, end) = Self::span_bounds(span);
        archived
            .string_blob
            .as_slice()
            .get(start..end)
            .is_some_and(|bytes| bytes == key)
    }

    #[cfg(feature = "unsafe_fast_path")]
    fn find_symbol_index(&self, hash: u64, key: &[u8]) -> Option<usize> {
        let table = self.archived().hash_table.as_slice();
        if table.is_empty() {
            return None;
        }

        let mask = table.len() - 1;
        let mut idx = (hash as usize) & mask;

        loop {
            // SAFETY: idx always < than table.len in binary mask
            let entry = unsafe { table.get_unchecked(idx) };
            let node_idx = entry.node_index.to_native();

            if node_idx == INVALID_INDEX {
                return None; // Наткнулись на пустой слот - ключа нет
            }

            if entry.hash.to_native() == hash && self.key_equals(node_idx as usize, key) {
                return Some(node_idx as usize);
            }

            idx = (idx + 1) & mask;
        }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        let archived = self.archived();
        let table = archived.hash_table.as_slice();
        if table.is_empty() {
            return None;
        }

        let key_spans = archived.key_spans.as_slice();
        let value_spans = archived.value_spans.as_slice();
        let blob = archived.string_blob.as_slice();

        let key_bytes = key.as_bytes();
        let hash = hash64(key_bytes);

        let mask = table.len() - 1;
        let mut idx = (hash as usize) & mask;

        loop {
            let entry = unsafe { table.get_unchecked(idx) };
            let node_idx = entry.node_index.to_native() as usize;

            if node_idx == INVALID_INDEX as usize {
                return None;
            }

            if entry.hash.to_native() == hash {
                let span = unsafe { key_spans.get_unchecked(node_idx) };
                let start = span.start.to_native() as usize;
                let end = start + span.len.to_native() as usize;

                if blob.get(start..end) == Some(key_bytes) {
                    let val_span = unsafe { value_spans.get_unchecked(node_idx) };
                    let v_start = val_span.start.to_native() as usize;
                    let v_end = v_start + val_span.len.to_native() as usize;

                    return Some(unsafe {
                        std::str::from_utf8_unchecked(blob.get_unchecked(v_start..v_end))
                    });
                }
            }
            idx = (idx + 1) & mask;
        }
    }
    pub fn len(&self) -> usize {
        self.archived().hash_table.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter_pairs(&self) -> impl Iterator<Item = (&str, &str)> {
        let archived = self.archived();
        let blob = archived.string_blob.as_slice();
        let key_spans = archived.key_spans.as_slice();
        let value_spans = archived.value_spans.as_slice();

        (1..archived.nodes.len()).filter_map(move |idx| {
            let key = key_spans.get(idx)?;
            let value = value_spans.get(idx)?;
            let (k_start, k_end) = Self::span_bounds(key);
            let (v_start, v_end) = Self::span_bounds(value);
            let k = std::str::from_utf8(blob.get(k_start..k_end)?).ok()?;
            let v = std::str::from_utf8(blob.get(v_start..v_end)?).ok()?;
            Some((k, v))
        })
    }
}

pub struct AamLoader;

impl AamLoader {
    pub fn load_fast(source_path: impl AsRef<Path>) -> Result<MappedAam, Vec<AamlError>> {
        let source_path = source_path.as_ref();
        let cache = cache_path(source_path);

        let needs_rebuild = cache_needs_rebuild(source_path, &cache).map_err(|e| vec![e])?;
        if needs_rebuild {
            AamCompiler::cook(source_path)?;
        }

        let file = File::open(&cache).map_err(|e| {
            vec![io_err(format!(
                "failed to open cooked cache '{}': {e}",
                cache.display()
            ))]
        })?;

        let mmap = unsafe {
            MmapOptions::new().map(&file).map_err(|e| {
                vec![io_err(format!(
                    "failed to mmap cooked cache '{}': {e}",
                    cache.display()
                ))]
            })?
        };

        #[cfg(not(feature = "unsafe_fast_path"))]
        rkyv::access::<rkyv::Archived<CookedAotBlob>, rkyv::rancor::Error>(&mmap).map_err(|e| {
            vec![AamlError::DirectiveError {
                directive: "load_fast".to_string(),
                message: format!("invalid cooked archive '{}': {e}", cache.display()),
                diagnostics: None,
            }]
        })?;

        let archived_version = {
            // Safe minimal check: read only the archived version field.
            let blob = unsafe { rkyv::access_unchecked::<rkyv::Archived<CookedAotBlob>>(&mmap) };
            blob.version.to_native()
        };

        if archived_version != CURRENT_AOT_VERSION {
            let _ = fs::remove_file(&cache);
            AamCompiler::cook(source_path)?;
            return Self::load_fast(source_path);
        }

        Ok(MappedAam { mmap })
    }
}
