//! Core width calculation for terminal text.
//!
//! Measures the display width of characters, grapheme clusters, and strings
//! in terminal cells. Uses Unicode East Asian Width for character widths and
//! grapheme cluster analysis for emoji sequences.

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthChar;

use super::ansi::strip_ansi;

/// Display width of a single Unicode codepoint in terminal cells.
///
/// - `0` for control characters, combining marks, zero-width characters
/// - `1` for normal-width characters (ASCII, Latin, Cyrillic, etc.)
/// - `2` for wide characters (CJK ideographs, fullwidth forms)
#[inline]
pub fn char_width(c: char) -> usize {
    // Force known emoji ranges to width 2 (terminal renderers usually treat them as wide)
    match c as u32 {
        // Sparkles ✨, Zap ⚡, etc
        0x2600..=0x27BF => 2,
        // Misc Symbols and Pictographs (typical emojis)
        0x1F300..=0x1F5FF => 2,
        // Emoticons (😀)
        0x1F600..=0x1F64F => 2,
        // Transport and Map Symbols (🚀)
        0x1F680..=0x1F6FF => 2,
        // Supplemental Symbols and Pictographs
        0x1F900..=0x1F9FF => 2,
        // Symbols and Pictographs Extended-A
        0x1FA70..=0x1FAFF => 2,
        _ => c.width().unwrap_or(0),
    }
}

/// Display width of a grapheme cluster in terminal cells.
///
/// A grapheme cluster is a user-perceived character that may span multiple
/// Unicode codepoints. Examples:
/// - `é` (e + combining acute) → width 1
/// - `👨‍👩‍👧‍👦` (family ZWJ sequence) → width 2
/// - `🇺🇸` (flag: regional indicator pair) → width 2
/// - `👍🏽` (thumbs up + skin tone) → width 2
///
/// # Rules
///
/// 1. Single codepoint → delegates to `char_width()`
/// 2. Emoji sequence (contains ZWJ, VS16, skin tone, keycap) → 2
/// 3. Regional indicator pair (flags) → 2
/// 4. Base + combining marks → base character width
pub fn grapheme_width(grapheme: &str) -> usize {
    let mut chars = grapheme.chars();
    let first = match chars.next() {
        Some(c) => c,
        None => return 0,
    };

    // Single codepoint: use char_width for proper emoji handling.
    if grapheme.len() == first.len_utf8() {
        return char_width(first);
    }

    // Multi-codepoint grapheme cluster.

    // Regional indicator pair (flag emoji: 🇺🇸)
    let first_cp = first as u32;
    if (0x1F1E6..=0x1F1FF).contains(&first_cp) {
        return 2;
    }

    // Scan trailing codepoints for emoji sequence modifiers.
    for c in grapheme.chars().skip(1) {
        match c as u32 {
            0x200D => return 2,            // Zero-Width Joiner → ZWJ sequence
            0xFE0F => return 2,            // VS16 → emoji presentation
            0x1F3FB..=0x1F3FF => return 2, // Fitzpatrick skin tone modifier
            0x20E3 => return 2,            // Combining enclosing keycap
            _ => {}
        }
    }

    // Base character + combining marks → base width only.
    first.width().unwrap_or(0)
}

