use std::cell::RefCell;
use std::io::Result;
use std::path::{Path, PathBuf};
use std::rc::Rc;

mod dbmap;
mod inner;

pub use dbmap::{DbBytes, DbI64, DbString, DbU64, DbVu64};
pub use dbmap::{FileDbMap, FileDbMapDbBytes, FileDbMapDbString};
pub use dbmap::{FileDbMapDbI64, FileDbMapDbU64, FileDbMapDbVu64};
pub use inner::dbxxx::FileDbXxxInner;
pub use inner::dbxxx::{DbXxxIntoIter, DbXxxIter, DbXxxIterMut, DbXxxKeys, DbXxxValues};
use inner::semtype::*;
use inner::FileDbInner;

/// File Database.
#[derive(Debug, Clone)]
pub struct FileDb(Rc<RefCell<FileDbInner>>);

impl FileDb {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self(Rc::new(RefCell::new(FileDbInner::open(path)?))))
    }
    pub fn path(&self) -> PathBuf {
        RefCell::borrow(&self.0).path().to_path_buf()
    }
    pub fn sync_all(&self) -> Result<()> {
        RefCell::borrow_mut(&self.0).sync_all()
    }
    pub fn sync_data(&self) -> Result<()> {
        RefCell::borrow_mut(&self.0).sync_data()
    }
}

impl FileDb {
    pub fn db_map_string(&self, name: &str) -> Result<FileDbMapDbString> {
        self.db_map_string_with_params(name, FileDbParams::default())
    }
    pub fn db_map_string_with_params(
        &self,
        name: &str,
        params: FileDbParams,
    ) -> Result<FileDbMapDbString> {
        if let Some(m) = RefCell::borrow(&self.0).db_map_string(name) {
            return Ok(m);
        }
        RefCell::borrow_mut(&self.0).create_db_map(name, params)?;
        match RefCell::borrow(&self.0).db_map_string(name) {
            Some(m) => Ok(m),
            None => panic!("Cannot create db_maps: {name}"),
        }
    }
    pub fn db_map_bytes(&self, name: &str) -> Result<FileDbMapDbBytes> {
        self.db_map_bytes_with_params(name, FileDbParams::default())
    }
    pub fn db_map_bytes_with_params(
        &self,
        name: &str,
        params: FileDbParams,
    ) -> Result<FileDbMapDbBytes> {
        if let Some(m) = RefCell::borrow(&self.0).db_map_bytes(name) {
            return Ok(m);
        }
        RefCell::borrow_mut(&self.0).create_db_map_bytes(name, params)?;
        match RefCell::borrow(&self.0).db_map_bytes(name) {
            Some(m) => Ok(m),
            None => panic!("Cannot create db_maps: {name}"),
        }
    }
    pub fn db_map_i64(&self, name: &str) -> Result<FileDbMapDbI64> {
        self.db_map_i64_with_params(name, FileDbParams::default())
    }
    pub fn db_map_i64_with_params(
        &self,
        name: &str,
        params: FileDbParams,
    ) -> Result<FileDbMapDbI64> {
        if let Some(m) = RefCell::borrow(&self.0).db_map_i64(name) {
            return Ok(m);
        }
        RefCell::borrow_mut(&self.0).create_db_map_dbi64(name, params)?;
        match RefCell::borrow(&self.0).db_map_i64(name) {
            Some(m) => Ok(m),
            None => panic!("Cannot create db_maps: {name}"),
        }
    }
    pub fn db_map_u64(&self, name: &str) -> Result<FileDbMapDbU64> {
        self.db_map_u64_with_params(name, FileDbParams::default())
    }
    pub fn db_map_u64_with_params(
        &self,
        name: &str,
        params: FileDbParams,
    ) -> Result<FileDbMapDbU64> {
        if let Some(m) = RefCell::borrow(&self.0).db_map_u64(name) {
            return Ok(m);
        }
        RefCell::borrow_mut(&self.0).create_db_map_dbu64(name, params)?;
        match RefCell::borrow(&self.0).db_map_u64(name) {
            Some(m) => Ok(m),
            None => panic!("Cannot create db_maps: {name}"),
        }
    }
    pub fn db_map_vu64(&self, name: &str) -> Result<FileDbMapDbVu64> {
        self.db_map_vu64_with_params(name, FileDbParams::default())
    }
    pub fn db_map_vu64_with_params(
        &self,
        name: &str,
        params: FileDbParams,
    ) -> Result<FileDbMapDbVu64> {
        if let Some(m) = RefCell::borrow(&self.0).db_map_vu64(name) {
            return Ok(m);
        }
        RefCell::borrow_mut(&self.0).create_db_map_dbvu64(name, params)?;
        match RefCell::borrow(&self.0).db_map_vu64(name) {
            Some(m) => Ok(m),
            None => panic!("Cannot create db_maps: {name}"),
        }
    }
}

