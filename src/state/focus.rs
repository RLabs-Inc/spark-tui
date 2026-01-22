//! Focus System - Keyboard navigation and focus state
//!
//! Manages focus state and navigation:
//! - `focused_index` signal (currently focused component)
//! - Focus cycling (Tab/Shift+Tab)
//! - Focus trapping for modals
//! - Focus history for restoration
//! - Focus callbacks (onFocus/onBlur)
//!
//! # Example
//!
//! ```ignore
//! use spark_tui::state::focus;
//!
//! // Navigate with Tab
//! focus::focus_next();
//! focus::focus_previous();
//!
//! // Focus specific component
//! focus::focus(component_index);
//!
//! // Register callbacks
//! let cleanup = focus::register_callbacks(index, FocusCallbacks {
//!     on_focus: Some(Box::new(|| println!("Focused!"))),
//!     on_blur: Some(Box::new(|| println!("Blurred!"))),
//! });
//! ```

use std::cell::RefCell;
use std::collections::HashMap;
use spark_signals::{signal, Signal};
use crate::engine::arrays::{core, interaction};
use crate::engine::{get_allocated_indices, get_id};

// =============================================================================
// FOCUSED INDEX SIGNAL
// =============================================================================

thread_local! {
    static FOCUSED_INDEX: Signal<i32> = signal(-1);
}

/// Get the currently focused component index (-1 if none)
pub fn get_focused_index() -> i32 {
    FOCUSED_INDEX.with(|s| s.get())
}

/// Check if any component is focused
pub fn has_focus() -> bool {
    get_focused_index() >= 0
}

/// Check if specific component is focused
pub fn is_focused(index: usize) -> bool {
    get_focused_index() == index as i32
}

// =============================================================================
// FOCUS CALLBACKS
// =============================================================================

/// Callbacks fired when focus changes
pub struct FocusCallbacks {
    pub on_focus: Option<Box<dyn Fn()>>,
    pub on_blur: Option<Box<dyn Fn()>>,
}

impl Default for FocusCallbacks {
    fn default() -> Self {
        Self {
            on_focus: None,
            on_blur: None,
        }
    }
}

thread_local! {
    // Multiple callbacks per index supported (cursor blink + user callback)
    static FOCUS_CALLBACK_REGISTRY: RefCell<HashMap<usize, Vec<FocusCallbacks>>> = RefCell::new(HashMap::new());
}

/// Register focus callbacks for a component.
/// Returns cleanup function to unregister.
pub fn register_callbacks(index: usize, callbacks: FocusCallbacks) -> impl FnOnce() {
    let callback_id = FOCUS_CALLBACK_REGISTRY.with(|reg| {
        let mut reg = reg.borrow_mut();
        let list = reg.entry(index).or_insert_with(Vec::new);
        let id = list.len();
        list.push(callbacks);
        id
    });

    move || {
        FOCUS_CALLBACK_REGISTRY.with(|reg| {
            let mut reg = reg.borrow_mut();
            if let Some(list) = reg.get_mut(&index) {
                if callback_id < list.len() {
                    // Mark as removed (can't easily remove from Vec while preserving IDs)
                    list[callback_id].on_focus = None;
                    list[callback_id].on_blur = None;
                }
                // Clean up if all callbacks removed
                if list.iter().all(|cb| cb.on_focus.is_none() && cb.on_blur.is_none()) {
                    reg.remove(&index);
                }
            }
        });
    }
}

/// Internal: Set focus and fire callbacks at the source
fn set_focus_with_callbacks(new_index: i32) {
    let old_index = get_focused_index();

    // No change, no callbacks
    if old_index == new_index {
        return;
    }

    // Fire onBlur for all callbacks on old focus
    if old_index >= 0 {
        FOCUS_CALLBACK_REGISTRY.with(|reg| {
            let reg = reg.borrow();
            if let Some(callbacks) = reg.get(&(old_index as usize)) {
                for cb in callbacks {
                    if let Some(ref on_blur) = cb.on_blur {
                        on_blur();
                    }
                }
            }
        });
    }

    // Update reactive state
    FOCUSED_INDEX.with(|s| s.set(new_index));

    // Fire onFocus for all callbacks on new focus
    if new_index >= 0 {
        FOCUS_CALLBACK_REGISTRY.with(|reg| {
            let reg = reg.borrow();
            if let Some(callbacks) = reg.get(&(new_index as usize)) {
                for cb in callbacks {
                    if let Some(ref on_focus) = cb.on_focus {
                        on_focus();
                    }
                }
            }
        });
    }
}

