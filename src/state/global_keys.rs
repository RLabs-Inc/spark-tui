//! Global Keys Module - Central keyboard event router
//!
//! This module is the **central router** for all keyboard events in the framework.
//! It enforces the correct priority order for event handling:
//!
//! 1. **Ctrl+C** (hardcoded) - Graceful shutdown, non-overridable
//! 2. **Tab/Shift+Tab** (always consumed) - Focus navigation
//! 3. **Focused component handlers** - Components get FIRST CHANCE
//! 4. **User handlers** - keyboard.on(), keyboard.on_key()
//! 5. **Framework defaults** - Scroll handling (FALLBACK, LAST)
//!
//! This architecture solves the problem where scroll handlers registered via
//! `keyboard::on()` would steal events before components could handle them.
//! Now focused components (like Input) can consume arrow keys for cursor
//! movement, and scroll only happens if nothing else handles the event.
//!
//! # Example
//!
//! ```ignore
//! use spark_tui::state::global_keys;
//! use std::sync::Arc;
//! use std::sync::atomic::AtomicBool;
//!
//! let running = Arc::new(AtomicBool::new(true));
//! let handle = global_keys::setup_global_keys(running.clone());
//!
//! // Events are routed through route_keyboard_event() from input.rs
//! // The handle just needs to be kept alive for cleanup
//!
//! // Later, on cleanup:
//! handle.cleanup();
//! ```

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::cell::RefCell;

use super::keyboard::{self, KeyboardEvent, KeyState};
use super::focus;
use super::scroll;
use super::mouse::ScrollDirection;

// =============================================================================
// THREAD-LOCAL STATE
// =============================================================================

thread_local! {
    /// The running flag set by setup_global_keys, used by the router.
    static RUNNING: RefCell<Option<Arc<AtomicBool>>> = RefCell::new(None);
}

/// Get a clone of the running flag (if set).
fn get_running() -> Option<Arc<AtomicBool>> {
    RUNNING.with(|r| r.borrow().clone())
}

/// Set the running flag (called by setup_global_keys).
fn set_running(running: Arc<AtomicBool>) {
    RUNNING.with(|r| *r.borrow_mut() = Some(running));
}

/// Clear the running flag (for testing).
fn clear_running() {
    RUNNING.with(|r| *r.borrow_mut() = None);
}

// =============================================================================
// GLOBAL KEYS HANDLE
// =============================================================================

/// Cleanup handle for global key handlers.
///
/// The handle stores the running flag reference and performs cleanup
/// when dropped or explicitly cleaned up.
pub struct GlobalKeysHandle {
    _running: Arc<AtomicBool>,
}

impl GlobalKeysHandle {
    /// Clean up all global key handlers.
    ///
    /// Clears the thread-local running flag.
    pub fn cleanup(self) {
        clear_running();
    }
}

// =============================================================================
// CENTRAL KEYBOARD ROUTER
// =============================================================================

