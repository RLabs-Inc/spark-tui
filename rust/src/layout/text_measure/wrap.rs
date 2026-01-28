//! Text wrapping for terminal layout.
//!
//! Provides two wrapping modes:
//! - **Character-break** (`wrap_text`): breaks at any grapheme boundary
//! - **Word-break** (`wrap_text_word`): breaks at word boundaries, falls
//!   back to grapheme-break for words wider than the line
//!
//! Both modes correctly handle:
//! - Explicit newlines (`\n`) as hard line breaks
//! - CJK wide characters (2 cells)
//! - Emoji sequences (measured as single grapheme clusters)
//! - Combining marks (zero-width, attached to base)

use unicode_segmentation::UnicodeSegmentation;

use super::width::grapheme_width;

/// Wrap text by breaking at any grapheme boundary.
///
/// Each explicit newline in the input produces a line break.
/// Lines are broken when the next grapheme would exceed `max_width`.
///
/// Returns an empty `Vec` for empty input.
pub fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    if text.is_empty() {
        return vec![];
    }
    if max_width == 0 {
        return vec![text.to_string()];
    }

    let mut lines: Vec<String> = Vec::new();

    for raw_line in text.split('\n') {
        let mut current = String::new();
        let mut current_width: usize = 0;

        for grapheme in raw_line.graphemes(true) {
            let gw = grapheme_width(grapheme);

            if current_width + gw > max_width && !current.is_empty() {
                lines.push(current);
                current = String::new();
                current_width = 0;
            }

            current.push_str(grapheme);
            current_width += gw;
        }

        lines.push(current);
    }

    lines
}

/// Wrap text by breaking at word boundaries.
///
/// Uses Unicode word boundary rules (UAX #29) to find natural break points.
/// Falls back to grapheme-break for words wider than `max_width`.
/// Leading whitespace on new lines is trimmed after a wrap break.
///
/// Returns an empty `Vec` for empty input.
pub fn wrap_text_word(text: &str, max_width: usize) -> Vec<String> {
    if text.is_empty() {
        return vec![];
    }
    if max_width == 0 {
        return vec![text.to_string()];
    }

    let mut lines: Vec<String> = Vec::new();

    for raw_line in text.split('\n') {
        wrap_line_word(raw_line, max_width, &mut lines);
    }

    lines
}

/// Wrap a single line by word boundaries.
fn wrap_line_word(line: &str, max_width: usize, lines: &mut Vec<String>) {
    let mut current = String::new();
    let mut current_width: usize = 0;

    for segment in line.split_word_bounds() {
        let seg_width: usize = segment.graphemes(true).map(grapheme_width).sum();

        if current_width + seg_width > max_width {
            if current_width > 0 {
                lines.push(current.trim_end().to_string());
                current = String::new();
                current_width = 0;
            }

            // Segment wider than max: force-break by grapheme.
            if seg_width > max_width {
                force_break_graphemes(segment, max_width, lines, &mut current, &mut current_width);
                continue;
            }

            // Skip leading whitespace on a new wrapped line.
            if is_whitespace(segment) {
                continue;
            }
        }

        current.push_str(segment);
        current_width += seg_width;
    }

    lines.push(current);
}

/// Force-break a segment that is wider than `max_width` by grapheme boundaries.
fn force_break_graphemes(
    segment: &str,
    max_width: usize,
    lines: &mut Vec<String>,
    current: &mut String,
    current_width: &mut usize,
) {
    for grapheme in segment.graphemes(true) {
        let gw = grapheme_width(grapheme);

        if *current_width + gw > max_width && !current.is_empty() {
            lines.push(std::mem::take(current));
            *current_width = 0;
        }

        current.push_str(grapheme);
        *current_width += gw;
    }
}

/// Check if a segment is entirely whitespace.
fn is_whitespace(s: &str) -> bool {
    s.chars().all(|c| c.is_whitespace())
}

