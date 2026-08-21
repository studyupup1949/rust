//! MCP resources exposed as `ac://` URIs.
//!
//! The server advertises a fixed set of "listable" resources via
//! [`LISTED`] and responds to `resources/read` requests through [`read`].
//! Beyond the listed URIs, the server also recognizes templated URIs such
//! as `ac://hosts/{name}` and `ac://examples/{id}`.

#![allow(dead_code, reason = "wired into the rmcp resource router in Task 35")]

pub mod examples;
pub mod hosts;
pub mod schema;

use adaptive_card_core::KnowledgeBase;
use serde_json::Value;

/// Static metadata for a resource exposed by this server.
#[derive(Debug, Clone, Copy)]
pub struct ResourceDef {
    pub uri: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub mime_type: &'static str,
}

pub const LISTED: &[ResourceDef] = &[
    ResourceDef {
        uri: "ac://schema/v1.6",
        name: "Adaptive Cards v1.6 Schema",
        description: "Official Microsoft JSON Schema for Adaptive Card v1.6",
        mime_type: "application/schema+json",
    },
    ResourceDef {
        uri: "ac://hosts",
        name: "Host capability matrix",
        description: "List of supported hosts with version and element caps",
        mime_type: "application/json",
    },
    ResourceDef {
        uri: "ac://hosts/{name}",
        name: "Host detail",
        description: "Per-host capability summary",
        mime_type: "application/json",
    },
    ResourceDef {
        uri: "ac://examples",
        name: "Knowledge base examples",
        description: "Summary of every curated example card",
        mime_type: "application/json",
    },
    ResourceDef {
        uri: "ac://examples/{id}",
        name: "Knowledge base example",
        description: "A single example by id",
        mime_type: "application/json",
    },
];

/// Resolve a URI to its JSON body.
///
/// Accepts both the exact static URIs from [`LISTED`] and the templated
/// variants `ac://hosts/{name}` / `ac://examples/{id}`.
#[must_use]
pub fn read(kb: &KnowledgeBase, uri: &str) -> Option<Value> {
    if uri == "ac://schema/v1.6" {
        return Some(schema::body());
    }
    if uri == "ac://hosts" {
        return Some(hosts::all_body());
    }
    if let Some(name) = uri.strip_prefix("ac://hosts/") {
        return hosts::one_body(name);
    }
    if uri == "ac://examples" {
        return Some(examples::all_body(kb));
    }
    if let Some(id) = uri.strip_prefix("ac://examples/") {
        return examples::one_body(kb, id);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn listed_has_five_entries() {
        assert_eq!(LISTED.len(), 5);
    }

    #[test]
    fn reads_schema_uri() {
        let kb = KnowledgeBase::default();
        let v = read(&kb, "ac://schema/v1.6").unwrap();
        assert!(v.is_object());
    }

    #[test]
    fn reads_hosts_uri() {
        let kb = KnowledgeBase::default();
        let v = read(&kb, "ac://hosts").unwrap();
        assert!(v.is_array());
    }

    #[test]
    fn reads_host_detail() {
        let kb = KnowledgeBase::default();
        let v = read(&kb, "ac://hosts/teams").unwrap();
        assert_eq!(v["max_version"], "1.6");
    }

    #[test]
    fn reads_examples_list() {
        let kb = KnowledgeBase::default();
        let v = read(&kb, "ac://examples").unwrap();
        assert!(v.is_array());
        assert_eq!(v.as_array().unwrap().len(), 0);
    }

    #[test]
    fn unknown_uri_returns_none() {
        let kb = KnowledgeBase::default();
        assert!(read(&kb, "ac://unknown/foo").is_none());
    }

    #[test]
    fn unknown_host_returns_none() {
        let kb = KnowledgeBase::default();
        assert!(read(&kb, "ac://hosts/mars").is_none());
    }
}
