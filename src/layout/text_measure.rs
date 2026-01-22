//! Text Measurement
//!
//! Utilities for measuring text dimensions in terminal cells.
//!
//! Terminal text width depends on Unicode character widths:
//! - ASCII characters: 1 cell
//! - CJK characters: 2 cells (fullwidth)
//! - Emoji: 2 cells (most)
//! - Zero-width characters: 0 cells
//!
//! For now, we use a simple approximation. A full implementation
//! would use unicode-width crate for accurate measurements.

/// Measure the display width of a string in terminal cells.
///
/// This is a simple implementation that counts:
/// - ASCII printable as 1 cell
/// - Other characters as 1-2 cells (approximation)
///
/// For production, consider using `unicode-width` crate.
pub fn string_width(s: &str) -> u16 {
    let mut width = 0u16;

    for c in s.chars() {
        let char_width = if c.is_ascii() {
            if c.is_ascii_control() {
                0  // Control characters have no width
            } else {
                1  // ASCII printable
            }
        } else {
            // Approximation for non-ASCII
            // CJK and emoji are typically 2 cells
            // This is a simplification - use unicode-width for accuracy
            let code = c as u32;
            if (0x1100..=0x115F).contains(&code)     // Hangul Jamo
                || (0x2E80..=0x9FFF).contains(&code)   // CJK
                || (0xAC00..=0xD7A3).contains(&code)   // Hangul Syllables
                || (0xF900..=0xFAFF).contains(&code)   // CJK Compatibility
                || (0xFE10..=0xFE1F).contains(&code)   // Vertical Forms
                || (0xFE30..=0xFE6F).contains(&code)   // CJK Compatibility Forms
                || (0xFF00..=0xFF60).contains(&code)   // Fullwidth Forms
                || (0xFFE0..=0xFFE6).contains(&code)   // Fullwidth Forms
                || (0x1F300..=0x1F9FF).contains(&code) // Emoji
                || (0x20000..=0x2FFFF).contains(&code) // CJK Extension B-F
            {
                2
            } else {
                1
            }
        };
        width = width.saturating_add(char_width);
    }

    width
}

/// Measure the height of text when wrapped to a given width.
///
/// Returns the number of lines the text would occupy.
///
/// # Arguments
///
/// * `text` - The text to measure
/// * `available_width` - The width to wrap at (in cells)
///
/// # Returns
///
/// The number of lines (minimum 1 for non-empty text, 0 for empty).
pub fn measure_text_height(text: &str, available_width: u16) -> u16 {
    if text.is_empty() {
        return 0;
    }

    if available_width == 0 {
        return 1;  // Degenerate case
    }

    let mut lines = 0u16;
    let mut current_line_width = 0u16;

    for c in text.chars() {
        if c == '\n' {
            // Explicit newline
            lines = lines.saturating_add(1);
            current_line_width = 0;
            continue;
        }

        let char_width = if c.is_ascii() {
            if c.is_ascii_control() { 0 } else { 1 }
        } else {
            let code = c as u32;
            if (0x1100..=0x115F).contains(&code)
                || (0x2E80..=0x9FFF).contains(&code)
                || (0xAC00..=0xD7A3).contains(&code)
                || (0xF900..=0xFAFF).contains(&code)
                || (0xFE10..=0xFE1F).contains(&code)
                || (0xFE30..=0xFE6F).contains(&code)
                || (0xFF00..=0xFF60).contains(&code)
                || (0xFFE0..=0xFFE6).contains(&code)
                || (0x1F300..=0x1F9FF).contains(&code)
                || (0x20000..=0x2FFFF).contains(&code)
            {
                2
            } else {
                1
            }
        };

        if current_line_width + char_width > available_width && current_line_width > 0 {
            // Wrap to next line
            lines = lines.saturating_add(1);
            current_line_width = char_width;
        } else {
            current_line_width += char_width;
        }
    }

    // Count the final line if it has content
    if current_line_width > 0 || lines == 0 {
        lines = lines.saturating_add(1);
    }

    lines.max(1)
}

