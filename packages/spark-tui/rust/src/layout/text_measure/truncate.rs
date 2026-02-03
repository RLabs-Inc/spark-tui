//! Text truncation with configurable suffix.
//!
//! Truncates text to fit within a terminal cell width, appending a suffix
//! (e.g., "…" or "...") when the text exceeds the available space.
//! Never breaks in the middle of a grapheme cluster.

use unicode_segmentation::UnicodeSegmentation;

use super::width::{grapheme_width, string_width};

/// Truncate text to fit within `max_width` terminal cells.
///
/// If the text is wider than `max_width`, it is truncated at a grapheme
/// boundary and `suffix` is appended. The suffix width is accounted for.
///
/// Returns the original text (owned) if it fits within `max_width`.
///
/// # Arguments
///
/// * `text` - The text to truncate
/// * `max_width` - Maximum display width in terminal cells
/// * `suffix` - String to append when truncated (e.g., `"…"` or `"..."`)
pub fn truncate_text(text: &str, max_width: usize, suffix: &str) -> String {
    if max_width == 0 {
        return String::new();
    }

    let text_width = string_width(text);
    if text_width <= max_width {
        return text.to_string();
    }

    let suffix_width = string_width(suffix);
    if suffix_width >= max_width {
        // Suffix alone exceeds max_width — truncate the suffix itself.
        return truncate_exact(suffix, max_width);
    }

    let target_width = max_width - suffix_width;
    let mut result = String::with_capacity(text.len());
    let mut current_width: usize = 0;

    for grapheme in text.graphemes(true) {
        let gw = grapheme_width(grapheme);
        if current_width + gw > target_width {
            break;
        }
        result.push_str(grapheme);
        current_width += gw;
    }

    result.push_str(suffix);
    result
}

/// Truncate text to exactly `max_width` cells with no suffix.
fn truncate_exact(text: &str, max_width: usize) -> String {
    let mut result = String::new();
    let mut current_width: usize = 0;

    for grapheme in text.graphemes(true) {
        let gw = grapheme_width(grapheme);
        if current_width + gw > max_width {
            break;
        }
        result.push_str(grapheme);
        current_width += gw;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_fits() {
        assert_eq!(truncate_text("hello", 10, "…"), "hello");
    }

    #[test]
    fn truncate_exact_fit() {
        assert_eq!(truncate_text("hello", 5, "…"), "hello");
    }

    #[test]
    fn truncate_with_ellipsis() {
        assert_eq!(truncate_text("hello world", 6, "…"), "hello…");
    }

    #[test]
    fn truncate_three_dot_suffix() {
        assert_eq!(truncate_text("hello world", 8, "..."), "hello...");
    }

    #[test]
    fn truncate_empty_text() {
        assert_eq!(truncate_text("", 5, "…"), "");
    }

    #[test]
    fn truncate_zero_width() {
        assert_eq!(truncate_text("hello", 0, "…"), "");
    }

    #[test]
    fn truncate_cjk() {
        // "你好世界" = 8 cells, max 5 with "…" (1 cell) → target 4 → "你好" (4 cells) + "…"
        assert_eq!(truncate_text("你好世界", 5, "…"), "你好…");
    }

    #[test]
    fn truncate_cjk_boundary() {
        // Target width 3 — "你" (2) fits, "好" (2) doesn't → "你" + "…"
        assert_eq!(truncate_text("你好世界", 4, "…"), "你…");
    }

    #[test]
    fn truncate_suffix_too_wide() {
        // Suffix "..." is 3 cells, max_width is 2 → truncate suffix itself to ".."
        assert_eq!(truncate_text("hello", 2, "..."), "..");
    }

    #[test]
    fn truncate_width_equals_suffix() {
        // max_width 1, suffix "…" is 1 → suffix fills exactly → return "…"
        assert_eq!(truncate_text("hello world", 1, "…"), "…");
    }

    #[test]
    fn truncate_no_suffix() {
        assert_eq!(truncate_text("hello world", 5, ""), "hello");
    }

    #[test]
    fn truncate_preserves_grapheme() {
        // "café" with combining accent: c-a-f-e+combining = 4 cells
        let text = "cafe\u{0301}xyz";
        // Width: c(1) a(1) f(1) e+accent(1) x(1) y(1) z(1) = 7
        // Truncate to 5 with "…" → target 4 → "café" (4 cells) + "…"
        let result = truncate_text(text, 5, "…");
        assert_eq!(string_width(&result), 5);
        assert!(result.ends_with('…'));
    }
}
