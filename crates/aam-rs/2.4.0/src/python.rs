//! PyO3 bindings — exposes `AAM` to Python as `aam_rs.AAM`.

use crate::aam::AAM;
use crate::builder::{AAMBuilder, SchemaField};
use crate::error::AamlError;
use crate::pipeline::formatter::{FormatRange, FormattingOptions as FormatterRules};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use std::collections::HashMap;

#[cfg(feature = "translator")]
use crate::translator::TOMLTranslator;

// ── Error conversion ─────────────────────────────────────────────────────────

fn to_py(err: AamlError) -> PyErr {
    PyRuntimeError::new_err(err.to_string())
}

fn first_error(errors: Vec<AamlError>) -> AamlError {
    errors.into_iter().next().unwrap_or(AamlError::ParseError {
        line: 1,
        content: String::new(),
        details: "unexpected empty parse error list".to_string(),
        diagnostics: None,
    })
}

// ── PySchemaField class ──────────────────────────────────────────────────────

#[pyclass(name = "SchemaField")]
#[derive(Clone)]
pub struct PySchemaField {
    inner: SchemaField,
}

#[pymethods]
impl PySchemaField {
    #[staticmethod]
    fn required(name: &str, type_name: &str) -> Self {
        Self {
            inner: SchemaField::required(name, type_name),
        }
    }

    #[staticmethod]
    fn optional(name: &str, type_name: &str) -> Self {
        Self {
            inner: SchemaField::optional(name, type_name),
        }
    }

    fn __repr__(&self) -> String {
        format!("SchemaField({})", self.inner.to_aaml())
    }
}

// ── PyAAMBuilder class ───────────────────────────────────────────────────────

#[pyclass(unsendable, name = "AAMBuilder")]
pub struct PyAamBuilder {
    inner: AAMBuilder,
}

#[pymethods]
impl PyAamBuilder {
    #[new]
    fn new() -> Self {
        Self {
            inner: AAMBuilder::new(),
        }
    }

    #[staticmethod]
    fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: AAMBuilder::with_capacity(capacity),
        }
    }

    fn add_line(&mut self, key: &str, value: &str) {
        self.inner.add_line(key, value);
    }

    fn comment(&mut self, text: &str) {
        self.inner.comment(text);
    }

    fn schema(&mut self, name: &str, fields: Vec<PySchemaField>) {
        self.inner
            .schema(name, fields.into_iter().map(|field| field.inner));
    }

    fn schema_multiline(&mut self, name: &str, fields: Vec<PySchemaField>) {
        self.inner
            .schema_multiline(name, fields.into_iter().map(|field| field.inner));
    }

    fn derive(&mut self, path: &str, schemas: Vec<String>) {
        self.inner.derive(path, schemas);
    }

    fn import(&mut self, path: &str) {
        self.inner.import(path);
    }

    fn type_alias(&mut self, alias: &str, type_name: &str) {
        self.inner.type_alias(alias, type_name);
    }

    fn as_string(&self) -> String {
        self.inner.as_string()
    }

    fn build(&self) -> String {
        self.inner.as_string()
    }

    fn __str__(&self) -> String {
        self.inner.as_string()
    }

    fn __repr__(&self) -> String {
        format!("AAMBuilder(len={})", self.inner.as_string().len())
    }
}

// ── PyAAM class ──────────────────────────────────────────────────────────────

#[pyclass(unsendable, name = "AAM")]
pub struct PyAam {
    inner: Option<AAM>,
}

impl PyAam {
    fn inner_ref(&self) -> PyResult<&AAM> {
        self.inner
            .as_ref()
            .ok_or_else(|| PyRuntimeError::new_err("AAM instance is closed"))
    }

    fn inner_mut(&mut self) -> PyResult<&mut AAM> {
        self.inner
            .as_mut()
            .ok_or_else(|| PyRuntimeError::new_err("AAM instance is closed"))
    }
}

#[pymethods]
impl PyAam {
    #[new]
    fn new() -> Self {
        Self {
            inner: Some(AAM::new()),
        }
    }

    #[staticmethod]
    fn parse(content: &str) -> PyResult<Self> {
        AAM::parse(content)
            .map_err(first_error)
            .map(|inner| PyAam { inner: Some(inner) })
            .map_err(to_py)
    }

    #[staticmethod]
    fn load(path: &str) -> PyResult<Self> {
        AAM::load(path)
            .map_err(first_error)
            .map(|inner| PyAam { inner: Some(inner) })
            .map_err(to_py)
    }