// =============================================================================
// FOCUS TRAP (for modals/dialogs)
// =============================================================================

thread_local! {
    static FOCUS_TRAP_STACK: RefCell<Vec<usize>> = RefCell::new(Vec::new());
}

/// Push a focus trap - focus will be contained within this component's children
pub fn push_focus_trap(container_index: usize) {
    FOCUS_TRAP_STACK.with(|stack| {
        stack.borrow_mut().push(container_index);
    });
}

/// Pop the current focus trap
pub fn pop_focus_trap() -> Option<usize> {
    FOCUS_TRAP_STACK.with(|stack| {
        stack.borrow_mut().pop()
    })
}

/// Check if focus is currently trapped
pub fn is_focus_trapped() -> bool {
    FOCUS_TRAP_STACK.with(|stack| !stack.borrow().is_empty())
}

/// Get the current focus trap container
pub fn get_focus_trap_container() -> Option<usize> {
    FOCUS_TRAP_STACK.with(|stack| {
        stack.borrow().last().copied()
    })
}

// =============================================================================
// FOCUS HISTORY (for restoration)
// =============================================================================

#[derive(Clone)]
struct FocusHistoryEntry {
    index: usize,
    id: Option<String>,
}

thread_local! {
    static FOCUS_HISTORY: RefCell<Vec<FocusHistoryEntry>> = RefCell::new(Vec::new());
}

const MAX_HISTORY: usize = 10;

/// Save current focus to history
pub fn save_focus_to_history() {
    let current = get_focused_index();
    if current >= 0 {
        let index = current as usize;
        let id = get_id(index);
        FOCUS_HISTORY.with(|history| {
            let mut history = history.borrow_mut();
            history.push(FocusHistoryEntry { index, id });
            if history.len() > MAX_HISTORY {
                history.remove(0);
            }
        });
    }
}

/// Restore focus from history
pub fn restore_focus_from_history() -> bool {
    loop {
        let entry = FOCUS_HISTORY.with(|history| {
            history.borrow_mut().pop()
        });

        match entry {
            None => return false,
            Some(entry) => {
                // Verify the index hasn't been recycled for a different component
                if get_id(entry.index) != entry.id {
                    continue;
                }
                // Check if component is still valid and focusable
                let is_visible = core::get_visible(entry.index);
                let is_focusable = interaction::get_focusable(entry.index);
                if is_focusable && is_visible {
                    set_focus_with_callbacks(entry.index as i32);
                    return true;
                }
            }
        }
    }
}

// =============================================================================
// FOCUSABLE QUERIES
// =============================================================================

/// Get all focusable component indices, sorted by tabIndex
pub fn get_focusable_indices() -> Vec<usize> {
    let indices = get_allocated_indices();
    let mut result: Vec<usize> = Vec::new();

    for i in indices {
        let is_focusable = interaction::get_focusable(i);
        let is_visible = core::get_visible(i);
        if is_focusable && is_visible {
            result.push(i);
        }
    }

    // Sort by tabIndex (components with same tabIndex keep allocation order)
    result.sort_by(|&a, &b| {
        let tab_a = interaction::get_tab_index(a);
        let tab_b = interaction::get_tab_index(b);
        if tab_a != tab_b {
            tab_a.cmp(&tab_b)
        } else {
            a.cmp(&b)
        }
    });

    result
}

// =============================================================================
// FOCUS NAVIGATION
// =============================================================================

/// Find next focusable component
fn find_next_focusable(from_index: i32, direction: i32) -> i32 {
    let focusables = get_focusable_indices();

    // Apply focus trap if active (filter to children of trap container)
    // TODO: Implement proper filtering to children of trap container

    if focusables.is_empty() {
        return -1;
    }

    let current_pos = if from_index >= 0 {
        focusables.iter().position(|&i| i == from_index as usize)
    } else {
        None
    };

    match current_pos {
        None => {
            // Not currently focused on a focusable
            if direction == 1 {
                focusables[0] as i32
            } else {
                focusables[focusables.len() - 1] as i32
            }
        }
        Some(pos) => {
            // Move in direction with wrap
            let len = focusables.len() as i32;
            let next_pos = ((pos as i32 + direction) % len + len) % len;
            focusables[next_pos as usize] as i32
        }
    }
}

