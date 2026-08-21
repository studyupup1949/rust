//! `"document-extractor"` node — extract text from PDF, DOCX, and PPTX files.
//!
//! Reads a file from the local filesystem (path provided via a variable selector)
//! and extracts its text content using format-specific parsers.
//!
//! # Config schema
//!
//! ```json
//! {
//!   "file_variable_selector": "upload.output_file_path",
//!   "output_format": "text",
//!   "pages": "1,3-5"
//! }
//! ```
//!
//! | Field | Type | Required | Description |
//! |-------|------|:--------:|-------------|
//! | `file_variable_selector` | string | ✅ | Variable path pointing to the file path |
//! | `output_format` | string | — | `text` (default), `markdown`, or `json` |
//! | `pages` | string | — | Page range for PDFs (e.g. `1,3-5`). Empty = all pages |
//!
//! # Output schema
//!
//! ```json
//! {
//!   "content": "Extracted text content...",
//!   "metadata": {
//!     "file_name": "document.pdf",
//!     "format": "pdf"
//!   }
//! }
//! ```

use async_trait::async_trait;
use regex::Regex;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

use crate::error::{FlowError, Result};
use crate::node::{ExecContext, Node};

// ─────────────────────────────────────────────────────────────────
// Main node
// ─────────────────────────────────────────────────────────────────

pub struct DocumentExtractorNode;

#[async_trait]
impl Node for DocumentExtractorNode {
    fn node_type(&self) -> &str {
        "document-extractor"
    }

    async fn execute(&self, ctx: ExecContext) -> Result<Value> {
        let selector = ctx
            .data
            .get("file_variable_selector")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                FlowError::InvalidDefinition(
                    "document-extractor: missing file_variable_selector".into(),
                )
            })?;

        let output_format = ctx
            .data
            .get("output_format")
            .and_then(|v| v.as_str())
            .unwrap_or("text");

        let pages = ctx.data.get("pages").and_then(|v| v.as_str()).unwrap_or("");

        let is_array_file = ctx
            .data
            .get("is_array_file")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if is_array_file {
            // Resolve selector to an array of file paths
            let file_paths = resolve_array_from_variables(&ctx.variables, selector)?;
            let mut results: Vec<Value> = Vec::with_capacity(file_paths.len());

            for file_path in file_paths {
                let result = extract_single_file(&file_path, output_format, pages)?;
                results.push(result);
            }

            Ok(json!({ "output": results }))
        } else {
            // Single file mode
            let file_path = resolve_string_from_variables(&ctx.variables, selector)?;
            let result = extract_single_file(&file_path, output_format, pages)?;
            Ok(result)
        }
    }
}

/// Extract content from a single file.
fn extract_single_file(file_path: &str, output_format: &str, pages: &str) -> Result<Value> {
    let path_ref = Path::new(file_path);
    let extension = path_ref
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    let file_name = path_ref
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let metadata_json = json!({
        "file_name": file_name,
        "format": extension,
    });

    let content: Vec<String> = match extension.as_str() {
        "pdf" => extract_pdf(file_path, pages)?,
        "docx" => extract_docx(file_path)?,
        "pptx" | "xlsx" => extract_office(file_path, &extension)?,
        "txt" | "md" | "csv" => {
            let text = std::fs::read_to_string(file_path)
                .map_err(|e| FlowError::Internal(format!("failed to read file: {}", e)))?;
            vec![text]
        }
        _ => {
            return Err(FlowError::InvalidDefinition(format!(
                "document-extractor: unsupported file format '{}' (supported: pdf, docx, pptx, xlsx, txt, md, csv)",
                extension
            )));
        }
    };

    let result = match output_format {
        "json" => json!({
            "pages": content,
            "metadata": metadata_json,
        }),
        "markdown" => {
            let pages_md: Vec<String> = content
                .into_iter()
                .enumerate()
                .map(|(i, text)| format!("## Page {}\n\n{}", i + 1, text))
                .collect();
            json!({
                "content": pages_md.join("\n\n"),
                "metadata": metadata_json,
            })
        }
        _ => {
            let full_text = content.join("\n\n");
            json!({
                "content": full_text,
                "metadata": metadata_json,
            })
        }
    };

    Ok(result)
}

// ─────────────────────────────────────────────────────────────────
// PDF extraction
// ─────────────────────────────────────────────────────────────────

fn extract_pdf(file_path: &str, pages: &str) -> Result<Vec<String>> {
    let doc = lopdf::Document::load(file_path)
        .map_err(|e| FlowError::Internal(format!("failed to load PDF: {}", e)))?;

    let page_count = doc.get_pages().len();
    let page_indices: Vec<usize> = parse_page_range(pages, page_count)?;

    let mut result = Vec::new();
    // lopdf uses 0-based page indices
    for idx in page_indices {
        let page_num = (idx as u32).saturating_sub(1);
        let page_ids = [page_num];
        if let Ok(content) = doc.extract_text(&page_ids) {
            result.push(content);
        } else {
            result.push(String::new());
        }
    }

    Ok(result)
}

