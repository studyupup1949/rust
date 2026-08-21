use serde::{de::DeserializeOwned, Serialize};
use std::{
    io::Write,
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};
use std::{
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    sync::Arc,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AcidJsonError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// A "smart pointer" to a JSON file on disk. Can be used in a RwLock-like fashion for thread-safe, ACID-guaranteed updates to the underlying file. Is "Arc-like" can can be cheaply cloned to create more references to the same file.
#[derive(Clone, Debug)]
pub struct AcidJson<T: Serialize + DeserializeOwned + Sync> {
    cached: Arc<RwLock<T>>,
    fname: PathBuf,
}

impl<T: Serialize + DeserializeOwned + Sync> AcidJson<T> {
    /// Opens an AcidJson.
    pub fn open(fname: &Path) -> Result<Self, AcidJsonError> {
        let file_contents = std::fs::read(fname)?;
        let parsed: T = serde_json::from_slice(&file_contents)?;
        Ok(Self {
            cached: RwLock::new(parsed).into(),
            fname: fname.to_owned(),
        })
    }

    /// Read-locks the AcidJson.
    pub fn read(&self) -> AcidJsonReadGuard<T> {
        let inner = self.cached.read().unwrap();
        AcidJsonReadGuard { inner }
    }

    /// Write-locks the AcidJson.
    pub fn write(&self) -> AcidJsonWriteGuard<T> {
        let inner = self.cached.write().unwrap();
        let init_serialized = serde_json::to_vec(inner.deref()).expect("cannot serialize");
        AcidJsonWriteGuard {
            inner,
            fname: self.fname.clone(),
            init_serialized,
        }
    }
}

/// A read guard for an acidjson.
pub struct AcidJsonReadGuard<'a, T: Serialize + DeserializeOwned + Sync> {
    inner: RwLockReadGuard<'a, T>,
}

impl<'a, T: Serialize + DeserializeOwned + Sync> Deref for AcidJsonReadGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// A write guard for an acidjson.
pub struct AcidJsonWriteGuard<'a, T: Serialize + DeserializeOwned + Sync> {
    inner: RwLockWriteGuard<'a, T>,
    fname: PathBuf,
    init_serialized: Vec<u8>,
}

impl<'a, T: Serialize + DeserializeOwned + Sync> Deref for AcidJsonWriteGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

impl<'a, T: Serialize + DeserializeOwned + Sync> DerefMut for AcidJsonWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.deref_mut()
    }
}

impl<'a, T: Serialize + DeserializeOwned + Sync> Drop for AcidJsonWriteGuard<'a, T> {
    fn drop(&mut self) {
        let serialized = serde_json::to_vec(self.inner.deref()).expect("cannot serialize");
        if serialized != self.init_serialized {
            atomicwrites::AtomicFile::new(
                &self.fname,
                atomicwrites::OverwriteBehavior::AllowOverwrite,
            )
            .write(|f| f.write_all(&serialized))
            .expect("could not write acidjson");
            log::debug!(
                "wrote {} bytes to {}",
                serialized.len(),
                self.fname.as_os_str().to_string_lossy()
            );
        }
    }
}
