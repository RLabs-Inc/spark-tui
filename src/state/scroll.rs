//! TUI Framework - Scroll State Module
//!
//! Manages scrolling behavior:
//! - Per-component scroll offset (user state via interaction arrays)
//! - Scroll bounds from layout (computed by TITAN)
//! - Scroll operations with clamping
//! - Parent chaining for mouse wheel
//! - Keyboard scroll handlers (arrow, page, home/end)
//!
//! Architecture:
//! - scrollOffsetX/Y = user state (interaction arrays)
//! - scrollable/maxScrollX/Y = computed by TITAN (read from ComputedLayout)
//!
//! # Example
//!
//! ```ignore
//! use spark_tui::state::scroll;
//! use spark_tui::layout::ComputedLayout;
//! use spark_tui::state::mouse::ScrollDirection;
//!
//! // Check if component is scrollable
//! let scrollable = scroll::is_scrollable(&layout, index);
//!
//! // Scroll by delta (returns true if scrolled)
//! let scrolled = scroll::scroll_by(&layout, index, 0, 1);
//!
//! // Keyboard scroll handlers
//! scroll::handle_arrow_scroll(&layout, ScrollDirection::Down);
//! scroll::handle_page_scroll(&layout, ScrollDirection::Up);
//! scroll::handle_home_end(&layout, true); // to_top
//! ```

use crate::engine::arrays::{core, interaction};
use crate::layout::ComputedLayout;
use crate::state::focus;
use crate::state::mouse::ScrollDirection;

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

// =============================================================================
// FIND SCROLLABLE
// =============================================================================

/// Get the focused component index if it's scrollable, else -1.
pub fn get_focused_scrollable(layout: &ComputedLayout) -> i32 {
    let focused = focus::get_focused_index();
    if focused >= 0 && is_scrollable(layout, focused as usize) {
        focused
    } else {
        -1
    }
}

// =============================================================================
// KEYBOARD SCROLL HANDLERS
// =============================================================================

/// Handle arrow key scroll for focused scrollable.
/// Returns true if scroll occurred.
///
/// Note: Keyboard scroll does NOT chain (per design decision -
/// would conflict with focus management).
pub fn handle_arrow_scroll(layout: &ComputedLayout, direction: ScrollDirection) -> bool {
    let scrollable = get_focused_scrollable(layout);
    if scrollable < 0 {
        return false;
    }

    let idx = scrollable as usize;
    match direction {
        ScrollDirection::Up => scroll_by(layout, idx, 0, -(LINE_SCROLL as i32)),
        ScrollDirection::Down => scroll_by(layout, idx, 0, LINE_SCROLL as i32),
        ScrollDirection::Left => scroll_by(layout, idx, -(LINE_SCROLL as i32), 0),
        ScrollDirection::Right => scroll_by(layout, idx, LINE_SCROLL as i32, 0),
    }
}

/// Handle PageUp/PageDown scroll.
/// Returns true if scroll occurred.
pub fn handle_page_scroll(layout: &ComputedLayout, direction: ScrollDirection) -> bool {
    let scrollable = get_focused_scrollable(layout);
    if scrollable < 0 {
        return false;
    }

    let idx = scrollable as usize;
    let viewport_height = layout.height.get(idx).copied().unwrap_or(10) as f32;
    let page_amount = (viewport_height * PAGE_SCROLL_FACTOR).max(1.0) as i32;

    match direction {
        ScrollDirection::Up => scroll_by(layout, idx, 0, -page_amount),
        ScrollDirection::Down => scroll_by(layout, idx, 0, page_amount),
        _ => false, // No horizontal page scroll
    }
}

/// Handle Ctrl+Home/End for scroll to top/bottom.
/// Returns true if action performed.
pub fn handle_home_end(layout: &ComputedLayout, to_top: bool) -> bool {
    let scrollable = get_focused_scrollable(layout);
    if scrollable < 0 {
        return false;
    }

    let idx = scrollable as usize;
    if to_top {
        scroll_to_top(layout, idx);
    } else {
        scroll_to_bottom(layout, idx);
    }
    true
}