/// Display width of a string in terminal cells.
///
/// Correctly handles:
/// - ANSI escape sequences (stripped, zero-width)
/// - East Asian wide characters (CJK = 2 cells)
/// - Emoji sequences (ZWJ, skin tones, flags = 2 cells)
/// - Combining marks (zero-width)
/// - Control characters (zero-width)
///
/// # Performance
///
/// - Fast path for pure ASCII strings (no allocation, byte counting)
/// - ANSI stripping uses `Cow` to avoid allocation when no escapes present
/// - Grapheme iteration only when non-ASCII content detected
pub fn string_width(s: &str) -> usize {
    if s.is_empty() {
        return 0;
    }

    // Fast path: pure ASCII with no escape sequences.
    // Count printable ASCII bytes directly — no allocation, no iteration overhead.
    if s.is_ascii() && !s.as_bytes().contains(&0x1B) {
        return s.bytes().filter(|&b| b >= 0x20).count();
    }

    let stripped = strip_ansi(s);
    stripped.graphemes(true).map(grapheme_width).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── char_width ──

    #[test]
    fn char_width_ascii() {
        assert_eq!(char_width('a'), 1);
        assert_eq!(char_width('Z'), 1);
        assert_eq!(char_width(' '), 1);
        assert_eq!(char_width('~'), 1);
    }

    #[test]
    fn char_width_control() {
        assert_eq!(char_width('\0'), 0);
        assert_eq!(char_width('\t'), 0);
        assert_eq!(char_width('\n'), 0);
        assert_eq!(char_width('\r'), 0);
        assert_eq!(char_width('\x7F'), 0); // DEL
    }

    #[test]
    fn char_width_cjk() {
        assert_eq!(char_width('你'), 2);
        assert_eq!(char_width('好'), 2);
        assert_eq!(char_width('世'), 2);
        assert_eq!(char_width('界'), 2);
    }

    #[test]
    fn char_width_hangul() {
        assert_eq!(char_width('한'), 2);
        assert_eq!(char_width('글'), 2);
    }

    #[test]
    fn char_width_fullwidth() {
        assert_eq!(char_width('Ａ'), 2); // Fullwidth A
        assert_eq!(char_width('０'), 2); // Fullwidth 0
    }

    #[test]
    fn char_width_combining() {
        assert_eq!(char_width('\u{0300}'), 0); // Combining grave accent
        assert_eq!(char_width('\u{0301}'), 0); // Combining acute accent
        assert_eq!(char_width('\u{0302}'), 0); // Combining circumflex
    }

    #[test]
    fn char_width_emoji() {
        assert_eq!(char_width('😀'), 2);
        assert_eq!(char_width('🎉'), 2);
        assert_eq!(char_width('🚀'), 2);
    }

    // ── grapheme_width ──

    #[test]
    fn grapheme_single_char() {
        assert_eq!(grapheme_width("a"), 1);
        assert_eq!(grapheme_width("你"), 2);
        assert_eq!(grapheme_width("😀"), 2);
    }

    #[test]
    fn grapheme_combining_marks() {
        // e + combining acute = é (width 1, not 2)
        assert_eq!(grapheme_width("e\u{0301}"), 1);
        // a + combining ring above = å
        assert_eq!(grapheme_width("a\u{030A}"), 1);
    }

    #[test]
    fn grapheme_emoji_zwj_sequence() {
        // Family: man + ZWJ + woman + ZWJ + girl + ZWJ + boy
        assert_eq!(grapheme_width("👨\u{200D}👩\u{200D}👧\u{200D}👦"), 2);
    }

    #[test]
    fn grapheme_emoji_skin_tone() {
        // Thumbs up + medium skin tone
        assert_eq!(grapheme_width("👍\u{1F3FD}"), 2);
    }

    #[test]
    fn grapheme_flag() {
        // Regional indicators U + S = US flag
        assert_eq!(grapheme_width("🇺🇸"), 2);
        // Regional indicators B + R = Brazil flag
        assert_eq!(grapheme_width("🇧🇷"), 2);
    }

    #[test]
    fn grapheme_keycap() {
        // 1 + VS16 + keycap
        assert_eq!(grapheme_width("1\u{FE0F}\u{20E3}"), 2);
    }

    #[test]
    fn grapheme_vs16_presentation() {
        // Sun with VS16 (emoji presentation)
        assert_eq!(grapheme_width("☀\u{FE0F}"), 2);
    }

    // ── string_width ──

    #[test]
    fn string_width_ascii() {
        assert_eq!(string_width("hello"), 5);
        assert_eq!(string_width(""), 0);
        assert_eq!(string_width("a b c"), 5);
    }

    #[test]
    fn string_width_control_chars() {
        assert_eq!(string_width("\t"), 0);
        assert_eq!(string_width("a\tb"), 2);
    }

    #[test]
    fn string_width_cjk() {
        assert_eq!(string_width("你好"), 4);
        assert_eq!(string_width("hello你好"), 9);
    }

    #[test]
    fn string_width_emoji_sequence() {
        // Family ZWJ sequence should be width 2, not 8
        assert_eq!(string_width("👨\u{200D}👩\u{200D}👧\u{200D}👦"), 2);
    }

    #[test]
    fn string_width_flag() {
        assert_eq!(string_width("🇺🇸"), 2);
    }

    #[test]
    fn string_width_combining() {
        // "café" with combining acute on e
        assert_eq!(string_width("cafe\u{0301}"), 4);
    }

    #[test]
    fn string_width_ansi_stripped() {
        assert_eq!(string_width("\x1b[31mred\x1b[0m"), 3);
        assert_eq!(string_width("\x1b[1m\x1b[31mBold Red\x1b[0m"), 8);
    }

    #[test]
    fn string_width_ansi_with_cjk() {
        assert_eq!(string_width("\x1b[31m你好\x1b[0m"), 4);
    }

    #[test]
    fn string_width_mixed() {
        // ASCII + CJK + emoji
        assert_eq!(string_width("hi你好😀"), 2 + 4 + 2);
    }
}
