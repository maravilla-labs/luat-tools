// Copyright 2026 Maravilla Labs
// SPDX-License-Identifier: MIT OR Apache-2.0

use regex::Regex;
use std::sync::LazyLock;

/// Type of region in a .luat document
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)] // Template variant reserved for future use
pub enum RegionType {
    /// HTML-like template markup (reserved for future template-specific features)
    Template,
    /// Module script: <script module> or <script context="module">
    LuaScriptModule,
    /// Regular script: <script>
    LuaScript,
    /// Lua expression in {expression}
    LuaExpression,
    /// Control flow condition: {#if condition}, {#each list as item}
    ControlFlow,
    /// Directive: {@html}, {@local}, {@render}
    Directive,
    /// HTML comment
    HtmlComment,
    /// Luat comment {/* */}
    LuatComment,
}

/// A region in the document with its type and span
#[derive(Debug, Clone)]
pub struct Region {
    pub region_type: RegionType,
    pub start: usize,
    pub end: usize,
    /// For script/expression regions, the extracted content
    pub content: Option<String>,
}

impl Region {
    pub fn contains(&self, offset: usize) -> bool {
        offset >= self.start && offset < self.end
    }
}

/// All regions in a document
#[derive(Debug)]
pub struct DocumentRegions {
    pub regions: Vec<Region>,
}

// Regex patterns for region detection
static SCRIPT_MODULE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?s)<script\s+(?:module|context\s*=\s*"module")\s*>(.*?)</script>"#).unwrap()
});

static SCRIPT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?s)<script\s*>(.*?)</script>"#).unwrap());

static CONTROL_FLOW_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"\{[#:/!][^}]*\}"#).unwrap());

static DIRECTIVE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"\{@[^}]*\}"#).unwrap());

// Note: regex crate doesn't support lookahead, so we match all braces
// and filter out control flow/directives/comments in post-processing
static MUSTACHE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"\{[^}]+\}"#).unwrap());

static LUAT_COMMENT_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"\{/\*.*?\*/\}"#).unwrap());

static HTML_COMMENT_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"<!--.*?-->"#).unwrap());

impl DocumentRegions {
    /// Parse a document into regions
    pub fn parse(text: &str) -> Self {
        let mut regions = Vec::new();

        // Find module scripts
        for cap in SCRIPT_MODULE_RE.captures_iter(text) {
            if let Some(full) = cap.get(0) {
                let content = cap.get(1).map(|m| m.as_str().to_string());
                regions.push(Region {
                    region_type: RegionType::LuaScriptModule,
                    start: full.start(),
                    end: full.end(),
                    content,
                });
            }
        }

        // Find regular scripts (excluding module scripts)
        for cap in SCRIPT_RE.captures_iter(text) {
            if let Some(full) = cap.get(0) {
                // Check if this overlaps with a module script
                let is_module = regions.iter().any(|r| {
                    r.region_type == RegionType::LuaScriptModule && r.start == full.start()
                });

                if !is_module {
                    let content = cap.get(1).map(|m| m.as_str().to_string());
                    regions.push(Region {
                        region_type: RegionType::LuaScript,
                        start: full.start(),
                        end: full.end(),
                        content,
                    });
                }
            }
        }

        // Find luat comments {/* */}
        for mat in LUAT_COMMENT_RE.find_iter(text) {
            regions.push(Region {
                region_type: RegionType::LuatComment,
                start: mat.start(),
                end: mat.end(),
                content: None,
            });
        }

        // Find HTML comments
        for mat in HTML_COMMENT_RE.find_iter(text) {
            regions.push(Region {
                region_type: RegionType::HtmlComment,
                start: mat.start(),
                end: mat.end(),
                content: None,
            });
        }

        // Find control flow blocks (not inside scripts or comments)
        for mat in CONTROL_FLOW_RE.find_iter(text) {
            if !Self::is_inside_region(&regions, mat.start()) {
                regions.push(Region {
                    region_type: RegionType::ControlFlow,
                    start: mat.start(),
                    end: mat.end(),
                    content: Some(mat.as_str().to_string()),
                });
            }
        }

        // Find directives
        for mat in DIRECTIVE_RE.find_iter(text) {
            if !Self::is_inside_region(&regions, mat.start()) {
                regions.push(Region {
                    region_type: RegionType::Directive,
                    start: mat.start(),
                    end: mat.end(),
                    content: Some(mat.as_str().to_string()),
                });
            }
        }

        // Find mustache expressions (not inside scripts, comments, control flow, or directives)
        for mat in MUSTACHE_RE.find_iter(text) {
            if !Self::is_inside_region(&regions, mat.start()) {
                let expr = mat.as_str();

                // Skip if it starts with special characters (control flow, directive, comment)
                if let Some(first_content_char) = expr.chars().nth(1) {
                    if matches!(first_content_char, '#' | ':' | '/' | '@' | '!') {
                        continue;
                    }
                }

                // Extract content between { and }
                let content = expr
                    .strip_prefix('{')
                    .and_then(|s| s.strip_suffix('}'))
                    .map(|s| s.to_string());

                regions.push(Region {
                    region_type: RegionType::LuaExpression,
                    start: mat.start(),
                    end: mat.end(),
                    content,
                });
            }
        }

        // Sort by start position
        regions.sort_by_key(|r| r.start);

        Self { regions }
    }

