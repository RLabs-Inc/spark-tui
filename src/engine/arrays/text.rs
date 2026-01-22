//! TUI Framework - Text Arrays
//!
//! Text-related properties:
//! - textContent: The text string to display
//! - textAttrs: Text attributes (bold, italic, etc.)
//! - textAlign: Alignment (left, center, right)
//! - textWrap: Wrap mode (wrap, nowrap, truncate)
//!
//! Uses `TrackedSlotArray` for stable reactive cells with fine-grained tracking.

use spark_signals::TrackedSlotArray;
use crate::types::{Attr, TextAlign, TextWrap};

// =============================================================================
// Arrays
// =============================================================================

thread_local! {
    /// Text content string.
    static TEXT_CONTENT: TrackedSlotArray<String> = TrackedSlotArray::new(Some(String::new()));

    /// Text attributes (bold, italic, etc.).
    static TEXT_ATTRS: TrackedSlotArray<Attr> = TrackedSlotArray::new(Some(Attr::NONE));

    /// Text alignment.
    static TEXT_ALIGN: TrackedSlotArray<TextAlign> = TrackedSlotArray::new(Some(TextAlign::Left));

    /// Text wrap mode.
    static TEXT_WRAP: TrackedSlotArray<TextWrap> = TrackedSlotArray::new(Some(TextWrap::Wrap));
}

// =============================================================================
// Capacity Management
// =============================================================================

/// Ensure arrays have capacity for the given index.
pub fn ensure_capacity(index: usize) {
    TEXT_CONTENT.with(|arr| { let _ = arr.peek(index); });
    TEXT_ATTRS.with(|arr| { let _ = arr.peek(index); });
    TEXT_ALIGN.with(|arr| { let _ = arr.peek(index); });
    TEXT_WRAP.with(|arr| { let _ = arr.peek(index); });
}

/// Clear values at index.
pub fn clear_at_index(index: usize) {
    TEXT_CONTENT.with(|arr| arr.clear(index));
    TEXT_ATTRS.with(|arr| arr.clear(index));
    TEXT_ALIGN.with(|arr| arr.clear(index));
    TEXT_WRAP.with(|arr| arr.clear(index));
}

/// Reset all arrays.
pub fn reset() {
    TEXT_CONTENT.with(|arr| arr.clear_all());
    TEXT_ATTRS.with(|arr| arr.clear_all());
    TEXT_ALIGN.with(|arr| arr.clear_all());
    TEXT_WRAP.with(|arr| arr.clear_all());
}

// =============================================================================
// Text Content
// =============================================================================

/// Get text content at index (reactive).
pub fn get_text_content(index: usize) -> String {
    TEXT_CONTENT.with(|arr| arr.get(index))
}

/// Set text content at index.
pub fn set_text_content(index: usize, content: String) {
    TEXT_CONTENT.with(|arr| arr.set_value(index, content));
}

/// Set text content from a getter function.
pub fn set_text_content_getter<F>(index: usize, getter: F)
where
    F: Fn() -> String + 'static,
{
    TEXT_CONTENT.with(|arr| arr.set_getter(index, getter));
}

/// Set text content from a signal.
pub fn set_text_content_signal(index: usize, sig: spark_signals::Signal<String>) {
    TEXT_CONTENT.with(|arr| arr.set_signal(index, sig));
}

// =============================================================================
// Text Attributes
// =============================================================================

/// Get text attributes at index (reactive).
pub fn get_text_attrs(index: usize) -> Attr {
    TEXT_ATTRS.with(|arr| arr.get(index))
}

/// Set text attributes at index.
pub fn set_text_attrs(index: usize, attrs: Attr) {
    TEXT_ATTRS.with(|arr| arr.set_value(index, attrs));
}

/// Set text attributes from a getter function.
pub fn set_text_attrs_getter<F>(index: usize, getter: F)
where
    F: Fn() -> Attr + 'static,
{
    TEXT_ATTRS.with(|arr| arr.set_getter(index, getter));
}

// =============================================================================
// Text Align
// =============================================================================

/// Get text alignment at index (reactive).
pub fn get_text_align(index: usize) -> TextAlign {
    TEXT_ALIGN.with(|arr| arr.get(index))
}

/// Set text alignment at index.
pub fn set_text_align(index: usize, align: TextAlign) {
    TEXT_ALIGN.with(|arr| arr.set_value(index, align));
}

/// Set text alignment from a getter function.
pub fn set_text_align_getter<F>(index: usize, getter: F)
where
    F: Fn() -> TextAlign + 'static,
{
    TEXT_ALIGN.with(|arr| arr.set_getter(index, getter));
}

// =============================================================================
// Text Wrap
// =============================================================================

/// Get text wrap mode at index (reactive).
pub fn get_text_wrap(index: usize) -> TextWrap {
    TEXT_WRAP.with(|arr| arr.get(index))
}

/// Set text wrap mode at index.
pub fn set_text_wrap(index: usize, wrap: TextWrap) {
    TEXT_WRAP.with(|arr| arr.set_value(index, wrap));
}

/// Set text wrap mode from a getter function.
pub fn set_text_wrap_getter<F>(index: usize, getter: F)
where
    F: Fn() -> TextWrap + 'static,
{
    TEXT_WRAP.with(|arr| arr.set_getter(index, getter));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() {
        reset();
    }

    #[test]
    fn test_text_content() {
        setup();

        assert_eq!(get_text_content(0), "");

        set_text_content(0, "Hello, World!".to_string());
        assert_eq!(get_text_content(0), "Hello, World!");
    }

    #[test]
    fn test_text_attrs() {
        setup();

        assert_eq!(get_text_attrs(0), Attr::NONE);

        set_text_attrs(0, Attr::BOLD | Attr::ITALIC);
        assert_eq!(get_text_attrs(0), Attr::BOLD | Attr::ITALIC);
    }

    #[test]
    fn test_text_align() {
        setup();

        assert_eq!(get_text_align(0), TextAlign::Left);

        set_text_align(0, TextAlign::Center);
        assert_eq!(get_text_align(0), TextAlign::Center);
    }

    #[test]
    fn test_text_wrap() {
        setup();

        assert_eq!(get_text_wrap(0), TextWrap::Wrap);

        set_text_wrap(0, TextWrap::Truncate);
        assert_eq!(get_text_wrap(0), TextWrap::Truncate);
    }
}