// ─────────────────────────────────────────────────────────────────
// DOCX extraction (via zip + regex-based XML text extraction)
// ─────────────────────────────────────────────────────────────────

fn extract_docx(file_path: &str) -> Result<Vec<String>> {
    let file_data = std::fs::read(file_path)
        .map_err(|e| FlowError::Internal(format!("failed to read DOCX: {}", e)))?;

    let cursor = std::io::Cursor::new(file_data);
    let mut archive = ZipArchive::new(cursor)
        .map_err(|e| FlowError::Internal(format!("failed to read DOCX as zip: {}", e)))?;

    let mut all_text = String::new();

    if let Ok(mut doc_file) = archive.by_name("word/document.xml") {
        let mut xml_content = String::new();
        if doc_file.read_to_string(&mut xml_content).is_ok() {
            // Extract text from <w:t> elements (Word text runs)
            let text_re = Regex::new(r"<w:t[^>]*>([^<]*)</w:t>").unwrap();
            for cap in text_re.captures_iter(&xml_content) {
                if let Some(text) = cap.get(1) {
                    let t = text.as_str();
                    if !t.is_empty() {
                        all_text.push_str(t);
                        all_text.push(' ');
                    }
                }
            }
        }
    }

    let content = all_text.trim().to_string();
    Ok(vec![content])
}

// ─────────────────────────────────────────────────────────────────
// PPTX/XLSX extraction (via zip + regex-based XML text extraction)
// ─────────────────────────────────────────────────────────────────

fn extract_office(file_path: &str, extension: &str) -> Result<Vec<String>> {
    let file_data = std::fs::read(file_path)
        .map_err(|e| FlowError::Internal(format!("failed to read {}: {}", extension, e)))?;

    let cursor = std::io::Cursor::new(file_data);
    let mut archive = ZipArchive::new(cursor)
        .map_err(|e| FlowError::Internal(format!("failed to read {} as zip: {}", extension, e)))?;

    match extension {
        "pptx" => extract_pptx_from_zip(&mut archive),
        "xlsx" => extract_xlsx_from_zip(&mut archive),
        _ => Err(FlowError::InvalidDefinition(format!(
            "unsupported office format: {}",
            extension
        ))),
    }
}

fn extract_pptx_from_zip(
    archive: &mut ZipArchive<std::io::Cursor<Vec<u8>>>,
) -> Result<Vec<String>> {
    let mut results = Vec::new();

    // PPTX: extract text from slide XML files
    let text_re = Regex::new(r"<a:t>([^<]*)</a:t>").unwrap();

    for i in 1..=100 {
        let slide_name = format!("ppt/slides/slide{}.xml", i);
        if let Ok(mut slide_file) = archive.by_name(&slide_name) {
            let mut xml_content = String::new();
            if slide_file.read_to_string(&mut xml_content).is_ok() {
                let mut slide_text = String::new();
                for cap in text_re.captures_iter(&xml_content) {
                    if let Some(text) = cap.get(1) {
                        let t = text.as_str();
                        if !t.is_empty() {
                            slide_text.push_str(t);
                            slide_text.push(' ');
                        }
                    }
                }
                if !slide_text.trim().is_empty() {
                    results.push(slide_text.trim().to_string());
                }
            }
        }
    }

    if results.is_empty() {
        results.push(String::new());
    }
    Ok(results)
}

