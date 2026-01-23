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
//! # Global Layout Access
//!
//! Keyboard scroll handlers use `crate::pipeline::get_layout()` to access
//! the current computed layout. This is set by the render effect in mount().
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
//! // Keyboard scroll handlers (no layout parameter - use global accessor)
//! scroll::handle_arrow_scroll(ScrollDirection::Down);
//! scroll::handle_page_scroll(ScrollDirection::Up);
//! scroll::handle_home_end(true); // to_top
//! ```

use crate::engine::arrays::{core, interaction};
use crate::layout::ComputedLayout;
use crate::pipeline::get_layout;
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
/// Uses global layout accessor `get_layout()` set by the render pipeline.
///
/// Note: Keyboard scroll does NOT chain (per design decision -
/// would conflict with focus management).
pub fn handle_arrow_scroll(direction: ScrollDirection) -> bool {
    let layout = get_layout();
    let scrollable = get_focused_scrollable(&layout);
    if scrollable < 0 {
        return false;
    }

    let idx = scrollable as usize;
    match direction {
        ScrollDirection::Up => scroll_by(&layout, idx, 0, -(LINE_SCROLL as i32)),
        ScrollDirection::Down => scroll_by(&layout, idx, 0, LINE_SCROLL as i32),
        ScrollDirection::Left => scroll_by(&layout, idx, -(LINE_SCROLL as i32), 0),
        ScrollDirection::Right => scroll_by(&layout, idx, LINE_SCROLL as i32, 0),
    }
}

/// Handle PageUp/PageDown scroll.
/// Returns true if scroll occurred.
///
/// Uses global layout accessor `get_layout()` set by the render pipeline.
pub fn handle_page_scroll(direction: ScrollDirection) -> bool {
    let layout = get_layout();
    let scrollable = get_focused_scrollable(&layout);
    if scrollable < 0 {
        return false;
    }

    let idx = scrollable as usize;
    let viewport_height = layout.height.get(idx).copied().unwrap_or(10) as f32;
    let page_amount = (viewport_height * PAGE_SCROLL_FACTOR).max(1.0) as i32;

    match direction {
        ScrollDirection::Up => scroll_by(&layout, idx, 0, -page_amount),
        ScrollDirection::Down => scroll_by(&layout, idx, 0, page_amount),
        _ => false, // No horizontal page scroll
    }
}

/// Handle Ctrl+Home/End for scroll to top/bottom.
/// Returns true if action performed.
///
/// Uses global layout accessor `get_layout()` set by the render pipeline.
pub fn handle_home_end(to_top: bool) -> bool {
    let layout = get_layout();
    let scrollable = get_focused_scrollable(&layout);
    if scrollable < 0 {
        return false;
    }

    let idx = scrollable as usize;
    if to_top {
        scroll_to_top(&layout, idx);
    } else {
        scroll_to_bottom(&layout, idx);
    }
    true
}

// =============================================================================
// MOUSE WHEEL HANDLER
// =============================================================================

/// Handle mouse wheel scroll at coordinates.
/// First tries element under cursor, then falls back to focused scrollable.
/// Uses chaining for BOTH cases (unlike keyboard scroll which doesn't chain).
///
/// Uses global layout accessor `get_layout()` set by the render pipeline.
pub fn handle_wheel_scroll(
    x: u16,
    y: u16,
    direction: ScrollDirection,
) -> bool {
    use crate::state::mouse::hit_test;

    let layout = get_layout();

    // Try element under cursor via hit test
    let component_at = hit_test(x, y);

    // If component at cursor is scrollable, scroll it (with chaining)
    if let Some(idx) = component_at {
        if is_scrollable(&layout, idx) {
            let delta = match direction {
                ScrollDirection::Up => (0, -(WHEEL_SCROLL as i32)),
                ScrollDirection::Down => (0, WHEEL_SCROLL as i32),
                ScrollDirection::Left => (-(WHEEL_SCROLL as i32), 0),
                ScrollDirection::Right => (WHEEL_SCROLL as i32, 0),
            };
            return scroll_by_with_chaining(&layout, idx, delta.0, delta.1);
        }
    }

    // Fallback to focused scrollable (NOW WITH CHAINING - fixed from previous version)
    let scrollable = get_focused_scrollable(&layout);
    if scrollable >= 0 {
        let idx = scrollable as usize;
        let delta = match direction {
            ScrollDirection::Up => (0, -(WHEEL_SCROLL as i32)),
            ScrollDirection::Down => (0, WHEEL_SCROLL as i32),
            ScrollDirection::Left => (-(WHEEL_SCROLL as i32), 0),
            ScrollDirection::Right => (WHEEL_SCROLL as i32, 0),
        };
        scroll_by_with_chaining(&layout, idx, delta.0, delta.1)
    } else {
        false
    }
}

