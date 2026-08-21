use crate::error::{AamlError, set_error_render_context};
use crate::pipeline::{
    DefaultLexer, DefaultParser, FormatRange, FormattingOptions, Lexer, Parser, Pipeline,
    PipelineHashMap, PipelineOutput, SchemaInfo, TypeInfo,
};
use smol_str::SmolStr;
use std::path::Path;

#[cfg(feature = "aot")]
use crate::aot::{AamCompiler, AamLoader, MappedAam};

/// Represents the underlying storage backend for AAM.
/// Can be either a dynamic in-memory structure or a zero-copy memory-mapped file.
#[derive(Debug)]
pub enum AamBackend {
    /// Fully parsed dynamic in-memory output.
    Dynamic(PipelineOutput),
    /// Zero-copy memory-mapped AOT (Ahead-of-Time) binary cache.
    #[cfg(feature = "aot")]
    Mapped(MappedAam),
}

/// The main AAM configuration store.
///
/// Holds the final, validated output of the AAM pipeline, including
/// the key-value map, schemas, and registered types.
#[derive(Debug)]
pub struct AAM {
    backend: AamBackend,
}

/// LSP-oriented helper payload that returns diagnostics plus optional formatting output.
#[derive(Debug)]
pub struct AamLspAssist {
    pub diagnostics: Vec<AamlError>,
    pub formatted: Option<String>,
}

impl AAM {
    fn parse_with_source_name(source_name: &str, text: &str) -> Result<Self, Vec<AamlError>> {
        set_error_render_context(source_name.to_string(), text);
        let pipeline = Pipeline::new();
        let output = pipeline.process(text)?;

        Ok(Self {
            backend: AamBackend::Dynamic(output),
        })
    }

    /// Creates an empty dynamic AAM document.
    #[must_use]
    pub fn new() -> Self {
        Self {
            backend: AamBackend::Dynamic(PipelineOutput::new()),
        }
    }

    /// Parses an AAM string using the default Pipeline and returns a new [`AAM`] instance.
    ///
    /// If `text` is a path to an existing file on disk, it is loaded and parsed dynamically
    /// (with full schema support), bypassing the AOT cache. Otherwise, `text` is treated as
    /// raw AAM content.
    ///
    /// # Errors
    ///
    /// Returns errors if the file cannot be read, the content fails to parse, or validation fails.
    pub fn parse(text: &str) -> Result<Self, Vec<AamlError>> {
        let path = Path::new(text);
        if path.is_file() {
            let content = std::fs::read_to_string(path).map_err(|e| {
                vec![AamlError::IoError {
                    details: format!("failed to read '{}': {e}", path.display()),
                    diagnostics: None,
                }]
            })?;
            set_error_render_context(text.to_string(), &content);
            let pipeline = Pipeline::new();
            let output = pipeline.process_with_source_dir(&content, path.parent())?;
            return Ok(Self {
                backend: AamBackend::Dynamic(output),
            });
        }
        Self::parse_with_source_name("raw_string", text)
    }

    /// Creates an [`AAM`] instance from a custom configured Pipeline.
    /// Use this if you need to register custom commands, parsers, or validators.
    ///
    /// If `text` is a path to an existing file on disk, it is loaded and parsed as a file.
    /// Otherwise, `text` is treated as raw AAM content.
    ///
    /// # Errors
    ///
    /// Returns errors if the file cannot be read, the content fails to parse, or validation fails.
    pub fn from_pipeline(pipeline: &Pipeline, text: &str) -> Result<Self, Vec<AamlError>> {
        let path = Path::new(text);
        if path.is_file() {
            let content = std::fs::read_to_string(path).map_err(|e| {
                vec![AamlError::IoError {
                    details: format!("failed to read '{}': {e}", path.display()),
                    diagnostics: None,
                }]
            })?;
            set_error_render_context(text.to_string(), &content);
            let output = pipeline.process_with_source_dir(&content, path.parent())?;
            return Ok(Self {
                backend: AamBackend::Dynamic(output),
            });
        }
        set_error_render_context("raw_string", text);
        let output = pipeline.process(text)?;
        Ok(Self {
            backend: AamBackend::Dynamic(output),
        })
    }