    /// Check if an offset is inside any existing region
    fn is_inside_region(regions: &[Region], offset: usize) -> bool {
        regions.iter().any(|r| r.contains(offset))
    }

    /// Get the region at a given offset
    pub fn region_at_offset(&self, offset: usize) -> Option<&Region> {
        self.regions.iter().find(|r| r.contains(offset))
    }

    /// Get all script regions
    pub fn scripts(&self) -> impl Iterator<Item = &Region> {
        self.regions.iter().filter(|r| {
            matches!(
                r.region_type,
                RegionType::LuaScript | RegionType::LuaScriptModule
            )
        })
    }

    /// Get all expression regions
    #[allow(dead_code)] // Useful for lua-language-server integration
    pub fn expressions(&self) -> impl Iterator<Item = &Region> {
        self.regions
            .iter()
            .filter(|r| r.region_type == RegionType::LuaExpression)
    }

    /// Generate a virtual Lua document from all Lua regions
    #[allow(dead_code)] // Useful for lua-language-server integration
    pub fn virtual_lua_document(&self) -> String {
        let mut doc = String::new();

        // Add module scripts first
        for region in self.regions.iter() {
            if region.region_type == RegionType::LuaScriptModule {
                if let Some(content) = &region.content {
                    doc.push_str(content);
                    doc.push_str("\n\n");
                }
            }
        }

        // Add regular scripts
        for region in self.regions.iter() {
            if region.region_type == RegionType::LuaScript {
                if let Some(content) = &region.content {
                    doc.push_str(content);
                    doc.push_str("\n\n");
                }
            }
        }

        // Wrap expressions in a function for analysis
        doc.push_str("local function __luat_template(props)\n");
        for region in self.regions.iter() {
            if region.region_type == RegionType::LuaExpression {
                if let Some(content) = &region.content {
                    doc.push_str(&format!("  local _ = {}\n", content));
                }
            }
        }
        doc.push_str("end\n");

        doc
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_script_regions() {
        let text = r#"
<script module>
local Card = require("Card")
</script>

<script>
local count = 0
</script>

<div>{count}</div>
"#;

        let regions = DocumentRegions::parse(text);

        let scripts: Vec<_> = regions.scripts().collect();
        assert_eq!(scripts.len(), 2);
        assert_eq!(scripts[0].region_type, RegionType::LuaScriptModule);
        assert_eq!(scripts[1].region_type, RegionType::LuaScript);
    }

    #[test]
    fn test_parse_expressions() {
        let text = "<p>{name}</p><span>{count + 1}</span>";
        let regions = DocumentRegions::parse(text);

        let exprs: Vec<_> = regions.expressions().collect();
        assert_eq!(exprs.len(), 2);
        assert_eq!(exprs[0].content.as_deref(), Some("name"));
        assert_eq!(exprs[1].content.as_deref(), Some("count + 1"));
    }

    #[test]
    fn test_parse_control_flow() {
        let text = "{#if visible}<p>Hello</p>{/if}";
        let regions = DocumentRegions::parse(text);

        let control: Vec<_> = regions
            .regions
            .iter()
            .filter(|r| r.region_type == RegionType::ControlFlow)
            .collect();
        assert_eq!(control.len(), 2);
    }
}