/// Move focus to next focusable component
pub fn focus_next() -> bool {
    let current = get_focused_index();
    let next = find_next_focusable(current, 1);
    if next != -1 && next != current {
        save_focus_to_history();
        set_focus_with_callbacks(next);
        return true;
    }
    false
}

/// Move focus to previous focusable component
pub fn focus_previous() -> bool {
    let current = get_focused_index();
    let prev = find_next_focusable(current, -1);
    if prev != -1 && prev != current {
        save_focus_to_history();
        set_focus_with_callbacks(prev);
        return true;
    }
    false
}

/// Focus a specific component by index
pub fn focus(index: usize) -> bool {
    let is_visible = core::get_visible(index);
    let is_focusable = interaction::get_focusable(index);

    if is_focusable && is_visible {
        let current = get_focused_index();
        if current != index as i32 {
            save_focus_to_history();
            set_focus_with_callbacks(index as i32);
        }
        return true;
    }
    false
}

/// Clear focus (no component focused)
pub fn blur() {
    if get_focused_index() >= 0 {
        save_focus_to_history();
        set_focus_with_callbacks(-1);
    }
}

/// Focus the first focusable component
pub fn focus_first() -> bool {
    let focusables = get_focusable_indices();
    if !focusables.is_empty() {
        return focus(focusables[0]);
    }
    false
}

/// Focus the last focusable component
pub fn focus_last() -> bool {
    let focusables = get_focusable_indices();
    if !focusables.is_empty() {
        return focus(focusables[focusables.len() - 1]);
    }
    false
}

// =============================================================================
// RESET (for testing)
// =============================================================================