    /// Loads an `.aam` file from disk.
    ///
    /// With `aot` enabled (default), this uses cooked `.aam.bin` cache as the
    /// primary path and only invokes parsing/cooking when cache is missing/stale.
    /// This provides zero-copy memory mapping without any allocations.
    ///
    /// # Errors
    ///
    /// Returns errors if the file cannot be read, the cache cannot be memory-mapped,
    /// or the content fails to parse/validate.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Vec<AamlError>> {
        #[cfg(feature = "aot")]
        {
            let mapped = AamLoader::load_fast(path)?;
            Ok(Self {
                backend: AamBackend::Mapped(mapped),
            })
        }

        #[cfg(not(feature = "aot"))]
        {
            let path = path.as_ref();
            let content = std::fs::read_to_string(path).map_err(|e| {
                vec![AamlError::IoError {
                    details: format!("failed to read '{}': {e}", path.display()),
                    diagnostics: None,
                }]
            })?;

            let source_name = path.display().to_string();
            set_error_render_context(source_name.clone(), &content);
            let pipeline = Pipeline::new();
            let source_dir = path.parent();
            let output = pipeline.process_with_source_dir(&content, source_dir)?;

            Ok(Self {
                backend: AamBackend::Dynamic(output),
            })
        }
    }

    /// Formats arbitrary AAM content using parser + pipeline formatter.
    ///
    /// # Errors
    ///
    /// Returns errors if parsing or formatting fails.
    pub fn format(&self, content: &str, options: &FormattingOptions) -> Result<String, AamlError> {
        set_error_render_context("<format>", content);
        let lexer = DefaultLexer::new();
        let parser = DefaultParser::new();
        let tokens = lexer.tokenize(content)?;
        let crate::pipeline::parser::ParseOutput { ast, errors } =
            parser.parse_with_recovery(&tokens);

        if let Some(first_error) = errors.into_iter().next() {
            return Err(first_error);
        }

        Pipeline::new().format(&ast, options)
    }

    /// Formats only a selected line range of arbitrary AAM content.
    ///
    /// # Errors
    ///
    /// Returns errors if parsing or formatting fails.
    pub fn format_range(
        &self,
        content: &str,
        range: FormatRange,
        options: &FormattingOptions,
    ) -> Result<String, AamlError> {
        set_error_render_context("<format>", content);
        let lexer = DefaultLexer::new();
        let parser = DefaultParser::new();
        let tokens = lexer.tokenize(content)?;
        let crate::pipeline::parser::ParseOutput { ast, errors } =
            parser.parse_with_recovery(&tokens);

        if let Some(first_error) = errors.into_iter().next() {
            return Err(first_error);
        }

        Pipeline::new().format_range(&ast, range, options)
    }

    /// Convenience method for LSP servers: parse with recovery and optional formatting result.
    #[must_use]
    pub fn lsp_assist(content: &str, options: &FormattingOptions) -> AamLspAssist {
        set_error_render_context("<lsp>", content);
        let lexer = DefaultLexer::new();
        let parser = DefaultParser::new();

        let tokens = match lexer.tokenize(content) {
            Ok(tokens) => tokens,
            Err(err) => {
                return AamLspAssist {
                    diagnostics: vec![err],
                    formatted: None,
                };
            }
        };

        let parse_output = parser.parse_with_recovery(&tokens);
        let formatted = if parse_output.errors.is_empty() {
            Pipeline::new().format(&parse_output.ast, options).ok()
        } else {
            None
        };

        AamLspAssist {
            diagnostics: parse_output.errors,
            formatted,
        }
    }

    /// Explicitly cooks an `.aam` file into `.aam.bin` cache.
    ///
    /// # Errors
    ///
    /// Returns errors if the file cannot be read, the cache cannot be created,
    /// or the content fails to parse/validate.
    #[cfg(feature = "aot")]
    pub fn cook(path: impl AsRef<Path>) -> Result<std::path::PathBuf, Vec<AamlError>> {
        AamCompiler::cook(path)
    }

    /// Exposes zero-copy fast loading for advanced runtime integrations.
    ///
    /// # Errors
    ///
    /// Returns errors if the file cannot be read, the cache cannot be memory-mapped,
    /// or the source file is invalid and rebuilding fails.
    #[cfg(feature = "aot")]
    pub fn load_fast(path: impl AsRef<Path>) -> Result<MappedAam, Vec<AamlError>> {
        AamLoader::load_fast(path)
    }

    // ── Search & Filtering ───────────────────────────────────────────

    /// Deep Search: Finds all key-value pairs where the key contains the specified pattern.
    #[must_use]
    pub fn deep_search(&self, pattern: &str) -> Vec<(&str, &str)> {
        self.iter().filter(|(k, _)| k.contains(pattern)).collect()
    }

    /// Reverse Search: Finds all keys that match the specified target value.
    #[must_use]
    pub fn reverse_search(&self, target_value: &str) -> Vec<&str> {
        self.iter()
            .filter(|(_, v)| *v == target_value)
            .map(|(k, _)| k)
            .collect()
    }

    /// Advanced search using a custom predicate function.
    pub fn find_by<F>(&self, predicate: F) -> Vec<(&str, &str)>
    where
        F: Fn(&str, &str) -> bool,
    {
        self.iter().filter(|(k, v)| predicate(k, v)).collect()
    }

    /// Find by key or value with fallback.
    /// First tries to find exactly by key (O(1) lookup), if not found,
    /// searches for matching values (O(N) iteration).
    #[must_use]
    pub fn find<'a>(&'a self, query: &'a str) -> Vec<(&'a str, &'a str)> {
        if let Some(v) = self.get(query) {
            return vec![(query, v)];
        }

        self.iter().filter(|(_, v)| *v == query).collect()
    }

    // ── Key-Value Data Accessors ─────────────────────────────────────────────

    /// Retrieves a string value by its key. Performs an O(1) lookup.
    /// When AOT is enabled, this is a zero-copy operation straight from the memory-mapped file.
    #[inline]
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&str> {
        match &self.backend {
            AamBackend::Dynamic(output) => output.map.get(key).map(std::convert::AsRef::as_ref),
            #[cfg(feature = "aot")]
            AamBackend::Mapped(mapped) => mapped.get(key),
        }
    }

    /// Iterates over all key-value pairs without allocating memory.
    /// Supports both dynamic `HashMaps` and memory-mapped AOT iterators.
    #[must_use]
    pub fn iter(&self) -> Box<dyn Iterator<Item = (&str, &str)> + '_> {
        match &self.backend {
            AamBackend::Dynamic(output) => {
                Box::new(output.map.iter().map(|(k, v)| (k.as_ref(), v.as_ref())))
            }
            #[cfg(feature = "aot")]
            AamBackend::Mapped(mapped) => Box::new(mapped.iter_pairs()),
        }
    }

    /// Returns all keys currently stored.
    /// Prefer [`AAM::iter`] for zero-allocation iteration.
    #[inline]
    #[must_use]
    pub fn keys(&self) -> Vec<&str> {
        self.iter().map(|(k, _)| k).collect()
    }

    /// Returns all key-value pairs as a standard allocated map.
    /// Prefer [`AAM::iter`] for zero-allocation iteration.
    #[inline]
    #[must_use]
    pub fn to_map(&self) -> PipelineHashMap<String, String> {
        self.iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    // ── Schema & Type Accessors ──────────────────────────────────────────────

    /// Returns a reference to all registered schemas, if loaded dynamically.
    /// Returns `None` if the configuration was loaded via an AOT memory map.
    #[inline]
    #[must_use]
    pub const fn schemas(&self) -> Option<&PipelineHashMap<SmolStr, SchemaInfo>> {
        match &self.backend {
            AamBackend::Dynamic(output) => Some(&output.schemas),
            #[cfg(feature = "aot")]
            AamBackend::Mapped(_) => None,
        }
    }

    /// Returns a specific schema by name, if it exists and was loaded dynamically.
    #[must_use]
    pub fn get_schema(&self, name: &str) -> Option<&SchemaInfo> {
        self.schemas().and_then(|schemas| schemas.get(name))
    }

    /// Returns a reference to all registered types, if loaded dynamically.
    /// Returns `None` if the configuration was loaded via an AOT memory map.
    #[inline]
    #[must_use]
    pub const fn types(&self) -> Option<&PipelineHashMap<SmolStr, TypeInfo>> {
        match &self.backend {
            AamBackend::Dynamic(output) => Some(&output.types),
            #[cfg(feature = "aot")]
            AamBackend::Mapped(_) => None,
        }
    }

    /// Returns a specific type info by name, if it exists and was loaded dynamically.
    #[must_use]
    pub fn get_type(&self, name: &str) -> Option<&TypeInfo> {
        self.types().and_then(|types| types.get(name))
    }
}

impl Default for AAM {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> IntoIterator for &'a AAM {
    type Item = (&'a str, &'a str);
    type IntoIter = Box<dyn Iterator<Item = (&'a str, &'a str)> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
