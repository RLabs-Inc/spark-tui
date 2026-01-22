//! TUI Framework - Interaction Arrays
//!
//! User interaction state:
//! - focusable: Can receive focus
//! - tabIndex: Tab order
//! - scrollOffset: Scroll position
//! - hovered/pressed: Mouse state
//! - cursorPosition: Input cursor position
//!
//! Uses `TrackedSlotArray` for stable reactive cells with fine-grained tracking.

use spark_signals::TrackedSlotArray;

// =============================================================================
// Arrays
// =============================================================================

thread_local! {
    /// Is component focusable.
    static FOCUSABLE: TrackedSlotArray<bool> = TrackedSlotArray::new(Some(false));

    /// Tab index for focus navigation (higher = later in order).
    static TAB_INDEX: TrackedSlotArray<i32> = TrackedSlotArray::new(Some(0));

    /// Scroll offset X.
    static SCROLL_OFFSET_X: TrackedSlotArray<u16> = TrackedSlotArray::new(Some(0));

    /// Scroll offset Y.
    static SCROLL_OFFSET_Y: TrackedSlotArray<u16> = TrackedSlotArray::new(Some(0));

    /// Is component hovered by mouse.
    static HOVERED: TrackedSlotArray<bool> = TrackedSlotArray::new(Some(false));

    /// Is component pressed (mouse down).
    static PRESSED: TrackedSlotArray<bool> = TrackedSlotArray::new(Some(false));

    /// Is mouse enabled for this component.
    static MOUSE_ENABLED: TrackedSlotArray<bool> = TrackedSlotArray::new(Some(true));

    /// Cursor position (for input components).
    static CURSOR_POSITION: TrackedSlotArray<u16> = TrackedSlotArray::new(Some(0));

    /// Selection start (for input components).
    static SELECTION_START: TrackedSlotArray<u16> = TrackedSlotArray::new(Some(0));

    /// Selection end (for input components).
    static SELECTION_END: TrackedSlotArray<u16> = TrackedSlotArray::new(Some(0));

    /// Cursor visible flag.
    static CURSOR_VISIBLE: TrackedSlotArray<bool> = TrackedSlotArray::new(Some(true));

    /// Cursor blink FPS (0 = no blink).
    static CURSOR_BLINK_FPS: TrackedSlotArray<u8> = TrackedSlotArray::new(Some(2));
}

// =============================================================================
// Capacity Management
// =============================================================================

/// Ensure arrays have capacity for the given index.
pub fn ensure_capacity(index: usize) {
    FOCUSABLE.with(|arr| { let _ = arr.peek(index); });
    TAB_INDEX.with(|arr| { let _ = arr.peek(index); });
    SCROLL_OFFSET_X.with(|arr| { let _ = arr.peek(index); });
    SCROLL_OFFSET_Y.with(|arr| { let _ = arr.peek(index); });
    HOVERED.with(|arr| { let _ = arr.peek(index); });
    PRESSED.with(|arr| { let _ = arr.peek(index); });
    MOUSE_ENABLED.with(|arr| { let _ = arr.peek(index); });
    CURSOR_POSITION.with(|arr| { let _ = arr.peek(index); });
    SELECTION_START.with(|arr| { let _ = arr.peek(index); });
    SELECTION_END.with(|arr| { let _ = arr.peek(index); });
    CURSOR_VISIBLE.with(|arr| { let _ = arr.peek(index); });
    CURSOR_BLINK_FPS.with(|arr| { let _ = arr.peek(index); });
}

/// Clear values at index.
pub fn clear_at_index(index: usize) {
    FOCUSABLE.with(|arr| arr.clear(index));
    TAB_INDEX.with(|arr| arr.clear(index));
    SCROLL_OFFSET_X.with(|arr| arr.clear(index));
    SCROLL_OFFSET_Y.with(|arr| arr.clear(index));
    HOVERED.with(|arr| arr.clear(index));
    PRESSED.with(|arr| arr.clear(index));
    MOUSE_ENABLED.with(|arr| arr.clear(index));
    CURSOR_POSITION.with(|arr| arr.clear(index));
    SELECTION_START.with(|arr| arr.clear(index));
    SELECTION_END.with(|arr| arr.clear(index));
    CURSOR_VISIBLE.with(|arr| arr.clear(index));
    CURSOR_BLINK_FPS.with(|arr| arr.clear(index));
}

