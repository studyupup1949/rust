//! PyO3 bindings — exposes `AAML` to Python as `aam_rs.AAML`.
//!
//! Build with:
//! ```sh
//! pip install maturin
//! maturin develop --features python
//! ```
//! Then in Python:
//! ```python
//! from aam_rs import AAML
//!
//! cfg = AAML.parse("host = localhost\nport = 8080")
//! print(cfg.find_obj("host"))   # "localhost"
//! ```

use crate::aaml::AAML;
use crate::error::AamlError;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use std::collections::HashMap;

// ── Error conversion ─────────────────────────────────────────────────────────

fn to_py(err: AamlError) -> PyErr {
    PyRuntimeError::new_err(err.to_string())
}

// ── PyAAML class ─────────────────────────────────────────────────────────────

/// Python-facing wrapper around the Rust `AAML` parser.
///
/// All methods that can fail raise `RuntimeError` with a descriptive message.
#[pyclass(unsendable, name = "AAML")]
pub struct PyAAML {
    inner: AAML,
}

#[pymethods]
impl PyAAML {
    // ── Constructors ─────────────────────────────────────────────────────────

    /// Create an empty AAML instance.
    #[new]
    fn new() -> Self {
        PyAAML { inner: AAML::new() }
    }

    /// Parse an AAML string and return a new instance.
    ///
    /// ```python
    /// cfg = AAML.parse("host = localhost\nport = 8080")
    /// ```
    #[staticmethod]
    fn parse(content: &str) -> PyResult<Self> {
        AAML::parse(content)
            .map(|inner| PyAAML { inner })
            .map_err(to_py)
    }

    /// Load an AAML file from disk and return a new instance.
    ///
    /// ```python
    /// cfg = AAML.load("config.aam")
    /// ```
    #[staticmethod]
    fn load(path: &str) -> PyResult<Self> {
        AAML::load(path)
            .map(|inner| PyAAML { inner })
            .map_err(to_py)
    }

    // ── Mutation ─────────────────────────────────────────────────────────────

    /// Merge AAML text into this instance (modifies in-place).
    fn merge_content(&mut self, content: &str) -> PyResult<()> {
        self.inner.merge_content(content).map_err(to_py)
    }

    /// Merge an AAML file into this instance (modifies in-place).
    fn merge_file(&mut self, path: &str) -> PyResult<()> {
        self.inner.merge_file(path).map_err(to_py)
    }

    // ── Lookups ──────────────────────────────────────────────────────────────

    /// Forward lookup: returns the value for `key`, or `None`.
    ///
    /// Also performs a reverse lookup (find key whose value equals `key`)
    /// when no direct key is found.
    fn find_obj(&self, key: &str) -> Option<String> {
        self.inner.find_obj(key).map(|v| v.as_str().to_string())
    }

    /// Reverse lookup: find the key whose value equals `value`, or `None`.
    fn find_key(&self, value: &str) -> Option<String> {
        self.inner.find_key(value).map(|v| v.as_str().to_string())
    }

    /// Deep lookup: follow key→value→key chains until a terminal or cycle.
    fn find_deep(&self, key: &str) -> Option<String> {
        self.inner.find_deep(key).map(|v| v.as_str().to_string())
    }

    /// Look up `key` and parse its value as a list `[a, b, c]`.
    ///
    /// Returns `None` if the key is absent or the value is not a list literal.
    fn find_list(&self, key: &str) -> Option<Vec<String>> {
        self.inner.find_obj(key).and_then(|v| v.as_list())
    }

    /// Look up `key` and parse its value as an inline object `{ k = v, ... }`.
    ///
    /// Returns `None` if the key is absent or the value is not an object literal.
    fn find_object(&self, key: &str) -> Option<HashMap<String, String>> {
        self.inner.find_obj(key).and_then(|v| v.as_object())
    }

    // ── Map introspection ────────────────────────────────────────────────────

    /// Returns all keys stored in this instance.
    fn keys(&self) -> Vec<String> {
        self.inner.keys().iter().map(|s| s.to_string()).collect()
    }

    /// Returns a Python `dict` of all key-value pairs.
    fn to_dict(&self) -> HashMap<String, String> {
        self.inner.to_map()
    }

    // ── Type validation ──────────────────────────────────────────────────────

    /// Validate `value` against a built-in or registered type name.
    ///
    /// Raises `RuntimeError` on validation failure.
    fn validate_value(&self, type_name: &str, value: &str) -> PyResult<()> {
        self.inner.validate_value(type_name, value).map_err(to_py)
    }

    // ── Dunder methods ───────────────────────────────────────────────────────

    fn __repr__(&self) -> String {
        format!("AAML({} keys)", self.inner.keys().len())
    }

    fn __len__(&self) -> usize {
        self.inner.keys().len()
    }

    fn __contains__(&self, key: &str) -> bool {
        self.inner.find_obj(key).is_some()
    }

    fn __getitem__(&self, key: &str) -> PyResult<String> {
        self.inner
            .find_obj(key)
            .map(|v| v.as_str().to_string())
            .ok_or_else(|| PyRuntimeError::new_err(format!("Key not found: '{key}'")))
    }
}

/// Register all Python-facing items into `m`.
///
/// Called from the crate-root `#[pymodule]` in `lib.rs`.
pub fn register(m: &pyo3::Bound<'_, pyo3::types::PyModule>) -> pyo3::PyResult<()> {
    m.add_class::<PyAAML>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
