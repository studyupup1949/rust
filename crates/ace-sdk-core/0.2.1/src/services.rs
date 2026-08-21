//! Service modules for bootstrap streaming, language detection, and import graph.

use crate::errors::AceError;
use crate::types::{BootstrapMode, VerbosityLevel};

// =============================================================================
// Bootstrap Streaming
// =============================================================================

/// Bootstrap SSE event from server.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BootstrapSSEEvent {
    pub stage: String,
    pub message: String,
    #[serde(default)]
    pub data: Option<serde_json::Value>,
    pub timestamp: String,
}

/// Options for bootstrap streaming.
#[derive(Debug, Clone)]
pub struct BootstrapStreamOptions {
    pub server_url: String,
    pub org_id: String,
    pub project_id: String,
    pub mode: BootstrapMode,
    pub code_blocks: Vec<String>,
    pub metadata: Option<serde_json::Value>,
    pub api_token: Option<String>,
    pub timeout_ms: u64,
    pub verbosity: VerbosityLevel,
}

impl Default for BootstrapStreamOptions {
    fn default() -> Self {
        Self {
            server_url: "https://ace-api.code-engine.app".to_string(),
            org_id: String::new(),
            project_id: String::new(),
            mode: BootstrapMode::default(),
            code_blocks: Vec::new(),
            metadata: None,
            api_token: None,
            timeout_ms: 120_000,
            verbosity: VerbosityLevel::default(),
        }
    }
}

/// Result of bootstrap streaming.
#[derive(Debug, Clone)]
pub struct BootstrapStreamResult {
    pub success: bool,
    pub patterns_extracted: Option<u32>,
    pub processing_time: Option<f64>,
    pub error: Option<BootstrapStreamError>,
}

/// Error info from bootstrap stream.
#[derive(Debug, Clone)]
pub struct BootstrapStreamError {
    pub code: String,
    pub message: String,
    pub retryable: bool,
}