/// Reset all arrays.
pub fn reset() {
    FOCUSABLE.with(|arr| arr.clear_all());
    TAB_INDEX.with(|arr| arr.clear_all());
    SCROLL_OFFSET_X.with(|arr| arr.clear_all());
    SCROLL_OFFSET_Y.with(|arr| arr.clear_all());
    HOVERED.with(|arr| arr.clear_all());
    PRESSED.with(|arr| arr.clear_all());
    MOUSE_ENABLED.with(|arr| arr.clear_all());
    CURSOR_POSITION.with(|arr| arr.clear_all());
    SELECTION_START.with(|arr| arr.clear_all());
    SELECTION_END.with(|arr| arr.clear_all());
    CURSOR_VISIBLE.with(|arr| arr.clear_all());
    CURSOR_BLINK_FPS.with(|arr| arr.clear_all());
}

// =============================================================================
// Focusable
// =============================================================================

/// Get focusable at index (reactive).
pub fn get_focusable(index: usize) -> bool {
    FOCUSABLE.with(|arr| arr.get(index))
}

/// Set focusable at index.
pub fn set_focusable(index: usize, focusable: bool) {
    FOCUSABLE.with(|arr| arr.set_value(index, focusable));
}

// =============================================================================
// Tab Index
// =============================================================================

/// Get tab index at index (reactive).
pub fn get_tab_index(index: usize) -> i32 {
    TAB_INDEX.with(|arr| arr.get(index))
}

/// Set tab index at index.
pub fn set_tab_index(index: usize, tab_index: i32) {
    TAB_INDEX.with(|arr| arr.set_value(index, tab_index));
}

// =============================================================================
// Scroll Offset
// =============================================================================

/// Get scroll offset X at index (reactive).
pub fn get_scroll_offset_x(index: usize) -> u16 {
    SCROLL_OFFSET_X.with(|arr| arr.get(index))
}

/// Get scroll offset Y at index (reactive).
pub fn get_scroll_offset_y(index: usize) -> u16 {
    SCROLL_OFFSET_Y.with(|arr| arr.get(index))
}

/// Set scroll offset at index.
pub fn set_scroll_offset(index: usize, x: u16, y: u16) {
    SCROLL_OFFSET_X.with(|arr| arr.set_value(index, x));
    SCROLL_OFFSET_Y.with(|arr| arr.set_value(index, y));
}

// =============================================================================
// Hovered
// =============================================================================

/// Get hovered state at index (reactive).
pub fn get_hovered(index: usize) -> bool {
    HOVERED.with(|arr| arr.get(index))
}

/// Set hovered state at index.
pub fn set_hovered(index: usize, hovered: bool) {
    HOVERED.with(|arr| arr.set_value(index, hovered));
}

// =============================================================================
// Pressed
// =============================================================================

/// Get pressed state at index (reactive).
pub fn get_pressed(index: usize) -> bool {
    PRESSED.with(|arr| arr.get(index))
}

/// Set pressed state at index.
pub fn set_pressed(index: usize, pressed: bool) {
    PRESSED.with(|arr| arr.set_value(index, pressed));
}

// =============================================================================
// Mouse Enabled
// =============================================================================

/// Get mouse enabled at index (reactive).
pub fn get_mouse_enabled(index: usize) -> bool {
    MOUSE_ENABLED.with(|arr| arr.get(index))
}

/// Set mouse enabled at index.
pub fn set_mouse_enabled(index: usize, enabled: bool) {
    MOUSE_ENABLED.with(|arr| arr.set_value(index, enabled));
}

// =============================================================================
// Cursor Position
// =============================================================================

/// Get cursor position at index (reactive).
pub fn get_cursor_position(index: usize) -> u16 {
    CURSOR_POSITION.with(|arr| arr.get(index))
}

/// Set cursor position at index.
pub fn set_cursor_position(index: usize, pos: u16) {
    CURSOR_POSITION.with(|arr| arr.set_value(index, pos));
}

