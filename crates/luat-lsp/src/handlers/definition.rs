// Copyright 2026 Maravilla Labs
// SPDX-License-Identifier: MIT OR Apache-2.0

use tower_lsp::lsp_types::{GotoDefinitionResponse, Location, Position, Range, Url};
use regex::Regex;
use std::sync::LazyLock;

use crate::document::Document;

static REQUIRE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"local\s+(\w+)\s*=\s*require\s*\(\s*["']([^"']+)["']\s*\)"#).unwrap());

static COMPONENT_TAG_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<([A-Z][A-Za-z0-9_]*)"#).unwrap());

/// Get definition for symbol at position
pub fn get_definition(doc: &Document, position: Position) -> Option<GotoDefinitionResponse> {
    let text = doc.text();

    // Get the word at position
    let (word, _range) = doc.word_at_position(position)?;

    // Check if it's a component tag
    if word.chars().next()?.is_uppercase() {
        return find_component_definition(&word, &text, doc);
    }

    // Check if cursor is on a require path
    if let Some(offset) = doc.position_to_offset(position) {
        if let Some(path) = find_require_path_at_offset(&text, offset) {
            return resolve_require_path(&path, doc);
        }
    }

    // For other symbols, we would delegate to lua-language-server
    None
}

/// Find component definition from imports
fn find_component_definition(
    component_name: &str,
    text: &str,
    doc: &Document,
) -> Option<GotoDefinitionResponse> {
    // Look for: local ComponentName = require("path")
    for cap in REQUIRE_RE.captures_iter(text) {
        if let (Some(name), Some(path)) = (cap.get(1), cap.get(2)) {
            if name.as_str() == component_name {
                return resolve_require_path(path.as_str(), doc);
            }
        }
    }

    None
}

/// Find if cursor is on a require path string
fn find_require_path_at_offset(text: &str, offset: usize) -> Option<String> {
    // Find require calls and check if offset is within the path string
    for cap in REQUIRE_RE.captures_iter(text) {
        if let Some(path_match) = cap.get(2) {
            if offset >= path_match.start() && offset <= path_match.end() {
                return Some(path_match.as_str().to_string());
            }
        }
    }
    None
}

/// Resolve a require path to a file location
fn resolve_require_path(path: &str, doc: &Document) -> Option<GotoDefinitionResponse> {
    // Get the document's directory
    let doc_path = doc.uri().to_file_path().ok()?;
    let doc_dir = doc_path.parent()?;

    // Try common extensions
    let extensions = ["luat", "lua"];
    let search_dirs = [
        doc_dir.to_path_buf(),
        doc_dir.join(".."),
        doc_dir.join("../.."),
    ];

    for dir in &search_dirs {
        for ext in &extensions {
            let candidate = dir.join(format!("{}.{}", path, ext));
            if candidate.exists() {
                let uri = Url::from_file_path(&candidate).ok()?;
                return Some(GotoDefinitionResponse::Scalar(Location {
                    uri,
                    range: Range::default(),
                }));
            }

            // Also try with src/ prefix
            let candidate = dir.join("src").join(format!("{}.{}", path, ext));
            if candidate.exists() {
                let uri = Url::from_file_path(&candidate).ok()?;
                return Some(GotoDefinitionResponse::Scalar(Location {
                    uri,
                    range: Range::default(),
                }));
            }
        }
    }

    None
}