// =============================================================================
// FIND SCROLLABLE ANCESTOR
// =============================================================================

/// Find the nearest scrollable ancestor of a component.
/// Returns the scrollable index, or None if no scrollable ancestor.
pub fn find_scrollable_ancestor(layout: &ComputedLayout, index: usize) -> Option<usize> {
    let mut current = core::get_parent_index(index);
    while let Some(parent_idx) = current {
        if is_scrollable(layout, parent_idx) {
            return Some(parent_idx);
        }
        current = core::get_parent_index(parent_idx);
    }
    None
}

// =============================================================================
// SCROLL INTO VIEW
// =============================================================================

/// High-level scroll into view for a focused component.
/// Finds the scrollable ancestor and computes positions automatically.
///
/// Call this when focus changes to ensure the newly focused element is visible.
pub fn scroll_focused_into_view(layout: &ComputedLayout, focused_index: usize) {
    // Find scrollable ancestor
    let scrollable = match find_scrollable_ancestor(layout, focused_index) {
        Some(idx) => idx,
        None => return, // No scrollable ancestor
    };

    // Get positions from layout
    let child_y = layout.y.get(focused_index).copied().unwrap_or(0);
    let child_height = layout.height.get(focused_index).copied().unwrap_or(0);
    let scrollable_y = layout.y.get(scrollable).copied().unwrap_or(0);
    let viewport_height = layout.height.get(scrollable).copied().unwrap_or(0);

    // Compute child position relative to scrollable
    let relative_y = child_y.saturating_sub(scrollable_y);

    scroll_into_view(layout, scrollable, relative_y, child_height, viewport_height);
}

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

// =============================================================================
// STICK TO BOTTOM (AUTO-SCROLL)
// =============================================================================

/// Check and apply stick-to-bottom behavior for a component.
///
/// If stick_to_bottom is enabled and the component was at the bottom
/// when content grew, automatically scroll to the new bottom.
///
/// This should be called after layout computation to detect content growth.
pub fn handle_stick_to_bottom(layout: &ComputedLayout, index: usize) {
    // Check if stick_to_bottom is enabled
    if !interaction::get_stick_to_bottom(index) {
        return;
    }

    // Check if scrollable
    if !is_scrollable(layout, index) {
        return;
    }

    let (_, current_y) = get_scroll_offset(index);
    let (_, max_y) = get_max_scroll(layout, index);
    let prev_max_y = interaction::get_prev_max_scroll_y(index);

    // Check if content grew (max scroll increased)
    if max_y > prev_max_y {
        // Check if we were at/near the bottom before content grew
        // We consider "at bottom" if within 1 line of the previous max
        let was_at_bottom = current_y >= prev_max_y.saturating_sub(LINE_SCROLL);

        if was_at_bottom {
            // Auto-scroll to new bottom
            scroll_to_bottom(layout, index);
        }
    }

    // Update previous max scroll for next comparison
    interaction::set_prev_max_scroll_y(index, max_y);
}

/// Update stick_to_bottom state based on user scroll action.
///
/// If the user scrolls away from the bottom, we disable auto-follow.
/// If the user scrolls back to the bottom, we re-enable auto-follow.
///
/// This should be called after any user-initiated scroll (arrow keys, mouse wheel, etc.)
pub fn update_stick_to_bottom_on_scroll(layout: &ComputedLayout, index: usize) {
    // Check if stick_to_bottom is enabled
    if !interaction::get_stick_to_bottom(index) {
        return;
    }

    // Check if scrollable
    if !is_scrollable(layout, index) {
        return;
    }

    let (_, current_y) = get_scroll_offset(index);
    let (_, max_y) = get_max_scroll(layout, index);

    // Check if at bottom (within tolerance)
    let at_bottom = current_y >= max_y.saturating_sub(LINE_SCROLL);

    // If user scrolled away from bottom, we don't need to do anything special
    // The handle_stick_to_bottom will check if they were at bottom before growing
    // If they're at bottom now, prev_max_y tracking handles the rest

    // Update prev_max_scroll_y so that if they scroll back to bottom,
    // content growth will trigger auto-scroll again
    if at_bottom {
        interaction::set_prev_max_scroll_y(index, max_y);
    }
}

