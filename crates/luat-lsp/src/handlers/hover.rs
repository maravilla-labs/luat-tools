// Copyright 2026 Maravilla Labs
// SPDX-License-Identifier: MIT OR Apache-2.0

use tower_lsp::lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, Position};

use crate::document::Document;
use crate::regions::RegionType;

/// Get hover information at a position
pub fn get_hover(doc: &Document, position: Position) -> Option<Hover> {
    // Check if we're in a special region
    if let Some(region) = doc.region_at_position(position) {
        match region.region_type {
            RegionType::ControlFlow => {
                return get_control_flow_hover(region.content.as_deref()?);
            }
            RegionType::Directive => {
                return get_directive_hover(region.content.as_deref()?);
            }
            _ => {}
        }
    }

    // Check for word at position
    let (word, _range) = doc.word_at_position(position)?;

    // Provide hover for known keywords/builtins
    get_keyword_hover(&word)
}

fn get_control_flow_hover(content: &str) -> Option<Hover> {
    let hover_text = if content.starts_with("{#if") {
        "**Conditional Block**\n\nRenders content only if the condition is truthy.\n\n```luat\n{#if condition}\n  <p>Shown when true</p>\n{:else}\n  <p>Shown when false</p>\n{/if}\n```"
    } else if content.starts_with("{#each") {
        "**Each Block**\n\nIterates over a list and renders content for each item.\n\n```luat\n{#each items as item, index}\n  <li>{index}: {item.name}</li>\n{:empty}\n  <li>No items</li>\n{/each}\n```"
    } else if content.starts_with("{:else}") {
        "**Else Clause**\n\nProvides an alternative branch for `{#if}` blocks."
    } else if content.starts_with("{:else if") {
        "**Else-If Clause**\n\nProvides an additional condition branch for `{#if}` blocks."
    } else if content.starts_with("{:empty}") {
        "**Empty Clause**\n\nRendered when an `{#each}` list is empty."
    } else if content.starts_with("{/if}") {
        "**End If**\n\nCloses a `{#if}` block."
    } else if content.starts_with("{/each}") {
        "**End Each**\n\nCloses a `{#each}` block."
    } else {
        return None;
    };

    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: hover_text.to_string(),
        }),
        range: None,
    })
}

fn get_directive_hover(content: &str) -> Option<Hover> {
    let hover_text = if content.starts_with("{@html") {
        "**Raw HTML Directive**\n\nOutputs the expression as raw HTML without escaping.\n\n```luat\n{@html props.richContent}\n```\n\n⚠️ **Warning:** Can cause XSS vulnerabilities if used with untrusted content."
    } else if content.starts_with("{@local") {
        "**Local Constant**\n\nDeclares a local variable in the template scope.\n\n```luat\n{@local name = props.user.name}\n<p>Hello, {name}!</p>\n```"
    } else if content.starts_with("{@render") {
        "**Render Directive**\n\nRenders slot content passed to the component.\n\n```luat\n{@render props.children()}\n{@render props.header?.()}  {/* Optional */}\n```"
    } else {
        return None;
    };

    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: hover_text.to_string(),
        }),
        range: None,
    })
}

fn get_keyword_hover(word: &str) -> Option<Hover> {
    let hover_text = match word {
        "props" => "**props**\n\nThe props object contains all properties passed to this component.\n\n```lua\n-- Access props\nlocal name = props.name\nlocal items = props.items or {}\n```",
        "require" => "**require(path)**\n\nImports a Lua module or Luat component.\n\n```lua\nlocal Card = require(\"components/Card\")\nlocal utils = require(\"lib/utils\")\n```",
        "children" => "**props.children**\n\nSlot content passed to this component. Call it as a function to render.\n\n```luat\n{@render props.children()}\n```",
        _ => return None,
    };

    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: hover_text.to_string(),
        }),
        range: None,
    })
}
