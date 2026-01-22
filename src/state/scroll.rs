//! TUI Framework - Scroll State Module
//!
//! Manages scrolling behavior:
//! - Per-component scroll offset (user state via interaction arrays)
//! - Scroll bounds from layout (computed by TITAN)
//! - Scroll operations with clamping
//! - Parent chaining for mouse wheel
//!
//! Architecture:
//! - scrollOffsetX/Y = user state (interaction arrays)
//! - scrollable/maxScrollX/Y = computed by TITAN (read from ComputedLayout)

use crate::engine::arrays::{core, interaction};
use crate::layout::ComputedLayout;

// =============================================================================
// SCROLL CONSTANTS
// =============================================================================

/// Default scroll amount for arrow keys (lines).
pub const LINE_SCROLL: u16 = 1;

/// Default scroll amount for mouse wheel.
pub const WHEEL_SCROLL: u16 = 3;

/// Default scroll amount for Page Up/Down (90% of viewport).
pub const PAGE_SCROLL_FACTOR: f32 = 0.9;

// =============================================================================
// SCROLL STATE ACCESS
// =============================================================================

/// Check if a component is scrollable (reads from computed layout).
pub fn is_scrollable(layout: &ComputedLayout, index: usize) -> bool {
    layout.scrollable.get(index).copied().unwrap_or(0) == 1
}

/// Get current scroll offset for a component (user state).
///
/// Returns (x, y) scroll offset.
pub fn get_scroll_offset(index: usize) -> (u16, u16) {
    (
        interaction::get_scroll_offset_x(index),
        interaction::get_scroll_offset_y(index),
    )
}

/// Get maximum scroll values for a component (reads from computed layout).
///
/// Returns (max_scroll_x, max_scroll_y).
pub fn get_max_scroll(layout: &ComputedLayout, index: usize) -> (u16, u16) {
    (
        layout.max_scroll_x.get(index).copied().unwrap_or(0),
        layout.max_scroll_y.get(index).copied().unwrap_or(0),
    )
}

// =============================================================================
// SCROLL OPERATIONS
// =============================================================================

/// Set scroll offset for a component (clamped to valid range).
///
/// Does nothing if the component is not scrollable.
pub fn set_scroll_offset(layout: &ComputedLayout, index: usize, x: u16, y: u16) {
    if !is_scrollable(layout, index) {
        return;
    }

    let (max_x, max_y) = get_max_scroll(layout, index);

    // Clamp values
    let clamped_x = x.min(max_x);
    let clamped_y = y.min(max_y);

    interaction::set_scroll_offset(index, clamped_x, clamped_y);
}

/// Scroll by a delta amount.
///
/// Returns `true` if scrolling occurred, `false` if already at boundary.
pub fn scroll_by(layout: &ComputedLayout, index: usize, delta_x: i32, delta_y: i32) -> bool {
    if !is_scrollable(layout, index) {
        return false;
    }

    let (current_x, current_y) = get_scroll_offset(index);
    let (max_x, max_y) = get_max_scroll(layout, index);

    // Compute new values with clamping (using i32 to handle negative deltas)
    let new_x = ((current_x as i32) + delta_x).clamp(0, max_x as i32) as u16;
    let new_y = ((current_y as i32) + delta_y).clamp(0, max_y as i32) as u16;

    // Check if we actually scrolled
    if new_x == current_x && new_y == current_y {
        return false; // Already at boundary
    }

    interaction::set_scroll_offset(index, new_x, new_y);
    true
}

/// Scroll to top (set Y offset to 0, preserve X).
pub fn scroll_to_top(layout: &ComputedLayout, index: usize) {
    let (current_x, _) = get_scroll_offset(index);
    set_scroll_offset(layout, index, current_x, 0);
}

/// Scroll to bottom (set Y offset to max, preserve X).
pub fn scroll_to_bottom(layout: &ComputedLayout, index: usize) {
    let (current_x, _) = get_scroll_offset(index);
    let (_, max_y) = get_max_scroll(layout, index);
    set_scroll_offset(layout, index, current_x, max_y);
}

/// Scroll to start (set X offset to 0, preserve Y).
pub fn scroll_to_start(layout: &ComputedLayout, index: usize) {
    let (_, current_y) = get_scroll_offset(index);
    set_scroll_offset(layout, index, 0, current_y);
}

