// Copyright 2026 Maravilla Labs
// SPDX-License-Identifier: MIT OR Apache-2.0

use ropey::Rope;
use tower_lsp::lsp_types::{Position, Range, TextDocumentContentChangeEvent, Url};

use crate::regions::{DocumentRegions, Region};

/// Represents an open .luat document
pub struct Document {
    uri: Url,
    /// The document text stored as a rope for efficient edits
    rope: Rope,
    /// Cached regions (invalidated on change)
    regions: Option<DocumentRegions>,
}

impl Document {
    pub fn new(uri: Url, text: String) -> Self {
        let rope = Rope::from_str(&text);
        let mut doc = Self {
            uri,
            rope,
            regions: None,
        };
        doc.parse_regions();
        doc
    }

    pub fn uri(&self) -> &Url {
        &self.uri
    }

    pub fn text(&self) -> String {
        self.rope.to_string()
    }

    /// Get the underlying rope for advanced text operations
    #[allow(dead_code)] // Useful for future features like semantic tokens
    pub fn rope(&self) -> &Rope {
        &self.rope
    }

    pub fn regions(&self) -> Option<&DocumentRegions> {
        self.regions.as_ref()
    }

    /// Apply an incremental change to the document
    pub fn apply_change(&mut self, change: &TextDocumentContentChangeEvent) {
        if let Some(range) = change.range {
            let start_idx = self.position_to_offset(range.start);
            let end_idx = self.position_to_offset(range.end);

            if let (Some(start), Some(end)) = (start_idx, end_idx) {
                self.rope.remove(start..end);
                self.rope.insert(start, &change.text);
            }
        } else {
            // Full document replace
            self.rope = Rope::from_str(&change.text);
        }

        // Invalidate and reparse regions
        self.regions = None;
        self.parse_regions();
    }

    /// Convert LSP position to rope char offset
    pub fn position_to_offset(&self, pos: Position) -> Option<usize> {
        let line = pos.line as usize;
        if line >= self.rope.len_lines() {
            return None;
        }

        let line_start = self.rope.line_to_char(line);
        let col = pos.character as usize;
        let line_len = self.rope.line(line).len_chars();

        if col > line_len {
            Some(line_start + line_len)
        } else {
            Some(line_start + col)
        }
    }

    /// Convert rope char offset to LSP position
    pub fn offset_to_position(&self, offset: usize) -> Position {
        let line = self.rope.char_to_line(offset);
        let line_start = self.rope.line_to_char(line);
        let col = offset - line_start;

        Position {
            line: line as u32,
            character: col as u32,
        }
    }

    /// Get the region at a given position
    pub fn region_at_position(&self, pos: Position) -> Option<&Region> {
        let offset = self.position_to_offset(pos)?;
        self.regions.as_ref()?.region_at_offset(offset)
    }

    /// Get text in a range
    #[allow(dead_code)] // Useful for future features like rename, extract refactoring
    pub fn get_text_range(&self, range: Range) -> Option<String> {
        let start = self.position_to_offset(range.start)?;
        let end = self.position_to_offset(range.end)?;
        Some(self.rope.slice(start..end).to_string())
    }

    /// Get the word at a position
    pub fn word_at_position(&self, pos: Position) -> Option<(String, Range)> {
        let offset = self.position_to_offset(pos)?;
        let text = self.rope.to_string();
        let bytes = text.as_bytes();

        // Find word boundaries
        let mut start = offset;
        while start > 0 && is_word_char(bytes[start - 1]) {
            start -= 1;
        }

        let mut end = offset;
        while end < bytes.len() && is_word_char(bytes[end]) {
            end += 1;
        }

        if start == end {
            return None;
        }

        let word = text[start..end].to_string();
        let range = Range {
            start: self.offset_to_position(start),
            end: self.offset_to_position(end),
        };

        Some((word, range))
    }

    /// Parse document into regions
    fn parse_regions(&mut self) {
        let text = self.text();
        self.regions = Some(DocumentRegions::parse(&text));
    }
}

fn is_word_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}
