//! TUI Framework - Visual Arrays
//!
//! Visual properties for rendering:
//! - fgColor, bgColor: Foreground and background colors
//! - opacity: Component opacity
//! - borderStyle: Border style enum
//! - borderColor: Border color
//! - zIndex: Stacking order
//!
//! Uses `TrackedSlotArray` for stable reactive cells with fine-grained tracking.

use spark_signals::{TrackedSlotArray, tracked_slot_array};
use crate::types::{Rgba, BorderStyle};
use super::dirty::VISUAL_DIRTY_SET;
use crate::engine::arrays::ClearAll;

// =============================================================================
// Arrays
// =============================================================================

thread_local! {
    /// Foreground color (text color).
    static FG_COLOR: TrackedSlotArray<Rgba> = tracked_slot_array(
        Some(Rgba::TERMINAL_DEFAULT),
        VISUAL_DIRTY_SET.with(|s| s.clone())
    );

    /// Background color.
    static BG_COLOR: TrackedSlotArray<Rgba> = tracked_slot_array(
        Some(Rgba::TERMINAL_DEFAULT),
        VISUAL_DIRTY_SET.with(|s| s.clone())
    );

    /// Opacity (0-255, 255 = fully opaque).
    static OPACITY: TrackedSlotArray<u8> = tracked_slot_array(
        Some(255),
        VISUAL_DIRTY_SET.with(|s| s.clone())
    );

    /// Border style.
    static BORDER_STYLE: TrackedSlotArray<BorderStyle> = tracked_slot_array(
        Some(BorderStyle::None),
        VISUAL_DIRTY_SET.with(|s| s.clone())
    );

    /// Border color.
    static BORDER_COLOR: TrackedSlotArray<Rgba> = tracked_slot_array(
        Some(Rgba::TERMINAL_DEFAULT),
        VISUAL_DIRTY_SET.with(|s| s.clone())
    );

    /// Per-side border styles (override the main border style).
    static BORDER_TOP_STYLE: TrackedSlotArray<BorderStyle> = tracked_slot_array(
        Some(BorderStyle::None),
        VISUAL_DIRTY_SET.with(|s| s.clone())
    );
    static BORDER_RIGHT_STYLE: TrackedSlotArray<BorderStyle> = tracked_slot_array(
        Some(BorderStyle::None),
        VISUAL_DIRTY_SET.with(|s| s.clone())
    );
    static BORDER_BOTTOM_STYLE: TrackedSlotArray<BorderStyle> = tracked_slot_array(
        Some(BorderStyle::None),
        VISUAL_DIRTY_SET.with(|s| s.clone())
    );
    static BORDER_LEFT_STYLE: TrackedSlotArray<BorderStyle> = tracked_slot_array(
        Some(BorderStyle::None),
        VISUAL_DIRTY_SET.with(|s| s.clone())
    );

    /// Z-index for stacking order.
    static Z_INDEX: TrackedSlotArray<i32> = tracked_slot_array(
        Some(0),
        VISUAL_DIRTY_SET.with(|s| s.clone())
    );
}

// =============================================================================
// Capacity Management
// =============================================================================

/// Ensure arrays have capacity for the given index.
pub fn ensure_capacity(index: usize) {
    // TrackedSlotArray auto-expands on access
    FG_COLOR.with(|arr| { let _ = arr.peek(index); });
    BG_COLOR.with(|arr| { let _ = arr.peek(index); });
    OPACITY.with(|arr| { let _ = arr.peek(index); });
    BORDER_STYLE.with(|arr| { let _ = arr.peek(index); });
    BORDER_COLOR.with(|arr| { let _ = arr.peek(index); });
    BORDER_TOP_STYLE.with(|arr| { let _ = arr.peek(index); });
    BORDER_RIGHT_STYLE.with(|arr| { let _ = arr.peek(index); });
    BORDER_BOTTOM_STYLE.with(|arr| { let _ = arr.peek(index); });
    BORDER_LEFT_STYLE.with(|arr| { let _ = arr.peek(index); });
    Z_INDEX.with(|arr| { let _ = arr.peek(index); });
}

/// Clear values at index.
pub fn clear_at_index(index: usize) {
    FG_COLOR.with(|arr| arr.clear(index));
    BG_COLOR.with(|arr| arr.clear(index));
    OPACITY.with(|arr| arr.clear(index));
    BORDER_STYLE.with(|arr| arr.clear(index));
    BORDER_COLOR.with(|arr| arr.clear(index));
    BORDER_TOP_STYLE.with(|arr| arr.clear(index));
    BORDER_RIGHT_STYLE.with(|arr| arr.clear(index));
    BORDER_BOTTOM_STYLE.with(|arr| arr.clear(index));
    BORDER_LEFT_STYLE.with(|arr| arr.clear(index));
    Z_INDEX.with(|arr| arr.clear(index));
}