/// Scroll to end (set X offset to max, preserve Y).
pub fn scroll_to_end(layout: &ComputedLayout, index: usize) {
    let (_, current_y) = get_scroll_offset(index);
    let (max_x, _) = get_max_scroll(layout, index);
    set_scroll_offset(layout, index, max_x, current_y);
}

// =============================================================================
// SCROLL CHAINING
// =============================================================================

/// Scroll with parent chaining (for mouse wheel).
///
/// If the target component is at its scroll boundary, tries to scroll
/// the parent instead. Returns `true` if any scrolling occurred.
pub fn scroll_by_with_chaining(
    layout: &ComputedLayout,
    index: usize,
    delta_x: i32,
    delta_y: i32,
) -> bool {
    // Try to scroll this component
    if scroll_by(layout, index, delta_x, delta_y) {
        return true;
    }

    // At boundary - try parent
    if let Some(parent_idx) = core::get_parent_index(index) {
        if is_scrollable(layout, parent_idx) {
            return scroll_by_with_chaining(layout, parent_idx, delta_x, delta_y);
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::arrays::interaction;

    fn create_test_layout(scrollable_indices: &[(usize, u16, u16)]) -> ComputedLayout {
        let max_idx = scrollable_indices
            .iter()
            .map(|(i, _, _)| *i)
            .max()
            .unwrap_or(0);
        let size = max_idx + 1;

        let mut layout = ComputedLayout {
            x: vec![0; size],
            y: vec![0; size],
            width: vec![0; size],
            height: vec![0; size],
            scrollable: vec![0; size],
            max_scroll_x: vec![0; size],
            max_scroll_y: vec![0; size],
            content_width: 0,
            content_height: 0,
        };

        for &(idx, max_x, max_y) in scrollable_indices {
            layout.scrollable[idx] = 1;
            layout.max_scroll_x[idx] = max_x;
            layout.max_scroll_y[idx] = max_y;
        }

        layout
    }

    fn setup() {
        interaction::reset();
    }

    #[test]
    fn test_is_scrollable() {
        setup();

        let layout = create_test_layout(&[(0, 10, 20)]);

        assert!(is_scrollable(&layout, 0));
        assert!(!is_scrollable(&layout, 1)); // Not in scrollable list
    }

    #[test]
    fn test_is_scrollable_empty_layout() {
        setup();

        let layout = ComputedLayout::new();

        assert!(!is_scrollable(&layout, 0));
        assert!(!is_scrollable(&layout, 999));
    }

    #[test]
    fn test_get_scroll_offset() {
        setup();

        // Initially zero
        assert_eq!(get_scroll_offset(0), (0, 0));

        // Set via interaction arrays
        interaction::set_scroll_offset(0, 5, 10);
        assert_eq!(get_scroll_offset(0), (5, 10));
    }

    #[test]
    fn test_get_max_scroll() {
        setup();

        let layout = create_test_layout(&[(0, 100, 200)]);

        assert_eq!(get_max_scroll(&layout, 0), (100, 200));
        assert_eq!(get_max_scroll(&layout, 999), (0, 0)); // Out of bounds
    }

    #[test]
    fn test_set_scroll_offset_clamps() {
        setup();

        let layout = create_test_layout(&[(0, 10, 20)]);

        // Within range
        set_scroll_offset(&layout, 0, 5, 10);
        assert_eq!(get_scroll_offset(0), (5, 10));

        // Exceeds max - should clamp
        set_scroll_offset(&layout, 0, 100, 200);
        assert_eq!(get_scroll_offset(0), (10, 20));

        // Zero is always valid
        set_scroll_offset(&layout, 0, 0, 0);
        assert_eq!(get_scroll_offset(0), (0, 0));
    }

    #[test]
    fn test_set_scroll_offset_not_scrollable() {
        setup();

        let layout = create_test_layout(&[(0, 10, 20)]);

        // Index 1 is not scrollable
        interaction::set_scroll_offset(1, 5, 10);
        set_scroll_offset(&layout, 1, 99, 99);

        // Should not change because not scrollable
        assert_eq!(get_scroll_offset(1), (5, 10));
    }

    #[test]
    fn test_scroll_by_returns_bool() {
        setup();

        let layout = create_test_layout(&[(0, 10, 20)]);

        // Initial scroll should succeed
        assert!(scroll_by(&layout, 0, 5, 5));
        assert_eq!(get_scroll_offset(0), (5, 5));

        // Scroll more
        assert!(scroll_by(&layout, 0, 3, 3));
        assert_eq!(get_scroll_offset(0), (8, 8));

        // Scroll to boundary
        assert!(scroll_by(&layout, 0, 10, 20));
        assert_eq!(get_scroll_offset(0), (10, 20));

        // At boundary - should return false
        assert!(!scroll_by(&layout, 0, 1, 1));
        assert_eq!(get_scroll_offset(0), (10, 20)); // Unchanged
    }

    #[test]
    fn test_scroll_by_negative() {
        setup();

        let layout = create_test_layout(&[(0, 10, 20)]);

        // Start at middle
        interaction::set_scroll_offset(0, 5, 10);

        // Scroll up (negative)
        assert!(scroll_by(&layout, 0, -3, -5));
        assert_eq!(get_scroll_offset(0), (2, 5));

        // Scroll to zero boundary
        assert!(scroll_by(&layout, 0, -10, -10));
        assert_eq!(get_scroll_offset(0), (0, 0));

        // At boundary - should return false
        assert!(!scroll_by(&layout, 0, -1, -1));
        assert_eq!(get_scroll_offset(0), (0, 0));
    }

    #[test]
    fn test_scroll_by_not_scrollable() {
        setup();

        let layout = create_test_layout(&[(0, 10, 20)]);

        // Index 1 is not scrollable
        assert!(!scroll_by(&layout, 1, 5, 5));
    }

    #[test]
    fn test_scroll_to_top_bottom() {
        setup();

        let layout = create_test_layout(&[(0, 10, 20)]);

        // Start at middle
        interaction::set_scroll_offset(0, 5, 10);

        // Scroll to bottom
        scroll_to_bottom(&layout, 0);
        assert_eq!(get_scroll_offset(0), (5, 20)); // X preserved, Y at max

        // Scroll to top
        scroll_to_top(&layout, 0);
        assert_eq!(get_scroll_offset(0), (5, 0)); // X preserved, Y at 0
    }

    #[test]
    fn test_scroll_to_start_end() {
        setup();

        let layout = create_test_layout(&[(0, 10, 20)]);

        // Start at middle
        interaction::set_scroll_offset(0, 5, 10);

        // Scroll to end
        scroll_to_end(&layout, 0);
        assert_eq!(get_scroll_offset(0), (10, 10)); // X at max, Y preserved

        // Scroll to start
        scroll_to_start(&layout, 0);
        assert_eq!(get_scroll_offset(0), (0, 10)); // X at 0, Y preserved
    }

    #[test]
    fn test_scroll_by_with_chaining() {
        setup();

        // Parent (index 0) is scrollable
        // Child (index 1) is scrollable
        let layout = create_test_layout(&[(0, 10, 20), (1, 5, 10)]);

        // Set up parent-child relationship
        crate::engine::arrays::core::set_parent_index(1, Some(0));

        // Child at middle
        interaction::set_scroll_offset(1, 2, 5);

        // Scroll child - should succeed
        assert!(scroll_by_with_chaining(&layout, 1, 1, 1));
        assert_eq!(get_scroll_offset(1), (3, 6));

        // Scroll child to boundary
        assert!(scroll_by_with_chaining(&layout, 1, 10, 10));
        assert_eq!(get_scroll_offset(1), (5, 10)); // At child max

        // Child at boundary - should chain to parent
        assert!(scroll_by_with_chaining(&layout, 1, 1, 1));
        assert_eq!(get_scroll_offset(1), (5, 10)); // Child unchanged
        assert_eq!(get_scroll_offset(0), (1, 1)); // Parent scrolled
    }

    #[test]
    fn test_scroll_by_with_chaining_no_parent() {
        setup();

        let layout = create_test_layout(&[(0, 10, 20)]);

        // Scroll to boundary
        interaction::set_scroll_offset(0, 10, 20);

        // No parent to chain to
        assert!(!scroll_by_with_chaining(&layout, 0, 1, 1));
    }

    #[test]
    fn test_constants() {
        assert_eq!(LINE_SCROLL, 1);
        assert_eq!(WHEEL_SCROLL, 3);
        assert!((PAGE_SCROLL_FACTOR - 0.9).abs() < 0.001);
    }
}
