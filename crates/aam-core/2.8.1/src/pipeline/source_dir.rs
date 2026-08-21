use std::cell::RefCell;
use std::path::PathBuf;

thread_local! {
    static PIPELINE_SOURCE_DIR: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

/// RAII guard that sets a source directory for the current scope and restores
/// the previous value (or clears it) when dropped — even across panics.
pub struct SourceDirGuard {
    prev: Option<PathBuf>,
}

impl SourceDirGuard {
    /// Overwrites the current thread-local source dir, saving the previous value.
    /// Restored on `drop`.
    pub(crate) fn new(dir: Option<PathBuf>) -> Self {
        let prev = get();
        set(dir);
        Self { prev }
    }
}

impl Drop for SourceDirGuard {
    fn drop(&mut self) {
        if let Some(prev) = self.prev.take() {
            set(Some(prev));
        } else {
            set(None);
        }
    }
}

/// Sets the source directory for resolving relative paths in @import/@derive directives.
pub fn set(dir: Option<PathBuf>) {
    PIPELINE_SOURCE_DIR.with(|d| *d.borrow_mut() = dir);
}

/// Gets the current source directory.
pub fn get() -> Option<PathBuf> {
    PIPELINE_SOURCE_DIR.with(|d| d.borrow().clone())
}