// =============================================================================
// MOUSE WHEEL HANDLER
// =============================================================================

/// Handle mouse wheel scroll at coordinates.
/// First tries element under cursor, then falls back to focused scrollable.
/// Uses chaining for mouse wheel (unlike keyboard scroll).
pub fn handle_wheel_scroll(
    layout: &ComputedLayout,
    x: u16,
    y: u16,
    direction: ScrollDirection,
) -> bool {
    use crate::state::mouse::hit_test;

    // Try element under cursor via hit test
    let component_at = hit_test(x, y);

    // If component at cursor is scrollable, scroll it (with chaining)
    if let Some(idx) = component_at {
        if is_scrollable(layout, idx) {
            let delta = match direction {
                ScrollDirection::Up => (0, -(WHEEL_SCROLL as i32)),
                ScrollDirection::Down => (0, WHEEL_SCROLL as i32),
                ScrollDirection::Left => (-(WHEEL_SCROLL as i32), 0),
                ScrollDirection::Right => (WHEEL_SCROLL as i32, 0),
            };
            return scroll_by_with_chaining(layout, idx, delta.0, delta.1);
        }
    }

    // Fallback to focused scrollable (no chaining for focused)
    let scrollable = get_focused_scrollable(layout);
    if scrollable >= 0 {
        let idx = scrollable as usize;
        match direction {
            ScrollDirection::Up => scroll_by(layout, idx, 0, -(WHEEL_SCROLL as i32)),
            ScrollDirection::Down => scroll_by(layout, idx, 0, WHEEL_SCROLL as i32),
            ScrollDirection::Left => scroll_by(layout, idx, -(WHEEL_SCROLL as i32), 0),
            ScrollDirection::Right => scroll_by(layout, idx, WHEEL_SCROLL as i32, 0),
        }
    } else {
        false
    }
}

// =============================================================================
// SCROLL INTO VIEW
// =============================================================================