/// Word-wrap text to a given width.
///
/// Returns a vector of lines, each fitting within the specified width.
///
/// # Arguments
///
/// * `text` - The text to wrap
/// * `width` - Maximum width per line (in cells)
///
/// # Returns
///
/// A vector of line strings.
pub fn wrap_text(text: &str, width: u16) -> Vec<String> {
    if text.is_empty() {
        return vec![];
    }

    if width == 0 {
        return vec![text.to_string()];
    }

    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0u16;

    for c in text.chars() {
        if c == '\n' {
            lines.push(current_line);
            current_line = String::new();
            current_width = 0;
            continue;
        }

        let char_width = if c.is_ascii() {
            if c.is_ascii_control() { 0 } else { 1 }
        } else {
            let code = c as u32;
            if (0x1100..=0x115F).contains(&code)
                || (0x2E80..=0x9FFF).contains(&code)
                || (0xAC00..=0xD7A3).contains(&code)
                || (0xF900..=0xFAFF).contains(&code)
                || (0xFE10..=0xFE1F).contains(&code)
                || (0xFE30..=0xFE6F).contains(&code)
                || (0xFF00..=0xFF60).contains(&code)
                || (0xFFE0..=0xFFE6).contains(&code)
                || (0x1F300..=0x1F9FF).contains(&code)
                || (0x20000..=0x2FFFF).contains(&code)
            {
                2
            } else {
                1
            }
        };

        if current_width + char_width > width && !current_line.is_empty() {
            lines.push(current_line);
            current_line = String::new();
            current_width = 0;
        }

        current_line.push(c);
        current_width += char_width;
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}

/// Truncate text to fit within a given width.
///
/// If text is longer than width, it's truncated and an ellipsis is added.
///
/// # Arguments
///
/// * `text` - The text to truncate
/// * `width` - Maximum width (in cells)
///
/// # Returns
///
/// The truncated string.
pub fn truncate_text(text: &str, width: u16) -> String {
    if width == 0 {
        return String::new();
    }

    let text_width = string_width(text);
    if text_width <= width {
        return text.to_string();
    }

    // Need to truncate - leave room for ellipsis
    let target_width = width.saturating_sub(1);
    let mut result = String::new();
    let mut current_width = 0u16;

    for c in text.chars() {
        let char_width = if c.is_ascii() {
            if c.is_ascii_control() { 0 } else { 1 }
        } else {
            let code = c as u32;
            if (0x1100..=0x115F).contains(&code)
                || (0x2E80..=0x9FFF).contains(&code)
                || (0xAC00..=0xD7A3).contains(&code)
                || (0xF900..=0xFAFF).contains(&code)
                || (0xFE10..=0xFE1F).contains(&code)
                || (0xFE30..=0xFE6F).contains(&code)
                || (0xFF00..=0xFF60).contains(&code)
                || (0xFFE0..=0xFFE6).contains(&code)
                || (0x1F300..=0x1F9FF).contains(&code)
                || (0x20000..=0x2FFFF).contains(&code)
            {
                2
            } else {
                1
            }
        };

        if current_width + char_width > target_width {
            break;
        }

        result.push(c);
        current_width += char_width;
    }

    result.push('…');
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_width_ascii() {
        assert_eq!(string_width("hello"), 5);
        assert_eq!(string_width(""), 0);
        assert_eq!(string_width("a b c"), 5);
    }

    #[test]
    fn test_string_width_control_chars() {
        assert_eq!(string_width("\t"), 0);  // Tab is control
        assert_eq!(string_width("a\tb"), 2);
    }

    #[test]
    fn test_measure_text_height_simple() {
        assert_eq!(measure_text_height("hello", 10), 1);
        assert_eq!(measure_text_height("hello world", 5), 3);  // hello, worl, d
        assert_eq!(measure_text_height("", 10), 0);
    }

    #[test]
    fn test_measure_text_height_newlines() {
        assert_eq!(measure_text_height("a\nb\nc", 10), 3);
        assert_eq!(measure_text_height("hello\nworld", 10), 2);
    }

    #[test]
    fn test_wrap_text() {
        let lines = wrap_text("hello world", 5);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "hello");
        assert_eq!(lines[1], " worl");
        assert_eq!(lines[2], "d");
    }

    #[test]
    fn test_wrap_text_newlines() {
        let lines = wrap_text("a\nb", 10);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "a");
        assert_eq!(lines[1], "b");
    }

    #[test]
    fn test_truncate_text() {
        assert_eq!(truncate_text("hello", 10), "hello");
        assert_eq!(truncate_text("hello world", 6), "hello…");
        assert_eq!(truncate_text("", 5), "");
    }

    #[test]
    fn test_truncate_text_exact() {
        assert_eq!(truncate_text("hello", 5), "hello");
        assert_eq!(truncate_text("hello", 4), "hel…");
    }
}
