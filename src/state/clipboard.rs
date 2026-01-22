//! Clipboard Module - Text copy/paste support
//!
//! Provides clipboard operations for input components with an internal buffer fallback.
//!
//! # Features
//!
//! - Copy text to clipboard
//! - Paste text from clipboard
//! - Cut (copy + return for deletion)
//! - Internal buffer fallback (no external dependencies)
//!
//! # Example
//!
//! ```ignore
//! use spark_tui::state::clipboard;
//!
//! // Copy text
//! clipboard::copy("Hello, World!");
//!
//! // Paste text
//! if let Some(text) = clipboard::paste() {
//!     println!("Pasted: {}", text);
//! }
//!
//! // Cut (returns text for deletion by caller)
//! if let Some(text) = clipboard::cut("Selected text") {
//!     // Text is now on clipboard
//!     // Caller should delete the original
//! }
//! ```

use std::cell::RefCell;

// =============================================================================
// Internal Buffer
// =============================================================================

thread_local! {
    /// Internal clipboard buffer.
    /// Used as fallback when system clipboard is unavailable.
    static CLIPBOARD_BUFFER: RefCell<Option<String>> = RefCell::new(None);
}

// =============================================================================
// Public API
// =============================================================================

/// Copy text to clipboard.
///
/// Stores text in internal buffer for later paste operations.
/// Empty strings are ignored (clipboard not modified).
pub fn copy(text: &str) {
    if text.is_empty() {
        return;
    }

    CLIPBOARD_BUFFER.with(|buf| {
        *buf.borrow_mut() = Some(text.to_string());
    });
}

/// Paste text from clipboard.
///
/// Returns the most recently copied text, or None if clipboard is empty.
pub fn paste() -> Option<String> {
    CLIPBOARD_BUFFER.with(|buf| {
        buf.borrow().clone()
    })
}

/// Cut text: copy to clipboard and return for deletion.
///
/// Convenience function that copies text to clipboard and returns it.
/// The caller is responsible for deleting the original text.
/// Returns None if text is empty.
pub fn cut(text: &str) -> Option<String> {
    if text.is_empty() {
        return None;
    }

    copy(text);
    Some(text.to_string())
}

/// Clear the clipboard.
pub fn clear() {
    CLIPBOARD_BUFFER.with(|buf| {
        *buf.borrow_mut() = None;
    });
}

/// Check if clipboard has content.
pub fn has_content() -> bool {
    CLIPBOARD_BUFFER.with(|buf| {
        buf.borrow().is_some()
    })
}

/// Get clipboard content length.
pub fn content_length() -> usize {
    CLIPBOARD_BUFFER.with(|buf| {
        buf.borrow().as_ref().map(|s| s.len()).unwrap_or(0)
    })
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() {
        clear();
    }

    #[test]
    fn test_copy_paste() {
        setup();

        // Initially empty
        assert!(paste().is_none());
        assert!(!has_content());

        // Copy text
        copy("Hello");

        // Paste should return it
        assert_eq!(paste(), Some("Hello".to_string()));
        assert!(has_content());
        assert_eq!(content_length(), 5);

        // Paste again (non-destructive)
        assert_eq!(paste(), Some("Hello".to_string()));
    }

    #[test]
    fn test_copy_overwrites() {
        setup();

        copy("First");
        assert_eq!(paste(), Some("First".to_string()));

        copy("Second");
        assert_eq!(paste(), Some("Second".to_string()));
    }

    #[test]
    fn test_copy_empty_ignored() {
        setup();

        copy("Something");
        copy(""); // Should not overwrite

        assert_eq!(paste(), Some("Something".to_string()));
    }

    #[test]
    fn test_cut() {
        setup();

        // Cut returns the text
        let result = cut("Cut me");
        assert_eq!(result, Some("Cut me".to_string()));

        // Text is also on clipboard
        assert_eq!(paste(), Some("Cut me".to_string()));
    }

    #[test]
    fn test_cut_empty() {
        setup();

        copy("Existing");

        // Cut empty returns None and doesn't modify clipboard
        let result = cut("");
        assert!(result.is_none());
        assert_eq!(paste(), Some("Existing".to_string()));
    }

    #[test]
    fn test_clear() {
        setup();

        copy("Something");
        assert!(has_content());

        clear();

        assert!(!has_content());
        assert!(paste().is_none());
        assert_eq!(content_length(), 0);
    }

    #[test]
    fn test_unicode() {
        setup();

        copy("Hello 世界 🚀");

        assert_eq!(paste(), Some("Hello 世界 🚀".to_string()));
        // Note: content_length() returns byte length, not char count
        assert!(content_length() > 10);
    }

    #[test]
    fn test_multiline() {
        setup();

        let multiline = "Line 1\nLine 2\nLine 3";
        copy(multiline);

        assert_eq!(paste(), Some(multiline.to_string()));
    }
}