/// Scroll to make a child visible within a scrollable parent.
/// This is called when focus changes to ensure focused element is visible.
pub fn scroll_into_view(
    layout: &ComputedLayout,
    scrollable_index: usize,
    child_y: u16,
    child_height: u16,
    viewport_height: u16,
) {
    if !is_scrollable(layout, scrollable_index) {
        return;
    }

    let (_, current_y) = get_scroll_offset(scrollable_index);
    let viewport_top = current_y;
    let viewport_bottom = viewport_top.saturating_add(viewport_height);

    let child_top = child_y;
    let child_bottom = child_y.saturating_add(child_height);

    // Check if child is already visible
    if child_top >= viewport_top && child_bottom <= viewport_bottom {
        return; // Already visible
    }

    let (current_x, _) = get_scroll_offset(scrollable_index);

    // Scroll to make visible (minimal scroll)
    if child_top < viewport_top {
        // Child is above viewport - scroll up
        set_scroll_offset(layout, scrollable_index, current_x, child_top);
    } else if child_bottom > viewport_bottom {
        // Child is below viewport - scroll down
        let new_y = child_bottom.saturating_sub(viewport_height);
        set_scroll_offset(layout, scrollable_index, current_x, new_y);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::arrays::interaction;
    use crate::engine::reset_registry;
    use crate::state::focus::reset_focus_state;

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
            width: vec![80; size],
            height: vec![24; size],
            scrollable: vec![0; size],
            max_scroll_x: vec![0; size],
            max_scroll_y: vec![0; size],
            content_width: 80,
            content_height: 24,
        };

        for &(idx, max_x, max_y) in scrollable_indices {
            layout.scrollable[idx] = 1;
            layout.max_scroll_x[idx] = max_x;
            layout.max_scroll_y[idx] = max_y;
        }

        layout
    }

    fn setup() {
        reset_registry();
        reset_focus_state();
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

    // =========================================================================
    // KEYBOARD SCROLL TESTS (04-02)
    // =========================================================================

    #[test]
    fn test_get_focused_scrollable() {
        setup();
        use crate::primitives::{box_primitive, BoxProps};

        // No focus
        let layout = create_test_layout(&[(0, 10, 50)]);
        assert_eq!(get_focused_scrollable(&layout), -1);

        // Focus non-scrollable
        let _cleanup = box_primitive(BoxProps {
            focusable: Some(true),
            ..Default::default()
        });
        focus::focus(0);
        let empty_layout = ComputedLayout::default();
        assert_eq!(get_focused_scrollable(&empty_layout), -1);

        // Focus scrollable
        assert_eq!(get_focused_scrollable(&layout), 0);
    }

    #[test]
    fn test_handle_arrow_scroll_down() {
        setup();
        use crate::primitives::{box_primitive, BoxProps};

        // Create focusable component
        let _cleanup = box_primitive(BoxProps {
            focusable: Some(true),
            ..Default::default()
        });

        // Focus it
        focus::focus(0);

        // Create layout where component 0 is scrollable
        let layout = create_test_layout(&[(0, 10, 50)]);

        assert!(handle_arrow_scroll(&layout, ScrollDirection::Down));
        assert_eq!(interaction::get_scroll_offset_y(0), LINE_SCROLL);
    }

    #[test]
    fn test_handle_arrow_scroll_up() {
        setup();
        use crate::primitives::{box_primitive, BoxProps};

        let _cleanup = box_primitive(BoxProps {
            focusable: Some(true),
            ..Default::default()
        });
        focus::focus(0);

        let layout = create_test_layout(&[(0, 10, 50)]);

        // Start in middle
        interaction::set_scroll_offset(0, 0, 25);

        assert!(handle_arrow_scroll(&layout, ScrollDirection::Up));
        assert_eq!(interaction::get_scroll_offset_y(0), 25 - LINE_SCROLL);
    }

    #[test]
    fn test_handle_arrow_scroll_at_boundary() {
        setup();
        use crate::primitives::{box_primitive, BoxProps};

        let _cleanup = box_primitive(BoxProps {
            focusable: Some(true),
            ..Default::default()
        });
        focus::focus(0);

        let layout = create_test_layout(&[(0, 10, 50)]);

        // Already at top
        assert!(!handle_arrow_scroll(&layout, ScrollDirection::Up));
        assert_eq!(interaction::get_scroll_offset_y(0), 0);
    }

    #[test]
    fn test_handle_arrow_scroll_horizontal() {
        setup();
        use crate::primitives::{box_primitive, BoxProps};

        let _cleanup = box_primitive(BoxProps {
            focusable: Some(true),
            ..Default::default()
        });
        focus::focus(0);

        let layout = create_test_layout(&[(0, 50, 50)]);

        // Scroll right
        assert!(handle_arrow_scroll(&layout, ScrollDirection::Right));
        assert_eq!(interaction::get_scroll_offset_x(0), LINE_SCROLL);

        // Scroll left
        assert!(handle_arrow_scroll(&layout, ScrollDirection::Left));
        assert_eq!(interaction::get_scroll_offset_x(0), 0);

        // At left boundary
        assert!(!handle_arrow_scroll(&layout, ScrollDirection::Left));
    }

    #[test]
    fn test_handle_page_scroll() {
        setup();
        use crate::primitives::{box_primitive, BoxProps};

        let _cleanup = box_primitive(BoxProps {
            focusable: Some(true),
            ..Default::default()
        });
        focus::focus(0);

        // Create layout with known height
        let mut layout = create_test_layout(&[(0, 10, 100)]);
        layout.height[0] = 10; // Viewport height

        assert!(handle_page_scroll(&layout, ScrollDirection::Down));
        // Should scroll by 10 * 0.9 = 9
        assert_eq!(interaction::get_scroll_offset_y(0), 9);
    }

    #[test]
    fn test_handle_page_scroll_up() {
        setup();
        use crate::primitives::{box_primitive, BoxProps};

        let _cleanup = box_primitive(BoxProps {
            focusable: Some(true),
            ..Default::default()
        });
        focus::focus(0);

        let mut layout = create_test_layout(&[(0, 10, 100)]);
        layout.height[0] = 10;

        // Start at position 50
        interaction::set_scroll_offset(0, 0, 50);

        assert!(handle_page_scroll(&layout, ScrollDirection::Up));
        // Should scroll up by 10 * 0.9 = 9
        assert_eq!(interaction::get_scroll_offset_y(0), 41);
    }

    #[test]
    fn test_handle_home_end() {
        setup();
        use crate::primitives::{box_primitive, BoxProps};

        let _cleanup = box_primitive(BoxProps {
            focusable: Some(true),
            ..Default::default()
        });
        focus::focus(0);

        let layout = create_test_layout(&[(0, 10, 50)]);

        // Scroll to bottom
        assert!(handle_home_end(&layout, false));
        assert_eq!(interaction::get_scroll_offset_y(0), 50);

        // Scroll to top
        assert!(handle_home_end(&layout, true));
        assert_eq!(interaction::get_scroll_offset_y(0), 0);
    }

    #[test]
    fn test_no_scroll_when_not_focused() {
        setup();

        reset_focus_state(); // Ensure no focus

        let layout = create_test_layout(&[(0, 10, 50)]);

        assert!(!handle_arrow_scroll(&layout, ScrollDirection::Down));
        assert!(!handle_page_scroll(&layout, ScrollDirection::Down));
        assert!(!handle_home_end(&layout, true));
    }

    #[test]
    fn test_no_scroll_when_focused_not_scrollable() {
        setup();
        use crate::primitives::{box_primitive, BoxProps};

        let _cleanup = box_primitive(BoxProps {
            focusable: Some(true),
            ..Default::default()
        });
        focus::focus(0);

        // Empty layout - component 0 not scrollable
        let layout = ComputedLayout::default();

        assert!(!handle_arrow_scroll(&layout, ScrollDirection::Down));
        assert!(!handle_page_scroll(&layout, ScrollDirection::Down));
        assert!(!handle_home_end(&layout, false));
    }

    #[test]
    fn test_scroll_into_view_above() {
        setup();
        let layout = create_test_layout(&[(0, 0, 100)]);

        // Viewport at y=50, height=20
        interaction::set_scroll_offset(0, 0, 50);

        // Child above viewport at y=10
        scroll_into_view(&layout, 0, 10, 5, 20);

        // Should scroll to show child (child_top = 10)
        assert_eq!(interaction::get_scroll_offset_y(0), 10);
    }

    #[test]
    fn test_scroll_into_view_below() {
        setup();
        let layout = create_test_layout(&[(0, 0, 100)]);

        // Viewport at y=0, height=20
        interaction::set_scroll_offset(0, 0, 0);

        // Child below viewport at y=30
        scroll_into_view(&layout, 0, 30, 5, 20);

        // Should scroll to show child (child_bottom - viewport_height = 35 - 20 = 15)
        assert_eq!(interaction::get_scroll_offset_y(0), 15);
    }

    #[test]
    fn test_scroll_into_view_already_visible() {
        setup();
        let layout = create_test_layout(&[(0, 0, 100)]);

        // Viewport at y=5, height=20 (shows y=5 to y=25)
        interaction::set_scroll_offset(0, 0, 5);

        // Child at y=10, height=5 (fully within viewport)
        scroll_into_view(&layout, 0, 10, 5, 20);

        // Should not change
        assert_eq!(interaction::get_scroll_offset_y(0), 5);
    }
}
