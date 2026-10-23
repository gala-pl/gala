//! Conversion between byte offsets in a document and LSP positions.
//!
//! LSP characters are counted in UTF-16 code units, so a character outside the Basic Multilingual
//! Plane counts as two.

use serde_json::{json, Value};

/// Zero-based `(line, character)` for a byte offset. Offsets past the end, or inside a character,
/// are clamped to the nearest character boundary before it.
pub fn offset_to_position(text: &str, offset: usize) -> (u32, u32) {
    let mut offset = offset.min(text.len());
    while !text.is_char_boundary(offset) {
        offset -= 1;
    }
    let before = &text[..offset];
    let line = before.matches('\n').count();
    let line_start = before.rfind('\n').map_or(0, |i| i + 1);
    let character = text[line_start..offset].encode_utf16().count();
    (line as u32, character as u32)
}

/// Byte offset for a zero-based `(line, character)`. A character past the end of its line maps
/// to the line end; a line past the end of the document maps to the document end.
pub fn position_to_offset(text: &str, line: u32, character: u32) -> usize {
    let mut line_start = 0;
    for _ in 0..line {
        match text[line_start..].find('\n') {
            Some(i) => line_start += i + 1,
            None => return text.len(),
        }
    }
    let mut units: u32 = 0;
    for (i, ch) in text[line_start..].char_indices() {
        if ch == '\n' || units >= character {
            return line_start + i;
        }
        units += ch.len_utf16() as u32;
    }
    text.len()
}

/// An LSP `Range` JSON object covering the byte range `start..end`.
pub fn lsp_range(text: &str, start: usize, end: usize) -> Value {
    let (sl, sc) = offset_to_position(text, start);
    let (el, ec) = offset_to_position(text, end);
    json!({
        "start": {"line": sl, "character": sc},
        "end": {"line": el, "character": ec},
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_positions() {
        let text = "ab\ncd";
        assert_eq!(offset_to_position(text, 0), (0, 0));
        assert_eq!(offset_to_position(text, 2), (0, 2));
        assert_eq!(offset_to_position(text, 4), (1, 1));
        assert_eq!(offset_to_position(text, 99), (1, 2));
    }

    #[test]
    fn multibyte_positions_use_utf16_units() {
        // 'é' is 2 bytes and 1 UTF-16 unit; '😀' is 4 bytes and 2 units.
        let text = "é😀x\nz";
        assert_eq!(offset_to_position(text, 2), (0, 1), "after é");
        assert_eq!(offset_to_position(text, 6), (0, 3), "x after the emoji");
        assert_eq!(offset_to_position(text, 8), (1, 0), "start of the second line");
        // An offset inside the emoji clamps to its start.
        assert_eq!(offset_to_position(text, 4), (0, 1));
    }

    #[test]
    fn round_trip_multibyte() {
        let text = "let é = '😀'\nfn f()";
        for (offset, _) in text.char_indices() {
            let (line, ch) = offset_to_position(text, offset);
            assert_eq!(position_to_offset(text, line, ch), offset, "offset {offset}");
        }
    }

    #[test]
    fn position_past_end_clamps() {
        let text = "ab\ncd";
        assert_eq!(position_to_offset(text, 0, 10), 2, "clamps to end of line 0");
        assert_eq!(position_to_offset(text, 7, 0), text.len(), "line past the end");
    }

    #[test]
    fn range_json_shape() {
        let v = lsp_range("é\nx", 0, 4);
        assert_eq!(v["start"], json!({"line": 0, "character": 0}));
        assert_eq!(v["end"], json!({"line": 1, "character": 1}));
    }
}