/// Parameters of buffer.
#[derive(Debug, Clone)]
pub enum FileBufSizeParam {
    /// Fixed buffer size
    Size(u32),
    /// Auto buffer size by file size.
    PerMille(u16),
    /// Default auto buffer size by file size.
    Auto,
}

/// Parameters of hash buckets (hash bucket table)
#[derive(Debug, Clone)]
pub enum HashBucketsParam {
    /// Buckets size at creation time.
    BucketsSize(u64),
    /// Capacity is calcurate to buckets size at creation time.
    Capacity(u64),
    /// Default buckets size.
    Default,
}

/// Parameters of filedb.
///
/// chunk_size is MUST power of 2.
#[derive(Debug, Clone)]
pub struct FileDbParams {
    /// buffer size of val file buffer. Default is auto buffer size.
    pub val_buf_size: FileBufSizeParam,
    /// buffer size of key file buffer. Default is full buffer size.
    pub key_buf_size: FileBufSizeParam,
    /// buffer size of idx file buffer. Default is full buffer size.
    pub idx_buf_size: FileBufSizeParam,
    /// buffer size of htx file buffer. Default is full buffer size.
    pub htx_buf_size: FileBufSizeParam,
    /// hash buckets size at cretation time.
    pub buckets_size: HashBucketsParam,
}

impl std::default::Default for FileDbParams {
    fn default() -> Self {
        Self {
            val_buf_size: FileBufSizeParam::Auto,
            key_buf_size: FileBufSizeParam::PerMille(1000),
            idx_buf_size: FileBufSizeParam::PerMille(1000),
            htx_buf_size: FileBufSizeParam::PerMille(1000),
            buckets_size: HashBucketsParam::Default,
        }
    }
}

/// Checks the file db map for debug.
pub trait CheckFileDbMap {
    /// hash table size and item counts in htx file.
    #[cfg(feature = "htx")]
    fn ht_size_and_count(&self) -> Result<(u64, u64)>;
    /// count of the free key piece
    fn count_of_free_key_piece(&self) -> Result<CountOfPerSize>;
    /// count of the free key piece
    fn count_of_free_value_piece(&self) -> Result<CountOfPerSize>;
    /// buffer statistics
    #[cfg(feature = "buf_stats")]
    fn buf_stats(&self) -> Vec<(String, i64)>;
    /// key piece size statistics
    fn key_piece_size_stats(&self) -> Result<RecordSizeStats<Key>>;
    /// value piece size statistics
    fn value_piece_size_stats(&self) -> Result<RecordSizeStats<Value>>;
    /// keys count statistics
    fn keys_count_stats(&self) -> Result<KeysCountStats>;
    /// key length statistics
    fn key_length_stats(&self) -> Result<LengthStats<Key>>;
    /// value length statistics
    fn value_length_stats(&self) -> Result<LengthStats<Value>>;
    /// htx filling rate per mill
    //#[cfg(feature = "htx")]
    fn htx_filling_rate_per_mill(&self) -> Result<(u64, u32)>;
    /*
    /// convert the index node tree to graph string for debug.
    fn graph_string(&self) -> Result<String>;
    /// convert the index node tree to graph string for debug.
    fn graph_string_with_key_string(&self) -> Result<String>;
    /// check the index node tree is balanced
    fn is_balanced(&self) -> Result<bool>;
    /// check the index node tree is multi search tree
    fn is_mst_valid(&self) -> Result<bool>;
    /// check the index node except the root and leaves of the tree has branches of hm or more.
    fn is_dense(&self) -> Result<bool>;
    /// get the depth of the index node.
    fn depth_of_node_tree(&self) -> Result<u64>;
    /// count of the free node
    fn count_of_free_node(&self) -> Result<CountOfPerSize>;
    /// count of the used piece and the used node
    fn count_of_used_node(&self) -> Result<(CountOfPerSize, CountOfPerSize, CountOfPerSize)>;
    */
}

