//! TUI Framework - Core Arrays
//!
//! The most fundamental component arrays:
//! - componentType: What kind of component (box, text, etc.)
//! - parentIndex: Parent in hierarchy
//! - visible: Is component rendered
//! - componentId: Component ID string
//!
//! Uses `TrackedSlotArray` for stable reactive cells with fine-grained tracking.
//! `componentType` is an exception - it stores values directly (not reactive).

use std::cell::RefCell;
use spark_signals::{TrackedSlotArray, tracked_slot_array};
use crate::types::ComponentType;
use super::dirty::CORE_DIRTY_SET;
use crate::engine::arrays::ClearAll;

// =============================================================================
// Arrays
// =============================================================================

thread_local! {
    /// Component type (box, text, input, etc.) - stores values directly (not reactive).
    static COMPONENT_TYPE: RefCell<Vec<ComponentType>> = RefCell::new(Vec::new());

    /// Parent component index (None for root).
    static PARENT_INDEX: TrackedSlotArray<Option<usize>> = tracked_slot_array(
        Some(None),
        CORE_DIRTY_SET.with(|s| s.clone())
    );

    /// Is component visible (false = hidden).
    static VISIBLE: TrackedSlotArray<bool> = tracked_slot_array(
        Some(true),
        CORE_DIRTY_SET.with(|s| s.clone())
    );

    /// Component ID (for debugging and lookups).
    static COMPONENT_ID: TrackedSlotArray<String> = tracked_slot_array(
        Some(String::new()),
        CORE_DIRTY_SET.with(|s| s.clone())
    );
}

// =============================================================================
// Capacity Management
// =============================================================================

/// Ensure arrays have capacity for the given index.
pub fn ensure_capacity(index: usize) {
    COMPONENT_TYPE.with(|arr| {
        let mut arr = arr.borrow_mut();
        while arr.len() <= index {
            arr.push(ComponentType::None);
        }
    });

    // TrackedSlotArray auto-expands, but we can ensure capacity
    PARENT_INDEX.with(|arr| {
        if arr.len() <= index {
            // Access to trigger auto-expansion
            let _ = arr.peek(index);
        }
    });
    VISIBLE.with(|arr| {
        if arr.len() <= index {
            let _ = arr.peek(index);
        }
    });
    COMPONENT_ID.with(|arr| {
        if arr.len() <= index {
            let _ = arr.peek(index);
        }
    });
}

/// Clear values at index (called when releasing).
pub fn clear_at_index(index: usize) {
    COMPONENT_TYPE.with(|arr| {
        let mut arr = arr.borrow_mut();
        if index < arr.len() {
            arr[index] = ComponentType::None;
        }
    });

    PARENT_INDEX.with(|arr| arr.clear(index));
    VISIBLE.with(|arr| arr.clear(index));
    COMPONENT_ID.with(|arr| arr.clear(index));
}

/// Reset all arrays.
pub fn reset() {
    COMPONENT_TYPE.with(|arr| arr.borrow_mut().clear());
    PARENT_INDEX.with(|arr| arr.clear_all());
    VISIBLE.with(|arr| arr.clear_all());
    COMPONENT_ID.with(|arr| arr.clear_all());
}

// =============================================================================
// Component Type
// =============================================================================

/// Get component type at index.
pub fn get_component_type(index: usize) -> ComponentType {
    COMPONENT_TYPE.with(|arr| {
        arr.borrow().get(index).copied().unwrap_or(ComponentType::None)
    })
}

/// Set component type at index.
pub fn set_component_type(index: usize, value: ComponentType) {
    COMPONENT_TYPE.with(|arr| {
        let mut arr = arr.borrow_mut();
        while arr.len() <= index {
            arr.push(ComponentType::None);
        }
        arr[index] = value;
    });
}

// =============================================================================
// Parent Index
// =============================================================================

/// Get parent index at index (reactive - tracks this index).
pub fn get_parent_index(index: usize) -> Option<usize> {
    PARENT_INDEX.with(|arr| arr.get(index)).flatten()
}

/// Set parent index at index.
pub fn set_parent_index(index: usize, parent: Option<usize>) {
    PARENT_INDEX.with(|arr| arr.set_value(index, parent));
}

/// Set parent index from a getter function.
pub fn set_parent_index_getter<F>(index: usize, getter: F)
where
    F: Fn() -> Option<usize> + 'static,
{
    PARENT_INDEX.with(|arr| arr.set_getter(index, getter));
}

// =============================================================================
// Visible
// =============================================================================

/// Get visibility at index (reactive - tracks this index).
pub fn get_visible(index: usize) -> bool {
    VISIBLE.with(|arr| arr.get(index)).unwrap_or(true)
}

/// Set visibility at index.
pub fn set_visible(index: usize, visible: bool) {
    VISIBLE.with(|arr| arr.set_value(index, visible));
}

/// Set visibility from a getter function.
pub fn set_visible_getter<F>(index: usize, getter: F)
where
    F: Fn() -> bool + 'static,
{
    VISIBLE.with(|arr| arr.set_getter(index, getter));
}

/// Set visibility from a signal.
pub fn set_visible_signal(index: usize, sig: spark_signals::Signal<bool>) {
    VISIBLE.with(|arr| arr.set_signal(index, &sig));
}

// =============================================================================
// Component ID
// =============================================================================

/// Get component ID at index (reactive - tracks this index).
pub fn get_component_id(index: usize) -> String {
    COMPONENT_ID.with(|arr| arr.get(index)).unwrap_or_default()
}

/// Set component ID at index.
pub fn set_component_id(index: usize, id: String) {
    COMPONENT_ID.with(|arr| arr.set_value(index, id));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() {
        reset();
    }

    #[test]
    fn test_component_type() {
        setup();

        set_component_type(0, ComponentType::Box);
        set_component_type(1, ComponentType::Text);

        assert_eq!(get_component_type(0), ComponentType::Box);
        assert_eq!(get_component_type(1), ComponentType::Text);
        assert_eq!(get_component_type(99), ComponentType::None);
    }

    #[test]
    fn test_parent_index() {
        setup();

        set_parent_index(1, Some(0));
        set_parent_index(2, Some(0));
        set_parent_index(3, Some(1));

        assert_eq!(get_parent_index(0), None);
        assert_eq!(get_parent_index(1), Some(0));
        assert_eq!(get_parent_index(2), Some(0));
        assert_eq!(get_parent_index(3), Some(1));
    }

    #[test]
    fn test_visible() {
        setup();

        // Default is true
        assert!(get_visible(0));

        set_visible(0, false);
        assert!(!get_visible(0));

        set_visible(0, true);
        assert!(get_visible(0));
    }

    #[test]
    fn test_clear_at_index() {
        setup();

        set_component_type(0, ComponentType::Box);
        set_parent_index(0, Some(5));
        set_visible(0, false);

        clear_at_index(0);

        assert_eq!(get_component_type(0), ComponentType::None);
        assert_eq!(get_parent_index(0), None);
        assert!(get_visible(0)); // Reset to default (true)
    }
}