/// Stream bootstrap progress from ACE Server.
///
/// Connects to the server's SSE endpoint and receives progress updates.
pub async fn bootstrap_with_streaming(
    options: BootstrapStreamOptions,
    on_event: impl Fn(BootstrapSSEEvent),
) -> Result<BootstrapStreamResult, AceError> {
    let client = reqwest::Client::new();

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert("Accept", "text/event-stream".parse().unwrap());
    headers.insert("X-ACE-Org", options.org_id.parse().unwrap());
    headers.insert(
        "X-ACE-Project",
        options.project_id.parse().unwrap(),
    );

    if let Some(ref token) = options.api_token {
        headers.insert(
            "Authorization",
            format!("Bearer {}", token).parse().unwrap(),
        );
    }

    let body = serde_json::json!({
        "mode": options.mode,
        "code_blocks": options.code_blocks,
        "metadata": options.metadata.unwrap_or(serde_json::json!({})),
        "verbosity": options.verbosity
    });

    let response = client
        .post(&format!("{}/bootstrap/stream", options.server_url))
        .headers(headers)
        .json(&body)
        .timeout(std::time::Duration::from_millis(options.timeout_ms))
        .send()
        .await?;

    if !response.status().is_success() {
        return Ok(BootstrapStreamResult {
            success: false,
            patterns_extracted: None,
            processing_time: None,
            error: Some(BootstrapStreamError {
                code: "HTTP_ERROR".to_string(),
                message: format!("Bootstrap request failed: {}", response.status()),
                retryable: response.status().is_server_error(),
            }),
        });
    }

    let text = response.text().await?;
    let mut result = BootstrapStreamResult {
        success: false,
        patterns_extracted: None,
        processing_time: None,
        error: None,
    };

    // Parse SSE events
    for line in text.lines() {
        if let Some(data) = line.strip_prefix("data: ") {
            if let Ok(event) = serde_json::from_str::<BootstrapSSEEvent>(data) {
                on_event(event.clone());

                match event.stage.as_str() {
                    "done" => {
                        result.success = true;
                        if let Some(ref d) = event.data {
                            result.patterns_extracted =
                                d.get("patterns_extracted").and_then(|v| v.as_u64()).map(|v| v as u32);
                            result.processing_time =
                                d.get("analysis_time_seconds").and_then(|v| v.as_f64());
                        }
                    }
                    "error" => {
                        if let Some(ref d) = event.data {
                            result.error = Some(BootstrapStreamError {
                                code: d
                                    .get("error_code")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("UNKNOWN")
                                    .to_string(),
                                message: d
                                    .get("message")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or(&event.message)
                                    .to_string(),
                                retryable: d
                                    .get("retryable")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(false),
                            });
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(result)
}

// =============================================================================
// Non-Streaming Bootstrap Fallback
// =============================================================================

/// Non-streaming bootstrap fallback.
///
/// For servers that don't support SSE streaming, this function
/// makes a regular POST request and waits for the response.
pub async fn bootstrap_without_streaming(
    options: BootstrapStreamOptions,
) -> Result<BootstrapStreamResult, AceError> {
    let client = reqwest::Client::new();

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert("X-ACE-Org", options.org_id.parse().unwrap());
    headers.insert(
        "X-ACE-Project",
        options.project_id.parse().unwrap(),
    );

    if let Some(ref token) = options.api_token {
        headers.insert(
            "Authorization",
            format!("Bearer {}", token).parse().unwrap(),
        );
    }

    let body = serde_json::json!({
        "mode": options.mode,
        "code_blocks": options.code_blocks,
        "metadata": options.metadata.unwrap_or(serde_json::json!({}))
    });

    let response = client
        .post(&format!("{}/bootstrap", options.server_url))
        .headers(headers)
        .json(&body)
        .timeout(std::time::Duration::from_millis(options.timeout_ms))
        .send()
        .await?;

    if !response.status().is_success() {
        return Ok(BootstrapStreamResult {
            success: false,
            patterns_extracted: None,
            processing_time: None,
            error: Some(BootstrapStreamError {
                code: if response.status().is_server_error() {
                    "HTTP_ERROR"
                } else {
                    "HTTP_ERROR"
                }
                .to_string(),
                message: format!("Bootstrap request failed: {}", response.status()),
                retryable: response.status().is_server_error(),
            }),
        });
    }

    let data: serde_json::Value = response.json().await?;

    let patterns_extracted = data
        .get("patterns_extracted")
        .or_else(|| data.get("total_patterns"))
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);

    let processing_time = data
        .get("analysis_time_seconds")
        .and_then(|v| v.as_f64());

    Ok(BootstrapStreamResult {
        success: true,
        patterns_extracted,
        processing_time,
        error: None,
    })
}

// =============================================================================
// Language Detector
// =============================================================================

/// Default ignored directory patterns.
pub const DEFAULT_IGNORED_DIRS: &[&str] = &[
    "node_modules",
    "dist",
    "build",
    ".git",
    ".next",
    "target",
    "__pycache__",
    "venv",
    "coverage",
];

/// Language detector using file extension heuristics.
///
/// Maps 40+ file extensions to programming language names.
pub struct LanguageDetector;

impl LanguageDetector {
    /// Create a new language detector.
    pub fn new() -> Self {
        Self
    }

    /// Analyze a list of file paths and return language breakdown.
    ///
    /// Returns a map of language name to file count.
    pub fn analyze(&self, files: &[String]) -> std::collections::HashMap<String, usize> {
        use std::collections::HashMap;

        let mut counts: HashMap<String, usize> = HashMap::new();

        for file in files {
            if let Some(lang) = self.detect_file_language(file) {
                *counts.entry(lang.to_string()).or_insert(0) += 1;
            }
        }

        counts
    }

    /// Detect language of a single file by extension.
    pub fn detect_file_language(&self, file_path: &str) -> Option<&'static str> {
        let ext = file_path.rsplit('.').next().unwrap_or("");
        extension_to_language(ext)
    }

    /// Get primary language from file list.
    pub fn get_primary_language(&self, files: &[String]) -> Option<String> {
        let counts = self.analyze(files);
        counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(lang, _)| lang)
    }

    /// Get language breakdown as percentages.
    pub fn get_language_breakdown(&self, files: &[String]) -> std::collections::HashMap<String, f64> {
        let counts = self.analyze(files);
        let total: usize = counts.values().sum();
        if total == 0 {
            return std::collections::HashMap::new();
        }
        counts
            .into_iter()
            .map(|(lang, count)| (lang, (count as f64 / total as f64) * 100.0))
            .collect()
    }
}

impl Default for LanguageDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Detect primary language from file extensions in a directory.
///
/// Convenience function wrapping LanguageDetector.
pub fn detect_primary_language(files: &[String]) -> Option<String> {
    LanguageDetector::new().get_primary_language(files)
}

// =============================================================================
// Import Graph
// =============================================================================

/// Minimum number of importers to be considered a hub.
pub const HUB_THRESHOLD: usize = 5;

/// Represents a node in the import graph.
#[derive(Debug, Clone)]
pub struct FileNode {
    pub path: String,
    pub imports: Vec<String>,
    pub imported_by: Vec<String>,
    pub is_entry_point: bool,
    pub is_hub: bool,
    pub is_leaf: bool,
}

/// Complete import graph analysis result.
#[derive(Debug, Clone, Default)]
pub struct ImportGraph {
    pub nodes: std::collections::HashMap<String, FileNode>,
    pub entry_points: Vec<String>,
    pub hub_files: Vec<String>,
    pub leaf_files: Vec<String>,
    pub circular_deps: Vec<Vec<String>>,
    pub dead_code: Vec<String>,
}

/// Metrics from import graph analysis (for ProjectDNA).
#[derive(Debug, Clone, Default)]
pub struct GraphMetrics {
    pub total_files: usize,
    pub hub_files: Vec<String>,
    pub entry_points: Vec<String>,
    pub leaf_nodes: Vec<String>,
    pub circular_deps: Vec<Vec<String>>,
}

/// Code health metrics derived from import graph.
#[derive(Debug, Clone, Default)]
pub struct CodeHealthMetrics {
    pub dead_code_percentage: f64,
    pub circular_deps_count: usize,
    pub avg_file_size: f64,
    pub max_file_size: u64,
}

/// Build an import graph from a map of file -> imports.
///
/// This is the Rust-native graph builder. For JS/TS projects, the TypeScript SDK
/// uses Skott; this function provides language-agnostic graph building from
/// pre-parsed dependency data.
pub fn build_import_graph(
    file_imports: &std::collections::HashMap<String, Vec<String>>,
) -> ImportGraph {
    use std::collections::HashMap;

    let mut nodes: HashMap<String, FileNode> = HashMap::new();

    // First pass: create nodes with imports
    for (file, imports) in file_imports {
        nodes.insert(
            file.clone(),
            FileNode {
                path: file.clone(),
                imports: imports.clone(),
                imported_by: Vec::new(),
                is_entry_point: false,
                is_hub: false,
                is_leaf: imports.is_empty(),
            },
        );
    }

    // Second pass: build reverse mapping
    let files_and_imports: Vec<(String, Vec<String>)> = nodes
        .iter()
        .map(|(k, v)| (k.clone(), v.imports.clone()))
        .collect();

    for (file, imports) in &files_and_imports {
        for dep in imports {
            if let Some(dep_node) = nodes.get_mut(dep) {
                dep_node.imported_by.push(file.clone());
            }
        }
    }

    // Third pass: classify nodes
    let mut entry_points = Vec::new();
    let mut hub_files = Vec::new();
    let mut leaf_files = Vec::new();
    let mut dead_code = Vec::new();

    for (path, node) in nodes.iter_mut() {
        if node.imported_by.is_empty() {
            node.is_entry_point = true;
            entry_points.push(path.clone());
        }

        if node.imported_by.len() >= HUB_THRESHOLD {
            node.is_hub = true;
            hub_files.push(path.clone());
        }

        if node.is_leaf {
            leaf_files.push(path.clone());
        }

        // Dead code: not imported and not a likely entry point
        if node.imported_by.is_empty() && !is_likely_entry_point(path) {
            dead_code.push(path.clone());
        }
    }

    ImportGraph {
        nodes,
        entry_points,
        hub_files,
        leaf_files,
        circular_deps: Vec::new(), // Circular dep detection requires DFS
        dead_code,
    }
}

/// Check if a file path looks like an entry point.
fn is_likely_entry_point(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path);
    matches!(
        name,
        "main.rs" | "lib.rs" | "mod.rs" | "index.ts" | "index.js"
            | "main.ts" | "main.js" | "app.ts" | "app.js"
            | "main.py" | "__init__.py" | "main.go" | "main.kt"
    )
}

/// Select priority files from an import graph for bootstrap.
///
/// Priority order:
/// 1. Entry points (always include)
/// 2. Hub files (most imported)
/// 3. Files in circular deps
/// 4. Fill remaining with diverse sampling
pub fn select_priority_files(graph: &ImportGraph, max_files: usize) -> Vec<String> {
    let mut priority: Vec<String> = Vec::new();
    let mut seen = std::collections::HashSet::new();

    fn add_unique(
        files: &[String],
        limit: usize,
        priority: &mut Vec<String>,
        seen: &mut std::collections::HashSet<String>,
    ) {
        let mut added = 0;
        for file in files {
            if !seen.contains(file) {
                seen.insert(file.clone());
                priority.push(file.clone());
                added += 1;
                if added >= limit {
                    break;
                }
            }
        }
    }

    // 1. Entry points
    add_unique(&graph.entry_points, 10, &mut priority, &mut seen);

    // 2. Hub files sorted by import count
    let mut sorted_hubs = graph.hub_files.clone();
    sorted_hubs.sort_by(|a, b| {
        let a_count = graph.nodes.get(a).map(|n| n.imported_by.len()).unwrap_or(0);
        let b_count = graph.nodes.get(b).map(|n| n.imported_by.len()).unwrap_or(0);
        b_count.cmp(&a_count)
    });
    add_unique(&sorted_hubs, 20, &mut priority, &mut seen);

    // 3. Files in circular deps
    let circular_files: Vec<String> = graph.circular_deps.iter().flatten().cloned().collect();
    add_unique(&circular_files, 10, &mut priority, &mut seen);

    // 4. Fill remaining
    if priority.len() < max_files {
        let remaining: Vec<String> = graph
            .nodes
            .keys()
            .filter(|k| !seen.contains(*k))
            .cloned()
            .collect();
        let fill = max_files - priority.len();
        add_unique(&remaining, fill, &mut priority, &mut seen);
    }

    priority.truncate(max_files);
    priority
}

/// Convert import graph to GraphMetrics for ProjectDNA.
pub fn graph_to_metrics(graph: &ImportGraph) -> GraphMetrics {
    GraphMetrics {
        total_files: graph.nodes.len(),
        hub_files: graph.hub_files.iter().take(20).cloned().collect(),
        entry_points: graph.entry_points.iter().take(10).cloned().collect(),
        leaf_nodes: graph.leaf_files.iter().take(20).cloned().collect(),
        circular_deps: graph.circular_deps.iter().take(10).cloned().collect(),
    }
}

/// Calculate code health metrics from import graph.
pub fn calculate_health_metrics(
    graph: &ImportGraph,
    total_files: usize,
    avg_file_size: f64,
    max_file_size: u64,
) -> CodeHealthMetrics {
    let dead_code_percentage = if total_files > 0 {
        (graph.dead_code.len() as f64 / total_files as f64) * 100.0
    } else {
        0.0
    };

    CodeHealthMetrics {
        dead_code_percentage,
        circular_deps_count: graph.circular_deps.len(),
        avg_file_size,
        max_file_size,
    }
}

/// Find circular dependencies using DFS.
///
/// Returns a list of cycles found in the import graph.
pub fn find_circular_deps(graph: &ImportGraph) -> Vec<Vec<String>> {
    use std::collections::HashSet;

    let mut cycles: Vec<Vec<String>> = Vec::new();
    let mut visited: HashSet<String> = HashSet::new();
    let mut rec_stack: Vec<String> = Vec::new();
    let mut rec_set: HashSet<String> = HashSet::new();

    fn dfs(
        node: &str,
        graph: &ImportGraph,
        visited: &mut HashSet<String>,
        rec_stack: &mut Vec<String>,
        rec_set: &mut HashSet<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        visited.insert(node.to_string());
        rec_stack.push(node.to_string());
        rec_set.insert(node.to_string());

        if let Some(file_node) = graph.nodes.get(node) {
            for dep in &file_node.imports {
                if !visited.contains(dep) {
                    dfs(dep, graph, visited, rec_stack, rec_set, cycles);
                } else if rec_set.contains(dep) {
                    // Found a cycle
                    let cycle_start = rec_stack.iter().position(|n| n == dep).unwrap_or(0);
                    let cycle: Vec<String> = rec_stack[cycle_start..].to_vec();
                    if cycle.len() >= 2 {
                        cycles.push(cycle);
                    }
                }
            }
        }

        rec_stack.pop();
        rec_set.remove(node);
    }

    for node in graph.nodes.keys() {
        if !visited.contains(node) {
            dfs(node, graph, &mut visited, &mut rec_stack, &mut rec_set, &mut cycles);
        }
    }

    cycles
}

// =============================================================================
// SSE Line Parsing
// =============================================================================

/// Parse a single SSE line into an event.
///
/// SSE format: `data: {json}`
pub fn parse_sse_line(line: &str) -> Option<BootstrapSSEEvent> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with(':') {
        return None;
    }
    if let Some(data) = trimmed.strip_prefix("data:") {
        let json_str = data.trim();
        if json_str.is_empty() {
            return None;
        }
        serde_json::from_str(json_str).ok()
    } else {
        None
    }
}

/// Map file extension to programming language name (40+ extensions).
pub fn extension_to_language(ext: &str) -> Option<&'static str> {
    match ext {
        // Rust
        "rs" => Some("Rust"),
        // TypeScript
        "ts" | "tsx" | "mts" | "cts" => Some("TypeScript"),
        // JavaScript
        "js" | "jsx" | "mjs" | "cjs" => Some("JavaScript"),
        // Python
        "py" | "pyi" | "pyw" => Some("Python"),
        // Go
        "go" => Some("Go"),
        // Java
        "java" => Some("Java"),
        // Kotlin
        "kt" | "kts" => Some("Kotlin"),
        // Ruby
        "rb" | "rake" => Some("Ruby"),
        // C#
        "cs" => Some("C#"),
        // C++
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" | "hh" => Some("C++"),
        // C
        "c" | "h" => Some("C"),
        // Swift
        "swift" => Some("Swift"),
        // PHP
        "php" => Some("PHP"),
        // Scala
        "scala" | "sc" => Some("Scala"),
        // Zig
        "zig" => Some("Zig"),
        // Dart
        "dart" => Some("Dart"),
        // Elixir
        "ex" | "exs" => Some("Elixir"),
        // Erlang
        "erl" | "hrl" => Some("Erlang"),
        // Haskell
        "hs" | "lhs" => Some("Haskell"),
        // Lua
        "lua" => Some("Lua"),
        // Perl
        "pl" | "pm" => Some("Perl"),
        // R
        "r" | "R" => Some("R"),
        // Shell
        "sh" | "bash" | "zsh" => Some("Shell"),
        // Clojure
        "clj" | "cljs" | "cljc" => Some("Clojure"),
        // Groovy
        "groovy" | "gvy" => Some("Groovy"),
        // OCaml
        "ml" | "mli" => Some("OCaml"),
        // F#
        "fs" | "fsi" | "fsx" => Some("F#"),
        // Objective-C
        "m" | "mm" => Some("Objective-C"),
        // Vue / Svelte
        "vue" => Some("Vue"),
        "svelte" => Some("Svelte"),
        // WASM text
        "wat" | "wast" => Some("WebAssembly"),
        // Nim
        "nim" => Some("Nim"),
        // Crystal
        "cr" => Some("Crystal"),
        // Julia
        "jl" => Some("Julia"),
        // V
        "v" => Some("V"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // detect_primary_language tests
    // =========================================================================

    #[test]
    fn test_detect_primary_language() {
        let files = vec![
            "src/main.rs".to_string(),
            "src/lib.rs".to_string(),
            "src/types.rs".to_string(),
            "README.md".to_string(),
        ];
        assert_eq!(detect_primary_language(&files), Some("Rust".to_string()));
    }

    #[test]
    fn test_detect_primary_language_mixed() {
        let files = vec![
            "index.ts".to_string(),
            "app.ts".to_string(),
            "main.rs".to_string(),
        ];
        assert_eq!(
            detect_primary_language(&files),
            Some("TypeScript".to_string())
        );
    }

    #[test]
    fn test_detect_primary_language_empty() {
        let files: Vec<String> = vec![];
        assert_eq!(detect_primary_language(&files), None);
    }

    // =========================================================================
    // LanguageDetector tests
    // =========================================================================

    #[test]
    fn test_language_detector_analyze() {
        let detector = LanguageDetector::new();
        let files = vec![
            "a.rs".to_string(),
            "b.rs".to_string(),
            "c.py".to_string(),
            "README.md".to_string(),
        ];
        let counts = detector.analyze(&files);
        assert_eq!(counts.get("Rust"), Some(&2));
        assert_eq!(counts.get("Python"), Some(&1));
        assert!(counts.get("Markdown").is_none()); // .md not in our extensions
    }

    #[test]
    fn test_language_detector_breakdown() {
        let detector = LanguageDetector::new();
        let files = vec![
            "a.rs".to_string(),
            "b.py".to_string(),
        ];
        let breakdown = detector.get_language_breakdown(&files);
        assert!((breakdown["Rust"] - 50.0).abs() < 0.01);
        assert!((breakdown["Python"] - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_language_detector_empty() {
        let detector = LanguageDetector::new();
        let files: Vec<String> = vec![];
        assert!(detector.get_primary_language(&files).is_none());
        assert!(detector.get_language_breakdown(&files).is_empty());
    }

    // =========================================================================
    // extension_to_language tests
    // =========================================================================

    #[test]
    fn test_extension_to_language_coverage() {
        assert_eq!(extension_to_language("rs"), Some("Rust"));
        assert_eq!(extension_to_language("ts"), Some("TypeScript"));
        assert_eq!(extension_to_language("tsx"), Some("TypeScript"));
        assert_eq!(extension_to_language("py"), Some("Python"));
        assert_eq!(extension_to_language("go"), Some("Go"));
        assert_eq!(extension_to_language("java"), Some("Java"));
        assert_eq!(extension_to_language("kt"), Some("Kotlin"));
        assert_eq!(extension_to_language("rb"), Some("Ruby"));
        assert_eq!(extension_to_language("cs"), Some("C#"));
        assert_eq!(extension_to_language("cpp"), Some("C++"));
        assert_eq!(extension_to_language("swift"), Some("Swift"));
        assert_eq!(extension_to_language("dart"), Some("Dart"));
        assert_eq!(extension_to_language("ex"), Some("Elixir"));
        assert_eq!(extension_to_language("hs"), Some("Haskell"));
        assert_eq!(extension_to_language("lua"), Some("Lua"));
        assert_eq!(extension_to_language("clj"), Some("Clojure"));
        assert_eq!(extension_to_language("vue"), Some("Vue"));
        assert_eq!(extension_to_language("svelte"), Some("Svelte"));
        assert_eq!(extension_to_language("jl"), Some("Julia"));
        assert_eq!(extension_to_language("nim"), Some("Nim"));
        assert_eq!(extension_to_language("unknown"), None);
    }

    // =========================================================================
    // ImportGraph tests
    // =========================================================================

    #[test]
    fn test_build_import_graph_basic() {
        use std::collections::HashMap;

        let mut deps: HashMap<String, Vec<String>> = HashMap::new();
        deps.insert("main.rs".to_string(), vec!["lib.rs".to_string(), "utils.rs".to_string()]);
        deps.insert("lib.rs".to_string(), vec!["utils.rs".to_string()]);
        deps.insert("utils.rs".to_string(), vec![]);

        let graph = build_import_graph(&deps);

        assert_eq!(graph.nodes.len(), 3);
        assert!(graph.leaf_files.contains(&"utils.rs".to_string()));
        // utils.rs is imported by main.rs and lib.rs
        assert_eq!(graph.nodes["utils.rs"].imported_by.len(), 2);
    }

    #[test]
    fn test_build_import_graph_hub_detection() {
        use std::collections::HashMap;

        let mut deps: HashMap<String, Vec<String>> = HashMap::new();
        // Make "shared.rs" imported by 6 files (>= HUB_THRESHOLD=5)
        deps.insert("shared.rs".to_string(), vec![]);
        for i in 0..6 {
            deps.insert(format!("file{}.rs", i), vec!["shared.rs".to_string()]);
        }

        let graph = build_import_graph(&deps);
        assert!(graph.hub_files.contains(&"shared.rs".to_string()));
        assert!(graph.nodes["shared.rs"].is_hub);
    }

    #[test]
    fn test_select_priority_files() {
        use std::collections::HashMap;

        let mut deps: HashMap<String, Vec<String>> = HashMap::new();
        deps.insert("main.rs".to_string(), vec!["core.rs".to_string()]);
        deps.insert("core.rs".to_string(), vec![]);
        deps.insert("test.rs".to_string(), vec!["core.rs".to_string()]);

        let graph = build_import_graph(&deps);
        let priority = select_priority_files(&graph, 5);

        // Should include entry points
        assert!(!priority.is_empty());
        assert!(priority.len() <= 5);
    }

    #[test]
    fn test_graph_to_metrics() {
        use std::collections::HashMap;

        let mut deps: HashMap<String, Vec<String>> = HashMap::new();
        deps.insert("a.rs".to_string(), vec!["b.rs".to_string()]);
        deps.insert("b.rs".to_string(), vec![]);

        let graph = build_import_graph(&deps);
        let metrics = graph_to_metrics(&graph);

        assert_eq!(metrics.total_files, 2);
    }

    #[test]
    fn test_calculate_health_metrics() {
        use std::collections::HashMap;

        let mut deps: HashMap<String, Vec<String>> = HashMap::new();
        deps.insert("a.rs".to_string(), vec![]);
        deps.insert("b.rs".to_string(), vec![]);

        let graph = build_import_graph(&deps);
        let health = calculate_health_metrics(&graph, 10, 150.0, 500);

        assert!(health.dead_code_percentage >= 0.0);
        assert_eq!(health.avg_file_size, 150.0);
        assert_eq!(health.max_file_size, 500);
    }

    #[test]
    fn test_find_circular_deps() {
        use std::collections::HashMap;

        let mut deps: HashMap<String, Vec<String>> = HashMap::new();
        deps.insert("a.rs".to_string(), vec!["b.rs".to_string()]);
        deps.insert("b.rs".to_string(), vec!["a.rs".to_string()]);

        let graph = build_import_graph(&deps);
        let cycles = find_circular_deps(&graph);

        assert!(!cycles.is_empty());
    }

    #[test]
    fn test_find_circular_deps_none() {
        use std::collections::HashMap;

        let mut deps: HashMap<String, Vec<String>> = HashMap::new();
        deps.insert("a.rs".to_string(), vec!["b.rs".to_string()]);
        deps.insert("b.rs".to_string(), vec![]);

        let graph = build_import_graph(&deps);
        let cycles = find_circular_deps(&graph);

        assert!(cycles.is_empty());
    }

    // =========================================================================
    // SSE parsing tests
    // =========================================================================

    #[test]
    fn test_parse_sse_line_valid() {
        let line = r#"data: {"stage":"progress","message":"Processing...","timestamp":"2025-01-01T00:00:00Z"}"#;
        let event = parse_sse_line(line);
        assert!(event.is_some());
        assert_eq!(event.unwrap().stage, "progress");
    }

    #[test]
    fn test_parse_sse_line_empty() {
        assert!(parse_sse_line("").is_none());
        assert!(parse_sse_line(": comment").is_none());
        assert!(parse_sse_line("not-data").is_none());
    }

    // =========================================================================
    // bootstrap_without_streaming is async - tested via integration
    // =========================================================================
}
