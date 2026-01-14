// Copyright 2026 Maravilla Labs
// SPDX-License-Identifier: MIT OR Apache-2.0

use regex::Regex;
use std::sync::LazyLock;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

use crate::document::Document;

// Useful for detecting unclosed braces at end of file
#[allow(dead_code)]
static UNCLOSED_BRACE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\{[^}]*$").unwrap());

static CONTROL_FLOW_OPEN_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{#(if|each)\b").unwrap());

static CONTROL_FLOW_CLOSE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{/(if|each)\}").unwrap());

/// Compute diagnostics for a document
pub fn compute_diagnostics(doc: &Document) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let text = doc.text();

    // Check for unclosed braces
    diagnostics.extend(check_unclosed_braces(&text, doc));

    // Check for mismatched control flow
    diagnostics.extend(check_control_flow_balance(&text, doc));

    // Check for unclosed tags
    diagnostics.extend(check_unclosed_tags(&text, doc));

    diagnostics
}

fn check_unclosed_braces(text: &str, doc: &Document) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut brace_depth = 0;
    let mut brace_start: Option<usize> = None;
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let ch = chars[i];

        // Check for start of multi-line comment {/*
        if ch == '{' && i + 2 < len && chars[i + 1] == '/' && chars[i + 2] == '*' {
            // Skip until we find */}
            i += 3;
            while i + 2 < len {
                if chars[i] == '*' && chars[i + 1] == '/' && chars[i + 2] == '}' {
                    i += 3;
                    break;
                }
                i += 1;
            }
            continue;
        }

        // Check for start of line comment {--
        if ch == '{' && i + 2 < len && chars[i + 1] == '-' && chars[i + 2] == '-' {
            // Skip until we find --}
            i += 3;
            while i + 2 < len {
                if chars[i] == '-' && chars[i + 1] == '-' && chars[i + 2] == '}' {
                    i += 3;
                    break;
                }
                i += 1;
            }
            continue;
        }

        // Calculate byte offset for position conversion
        let byte_offset: usize = chars[..i].iter().map(|c| c.len_utf8()).sum();

        match ch {
            '{' => {
                if brace_depth == 0 {
                    brace_start = Some(byte_offset);
                }
                brace_depth += 1;
            }
            '}' => {
                if brace_depth > 0 {
                    brace_depth -= 1;
                }
            }
            '\n' => {
                // If we have an unclosed brace at end of line (outside strings/scripts), report it
                if brace_depth > 0 && !is_in_script_block(text, byte_offset) {
                    if let Some(start) = brace_start {
                        let start_pos = doc.offset_to_position(start);
                        let end_pos = doc.offset_to_position(byte_offset);
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: start_pos,
                                end: end_pos,
                            },
                            severity: Some(DiagnosticSeverity::ERROR),
                            message: "Unclosed brace".to_string(),
                            source: Some("luat".to_string()),
                            ..Default::default()
                        });
                    }
                    brace_depth = 0;
                    brace_start = None;
                }
            }
            _ => {}
        }
        i += 1;
    }

    diagnostics
}

fn check_control_flow_balance(text: &str, doc: &Document) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Track open blocks: (type, position)
    let mut stack: Vec<(&str, usize)> = Vec::new();

    // Find all opening blocks
    for cap in CONTROL_FLOW_OPEN_RE.captures_iter(text) {
        if let (Some(full), Some(kind)) = (cap.get(0), cap.get(1)) {
            stack.push((kind.as_str(), full.start()));
        }
    }

    // Find all closing blocks and match them
    let mut close_positions: Vec<(&str, usize)> = Vec::new();
    for cap in CONTROL_FLOW_CLOSE_RE.captures_iter(text) {
        if let (Some(full), Some(kind)) = (cap.get(0), cap.get(1)) {
            close_positions.push((kind.as_str(), full.start()));
        }
    }

    // Simple balance check
    let if_opens = stack.iter().filter(|(k, _)| *k == "if").count();
    let if_closes = close_positions.iter().filter(|(k, _)| *k == "if").count();

    if if_opens > if_closes {
        // Find the unclosed {#if}
        for (kind, pos) in &stack {
            if *kind == "if" {
                let start_pos = doc.offset_to_position(*pos);
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: start_pos,
                        end: Position {
                            line: start_pos.line,
                            character: start_pos.character + 4,
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: "Unclosed {#if} block - missing {/if}".to_string(),
                    source: Some("luat".to_string()),
                    ..Default::default()
                });
                break;
            }
        }
    }

    let each_opens = stack.iter().filter(|(k, _)| *k == "each").count();
    let each_closes = close_positions.iter().filter(|(k, _)| *k == "each").count();

    if each_opens > each_closes {
        for (kind, pos) in &stack {
            if *kind == "each" {
                let start_pos = doc.offset_to_position(*pos);
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: start_pos,
                        end: Position {
                            line: start_pos.line,
                            character: start_pos.character + 6,
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: "Unclosed {#each} block - missing {/each}".to_string(),
                    source: Some("luat".to_string()),
                    ..Default::default()
                });
                break;
            }
        }
    }

    diagnostics
}

fn check_unclosed_tags(text: &str, doc: &Document) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Check for unclosed script tags
    let script_opens = text.matches("<script").count();
    let script_closes = text.matches("</script>").count();

    if script_opens > script_closes {
        if let Some(pos) = text.find("<script") {
            let start_pos = doc.offset_to_position(pos);
            diagnostics.push(Diagnostic {
                range: Range {
                    start: start_pos,
                    end: Position {
                        line: start_pos.line,
                        character: start_pos.character + 7,
                    },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                message: "Unclosed <script> tag".to_string(),
                source: Some("luat".to_string()),
                ..Default::default()
            });
        }
    }

    diagnostics
}

/// Check if position is inside a script block (simplified)
fn is_in_script_block(text: &str, pos: usize) -> bool {
    let before = &text[..pos];

    let last_script_open = before.rfind("<script");
    let last_script_close = before.rfind("</script>");

    match (last_script_open, last_script_close) {
        (Some(open), Some(close)) => open > close,
        (Some(_), None) => true,
        _ => false,
    }
}
