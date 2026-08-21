use super::super::DbXxxBase;
use super::{FileDbMapDbBytes, FileDbMapDbString, FileDbParams};
use super::{FileDbMapDbI64, FileDbMapDbU64, FileDbMapDbVu64};
use std::collections::BTreeMap;
use std::io::Result;
use std::path::{Path, PathBuf};

pub(crate) mod dbxxx;
pub(crate) mod semtype;

mod piece;
//mod tr;

//mod idx;
mod key;
mod val;
mod vfile;

//#[cfg(feature = "htx")]
mod htx;

//#[cfg(feature = "node_cache")]
//mod nc;

//#[cfg(feature = "node_cache")]
//mod offidx;

// _cold() is from hashbrown.
// On stable we can use #[cold] to get a equivalent effect: this attributes
// suggests that the function is unlikely to be called
#[inline]
#[cold]
fn _cold() {}

#[derive(Debug)]
pub struct FileDbInner {
    db_bytes_map: BTreeMap<String, FileDbMapDbBytes>,
    db_string_map: BTreeMap<String, FileDbMapDbString>,
    db_i64_map: BTreeMap<String, FileDbMapDbI64>,
    db_u64_map: BTreeMap<String, FileDbMapDbU64>,
    db_vu64_map: BTreeMap<String, FileDbMapDbVu64>,
    //
    path: PathBuf,
}

impl FileDbInner {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<FileDbInner> {
        let path = path.as_ref();
        if !path.is_dir() {
            std::fs::create_dir_all(path)?;
        }
        Ok(FileDbInner {
            db_bytes_map: BTreeMap::new(),
            db_string_map: BTreeMap::new(),
            db_i64_map: BTreeMap::new(),
            db_u64_map: BTreeMap::new(),
            db_vu64_map: BTreeMap::new(),
            path: path.to_path_buf(),
        })
    }
    #[inline]
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }
    pub fn sync_all(&self) -> Result<()> {
        self.applay_all(|o| o.sync_all())
    }
    pub fn sync_data(&self) -> Result<()> {
        self.applay_all(|o| o.sync_data())
    }
    fn applay_all<F>(&self, func: F) -> Result<()>
    where
        F: Fn(&mut dyn DbXxxBase) -> Result<()>,
    {
        {
            let keys: Vec<_> = self.db_bytes_map.keys().cloned().collect();
            for a in keys {
                let mut b = self.db_map_bytes(&a).unwrap();
                func(&mut b)?;
            }
        }
        {
            let keys: Vec<_> = self.db_string_map.keys().cloned().collect();
            for a in keys {
                let mut b = self.db_map_string(&a).unwrap();
                func(&mut b)?;
            }
        }
        {
            let keys: Vec<_> = self.db_i64_map.keys().cloned().collect();
            for a in keys {
                let mut b = self.db_map_i64(&a).unwrap();
                func(&mut b)?;
            }
        }
        {
            let keys: Vec<_> = self.db_u64_map.keys().cloned().collect();
            for a in keys {
                let mut b = self.db_map_u64(&a).unwrap();
                func(&mut b)?;
            }
        }
        {
            let keys: Vec<_> = self.db_vu64_map.keys().cloned().collect();
            for a in keys {
                let mut b = self.db_map_vu64(&a).unwrap();
                func(&mut b)?;
            }
        }
        Ok(())
    }
}

impl FileDbInner {
    #[inline]
    pub fn db_map_bytes(&self, name: &str) -> Option<FileDbMapDbBytes> {
        self.db_bytes_map.get(name).cloned()
    }
    #[inline]
    pub fn db_map_string(&self, name: &str) -> Option<FileDbMapDbString> {
        self.db_string_map.get(name).cloned()
    }
    #[inline]
    pub fn db_map_i64(&self, name: &str) -> Option<FileDbMapDbI64> {
        self.db_i64_map.get(name).cloned()
    }
    #[inline]
    pub fn db_map_u64(&self, name: &str) -> Option<FileDbMapDbU64> {
        self.db_u64_map.get(name).cloned()
    }
    #[inline]
    pub fn db_map_vu64(&self, name: &str) -> Option<FileDbMapDbVu64> {
        self.db_vu64_map.get(name).cloned()
    }
    #[inline]
    pub fn db_map_bytes_insert(
        &mut self,
        name: &str,
        child: FileDbMapDbBytes,
    ) -> Option<FileDbMapDbBytes> {
        self.db_bytes_map.insert(name.to_string(), child)
    }
    #[inline]
    pub fn db_map_insert(
        &mut self,
        name: &str,
        child: FileDbMapDbString,
    ) -> Option<FileDbMapDbString> {
        self.db_string_map.insert(name.to_string(), child)
    }
    #[inline]
    pub fn db_map_dbi64_insert(
        &mut self,
        name: &str,
        child: FileDbMapDbI64,
    ) -> Option<FileDbMapDbI64> {
        self.db_i64_map.insert(name.to_string(), child)
    }
    #[inline]
    pub fn db_map_dbu64_insert(
        &mut self,
        name: &str,
        child: FileDbMapDbU64,
    ) -> Option<FileDbMapDbU64> {
        self.db_u64_map.insert(name.to_string(), child)
    }
    #[inline]
    pub fn db_map_dbvu64_insert(
        &mut self,
        name: &str,
        child: FileDbMapDbVu64,
    ) -> Option<FileDbMapDbVu64> {
        self.db_vu64_map.insert(name.to_string(), child)
    }
}

impl FileDbInner {
    pub(super) fn create_db_map(&mut self, name: &str, params: FileDbParams) -> Result<()> {
        let child: FileDbMapDbString = FileDbMapDbString::open(self.path(), name, params)?;
        let _ = self.db_map_insert(name, child);
        Ok(())
    }
    pub(super) fn create_db_map_bytes(&mut self, name: &str, params: FileDbParams) -> Result<()> {
        let child: FileDbMapDbBytes = FileDbMapDbBytes::open(self.path(), name, params)?;
        let _ = self.db_map_bytes_insert(name, child);
        Ok(())
    }
    pub(super) fn create_db_map_dbi64(&mut self, name: &str, params: FileDbParams) -> Result<()> {
        let child: FileDbMapDbI64 = FileDbMapDbI64::open(self.path(), name, params)?;
        let _ = self.db_map_dbi64_insert(name, child);
        Ok(())
    }
    pub(super) fn create_db_map_dbu64(&mut self, name: &str, params: FileDbParams) -> Result<()> {
        let child: FileDbMapDbU64 = FileDbMapDbU64::open(self.path(), name, params)?;
        let _ = self.db_map_dbu64_insert(name, child);
        Ok(())
    }
    pub(super) fn create_db_map_dbvu64(&mut self, name: &str, params: FileDbParams) -> Result<()> {
        let child: FileDbMapDbVu64 = FileDbMapDbVu64::open(self.path(), name, params)?;
        let _ = self.db_map_dbvu64_insert(name, child);
        Ok(())
    }
}