fn extract_xlsx_from_zip(
    archive: &mut ZipArchive<std::io::Cursor<Vec<u8>>>,
) -> Result<Vec<String>> {
    let mut results = Vec::new();

    // XLSX: extract text from shared strings (xl/sharedStrings.xml) and sheet XML
    let shared_strings_re = Regex::new(r"<t>([^<]*)</t>").unwrap();
    let cell_value_re = Regex::new(r#"<c[^>]*r="[A-Z]+\d+"[^>]*><v>([^<]*)</v></c>"#).unwrap();

    let mut shared_strings: Vec<String> = Vec::new();

    // First, load shared strings
    if let Ok(mut ss_file) = archive.by_name("xl/sharedStrings.xml") {
        let mut xml_content = String::new();
        if ss_file.read_to_string(&mut xml_content).is_ok() {
            for cap in shared_strings_re.captures_iter(&xml_content) {
                if let Some(text) = cap.get(1) {
                    shared_strings.push(text.as_str().to_string());
                }
            }
        }
    }

    // Then, extract cell values from sheets
    for i in 1..=100 {
        let sheet_name = format!("xl/worksheets/sheet{}.xml", i);
        if let Ok(mut sheet_file) = archive.by_name(&sheet_name) {
            let mut xml_content = String::new();
            if sheet_file.read_to_string(&mut xml_content).is_ok() {
                let mut sheet_text = String::new();
                for cap in cell_value_re.captures_iter(&xml_content) {
                    if let Some(val_match) = cap.get(1) {
                        let v = val_match.as_str();
                        // Try to parse as shared string index
                        if let Ok(idx) = v.parse::<usize>() {
                            if idx < shared_strings.len() {
                                sheet_text.push_str(&shared_strings[idx]);
                                sheet_text.push('\t');
                            }
                        } else {
                            // Numeric value
                            sheet_text.push_str(v);
                            sheet_text.push('\t');
                        }
                    }
                }
                if !sheet_text.trim().is_empty() {
                    results.push(sheet_text.trim().to_string());
                }
            }
        }
    }

    if results.is_empty() {
        results.push(String::new());
    }
    Ok(results)
}

// ─────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────

fn resolve_string_from_variables(
    variables: &HashMap<String, Value>,
    selector: &str,
) -> Result<String> {
    let parts: Vec<&str> = selector.split('.').collect();
    if parts.is_empty() {
        return Err(FlowError::InvalidDefinition(
            "document-extractor: empty variable selector".into(),
        ));
    }

    let mut current: &Value = variables.get(parts[0]).ok_or_else(|| {
        FlowError::Internal(format!(
            "document-extractor: variable '{}' not found",
            parts[0]
        ))
    })?;

    for part in &parts[1..] {
        current = current.get(*part).ok_or_else(|| {
            FlowError::Internal(format!(
                "document-extractor: path '{}' not found in variable",
                part
            ))
        })?;
    }

    current.as_str().map(String::from).ok_or_else(|| {
        FlowError::InvalidDefinition(format!(
            "document-extractor: variable at '{}' is not a string",
            selector
        ))
    })
}

/// Resolve a selector to an array of strings.
fn resolve_array_from_variables(
    variables: &HashMap<String, Value>,
    selector: &str,
) -> Result<Vec<String>> {
    let parts: Vec<&str> = selector.split('.').collect();
    if parts.is_empty() {
        return Err(FlowError::InvalidDefinition(
            "document-extractor: empty variable selector".into(),
        ));
    }

    let mut current: &Value = variables.get(parts[0]).ok_or_else(|| {
        FlowError::Internal(format!(
            "document-extractor: variable '{}' not found",
            parts[0]
        ))
    })?;

    for part in &parts[1..] {
        current = current.get(*part).ok_or_else(|| {
            FlowError::Internal(format!(
                "document-extractor: path '{}' not found in variable",
                part
            ))
        })?;
    }

    current
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .ok_or_else(|| {
            FlowError::InvalidDefinition(format!(
                "document-extractor: variable at '{}' is not an array",
                selector
            ))
        })
}

fn parse_page_range(pages: &str, total_pages: usize) -> Result<Vec<usize>> {
    if pages.trim().is_empty() {
        return Ok((1..=total_pages).collect());
    }

    let mut result = Vec::new();
    for part in pages.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        if part.contains('-') {
            let range_parts: Vec<&str> = part.split('-').collect();
            if range_parts.len() != 2 {
                return Err(FlowError::InvalidDefinition(format!(
                    "invalid page range: '{}'",
                    part
                )));
            }
            let start: usize = range_parts[0].trim().parse().map_err(|_| {
                FlowError::InvalidDefinition(format!("invalid page number: '{}'", range_parts[0]))
            })?;
            let end: usize = range_parts[1].trim().parse().map_err(|_| {
                FlowError::InvalidDefinition(format!("invalid page number: '{}'", range_parts[1]))
            })?;

            if start == 0 || end == 0 {
                return Err(FlowError::InvalidDefinition(
                    "page numbers must be 1-based (1, 2, 3...)".into(),
                ));
            }
            for p in start..=end {
                if p <= total_pages {
                    result.push(p);
                }
            }
        } else {
            let page: usize = part.parse().map_err(|_| {
                FlowError::InvalidDefinition(format!("invalid page number: '{}'", part))
            })?;
            if page == 0 {
                return Err(FlowError::InvalidDefinition(
                    "page numbers must be 1-based (1, 2, 3...)".into(),
                ));
            }
            if page <= total_pages {
                result.push(page);
            }
        }
    }

    Ok(result)
}

// ─────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_page_range_all() {
        let pages = parse_page_range("", 10).unwrap();
        assert_eq!(pages, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    }

    #[test]
    fn test_parse_page_range_single() {
        let pages = parse_page_range("3", 10).unwrap();
        assert_eq!(pages, vec![3]);
    }

    #[test]
    fn test_parse_page_range_list() {
        let pages = parse_page_range("1,3,5", 10).unwrap();
        assert_eq!(pages, vec![1, 3, 5]);
    }

    #[test]
    fn test_parse_page_range_with_dash() {
        let pages = parse_page_range("1-3", 10).unwrap();
        assert_eq!(pages, vec![1, 2, 3]);
    }

    #[test]
    fn test_parse_page_range_mixed() {
        let pages = parse_page_range("1,3-5,8", 10).unwrap();
        assert_eq!(pages, vec![1, 3, 4, 5, 8]);
    }

    #[test]
    fn test_resolve_string_from_variables() {
        let mut vars = HashMap::new();
        vars.insert(
            "upload".into(),
            json!({"output": {"file_path": "/tmp/test.pdf"}}),
        );

        let path = resolve_string_from_variables(&vars, "upload.output.file_path").unwrap();
        assert_eq!(path, "/tmp/test.pdf");
    }
}