    #[staticmethod]
    fn lsp_assist(content: &str) -> (Vec<String>, Option<String>) {
        let rules = FormatterRules::default();
        let report = AAM::lsp_assist(content, &rules);
        (
            report
                .diagnostics
                .into_iter()
                .map(|err| err.to_string())
                .collect(),
            report.formatted,
        )
    }

    fn format(&self, content: &str) -> PyResult<String> {
        let rules = FormatterRules::default();
        self.inner_ref()?.format(content, &rules).map_err(to_py)
    }

    fn format_range(&self, content: &str, start_line: usize, end_line: usize) -> PyResult<String> {
        let rules = FormatterRules::default();
        let range = FormatRange {
            start_line,
            end_line,
        };
        self.inner_ref()?
            .format_range(content, range, &rules)
            .map_err(to_py)
    }

    fn get(&self, key: &str) -> Option<String> {
        self.inner_ref()
            .ok()
            .and_then(|i| i.get(key).map(ToString::to_string))
    }

    fn keys(&self) -> Vec<String> {
        self.inner_ref().map_or_else(
            |_| Vec::new(),
            |inner| inner.keys().iter().map(|s| s.to_string()).collect(),
        )
    }

    fn to_dict(&self) -> HashMap<String, String> {
        self.inner_ref()
            .map_or_else(|_| HashMap::new(), |i| i.to_map().into_iter().collect())
    }

    fn find(&self, query: &str) -> HashMap<String, String> {
        self.inner_ref().map_or_else(
            |_| HashMap::new(),
            |inner| {
                inner
                    .find(query)
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect()
            },
        )
    }

    fn deep_search(&self, pattern: &str) -> HashMap<String, String> {
        self.inner_ref().map_or_else(
            |_| HashMap::new(),
            |inner| {
                inner
                    .deep_search(pattern)
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect()
            },
        )
    }

    fn reverse_search(&self, target_value: &str) -> Vec<String> {
        self.inner_ref().map_or_else(
            |_| Vec::new(),
            |inner| {
                inner
                    .reverse_search(target_value)
                    .into_iter()
                    .map(ToString::to_string)
                    .collect()
            },
        )
    }

    fn schema_names(&self) -> Vec<String> {
        self.inner_ref()
            .ok()
            .and_then(|inner| inner.schemas())
            .map(|schemas| schemas.keys().map(|k| k.to_string()).collect())
            .unwrap_or_default()
    }

    fn type_names(&self) -> Vec<String> {
        self.inner_ref()
            .ok()
            .and_then(|inner| inner.types())
            .map(|types| types.keys().map(|k| k.to_string()).collect())
            .unwrap_or_default()
    }

    fn close(&mut self) {
        self.inner = None;
    }

    fn is_closed(&self) -> bool {
        self.inner.is_none()
    }

    // ── Python Dunder Methods ─────────────────────────────────────────────────

    fn __repr__(&self) -> String {
        match self.inner_ref() {
            Ok(inner) => format!("AAM({} keys)", inner.keys().len()),
            Err(_) => "AAM(closed)".to_string(),
        }
    }

    fn __len__(&self) -> usize {
        self.inner_ref().map_or(0, |i| i.keys().len())
    }

    fn __contains__(&self, key: &str) -> bool {
        self.inner_ref()
            .map(|i| i.get(key).is_some())
            .unwrap_or(false)
    }

    fn __getitem__(&self, key: &str) -> PyResult<String> {
        self.inner_ref()?
            .get(key)
            .map(ToString::to_string)
            .ok_or_else(|| PyRuntimeError::new_err(format!("Key not found: '{key}'")))
    }
}

// ── PyTOMLTranslator class ───────────────────────────────────────────────

#[cfg(feature = "translator")]
#[pyclass(name = "TOMLTranslator")]
pub struct PyTOMLTranslator;

#[cfg(feature = "translator")]
#[pymethods]
impl PyTOMLTranslator {
    #[staticmethod]
    fn toml_to_aam(toml_source: &str) -> PyResult<Vec<String>> {
        TOMLTranslator::toml_to_aam(toml_source)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
            .map(|builders| builders.into_iter().map(|b| b.build()).collect())
    }
}

// ── Module registration ──────────────────────────────────────────────────────

pub fn register(m: &pyo3::Bound<'_, pyo3::types::PyModule>) -> pyo3::PyResult<()> {
    m.add_class::<PyAam>()?;
    m.add_class::<PyAamBuilder>()?;
    m.add_class::<PySchemaField>()?;
    #[cfg(feature = "translator")]
    {
        m.add_class::<PyTOMLTranslator>()?;
    }
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