/// Measure the height of text when wrapped to a given width.
///
/// Counts lines without allocating wrapped content.
/// Uses character-break wrapping rules (same as `wrap_text`).
///
/// Returns 0 for empty text.
pub fn measure_text_height(text: &str, max_width: usize) -> usize {
    if text.is_empty() {
        return 0;
    }
    if max_width == 0 {
        return text.split('\n').count();
    }

    let mut lines: usize = 0;

    for raw_line in text.split('\n') {
        lines += 1;

        if raw_line.is_empty() {
            continue;
        }

        let mut current_width: usize = 0;

        for grapheme in raw_line.graphemes(true) {
            let gw = grapheme_width(grapheme);

            if current_width + gw > max_width && current_width > 0 {
                lines += 1;
                current_width = gw;
            } else {
                current_width += gw;
            }
        }
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── wrap_text (character-break) ──

    #[test]
    fn wrap_empty() {
        let lines = wrap_text("", 10);
        assert!(lines.is_empty());
    }

    #[test]
    fn wrap_fits() {
        let lines = wrap_text("hello", 10);
        assert_eq!(lines, vec!["hello"]);
    }

    #[test]
    fn wrap_exact_fit() {
        let lines = wrap_text("hello", 5);
        assert_eq!(lines, vec!["hello"]);
    }

    #[test]
    fn wrap_break_mid_word() {
        let lines = wrap_text("abcdef", 4);
        assert_eq!(lines, vec!["abcd", "ef"]);
    }

    #[test]
    fn wrap_newlines() {
        let lines = wrap_text("a\nb\nc", 10);
        assert_eq!(lines, vec!["a", "b", "c"]);
    }

    #[test]
    fn wrap_empty_newline() {
        let lines = wrap_text("a\n\nb", 10);
        assert_eq!(lines, vec!["a", "", "b"]);
    }

    #[test]
    fn wrap_cjk() {
        // Each CJK char is 2 cells. Width 5 fits 2 chars (4 cells), wraps on 3rd.
        let lines = wrap_text("你好世界", 5);
        assert_eq!(lines, vec!["你好", "世界"]);
    }

    #[test]
    fn wrap_mixed_ascii_cjk() {
        let lines = wrap_text("hi你好", 5);
        // "hi" = 2, "你" = 2 → total 4, fits. "好" = 2 → 6 > 5, wrap.
        assert_eq!(lines, vec!["hi你", "好"]);
    }

    #[test]
    fn wrap_width_zero() {
        let lines = wrap_text("hello", 0);
        assert_eq!(lines, vec!["hello"]);
    }

    // ── wrap_text_word (word-break) ──

    #[test]
    fn word_wrap_simple() {
        let lines = wrap_text_word("hello world", 8);
        assert_eq!(lines, vec!["hello", "world"]);
    }

    #[test]
    fn word_wrap_fits() {
        let lines = wrap_text_word("hello world", 20);
        assert_eq!(lines, vec!["hello world"]);
    }

    #[test]
    fn word_wrap_long_word() {
        // "abcdefghij" is 10 chars, width is 5 → must force-break.
        let lines = wrap_text_word("abcdefghij", 5);
        assert_eq!(lines, vec!["abcde", "fghij"]);
    }

    #[test]
    fn word_wrap_multiple_words() {
        let lines = wrap_text_word("one two three four", 9);
        assert_eq!(lines, vec!["one two", "three", "four"]);
    }

    #[test]
    fn word_wrap_newlines() {
        let lines = wrap_text_word("hello\nworld", 20);
        assert_eq!(lines, vec!["hello", "world"]);
    }

    #[test]
    fn word_wrap_empty() {
        let lines = wrap_text_word("", 10);
        assert!(lines.is_empty());
    }

    // ── measure_text_height ──

    #[test]
    fn height_empty() {
        assert_eq!(measure_text_height("", 10), 0);
    }

    #[test]
    fn height_single_line() {
        assert_eq!(measure_text_height("hello", 10), 1);
    }

    #[test]
    fn height_wrapping() {
        assert_eq!(measure_text_height("abcdef", 4), 2); // "abcd" + "ef"
    }

    #[test]
    fn height_newlines() {
        assert_eq!(measure_text_height("a\nb\nc", 10), 3);
    }

    #[test]
    fn height_newline_and_wrap() {
        assert_eq!(measure_text_height("abcdef\nghi", 4), 3); // "abcd"+"ef" + "ghi"
    }

    #[test]
    fn height_cjk() {
        // 4 CJK chars = 8 cells, width 5 → 2 lines (4+4 cells)
        assert_eq!(measure_text_height("你好世界", 5), 2);
    }

    #[test]
    fn height_width_zero() {
        assert_eq!(measure_text_height("a\nb", 0), 2);
    }

    #[test]
    fn height_empty_line() {
        assert_eq!(measure_text_height("a\n\nb", 10), 3);
    }
}