/// Central keyboard event router - enforces correct priority order.
///
/// This function implements the TypeScript keyboard architecture:
///
/// 1. Ctrl+C (hardcoded, non-overridable)
/// 2. Tab/Shift+Tab (always consumed for focus navigation)
/// 3. Focused component onKey handlers (components get FIRST CHANCE)
/// 4. User keyboard.on/on_key handlers
/// 5. Framework defaults like scroll (FALLBACK, LAST)
///
/// # Arguments
/// * `event` - The keyboard event to route
/// * `running` - Atomic bool to set to false on Ctrl+C
///
/// # Returns
/// `true` if the event was consumed, `false` otherwise
pub fn route_keyboard_event(event: &KeyboardEvent, running: &Arc<AtomicBool>) -> bool {
    let is_press = event.state == KeyState::Press;

    // ═══════════════════════════════════════════════════════════════════════════
    // 1. HARDCODED SYSTEM SHORTCUTS (non-overridable)
    // ═══════════════════════════════════════════════════════════════════════════
    if is_press && event.modifiers.ctrl && event.key == "c" {
        running.store(false, Ordering::SeqCst);
        return true;
    }

    // Always update last_event state regardless of press/release
    keyboard::update_last_event(event.clone());

    // Non-press events: dispatch for monitoring but don't handle navigation/actions
    if !is_press {
        // Still dispatch to focused handler for release events (some components track state)
        let focused = focus::get_focused_index();
        if focused >= 0 {
            keyboard::dispatch_focused(focused, event);
        }
        return false;
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // 2. TAB NAVIGATION (always consumed, never reaches components)
    // ═══════════════════════════════════════════════════════════════════════════
    if event.key == "Tab" && !event.modifiers.ctrl && !event.modifiers.alt {
        if event.modifiers.shift {
            focus::focus_previous();
        } else {
            focus::focus_next();
        }
        return true;
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // 3. FOCUSED COMPONENT HANDLERS (components get FIRST CHANCE)
    // ═══════════════════════════════════════════════════════════════════════════
    let focused = focus::get_focused_index();
    if focused >= 0 {
        if keyboard::dispatch_focused(focused, event) {
            return true;
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // 4. USER HANDLERS (keyboard.on, keyboard.on_key)
    // ═══════════════════════════════════════════════════════════════════════════
    if keyboard::dispatch_to_handlers(event) {
        return true;
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // 5. FRAMEWORK DEFAULTS (scroll - only if nothing else handled)
    // ═══════════════════════════════════════════════════════════════════════════
    route_framework_defaults(event)
}

/// Route framework default handlers (scroll keys).
///
/// These are the LAST resort - only fire if no component or user handler consumed the event.
fn route_framework_defaults(event: &KeyboardEvent) -> bool {
    // Arrow keys for scrolling (without modifiers, or with just shift)
    // Note: Ctrl+Arrow is used for word navigation in inputs
    if !event.modifiers.ctrl && !event.modifiers.alt {
        match event.key.as_str() {
            "ArrowUp" if !event.modifiers.shift => {
                return scroll::with_current_layout(|layout| {
                    scroll::handle_arrow_scroll(layout, ScrollDirection::Up)
                }).unwrap_or(false);
            }
            "ArrowDown" if !event.modifiers.shift => {
                return scroll::with_current_layout(|layout| {
                    scroll::handle_arrow_scroll(layout, ScrollDirection::Down)
                }).unwrap_or(false);
            }
            "ArrowLeft" if !event.modifiers.shift => {
                return scroll::with_current_layout(|layout| {
                    scroll::handle_arrow_scroll(layout, ScrollDirection::Left)
                }).unwrap_or(false);
            }
            "ArrowRight" if !event.modifiers.shift => {
                return scroll::with_current_layout(|layout| {
                    scroll::handle_arrow_scroll(layout, ScrollDirection::Right)
                }).unwrap_or(false);
            }
            _ => {}
        }
    }

    // PageUp/PageDown (no modifiers needed)
    match event.key.as_str() {
        "PageUp" => {
            return scroll::with_current_layout(|layout| {
                scroll::handle_page_scroll(layout, ScrollDirection::Up)
            }).unwrap_or(false);
        }
        "PageDown" => {
            return scroll::with_current_layout(|layout| {
                scroll::handle_page_scroll(layout, ScrollDirection::Down)
            }).unwrap_or(false);
        }
        _ => {}
    }

    // Ctrl+Home/End for scroll to boundaries
    if event.modifiers.ctrl {
        match event.key.as_str() {
            "Home" => {
                return scroll::with_current_layout(|layout| {
                    scroll::handle_home_end(layout, true)
                }).unwrap_or(false);
            }
            "End" => {
                return scroll::with_current_layout(|layout| {
                    scroll::handle_home_end(layout, false)
                }).unwrap_or(false);
            }
            _ => {}
        }
    }

    false
}

// =============================================================================
// CONVENIENCE WRAPPER
// =============================================================================

/// Route a keyboard event using the stored running flag.
///
/// This is the function called by `input::route_event()`. It retrieves
/// the running flag from thread-local storage (set by `setup_global_keys`)
/// and calls `route_keyboard_event`.
///
/// Returns `true` if the event was consumed, `false` otherwise.
/// Returns `false` if no running flag is set (global keys not initialized).
pub fn route_key_event(event: &KeyboardEvent) -> bool {
    match get_running() {
        Some(running) => route_keyboard_event(event, &running),
        None => {
            // No running flag set - just do basic dispatch
            // This happens if route_event is called before setup_global_keys
            keyboard::update_last_event(event.clone());
            if event.state == KeyState::Press {
                keyboard::dispatch_to_handlers(event)
            } else {
                false
            }
        }
    }
}

// =============================================================================
// SETUP FUNCTIONS
// =============================================================================

/// Set up global key handlers.
/// Returns a handle for cleanup.
///
/// # Arguments
/// * `running` - Atomic bool to set to false on Ctrl+C
///
/// # Note
///
/// This stores the running flag in thread-local storage so that
/// `route_key_event()` can access it when called from the input event loop.
pub fn setup_global_keys(running: Arc<AtomicBool>) -> GlobalKeysHandle {
    set_running(running.clone());
    GlobalKeysHandle {
        _running: running,
    }
}

/// Clean up all global keys state (for testing).
pub fn cleanup_global_keys() {
    // Nothing to clean up beyond the handle
    // This function exists for API consistency
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::keyboard::{KeyboardEvent, Modifiers, reset_keyboard_state};
    use crate::engine::reset_registry;
    use crate::state::focus::reset_focus_state;
    use crate::primitives::{box_primitive, BoxProps};
    use std::rc::Rc;
    use std::cell::Cell;

    fn setup() {
        reset_registry();
        reset_focus_state();
        reset_keyboard_state();
        scroll::clear_current_layout();
    }

    /// Helper to route an event through the new central router
    fn route_event(event: KeyboardEvent, running: &Arc<AtomicBool>) -> bool {
        route_keyboard_event(&event, running)
    }

    #[test]
    fn test_ctrl_c_sets_running_false() {
        setup();

        let running = Arc::new(AtomicBool::new(true));
        let _handle = setup_global_keys(running.clone());

        assert!(running.load(Ordering::SeqCst));

        // Dispatch Ctrl+C via router
        let event = KeyboardEvent::with_modifiers("c", Modifiers::ctrl());
        route_event(event, &running);

        assert!(!running.load(Ordering::SeqCst));
    }

    #[test]
    fn test_regular_c_does_not_stop() {
        setup();

        let running = Arc::new(AtomicBool::new(true));
        let _handle = setup_global_keys(running.clone());

        // Dispatch plain 'c' (no Ctrl) via router
        let event = KeyboardEvent::new("c");
        route_event(event, &running);

        // Should still be running
        assert!(running.load(Ordering::SeqCst));
    }

    #[test]
    fn test_tab_calls_focus_next() {
        setup();

        // Create focusable components
        let _c1 = box_primitive(BoxProps {
            focusable: Some(true),
            tab_index: Some(1),
            ..Default::default()
        });
        let _c2 = box_primitive(BoxProps {
            focusable: Some(true),
            tab_index: Some(2),
            ..Default::default()
        });

        // Focus first component
        focus::focus(0);
        assert_eq!(focus::get_focused_index(), 0);

        let running = Arc::new(AtomicBool::new(true));
        let _handle = setup_global_keys(running.clone());

        // Dispatch Tab via router
        let event = KeyboardEvent::new("Tab");
        route_event(event, &running);

        // Should have moved to next component
        assert_eq!(focus::get_focused_index(), 1);
    }

    #[test]
    fn test_shift_tab_calls_focus_previous() {
        setup();

        // Create focusable components
        let _c1 = box_primitive(BoxProps {
            focusable: Some(true),
            tab_index: Some(1),
            ..Default::default()
        });
        let _c2 = box_primitive(BoxProps {
            focusable: Some(true),
            tab_index: Some(2),
            ..Default::default()
        });

        // Focus second component
        focus::focus(1);
        assert_eq!(focus::get_focused_index(), 1);

        let running = Arc::new(AtomicBool::new(true));
        let _handle = setup_global_keys(running.clone());

        // Dispatch Shift+Tab via router
        let event = KeyboardEvent::with_modifiers("Tab", Modifiers::shift());
        route_event(event, &running);

        // Should have moved to previous component
        assert_eq!(focus::get_focused_index(), 0);
    }

    // =========================================================================
    // PRIORITY TESTS - The core fix verification
    // =========================================================================

    #[test]
    fn test_focused_component_blocks_scroll() {
        setup();
        interaction::reset();

        // Create focusable box with onKey that captures ArrowDown
        let arrow_count = Rc::new(Cell::new(0));
        let arrow_count_clone = arrow_count.clone();

        let _c1 = box_primitive(BoxProps {
            focusable: Some(true),
            on_key: Some(Rc::new(move |event| {
                if event.key == "ArrowDown" {
                    arrow_count_clone.set(arrow_count_clone.get() + 1);
                    return true; // CONSUME - prevent scroll
                }
                false
            })),
            ..Default::default()
        });

        // Focus the component
        focus::focus(0);

        // Set up layout where component 0 is scrollable
        let layout = create_test_layout(&[(0, 10, 50)]);
        scroll::set_current_layout(layout);

        let running = Arc::new(AtomicBool::new(true));
        let _handle = setup_global_keys(running.clone());

        // Dispatch ArrowDown via router
        let event = KeyboardEvent::new("ArrowDown");
        let consumed = route_event(event, &running);

        // Component handler should have been called
        assert_eq!(arrow_count.get(), 1);
        // Event should be consumed
        assert!(consumed);
        // Scroll should NOT have happened (component consumed the event)
        assert_eq!(interaction::get_scroll_offset_y(0), 0);

        scroll::clear_current_layout();
    }

    #[test]
    fn test_scroll_fires_when_unhandled() {
        setup();
        interaction::reset();

        // Create focusable box WITHOUT onKey handler
        let _c1 = box_primitive(BoxProps {
            focusable: Some(true),
            // NO on_key - event falls through to scroll
            ..Default::default()
        });

        // Focus the component
        focus::focus(0);

        // Set up layout where component 0 is scrollable
        let layout = create_test_layout(&[(0, 10, 50)]);
        scroll::set_current_layout(layout);

        let running = Arc::new(AtomicBool::new(true));
        let _handle = setup_global_keys(running.clone());

        // Dispatch ArrowDown via router
        let event = KeyboardEvent::new("ArrowDown");
        let consumed = route_event(event, &running);

        // Event should be consumed by scroll
        assert!(consumed);
        // Scroll SHOULD have happened
        assert_eq!(interaction::get_scroll_offset_y(0), scroll::LINE_SCROLL);

        scroll::clear_current_layout();
    }

    #[test]
    fn test_tab_always_consumed_before_component() {
        setup();

        // Create focusable box that would capture Tab
        let tab_count = Rc::new(Cell::new(0));
        let tab_count_clone = tab_count.clone();

        let _c1 = box_primitive(BoxProps {
            focusable: Some(true),
            tab_index: Some(1),
            on_key: Some(Rc::new(move |event| {
                if event.key == "Tab" {
                    tab_count_clone.set(tab_count_clone.get() + 1);
                    return true;
                }
                false
            })),
            ..Default::default()
        });
        let _c2 = box_primitive(BoxProps {
            focusable: Some(true),
            tab_index: Some(2),
            ..Default::default()
        });

        // Focus first component
        focus::focus(0);

        let running = Arc::new(AtomicBool::new(true));
        let _handle = setup_global_keys(running.clone());

        // Dispatch Tab via router
        let event = KeyboardEvent::new("Tab");
        route_event(event, &running);

        // Component handler should NOT have been called (Tab handled at step 2)
        assert_eq!(tab_count.get(), 0);
        // Focus should have moved (Tab navigation worked)
        assert_eq!(focus::get_focused_index(), 1);
    }

    #[test]
    fn test_user_handler_runs_after_component() {
        setup();

        // Register user handler
        let user_handler_count = Rc::new(Cell::new(0));
        let user_handler_clone = user_handler_count.clone();
        let _cleanup = keyboard::on(move |event| {
            if event.key == "x" {
                user_handler_clone.set(user_handler_clone.get() + 1);
                return true;
            }
            false
        });

        // Create focusable box that consumes 'x'
        let component_count = Rc::new(Cell::new(0));
        let component_clone = component_count.clone();
        let _c1 = box_primitive(BoxProps {
            focusable: Some(true),
            on_key: Some(Rc::new(move |event| {
                if event.key == "x" {
                    component_clone.set(component_clone.get() + 1);
                    return true; // CONSUME
                }
                false
            })),
            ..Default::default()
        });

        // Focus component
        focus::focus(0);

        let running = Arc::new(AtomicBool::new(true));
        let _handle = setup_global_keys(running.clone());

        // Dispatch 'x' via router
        let event = KeyboardEvent::new("x");
        route_event(event, &running);

        // Component should have handled it
        assert_eq!(component_count.get(), 1);
        // User handler should NOT have been called (component consumed it)
        assert_eq!(user_handler_count.get(), 0);
    }

    #[test]
    fn test_user_handler_runs_when_component_doesnt_handle() {
        setup();

        // Register user handler
        let user_handler_count = Rc::new(Cell::new(0));
        let user_handler_clone = user_handler_count.clone();
        let _cleanup = keyboard::on(move |event| {
            if event.key == "x" {
                user_handler_clone.set(user_handler_clone.get() + 1);
                return true;
            }
            false
        });

        // Create focusable box that does NOT handle 'x'
        let component_count = Rc::new(Cell::new(0));
        let component_clone = component_count.clone();
        let _c1 = box_primitive(BoxProps {
            focusable: Some(true),
            on_key: Some(Rc::new(move |event| {
                if event.key == "y" { // Only handles 'y', not 'x'
                    component_clone.set(component_clone.get() + 1);
                    return true;
                }
                false
            })),
            ..Default::default()
        });

        // Focus component
        focus::focus(0);

        let running = Arc::new(AtomicBool::new(true));
        let _handle = setup_global_keys(running.clone());

        // Dispatch 'x' via router
        let event = KeyboardEvent::new("x");
        route_event(event, &running);

        // Component handler was called but didn't handle
        assert_eq!(component_count.get(), 0);
        // User handler SHOULD have been called
        assert_eq!(user_handler_count.get(), 1);
    }

    // =========================================================================
    // SCROLL KEY TESTS
    // =========================================================================

    use crate::engine::arrays::interaction;
    use crate::layout::ComputedLayout;

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

    #[test]
    fn test_arrow_down_scrolls_focused_scrollable() {
        setup();
        interaction::reset();

        // Create focusable component (no on_key - falls through to scroll)
        let _c1 = box_primitive(BoxProps {
            focusable: Some(true),
            ..Default::default()
        });

        // Focus it
        focus::focus(0);

        // Set up layout where component 0 is scrollable
        let layout = create_test_layout(&[(0, 10, 50)]);
        scroll::set_current_layout(layout);

        let running = Arc::new(AtomicBool::new(true));
        let _handle = setup_global_keys(running.clone());

        // Dispatch ArrowDown via router
        let event = KeyboardEvent::new("ArrowDown");
        route_event(event, &running);

        // Should have scrolled
        assert_eq!(interaction::get_scroll_offset_y(0), scroll::LINE_SCROLL);

        scroll::clear_current_layout();
    }

    #[test]
    fn test_page_down_scrolls_by_viewport() {
        setup();
        interaction::reset();

        let _c1 = box_primitive(BoxProps {
            focusable: Some(true),
            ..Default::default()
        });
        focus::focus(0);

        // Layout with height=10, max_scroll=100
        let mut layout = create_test_layout(&[(0, 10, 100)]);
        layout.height[0] = 10;
        scroll::set_current_layout(layout);

        let running = Arc::new(AtomicBool::new(true));
        let _handle = setup_global_keys(running.clone());

        // Dispatch PageDown via router
        let event = KeyboardEvent::new("PageDown");
        route_event(event, &running);

        // Should scroll by viewport * 0.9 = 9
        assert_eq!(interaction::get_scroll_offset_y(0), 9);

        scroll::clear_current_layout();
    }

    #[test]
    fn test_ctrl_end_scrolls_to_bottom() {
        setup();
        interaction::reset();

        let _c1 = box_primitive(BoxProps {
            focusable: Some(true),
            ..Default::default()
        });
        focus::focus(0);

        let layout = create_test_layout(&[(0, 10, 50)]);
        scroll::set_current_layout(layout);

        let running = Arc::new(AtomicBool::new(true));
        let _handle = setup_global_keys(running.clone());

        // Dispatch Ctrl+End via router
        let event = KeyboardEvent::with_modifiers("End", Modifiers::ctrl());
        route_event(event, &running);

        // Should be at bottom
        assert_eq!(interaction::get_scroll_offset_y(0), 50);

        scroll::clear_current_layout();
    }

    #[test]
    fn test_ctrl_home_scrolls_to_top() {
        setup();
        interaction::reset();

        let _c1 = box_primitive(BoxProps {
            focusable: Some(true),
            ..Default::default()
        });
        focus::focus(0);

        let layout = create_test_layout(&[(0, 10, 50)]);
        scroll::set_current_layout(layout);

        // Start in middle
        interaction::set_scroll_offset(0, 0, 25);

        let running = Arc::new(AtomicBool::new(true));
        let _handle = setup_global_keys(running.clone());

        // Dispatch Ctrl+Home via router
        let event = KeyboardEvent::with_modifiers("Home", Modifiers::ctrl());
        route_event(event, &running);

        // Should be at top
        assert_eq!(interaction::get_scroll_offset_y(0), 0);

        scroll::clear_current_layout();
    }

    #[test]
    fn test_arrow_keys_dont_scroll_without_layout() {
        setup();
        interaction::reset();

        let _c1 = box_primitive(BoxProps {
            focusable: Some(true),
            ..Default::default()
        });
        focus::focus(0);

        // No layout set
        scroll::clear_current_layout();

        let running = Arc::new(AtomicBool::new(true));
        let _handle = setup_global_keys(running.clone());

        // Dispatch ArrowDown via router
        let event = KeyboardEvent::new("ArrowDown");
        route_event(event, &running);

        // Should not have scrolled (no layout)
        assert_eq!(interaction::get_scroll_offset_y(0), 0);
    }

    #[test]
    fn test_scroll_only_affects_focused_scrollable() {
        setup();
        interaction::reset();

        // Create two components
        let _c1 = box_primitive(BoxProps {
            focusable: Some(true),
            ..Default::default()
        });
        let _c2 = box_primitive(BoxProps {
            focusable: Some(true),
            ..Default::default()
        });

        // Focus component 0
        focus::focus(0);

        // Layout where ONLY component 1 is scrollable (0 is not)
        let layout = create_test_layout(&[(1, 10, 50)]);
        scroll::set_current_layout(layout);

        let running = Arc::new(AtomicBool::new(true));
        let _handle = setup_global_keys(running.clone());

        // Dispatch ArrowDown via router
        let event = KeyboardEvent::new("ArrowDown");
        route_event(event, &running);

        // Component 0 is not scrollable, so nothing should scroll
        assert_eq!(interaction::get_scroll_offset_y(0), 0);
        assert_eq!(interaction::get_scroll_offset_y(1), 0);

        scroll::clear_current_layout();
    }
}
