// Copyright 2026 Maravilla Labs
// SPDX-License-Identifier: MIT OR Apache-2.0

use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, Documentation, InsertTextFormat, MarkupContent,
    MarkupKind, Position,
};

use crate::document::Document;
use crate::regions::RegionType;

/// Get completions at a position
pub fn get_completions(doc: &Document, position: Position) -> Vec<CompletionItem> {
    let mut completions = Vec::new();

    // Determine context from region
    if let Some(region) = doc.region_at_position(position) {
        match region.region_type {
            RegionType::LuaScript | RegionType::LuaScriptModule => {
                // Inside script - would delegate to lua-language-server
                // For now, provide basic Lua completions
                completions.extend(lua_basic_completions());
            }
            RegionType::LuaExpression => {
                // Inside expression - Lua completions + props
                completions.extend(expression_completions());
            }
            RegionType::ControlFlow | RegionType::Directive => {
                // Inside control flow or directive - limited completions
            }
            _ => {
                // Template context
                completions.extend(template_completions());
            }
        }
    } else {
        // Not in any special region - template context
        // Check what character triggered completion
        if let Some(offset) = doc.position_to_offset(position) {
            let text = doc.text();
            if offset > 0 {
                let prev_char = text.chars().nth(offset - 1);
                match prev_char {
                    Some('{') => {
                        // After { - provide control flow and expression completions
                        completions.extend(after_brace_completions());
                    }
                    Some('<') => {
                        // After < - provide HTML and component completions
                        completions.extend(tag_completions());
                    }
                    _ => {
                        completions.extend(template_completions());
                    }
                }
            }
        }
    }

    completions
}

/// Completions after typing {
fn after_brace_completions() -> Vec<CompletionItem> {
    vec![
        CompletionItem {
            label: "#if".to_string(),
            kind: Some(CompletionItemKind::SNIPPET),
            detail: Some("Conditional block".to_string()),
            insert_text: Some("#if $1}\n\t$0\n{/if".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "Conditionally render content.\n\n```luat\n{#if condition}\n  <p>Shown if true</p>\n{/if}\n```".to_string(),
            })),
            ..Default::default()
        },
        CompletionItem {
            label: "#each".to_string(),
            kind: Some(CompletionItemKind::SNIPPET),
            detail: Some("Loop block".to_string()),
            insert_text: Some("#each $1 as $2}\n\t$0\n{/each".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "Iterate over a list.\n\n```luat\n{#each items as item}\n  <li>{item.name}</li>\n{/each}\n```".to_string(),
            })),
            ..Default::default()
        },
        CompletionItem {
            label: ":else".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Else clause".to_string()),
            insert_text: Some(":else}".to_string()),
            documentation: Some(Documentation::String(
                "Else branch for {#if} blocks".to_string(),
            )),
            ..Default::default()
        },
        CompletionItem {
            label: ":else if".to_string(),
            kind: Some(CompletionItemKind::SNIPPET),
            detail: Some("Else-if clause".to_string()),
            insert_text: Some(":else if $1}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: ":empty".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Empty list handler".to_string()),
            insert_text: Some(":empty}".to_string()),
            documentation: Some(Documentation::String(
                "Shown when {#each} list is empty".to_string(),
            )),
            ..Default::default()
        },
        CompletionItem {
            label: "@html".to_string(),
            kind: Some(CompletionItemKind::SNIPPET),
            detail: Some("Raw HTML output".to_string()),
            insert_text: Some("@html $1}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "Output raw HTML (unescaped).\n\n**Warning:** Can cause XSS if used with untrusted content.".to_string(),
            })),
            ..Default::default()
        },
        CompletionItem {
            label: "@local".to_string(),
            kind: Some(CompletionItemKind::SNIPPET),
            detail: Some("Local variable".to_string()),
            insert_text: Some("@local $1 = $2}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            documentation: Some(Documentation::String(
                "Declare a local constant in the template".to_string(),
            )),
            ..Default::default()
        },
        CompletionItem {
            label: "@render".to_string(),
            kind: Some(CompletionItemKind::SNIPPET),
            detail: Some("Render slot".to_string()),
            insert_text: Some("@render $1()}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            documentation: Some(Documentation::String(
                "Render slot content (e.g., children)".to_string(),
            )),
            ..Default::default()
        },
        CompletionItem {
            label: "/* comment */".to_string(),
            kind: Some(CompletionItemKind::SNIPPET),
            detail: Some("Luat comment".to_string()),
            insert_text: Some("/* $0 */}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            documentation: Some(Documentation::String(
                "Comment that won't appear in output".to_string(),
            )),
            ..Default::default()
        },
    ]
}