/// Check if a scrollable component is currently at the bottom.
pub fn is_at_bottom(layout: &ComputedLayout, index: usize) -> bool {
    if !is_scrollable(layout, index) {
        return true; // Non-scrollable is always "at bottom"
    }

    let (_, current_y) = get_scroll_offset(index);
    let (_, max_y) = get_max_scroll(layout, index);

    current_y >= max_y.saturating_sub(LINE_SCROLL)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::arrays::interaction;
    use crate::engine::reset_registry;
    use crate::pipeline::{set_layout, clear_layout};
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
        clear_layout();
    }

    /// Setup with a layout for the global accessor.
    /// Handlers that use get_layout() need this.
    fn setup_with_layout(layout: ComputedLayout) {
        setup();
        set_layout(layout);
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
        use crate::primitives::{box_primitive, BoxProps};

        // Create layout where component 0 is scrollable
        let layout = create_test_layout(&[(0, 10, 50)]);
        setup_with_layout(layout);

        // Create focusable component
        let _cleanup = box_primitive(BoxProps {
            focusable: Some(true),
            ..Default::default()
        });

        // Focus it
        focus::focus(0);

        assert!(handle_arrow_scroll(ScrollDirection::Down));
        assert_eq!(interaction::get_scroll_offset_y(0), LINE_SCROLL);
    }

    #[test]
    fn test_handle_arrow_scroll_up() {
        use crate::primitives::{box_primitive, BoxProps};

        let layout = create_test_layout(&[(0, 10, 50)]);
        setup_with_layout(layout);

        let _cleanup = box_primitive(BoxProps {
            focusable: Some(true),
            ..Default::default()
        });
        focus::focus(0);

        // Start in middle
        interaction::set_scroll_offset(0, 0, 25);

        assert!(handle_arrow_scroll(ScrollDirection::Up));
        assert_eq!(interaction::get_scroll_offset_y(0), 25 - LINE_SCROLL);
    }

    #[test]
    fn test_handle_arrow_scroll_at_boundary() {
        use crate::primitives::{box_primitive, BoxProps};

        let layout = create_test_layout(&[(0, 10, 50)]);
        setup_with_layout(layout);

        let _cleanup = box_primitive(BoxProps {
            focusable: Some(true),
            ..Default::default()
        });
        focus::focus(0);

        // Already at top
        assert!(!handle_arrow_scroll(ScrollDirection::Up));
        assert_eq!(interaction::get_scroll_offset_y(0), 0);
    }

    #[test]
    fn test_handle_arrow_scroll_horizontal() {
        use crate::primitives::{box_primitive, BoxProps};

        let layout = create_test_layout(&[(0, 50, 50)]);
        setup_with_layout(layout);

        let _cleanup = box_primitive(BoxProps {
            focusable: Some(true),
            ..Default::default()
        });
        focus::focus(0);

        // Scroll right
        assert!(handle_arrow_scroll(ScrollDirection::Right));
        assert_eq!(interaction::get_scroll_offset_x(0), LINE_SCROLL);

        // Scroll left
        assert!(handle_arrow_scroll(ScrollDirection::Left));
        assert_eq!(interaction::get_scroll_offset_x(0), 0);

        // At left boundary
        assert!(!handle_arrow_scroll(ScrollDirection::Left));
    }

    #[test]
    fn test_handle_page_scroll() {
        use crate::primitives::{box_primitive, BoxProps};

        // Create layout with known height
        let mut layout = create_test_layout(&[(0, 10, 100)]);
        layout.height[0] = 10; // Viewport height
        setup_with_layout(layout);

        let _cleanup = box_primitive(BoxProps {
            focusable: Some(true),
            ..Default::default()
        });
        focus::focus(0);

        assert!(handle_page_scroll(ScrollDirection::Down));
        // Should scroll by 10 * 0.9 = 9
        assert_eq!(interaction::get_scroll_offset_y(0), 9);
    }

    #[test]
    fn test_handle_page_scroll_up() {
        use crate::primitives::{box_primitive, BoxProps};

        let mut layout = create_test_layout(&[(0, 10, 100)]);
        layout.height[0] = 10;
        setup_with_layout(layout);

        let _cleanup = box_primitive(BoxProps {
            focusable: Some(true),
            ..Default::default()
        });
        focus::focus(0);

        // Start at position 50
        interaction::set_scroll_offset(0, 0, 50);

        assert!(handle_page_scroll(ScrollDirection::Up));
        // Should scroll up by 10 * 0.9 = 9
        assert_eq!(interaction::get_scroll_offset_y(0), 41);
    }

    #[test]
    fn test_handle_home_end() {
        use crate::primitives::{box_primitive, BoxProps};

        let layout = create_test_layout(&[(0, 10, 50)]);
        setup_with_layout(layout);

        let _cleanup = box_primitive(BoxProps {
            focusable: Some(true),
            ..Default::default()
        });
        focus::focus(0);

        // Scroll to bottom
        assert!(handle_home_end(false));
        assert_eq!(interaction::get_scroll_offset_y(0), 50);

        // Scroll to top
        assert!(handle_home_end(true));
        assert_eq!(interaction::get_scroll_offset_y(0), 0);
    }

    #[test]
    fn test_no_scroll_when_not_focused() {
        let layout = create_test_layout(&[(0, 10, 50)]);
        setup_with_layout(layout);

        reset_focus_state(); // Ensure no focus

        assert!(!handle_arrow_scroll(ScrollDirection::Down));
        assert!(!handle_page_scroll(ScrollDirection::Down));
        assert!(!handle_home_end(true));
    }

    #[test]
    fn test_no_scroll_when_focused_not_scrollable() {
        use crate::primitives::{box_primitive, BoxProps};

        // Empty layout - component 0 not scrollable
        let layout = ComputedLayout::default();
        setup_with_layout(layout);

        let _cleanup = box_primitive(BoxProps {
            focusable: Some(true),
            ..Default::default()
        });
        focus::focus(0);

        assert!(!handle_arrow_scroll(ScrollDirection::Down));
        assert!(!handle_page_scroll(ScrollDirection::Down));
        assert!(!handle_home_end(false));
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

    // =========================================================================
    // FIND SCROLLABLE ANCESTOR TESTS (04-03)
    // =========================================================================

    #[test]
    fn test_find_scrollable_ancestor_immediate_parent() {
        setup();

        // Parent (index 0) is scrollable
        // Child (index 1) is not scrollable
        let layout = create_test_layout(&[(0, 10, 50)]);

        // Set up parent-child relationship
        core::set_parent_index(1, Some(0));

        // Find ancestor of child
        let ancestor = find_scrollable_ancestor(&layout, 1);
        assert_eq!(ancestor, Some(0));
    }

    #[test]
    fn test_find_scrollable_ancestor_grandparent() {
        setup();

        // Grandparent (index 0) is scrollable
        // Parent (index 1) is not scrollable
        // Child (index 2) is not scrollable
        let layout = create_test_layout(&[(0, 10, 50)]);

        // Set up hierarchy: 2 -> 1 -> 0
        core::set_parent_index(2, Some(1));
        core::set_parent_index(1, Some(0));

        // Find ancestor of grandchild
        let ancestor = find_scrollable_ancestor(&layout, 2);
        assert_eq!(ancestor, Some(0));
    }

    #[test]
    fn test_find_scrollable_ancestor_no_ancestor() {
        setup();

        // No scrollable components
        let layout = ComputedLayout::default();

        // Component with no parent
        let ancestor = find_scrollable_ancestor(&layout, 0);
        assert_eq!(ancestor, None);
    }

    #[test]
    fn test_find_scrollable_ancestor_skips_non_scrollable_parent() {
        setup();

        // Grandparent (index 0) is scrollable
        // Parent (index 1) is also scrollable (closer)
        // Child (index 2) is not scrollable
        let layout = create_test_layout(&[(0, 10, 50), (1, 5, 25)]);

        // Set up hierarchy
        core::set_parent_index(2, Some(1));
        core::set_parent_index(1, Some(0));

        // Should find the closest scrollable parent (1)
        let ancestor = find_scrollable_ancestor(&layout, 2);
        assert_eq!(ancestor, Some(1));
    }

    // =========================================================================
    // SCROLL FOCUSED INTO VIEW TESTS (04-03)
    // =========================================================================

    #[test]
    fn test_scroll_focused_into_view_scrolls_to_show_child() {
        setup();

        // Scrollable parent (index 0) with viewport at y=0
        // Child (index 1) at y=50, which is below viewport
        let mut layout = create_test_layout(&[(0, 0, 100)]);
        layout.y = vec![0, 50];
        layout.height = vec![20, 5];

        // Set up parent-child relationship
        core::set_parent_index(1, Some(0));

        // No scroll yet
        interaction::set_scroll_offset(0, 0, 0);

        // Call scroll_focused_into_view for child
        scroll_focused_into_view(&layout, 1);

        // Should have scrolled to show child
        // child_bottom (55) - viewport_height (20) = 35
        assert_eq!(interaction::get_scroll_offset_y(0), 35);
    }

    #[test]
    fn test_scroll_focused_into_view_no_scrollable_ancestor() {
        setup();

        // No scrollable components
        let mut layout = ComputedLayout::default();
        layout.y = vec![0, 50];
        layout.height = vec![20, 5];

        // No scroll should happen (no ancestor to scroll)
        scroll_focused_into_view(&layout, 1);

        // Nothing should crash, nothing should change
    }

    // =========================================================================
    // MOUSE WHEEL SCROLL TESTS (04-03)
    // =========================================================================

    use crate::state::mouse;

    #[test]
    fn test_handle_wheel_scroll_uses_hovered_component() {
        // Create scrollable component at index 0
        let layout = create_test_layout(&[(0, 10, 50)]);
        setup_with_layout(layout);

        // Set up hit grid: component 0 occupies area
        mouse::clear_hit_grid();
        mouse::fill_hit_rect(0, 0, 10, 10, 0);

        // Mouse wheel at (5, 5) should scroll component 0
        let scrolled = handle_wheel_scroll(5, 5, ScrollDirection::Down);
        assert!(scrolled);
        assert_eq!(interaction::get_scroll_offset_y(0), WHEEL_SCROLL);
    }

    #[test]
    fn test_handle_wheel_scroll_falls_back_to_focused() {
        use crate::primitives::{box_primitive, BoxProps};

        let layout = create_test_layout(&[(0, 10, 50)]);
        setup_with_layout(layout);

        // Create focusable component that is scrollable
        let _cleanup = box_primitive(BoxProps {
            focusable: Some(true),
            ..Default::default()
        });
        focus::focus(0);

        // Clear hit grid - no component under cursor
        mouse::clear_hit_grid();

        // Should fall back to focused scrollable
        let scrolled = handle_wheel_scroll(50, 50, ScrollDirection::Down);
        assert!(scrolled);
        assert_eq!(interaction::get_scroll_offset_y(0), WHEEL_SCROLL);
    }

    #[test]
    fn test_handle_wheel_scroll_with_chaining() {
        // Parent (index 0) and child (index 1) both scrollable
        let layout = create_test_layout(&[(0, 10, 50), (1, 5, 10)]);
        setup_with_layout(layout);

        // Set up parent-child relationship
        core::set_parent_index(1, Some(0));

        // Child at boundary (max scroll)
        interaction::set_scroll_offset(1, 5, 10);

        // Set up hit grid: child at cursor position
        mouse::clear_hit_grid();
        mouse::fill_hit_rect(5, 5, 10, 10, 1);

        // Wheel scroll down should chain to parent
        let scrolled = handle_wheel_scroll(7, 7, ScrollDirection::Down);
        assert!(scrolled);
        assert_eq!(interaction::get_scroll_offset_y(1), 10); // Child unchanged
        assert_eq!(interaction::get_scroll_offset_y(0), WHEEL_SCROLL); // Parent scrolled
    }

    #[test]
    fn test_handle_wheel_scroll_no_scrollable() {
        // Empty layout - nothing scrollable
        let layout = ComputedLayout::default();
        setup_with_layout(layout);

        // Clear hit grid
        mouse::clear_hit_grid();
        reset_focus_state();

        // Should return false
        let scrolled = handle_wheel_scroll(5, 5, ScrollDirection::Down);
        assert!(!scrolled);
    }

    #[test]
    fn test_handle_wheel_scroll_horizontal() {
        let layout = create_test_layout(&[(0, 50, 50)]);
        setup_with_layout(layout);

        mouse::clear_hit_grid();
        mouse::fill_hit_rect(0, 0, 10, 10, 0);

        // Scroll right
        let scrolled = handle_wheel_scroll(5, 5, ScrollDirection::Right);
        assert!(scrolled);
        assert_eq!(interaction::get_scroll_offset_x(0), WHEEL_SCROLL);

        // Scroll left
        let scrolled = handle_wheel_scroll(5, 5, ScrollDirection::Left);
        assert!(scrolled);
        assert_eq!(interaction::get_scroll_offset_x(0), 0);
    }

    // =========================================================================
    // STICK TO BOTTOM TESTS (04-04)
    // =========================================================================

    #[test]
    fn test_stick_to_bottom_auto_scrolls_on_content_growth() {
        setup();

        // Enable stick_to_bottom
        interaction::set_stick_to_bottom(0, true);

        // Create layout with max_scroll_y = 50
        let mut layout = create_test_layout(&[(0, 10, 50)]);
        layout.height[0] = 20; // Viewport height

        // Start at bottom (scroll_y = max_scroll_y = 50)
        interaction::set_scroll_offset(0, 0, 50);
        interaction::set_prev_max_scroll_y(0, 50);

        // Simulate content growth: max_scroll_y increases to 70
        layout.max_scroll_y[0] = 70;

        // Handle stick_to_bottom
        handle_stick_to_bottom(&layout, 0);

        // Should auto-scroll to new bottom
        assert_eq!(interaction::get_scroll_offset_y(0), 70);
    }

    #[test]
    fn test_stick_to_bottom_no_scroll_when_user_scrolled_up() {
        setup();

        // Enable stick_to_bottom
        interaction::set_stick_to_bottom(0, true);

        // Create layout with max_scroll_y = 50
        let mut layout = create_test_layout(&[(0, 10, 50)]);
        layout.height[0] = 20;

        // User has scrolled up to position 20 (not at bottom)
        interaction::set_scroll_offset(0, 0, 20);
        interaction::set_prev_max_scroll_y(0, 50);

        // Simulate content growth: max_scroll_y increases to 70
        layout.max_scroll_y[0] = 70;

        // Handle stick_to_bottom
        handle_stick_to_bottom(&layout, 0);

        // Should NOT auto-scroll because user was not at bottom
        assert_eq!(interaction::get_scroll_offset_y(0), 20);
    }

    #[test]
    fn test_stick_to_bottom_updates_prev_max() {
        setup();

        // Enable stick_to_bottom
        interaction::set_stick_to_bottom(0, true);

        let layout = create_test_layout(&[(0, 10, 50)]);

        // Initial state
        interaction::set_scroll_offset(0, 0, 50);
        interaction::set_prev_max_scroll_y(0, 0);

        // Handle should update prev_max_scroll_y
        handle_stick_to_bottom(&layout, 0);

        assert_eq!(interaction::get_prev_max_scroll_y(0), 50);
    }

    #[test]
    fn test_is_at_bottom() {
        setup();

        let layout = create_test_layout(&[(0, 10, 50)]);

        // At bottom
        interaction::set_scroll_offset(0, 0, 50);
        assert!(is_at_bottom(&layout, 0));

        // Near bottom (within LINE_SCROLL)
        interaction::set_scroll_offset(0, 0, 49);
        assert!(is_at_bottom(&layout, 0));

        // Not at bottom
        interaction::set_scroll_offset(0, 0, 40);
        assert!(!is_at_bottom(&layout, 0));

        // At top
        interaction::set_scroll_offset(0, 0, 0);
        assert!(!is_at_bottom(&layout, 0));
    }

    #[test]
    fn test_stick_to_bottom_disabled_does_nothing() {
        setup();

        // stick_to_bottom is disabled by default
        assert!(!interaction::get_stick_to_bottom(0));

        let mut layout = create_test_layout(&[(0, 10, 50)]);
        layout.height[0] = 20;

        interaction::set_scroll_offset(0, 0, 50);
        interaction::set_prev_max_scroll_y(0, 50);

        // Simulate content growth
        layout.max_scroll_y[0] = 70;

        // Handle stick_to_bottom
        handle_stick_to_bottom(&layout, 0);

        // Should NOT auto-scroll because stick_to_bottom is disabled
        assert_eq!(interaction::get_scroll_offset_y(0), 50);
    }
}