/// Set cursor position from a getter function.
pub fn set_cursor_position_getter<F>(index: usize, getter: F)
where
    F: Fn() -> u16 + 'static,
{
    CURSOR_POSITION.with(|arr| arr.set_getter(index, getter));
}

// =============================================================================
// Cursor Visible
// =============================================================================

/// Get cursor visible at index (reactive).
pub fn get_cursor_visible(index: usize) -> bool {
    CURSOR_VISIBLE.with(|arr| arr.get(index))
}

/// Set cursor visible at index.
pub fn set_cursor_visible(index: usize, visible: bool) {
    CURSOR_VISIBLE.with(|arr| arr.set_value(index, visible));
}

/// Set cursor visible from a getter function.
pub fn set_cursor_visible_getter<F>(index: usize, getter: F)
where
    F: Fn() -> bool + 'static,
{
    CURSOR_VISIBLE.with(|arr| arr.set_getter(index, getter));
}

// =============================================================================
// Cursor Blink FPS
// =============================================================================

/// Get cursor blink FPS at index (reactive).
pub fn get_cursor_blink_fps(index: usize) -> u8 {
    CURSOR_BLINK_FPS.with(|arr| arr.get(index))
}

/// Set cursor blink FPS at index.
pub fn set_cursor_blink_fps(index: usize, fps: u8) {
    CURSOR_BLINK_FPS.with(|arr| arr.set_value(index, fps));
}

// =============================================================================
// Selection
// =============================================================================

/// Get selection start at index (reactive).
pub fn get_selection_start(index: usize) -> u16 {
    SELECTION_START.with(|arr| arr.get(index))
}

/// Set selection start at index.
pub fn set_selection_start(index: usize, pos: u16) {
    SELECTION_START.with(|arr| arr.set_value(index, pos));
}

/// Get selection end at index (reactive).
pub fn get_selection_end(index: usize) -> u16 {
    SELECTION_END.with(|arr| arr.get(index))
}

/// Set selection end at index.
pub fn set_selection_end(index: usize, pos: u16) {
    SELECTION_END.with(|arr| arr.set_value(index, pos));
}

/// Set selection range at index (convenience function).
pub fn set_selection(index: usize, start: u16, end: u16) {
    SELECTION_START.with(|arr| arr.set_value(index, start));
    SELECTION_END.with(|arr| arr.set_value(index, end));
}

/// Clear selection at index (set start == end).
pub fn clear_selection(index: usize) {
    set_selection(index, 0, 0);
}

/// Check if there is an active selection at index.
pub fn has_selection(index: usize) -> bool {
    let start = get_selection_start(index);
    let end = get_selection_end(index);
    start != end
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() {
        reset();
    }

    #[test]
    fn test_focusable() {
        setup();

        assert!(!get_focusable(0));

        set_focusable(0, true);
        assert!(get_focusable(0));
    }

    #[test]
    fn test_tab_index() {
        setup();

        assert_eq!(get_tab_index(0), 0);

        set_tab_index(0, 5);
        assert_eq!(get_tab_index(0), 5);
    }

    #[test]
    fn test_scroll_offset() {
        setup();

        assert_eq!(get_scroll_offset_x(0), 0);
        assert_eq!(get_scroll_offset_y(0), 0);

        set_scroll_offset(0, 10, 20);
        assert_eq!(get_scroll_offset_x(0), 10);
        assert_eq!(get_scroll_offset_y(0), 20);
    }

    #[test]
    fn test_hover_press() {
        setup();

        assert!(!get_hovered(0));
        assert!(!get_pressed(0));

        set_hovered(0, true);
        set_pressed(0, true);

        assert!(get_hovered(0));
        assert!(get_pressed(0));
    }

    #[test]
    fn test_cursor() {
        setup();

        assert_eq!(get_cursor_position(0), 0);
        assert!(get_cursor_visible(0));
        assert_eq!(get_cursor_blink_fps(0), 2);

        set_cursor_position(0, 5);
        set_cursor_visible(0, false);
        set_cursor_blink_fps(0, 4);

        assert_eq!(get_cursor_position(0), 5);
        assert!(!get_cursor_visible(0));
        assert_eq!(get_cursor_blink_fps(0), 4);
    }
}