/// Completions after typing <
fn tag_completions() -> Vec<CompletionItem> {
    let html_tags = [
        ("div", "Generic container"),
        ("span", "Inline container"),
        ("p", "Paragraph"),
        ("h1", "Heading level 1"),
        ("h2", "Heading level 2"),
        ("h3", "Heading level 3"),
        ("a", "Hyperlink"),
        ("button", "Button"),
        ("input", "Input field"),
        ("form", "Form"),
        ("ul", "Unordered list"),
        ("ol", "Ordered list"),
        ("li", "List item"),
        ("img", "Image"),
        ("table", "Table"),
        ("tr", "Table row"),
        ("td", "Table cell"),
        ("th", "Table header"),
        ("script", "Script block"),
        ("style", "Style block"),
    ];

    html_tags
        .iter()
        .map(|(tag, desc)| CompletionItem {
            label: tag.to_string(),
            kind: Some(CompletionItemKind::PROPERTY),
            detail: Some(desc.to_string()),
            insert_text: if *tag == "input" || *tag == "img" {
                Some(format!("{} $0/>", tag))
            } else {
                Some(format!("{}$1>$0</{}>", tag, tag))
            },
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        })
        .collect()
}

/// General template context completions
fn template_completions() -> Vec<CompletionItem> {
    vec![
        CompletionItem {
            label: "script".to_string(),
            kind: Some(CompletionItemKind::SNIPPET),
            detail: Some("Script block".to_string()),
            insert_text: Some("<script>\n$0\n</script>".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "script module".to_string(),
            kind: Some(CompletionItemKind::SNIPPET),
            detail: Some("Module script block".to_string()),
            insert_text: Some("<script module>\n$0\n</script>".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            documentation: Some(Documentation::String(
                "Script that runs once when the module is loaded".to_string(),
            )),
            ..Default::default()
        },
    ]
}

/// Basic Lua completions (placeholder until lua-ls integration)
fn lua_basic_completions() -> Vec<CompletionItem> {
    vec![
        CompletionItem {
            label: "local".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            ..Default::default()
        },
        CompletionItem {
            label: "function".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            ..Default::default()
        },
        CompletionItem {
            label: "if".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            ..Default::default()
        },
        CompletionItem {
            label: "for".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            ..Default::default()
        },
        CompletionItem {
            label: "while".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            ..Default::default()
        },
        CompletionItem {
            label: "return".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            ..Default::default()
        },
        CompletionItem {
            label: "require".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            insert_text: Some("require(\"$0\")".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
    ]
}

/// Completions for expressions
fn expression_completions() -> Vec<CompletionItem> {
    vec![
        CompletionItem {
            label: "props".to_string(),
            kind: Some(CompletionItemKind::VARIABLE),
            detail: Some("Component props".to_string()),
            documentation: Some(Documentation::String(
                "Props passed to this component".to_string(),
            )),
            ..Default::default()
        },
        CompletionItem {
            label: "props.children".to_string(),
            kind: Some(CompletionItemKind::PROPERTY),
            detail: Some("Slot content".to_string()),
            ..Default::default()
        },
    ]
}