/// Reset all focus state (for testing)
pub fn reset_focus_state() {
    set_focus_with_callbacks(-1);
    FOCUS_TRAP_STACK.with(|stack| stack.borrow_mut().clear());
    FOCUS_HISTORY.with(|history| history.borrow_mut().clear());
    FOCUS_CALLBACK_REGISTRY.with(|reg| reg.borrow_mut().clear());
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::reset_registry;
    use crate::primitives::{box_primitive, BoxProps};
    use std::cell::Cell;
    use std::rc::Rc;

    fn setup() {
        reset_registry();
        reset_focus_state();
    }

    #[test]
    fn test_initial_state() {
        setup();
        assert_eq!(get_focused_index(), -1);
        assert!(!has_focus());
    }

    #[test]
    fn test_focus_single_component() {
        setup();

        // Create a focusable box
        let _cleanup = box_primitive(BoxProps {
            focusable: Some(true),
            tab_index: Some(0),
            ..Default::default()
        });

        assert!(focus(0));
        assert_eq!(get_focused_index(), 0);
        assert!(has_focus());
        assert!(is_focused(0));
    }

    #[test]
    fn test_focus_non_focusable() {
        setup();

        // Create a non-focusable box
        let _cleanup = box_primitive(BoxProps::default());

        assert!(!focus(0));
        assert_eq!(get_focused_index(), -1);
    }

    #[test]
    fn test_focus_next_previous() {
        setup();

        // Create three focusable boxes
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
        let _c3 = box_primitive(BoxProps {
            focusable: Some(true),
            tab_index: Some(3),
            ..Default::default()
        });

        // Focus first
        assert!(focus_first());
        assert_eq!(get_focused_index(), 0);

        // Tab forward
        assert!(focus_next());
        assert_eq!(get_focused_index(), 1);

        assert!(focus_next());
        assert_eq!(get_focused_index(), 2);

        // Wrap around
        assert!(focus_next());
        assert_eq!(get_focused_index(), 0);

        // Tab backward
        assert!(focus_previous());
        assert_eq!(get_focused_index(), 2);
    }

    #[test]
    fn test_focus_callbacks() {
        setup();

        let focus_count = Rc::new(Cell::new(0));
        let blur_count = Rc::new(Cell::new(0));

        let _c1 = box_primitive(BoxProps {
            focusable: Some(true),
            ..Default::default()
        });
        let _c2 = box_primitive(BoxProps {
            focusable: Some(true),
            ..Default::default()
        });

        let focus_count_clone = focus_count.clone();
        let blur_count_clone = blur_count.clone();

        let _cleanup = register_callbacks(0, FocusCallbacks {
            on_focus: Some(Box::new(move || {
                focus_count_clone.set(focus_count_clone.get() + 1);
            })),
            on_blur: Some(Box::new(move || {
                blur_count_clone.set(blur_count_clone.get() + 1);
            })),
        });

        // Focus component 0
        focus(0);
        assert_eq!(focus_count.get(), 1);
        assert_eq!(blur_count.get(), 0);

        // Focus component 1 (blurs 0)
        focus(1);
        assert_eq!(focus_count.get(), 1);
        assert_eq!(blur_count.get(), 1);

        // Focus back to 0
        focus(0);
        assert_eq!(focus_count.get(), 2);
        assert_eq!(blur_count.get(), 1);
    }

    #[test]
    fn test_blur() {
        setup();

        let _c1 = box_primitive(BoxProps {
            focusable: Some(true),
            ..Default::default()
        });

        focus(0);
        assert!(has_focus());

        blur();
        assert!(!has_focus());
        assert_eq!(get_focused_index(), -1);
    }

    #[test]
    fn test_focus_history() {
        setup();

        let _c1 = box_primitive(BoxProps {
            focusable: Some(true),
            ..Default::default()
        });
        let _c2 = box_primitive(BoxProps {
            focusable: Some(true),
            ..Default::default()
        });

        // focus(0) - no previous, just sets focus to 0
        // focus(1) - saves 0 to history, sets focus to 1
        // blur() - saves 1 to history, clears focus
        // History is now: [0, 1]

        focus(0);
        focus(1);
        assert_eq!(get_focused_index(), 1);

        blur();
        assert_eq!(get_focused_index(), -1);

        // restore_focus_from_history pops from end: gets 1 (most recent)
        assert!(restore_focus_from_history());
        assert_eq!(get_focused_index(), 1);

        // History is now: [0]
        // Manually clear (don't blur which would save 1 again)
        reset_focus_state();

        // Create components again and test history accumulation
        let _c1 = box_primitive(BoxProps {
            focusable: Some(true),
            ..Default::default()
        });

        focus(0);
        blur();
        // History: [0]

        assert!(restore_focus_from_history());
        assert_eq!(get_focused_index(), 0);
    }

    #[test]
    fn test_focus_trap() {
        setup();

        assert!(!is_focus_trapped());

        push_focus_trap(0);
        assert!(is_focus_trapped());
        assert_eq!(get_focus_trap_container(), Some(0));

        push_focus_trap(1);
        assert_eq!(get_focus_trap_container(), Some(1));

        assert_eq!(pop_focus_trap(), Some(1));
        assert_eq!(get_focus_trap_container(), Some(0));

        assert_eq!(pop_focus_trap(), Some(0));
        assert!(!is_focus_trapped());
    }

    #[test]
    fn test_tab_index_ordering() {
        setup();

        // Create boxes with non-sequential tab indices
        let _c1 = box_primitive(BoxProps {
            focusable: Some(true),
            tab_index: Some(30),
            ..Default::default()
        });
        let _c2 = box_primitive(BoxProps {
            focusable: Some(true),
            tab_index: Some(10),
            ..Default::default()
        });
        let _c3 = box_primitive(BoxProps {
            focusable: Some(true),
            tab_index: Some(20),
            ..Default::default()
        });

        let focusables = get_focusable_indices();
        // Should be sorted by tab_index: 1 (10), 2 (20), 0 (30)
        assert_eq!(focusables, vec![1, 2, 0]);

        // Focus first (should be index 1 with tab_index 10)
        focus_first();
        assert_eq!(get_focused_index(), 1);

        // Next should be index 2 (tab_index 20)
        focus_next();
        assert_eq!(get_focused_index(), 2);

        // Next should be index 0 (tab_index 30)
        focus_next();
        assert_eq!(get_focused_index(), 0);
    }
}