/// Reset all arrays.
pub fn reset() {
    FG_COLOR.with(|arr| arr.clear_all());
    BG_COLOR.with(|arr| arr.clear_all());
    OPACITY.with(|arr| arr.clear_all());
    BORDER_STYLE.with(|arr| arr.clear_all());
    BORDER_COLOR.with(|arr| arr.clear_all());
    BORDER_TOP_STYLE.with(|arr| arr.clear_all());
    BORDER_RIGHT_STYLE.with(|arr| arr.clear_all());
    BORDER_BOTTOM_STYLE.with(|arr| arr.clear_all());
    BORDER_LEFT_STYLE.with(|arr| arr.clear_all());
    Z_INDEX.with(|arr| arr.clear_all());
}

// =============================================================================
// Foreground Color
// =============================================================================

/// Get foreground color at index (reactive).
pub fn get_fg_color(index: usize) -> Rgba {
    FG_COLOR.with(|arr| arr.get(index)).unwrap_or(Rgba::TERMINAL_DEFAULT)
}

/// Set foreground color at index.
pub fn set_fg_color(index: usize, color: Rgba) {
    FG_COLOR.with(|arr| arr.set_value(index, color));
}

/// Set foreground color from a getter function.
pub fn set_fg_color_getter<F>(index: usize, getter: F)
where
    F: Fn() -> Rgba + 'static,
{
    FG_COLOR.with(|arr| arr.set_getter(index, getter));
}

// =============================================================================
// Background Color
// =============================================================================

/// Get background color at index (reactive).
pub fn get_bg_color(index: usize) -> Rgba {
    BG_COLOR.with(|arr| arr.get(index)).unwrap_or(Rgba::TERMINAL_DEFAULT)
}

/// Set background color at index.
pub fn set_bg_color(index: usize, color: Rgba) {
    BG_COLOR.with(|arr| arr.set_value(index, color));
}

/// Set background color from a getter function.
pub fn set_bg_color_getter<F>(index: usize, getter: F)
where
    F: Fn() -> Rgba + 'static,
{
    BG_COLOR.with(|arr| arr.set_getter(index, getter));
}

// =============================================================================
// Opacity
// =============================================================================

/// Get opacity at index (reactive).
pub fn get_opacity(index: usize) -> u8 {
    OPACITY.with(|arr| arr.get(index)).unwrap_or(255)
}

/// Set opacity at index.
pub fn set_opacity(index: usize, opacity: u8) {
    OPACITY.with(|arr| arr.set_value(index, opacity));
}

// =============================================================================
// Border Style
// =============================================================================

/// Get border style at index (reactive).
pub fn get_border_style(index: usize) -> BorderStyle {
    BORDER_STYLE.with(|arr| arr.get(index)).unwrap_or(BorderStyle::None)
}

/// Set border style at index.
pub fn set_border_style(index: usize, style: BorderStyle) {
    BORDER_STYLE.with(|arr| arr.set_value(index, style));
}

/// Set border style from a getter function.
pub fn set_border_style_getter<F>(index: usize, getter: F)
where
    F: Fn() -> BorderStyle + 'static,
{
    BORDER_STYLE.with(|arr| arr.set_getter(index, getter));
}

// =============================================================================
// Border Color
// =============================================================================

/// Get border color at index (reactive).
pub fn get_border_color(index: usize) -> Rgba {
    BORDER_COLOR.with(|arr| arr.get(index)).unwrap_or(Rgba::TERMINAL_DEFAULT)
}

/// Set border color at index.
pub fn set_border_color(index: usize, color: Rgba) {
    BORDER_COLOR.with(|arr| arr.set_value(index, color));
}

/// Set border color from a getter function.
pub fn set_border_color_getter<F>(index: usize, getter: F)
where
    F: Fn() -> Rgba + 'static,
{
    BORDER_COLOR.with(|arr| arr.set_getter(index, getter));
}

// =============================================================================
// Z-Index
// =============================================================================

/// Get z-index at index (reactive).
pub fn get_z_index(index: usize) -> i32 {
    Z_INDEX.with(|arr| arr.get(index)).unwrap_or(0)
}

/// Set z-index at index.
pub fn set_z_index(index: usize, z: i32) {
    Z_INDEX.with(|arr| arr.set_value(index, z));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() {
        reset();
    }

    #[test]
    fn test_fg_color() {
        setup();

        assert!(get_fg_color(0).is_terminal_default());

        let red = Rgba::rgb(255, 0, 0);
        set_fg_color(0, red);
        assert_eq!(get_fg_color(0), red);
    }

    #[test]
    fn test_border_style() {
        setup();

        assert_eq!(get_border_style(0), BorderStyle::None);

        set_border_style(0, BorderStyle::Single);
        assert_eq!(get_border_style(0), BorderStyle::Single);
    }

    #[test]
    fn test_z_index() {
        setup();

        assert_eq!(get_z_index(0), 0);

        set_z_index(0, 10);
        assert_eq!(get_z_index(0), 10);

        set_z_index(1, -5);
        assert_eq!(get_z_index(1), -5);
    }
}
