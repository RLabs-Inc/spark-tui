//! ANSI escape sequence stripping.
//!
//! Removes ANSI escape sequences from strings so they don't affect
//! text width measurement. Handles:
//! - CSI sequences: `ESC [` ... final byte (0x40-0x7E)
//! - OSC sequences: `ESC ]` ... BEL (0x07) or ST (ESC \)
//! - DCS/PM/APC sequences: `ESC P`/`ESC ^`/`ESC _` ... ST
//! - Two-character sequences: `ESC` + single char

use std::borrow::Cow;

/// Strip ANSI escape sequences from a string.
///
/// Returns `Cow::Borrowed` when no escape sequences are present (zero allocation).
/// Returns `Cow::Owned` with sequences removed otherwise.
pub fn strip_ansi(s: &str) -> Cow<'_, str> {
    if !s.as_bytes().contains(&0x1B) {
        return Cow::Borrowed(s);
    }

    let mut result = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if bytes[i] == 0x1B {
            i = skip_escape_sequence(bytes, i);
        } else {
            // Regular content: copy everything up to the next ESC.
            // Safe to slice because ESC (0x1B) is a single-byte ASCII character,
            // so splitting at ESC positions never breaks a UTF-8 sequence.
            let start = i;
            while i < len && bytes[i] != 0x1B {
                i += 1;
            }
            result.push_str(&s[start..i]);
        }
    }

    Cow::Owned(result)
}

/// Skip an escape sequence starting at `pos` (which points to ESC byte).
/// Returns the byte index after the complete sequence.
fn skip_escape_sequence(bytes: &[u8], pos: usize) -> usize {
    let next = pos + 1;
    if next >= bytes.len() {
        return bytes.len();
    }

    match bytes[next] {
        b'[' => skip_csi(bytes, next + 1),
        b']' => skip_string_terminated(bytes, next + 1),
        b'P' | b'^' | b'_' => skip_string_terminated(bytes, next + 1),
        _ => next + 1, // Two-character sequence
    }
}

/// Skip a CSI sequence. `pos` is the byte after `[`.
///
/// CSI format: parameter bytes (0x30-0x3F), intermediate bytes (0x20-0x2F),
/// final byte (0x40-0x7E).
fn skip_csi(bytes: &[u8], pos: usize) -> usize {
    let len = bytes.len();
    let mut i = pos;

    while i < len {
        let b = bytes[i];
        if (0x40..=0x7E).contains(&b) {
            return i + 1; // Final byte — sequence complete
        }
        if b < 0x20 || b > 0x7E {
            return i; // Invalid byte — abort sequence
        }
        i += 1;
    }

    len // Unterminated — consume all
}

/// Skip a string-terminated sequence (OSC, DCS, PM, APC).
/// `pos` is the byte after the type indicator.
///
/// Terminates with BEL (0x07) or ST (ESC \).
fn skip_string_terminated(bytes: &[u8], pos: usize) -> usize {
    let len = bytes.len();
    let mut i = pos;

    while i < len {
        match bytes[i] {
            0x07 => return i + 1,
            0x1B if i + 1 < len && bytes[i + 1] == b'\\' => return i + 2,
            _ => i += 1,
        }
    }

    len // Unterminated — consume all
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_ansi() {
        assert!(matches!(strip_ansi("hello"), Cow::Borrowed(_)));
        assert_eq!(strip_ansi("hello"), "hello");
    }

    #[test]
    fn csi_color() {
        assert_eq!(strip_ansi("\x1b[31mred\x1b[0m"), "red");
    }

    #[test]
    fn csi_256_color() {
        assert_eq!(strip_ansi("\x1b[38;5;196mred\x1b[0m"), "red");
    }

    #[test]
    fn csi_truecolor() {
        assert_eq!(strip_ansi("\x1b[38;2;255;0;0mred\x1b[0m"), "red");
    }

    #[test]
    fn csi_cursor_movement() {
        assert_eq!(strip_ansi("\x1b[2J\x1b[Htext"), "text");
    }

    #[test]
    fn osc_hyperlink() {
        assert_eq!(
            strip_ansi("\x1b]8;;https://example.com\x07click\x1b]8;;\x07"),
            "click"
        );
    }

    #[test]
    fn osc_with_st_terminator() {
        assert_eq!(strip_ansi("\x1b]0;window title\x1b\\text"), "text");
    }

    #[test]
    fn two_char_sequence() {
        assert_eq!(strip_ansi("\x1b=normal mode"), "normal mode");
    }

    #[test]
    fn mixed_ansi_and_text() {
        assert_eq!(
            strip_ansi("\x1b[1m\x1b[31mBold Red\x1b[0m normal"),
            "Bold Red normal"
        );
    }

    #[test]
    fn empty_string() {
        assert_eq!(strip_ansi(""), "");
    }

    #[test]
    fn bare_esc_at_end() {
        assert_eq!(strip_ansi("text\x1b"), "text");
    }

    #[test]
    fn unterminated_csi() {
        assert_eq!(strip_ansi("\x1b[31"), "");
    }

    #[test]
    fn unterminated_osc() {
        assert_eq!(strip_ansi("\x1b]8;;url"), "");
    }

    #[test]
    fn dcs_sequence() {
        assert_eq!(strip_ansi("\x1bPdata\x1b\\after"), "after");
    }

    #[test]
    fn unicode_outside_ansi() {
        assert_eq!(strip_ansi("\x1b[31m你好\x1b[0m"), "你好");
    }
}
