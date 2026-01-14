// Copyright 2026 Maravilla Labs
// SPDX-License-Identifier: MIT OR Apache-2.0

use regex::Regex;
use std::sync::LazyLock;
use tower_lsp::lsp_types::{DocumentSymbol, DocumentSymbolResponse, Position, Range, SymbolKind};

use crate::document::Document;

static FUNCTION_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[\t ]*(?:local\s+)?function\s+(\w+)").unwrap());

static COMPONENT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"local\s+(\w+)\s*=\s*require\s*\("#).unwrap());

/// Get document symbols for outline view
#[allow(deprecated)]
pub fn get_document_symbols(doc: &Document) -> Option<DocumentSymbolResponse> {
    let mut symbols = Vec::new();
    let text = doc.text();

    // Find script regions and extract symbols from them
    if let Some(regions) = doc.regions() {
        for region in regions.scripts() {
            if let Some(content) = &region.content {
                // Find functions
                for cap in FUNCTION_RE.captures_iter(content) {
                    if let Some(name_match) = cap.get(1) {
                        let name = name_match.as_str().to_string();
                        let offset_in_content = name_match.start();

                        // Calculate position relative to document
                        let abs_offset = region.start + offset_in_content + "<script>".len();
                        let pos = doc.offset_to_position(abs_offset);

                        symbols.push(DocumentSymbol {
                            name: name.clone(),
                            detail: Some("function".to_string()),
                            kind: SymbolKind::FUNCTION,
                            range: Range {
                                start: pos,
                                end: Position {
                                    line: pos.line,
                                    character: pos.character + name.len() as u32,
                                },
                            },
                            selection_range: Range {
                                start: pos,
                                end: Position {
                                    line: pos.line,
                                    character: pos.character + name.len() as u32,
                                },
                            },
                            tags: None,
                            deprecated: None,
                            children: None,
                        });
                    }
                }

                // Find component imports
                for cap in COMPONENT_RE.captures_iter(content) {
                    if let Some(name_match) = cap.get(1) {
                        let name = name_match.as_str().to_string();
                        if name
                            .chars()
                            .next()
                            .map(|c| c.is_uppercase())
                            .unwrap_or(false)
                        {
                            let offset_in_content = name_match.start();
                            let abs_offset = region.start + offset_in_content + "<script>".len();
                            let pos = doc.offset_to_position(abs_offset);

                            symbols.push(DocumentSymbol {
                                name: name.clone(),
                                detail: Some("component".to_string()),
                                kind: SymbolKind::CLASS,
                                range: Range {
                                    start: pos,
                                    end: Position {
                                        line: pos.line,
                                        character: pos.character + name.len() as u32,
                                    },
                                },
                                selection_range: Range {
                                    start: pos,
                                    end: Position {
                                        line: pos.line,
                                        character: pos.character + name.len() as u32,
                                    },
                                },
                                tags: None,
                                deprecated: None,
                                children: None,
                            });
                        }
                    }
                }
            }
        }
    }

    // Find component usages in template
    let component_re = Regex::new(r"<([A-Z][A-Za-z0-9_]*)").unwrap();
    for cap in component_re.captures_iter(&text) {
        if let (Some(full), Some(name_match)) = (cap.get(0), cap.get(1)) {
            let name = name_match.as_str().to_string();
            let pos = doc.offset_to_position(full.start());

            // Check if already added as import
            if !symbols
                .iter()
                .any(|s| s.name == name && s.kind == SymbolKind::CLASS)
            {
                symbols.push(DocumentSymbol {
                    name: format!("<{}>", name),
                    detail: Some("component usage".to_string()),
                    kind: SymbolKind::OBJECT,
                    range: Range {
                        start: pos,
                        end: Position {
                            line: pos.line,
                            character: pos.character + name.len() as u32 + 1,
                        },
                    },
                    selection_range: Range {
                        start: pos,
                        end: Position {
                            line: pos.line,
                            character: pos.character + name.len() as u32 + 1,
                        },
                    },
                    tags: None,
                    deprecated: None,
                    children: None,
                });
            }
        }
    }

    if symbols.is_empty() {
        None
    } else {
        Some(DocumentSymbolResponse::Nested(symbols))
    }
}
