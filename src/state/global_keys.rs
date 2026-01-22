//! Global Keys Module - Global keyboard shortcuts
//!
//! Provides global key handlers for:
//! - Ctrl+C: Graceful shutdown
//! - Tab: Focus next component
//! - Shift+Tab: Focus previous component
//! - Arrow keys: Scroll focused scrollable (when applicable)
//! - PageUp/PageDown: Page scroll
//! - Ctrl+Home/End: Scroll to top/bottom
//!
//! These handlers are registered on mount and cleaned up on unmount.
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
//! // Later, on cleanup:
//! handle.cleanup();
//! ```

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use super::keyboard;
use super::focus;
use super::scroll;
use super::mouse::ScrollDirection;

// =============================================================================
// GLOBAL KEYS HANDLE
// =============================================================================

/// Cleanup handle for global key handlers
pub struct GlobalKeysHandle {
    ctrl_c_cleanup: Option<Box<dyn FnOnce()>>,
    tab_cleanup: Option<Box<dyn FnOnce()>>,
    shift_tab_cleanup: Option<Box<dyn FnOnce()>>,
    scroll_cleanup: Option<Box<dyn FnOnce()>>,
}

impl GlobalKeysHandle {
    /// Clean up all global key handlers
    pub fn cleanup(mut self) {
        if let Some(cleanup) = self.ctrl_c_cleanup.take() {
            cleanup();
        }
        if let Some(cleanup) = self.tab_cleanup.take() {
            cleanup();
        }
        if let Some(cleanup) = self.shift_tab_cleanup.take() {
            cleanup();
        }
        if let Some(cleanup) = self.scroll_cleanup.take() {
            cleanup();
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
/// # Handlers
///
/// - **Ctrl+C**: Sets `running` to false for graceful shutdown
/// - **Tab**: Calls `focus::focus_next()` to move to next focusable component
/// - **Shift+Tab**: Calls `focus::focus_previous()` to move to previous focusable component
pub fn setup_global_keys(running: Arc<AtomicBool>) -> GlobalKeysHandle {
    // Ctrl+C - Graceful shutdown
    // Use global handler to check for Ctrl modifier
    let running_clone = running.clone();
    let ctrl_c_cleanup = keyboard::on(move |event| {
        if event.modifiers.ctrl && event.key == "c" {
            running_clone.store(false, Ordering::SeqCst);
            true // Consume
        } else {
            false
        }
    });

    // Shift+Tab - Focus previous
    // Must register before Tab handler so it can check shift modifier first
    let shift_tab_cleanup = keyboard::on(move |event| {
        if event.key == "Tab" && event.modifiers.shift {
            focus::focus_previous();
            true // Consume
        } else {
            false
        }
    });

    // Tab - Focus next (plain Tab without Shift)
    let tab_cleanup = keyboard::on(move |event| {
        if event.key == "Tab" && !event.modifiers.shift {
            focus::focus_next();
            true // Consume
        } else {
            false
        }
    });

    // Scroll keys - Arrow keys, PageUp/Down, Ctrl+Home/End
    // These only activate when focused component is scrollable
    // Note: Arrow keys should NOT conflict with input navigation (input handlers
    // are registered per-component and have priority)
    let scroll_cleanup = keyboard::on(move |event| {
        // Arrow keys for scrolling (without modifiers, or with just shift)
        // Plain arrow keys scroll focused scrollable
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
    });

    GlobalKeysHandle {
        ctrl_c_cleanup: Some(Box::new(ctrl_c_cleanup)),
        tab_cleanup: Some(Box::new(tab_cleanup)),
        shift_tab_cleanup: Some(Box::new(shift_tab_cleanup)),
        scroll_cleanup: Some(Box::new(scroll_cleanup)),
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

    fn setup() {
        reset_registry();
        reset_focus_state();
        reset_keyboard_state();
    }

    #[test]
    fn test_ctrl_c_sets_running_false() {
        setup();

        let running = Arc::new(AtomicBool::new(true));
        let handle = setup_global_keys(running.clone());

        assert!(running.load(Ordering::SeqCst));

        // Dispatch Ctrl+C
        let event = KeyboardEvent::with_modifiers("c", Modifiers::ctrl());
        keyboard::dispatch(event);

        assert!(!running.load(Ordering::SeqCst));

        handle.cleanup();
    }

    #[test]
    fn test_regular_c_does_not_stop() {
        setup();

        let running = Arc::new(AtomicBool::new(true));
        let handle = setup_global_keys(running.clone());

        // Dispatch plain 'c' (no Ctrl)
        let event = KeyboardEvent::new("c");
        keyboard::dispatch(event);

        // Should still be running
        assert!(running.load(Ordering::SeqCst));

        handle.cleanup();
    }

    #[test]
    fn test_cleanup_removes_handlers() {
        setup();

        let running = Arc::new(AtomicBool::new(true));
        let handle = setup_global_keys(running.clone());

        // Clean up handlers
        handle.cleanup();

        // Now Ctrl+C should not affect running
        let event = KeyboardEvent::with_modifiers("c", Modifiers::ctrl());
        keyboard::dispatch(event);

        assert!(running.load(Ordering::SeqCst));
    }

    #[test]
    fn test_tab_calls_focus_next() {
        setup();

        // Create focusable components
        use crate::primitives::{box_primitive, BoxProps};

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
        let handle = setup_global_keys(running);

        // Dispatch Tab
        let event = KeyboardEvent::new("Tab");
        keyboard::dispatch(event);

        // Should have moved to next component
        assert_eq!(focus::get_focused_index(), 1);

        handle.cleanup();
    }

    #[test]
    fn test_shift_tab_calls_focus_previous() {
        setup();

        // Create focusable components
        use crate::primitives::{box_primitive, BoxProps};

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
        let handle = setup_global_keys(running);

        // Dispatch Shift+Tab
        let event = KeyboardEvent::with_modifiers("Tab", Modifiers::shift());
        keyboard::dispatch(event);

        // Should have moved to previous component
        assert_eq!(focus::get_focused_index(), 0);

        handle.cleanup();
    }
}