pub type CountOfPerSize = Vec<(u32, u64)>;

/// piece size statistics.
#[derive(Debug, Default)]
pub struct RecordSizeStats<T>(Vec<(PieceSize<T>, u64)>);

impl<T: Copy + Ord> RecordSizeStats<T> {
    pub fn new(vec: Vec<(PieceSize<T>, u64)>) -> Self {
        Self(vec)
    }
    pub fn touch_size(&mut self, piece_size: PieceSize<T>) {
        match self.0.binary_search_by_key(&piece_size, |&(a, _b)| a) {
            Ok(sz_idx) => {
                self.0[sz_idx].1 += 1;
            }
            Err(sz_idx) => {
                self.0.insert(sz_idx, (piece_size, 1));
            }
        }
    }
}

impl<T: Copy> std::fmt::Display for RecordSizeStats<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("[")?;
        if self.0.len() > 1 {
            for (a, b) in self.0.iter().take(self.0.len() - 1) {
                formatter.write_fmt(format_args!("({a}, {b})"))?;
                formatter.write_str(", ")?;
            }
        }
        if !self.0.is_empty() {
            let (a, b) = self.0[self.0.len() - 1];
            formatter.write_fmt(format_args!("({a}, {b})"))?;
        }
        formatter.write_str("]")?;
        Ok(())
    }
}

pub type KeyPieceSizeStats = RecordSizeStats<Key>;
pub type ValueRecordSizeStats = RecordSizeStats<Value>;

/// piece size statistics.
#[derive(Debug, Default)]
pub struct KeysCountStats(Vec<(KeysCount, u64)>);

impl KeysCountStats {
    pub fn new(vec: Vec<(KeysCount, u64)>) -> Self {
        Self(vec)
    }
    pub fn touch_size(&mut self, keys_count: KeysCount) {
        match self.0.binary_search_by_key(&keys_count, |&(a, _b)| a) {
            Ok(sz_idx) => {
                self.0[sz_idx].1 += 1;
            }
            Err(sz_idx) => {
                self.0.insert(sz_idx, (keys_count, 1));
            }
        }
    }
}

impl std::fmt::Display for KeysCountStats {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("[")?;
        if self.0.len() > 1 {
            for (a, b) in self.0.iter().take(self.0.len() - 1) {
                formatter.write_fmt(format_args!("({a}, {b})"))?;
                formatter.write_str(", ")?;
            }
        }
        if !self.0.is_empty() {
            let (a, b) = self.0[self.0.len() - 1];
            formatter.write_fmt(format_args!("({a}, {b})"))?;
        }
        formatter.write_str("]")?;
        Ok(())
    }
}

/// key or value length statistics.
#[derive(Debug, Default)]
pub struct LengthStats<T: Default>(Vec<(Length<T>, u64)>);

impl<T: Ord + Default + Copy> LengthStats<T> {
    pub fn new(vec: Vec<(Length<T>, u64)>) -> Self {
        Self(vec)
    }
    pub fn touch_length(&mut self, key_length: Length<T>) {
        match self.0.binary_search_by_key(&key_length, |&(a, _b)| a) {
            Ok(sz_idx) => {
                self.0[sz_idx].1 += 1;
            }
            Err(sz_idx) => {
                self.0.insert(sz_idx, (key_length, 1));
            }
        }
    }
}

impl<T: Default + Copy> std::fmt::Display for LengthStats<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("[")?;
        if self.0.len() > 1 {
            for (a, b) in self.0.iter().take(self.0.len() - 1) {
                formatter.write_fmt(format_args!("({a}, {b})"))?;
                formatter.write_str(", ")?;
            }
        }
        if !self.0.is_empty() {
            let (a, b) = self.0[self.0.len() - 1];
            formatter.write_fmt(format_args!("({a}, {b})"))?;
        }
        formatter.write_str("]")?;
        Ok(())
    }
}
