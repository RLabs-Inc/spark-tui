//! Component Registry - Index allocation for parallel arrays.
//!
//! Manages the lifecycle of component indices:
//! - ID ↔ Index bidirectional mapping
//! - Free index pool for O(1) reuse
//! - ReactiveSet for allocatedIndices (deriveds react to add/remove)
//! - Parent context stack for nested component creation

use std::cell::RefCell;
use std::collections::HashMap;
use spark_signals::ReactiveSet;

use super::arrays;
use super::flex_node_registry;

// =============================================================================
// Registry State
// =============================================================================

thread_local! {
    /// Map component ID to array index.
    static ID_TO_INDEX: RefCell<HashMap<String, usize>> = RefCell::new(HashMap::new());

    /// Map array index to component ID.
    static INDEX_TO_ID: RefCell<HashMap<usize, String>> = RefCell::new(HashMap::new());

    /// Set of currently allocated indices (for iteration).
    /// Using ReactiveSet so deriveds that iterate over this set
    /// automatically react when components are added or removed.
    static ALLOCATED_INDICES: ReactiveSet<usize> = ReactiveSet::new();

    /// Pool of freed indices for reuse.
    static FREE_INDICES: RefCell<Vec<usize>> = RefCell::new(Vec::new());

    /// Next index to allocate if pool is empty.
    static NEXT_INDEX: RefCell<usize> = const { RefCell::new(0) };

    /// Counter for generating unique IDs.
    static ID_COUNTER: RefCell<usize> = const { RefCell::new(0) };

    /// Stack of parent indices for nested component creation.
    static PARENT_STACK: RefCell<Vec<usize>> = RefCell::new(Vec::new());

    /// Destroy callbacks registered per index.
    static DESTROY_CALLBACKS: RefCell<HashMap<usize, Vec<Box<dyn FnOnce()>>>> = RefCell::new(HashMap::new());
}

// =============================================================================
// Parent Context Stack
// =============================================================================

/// Get current parent index (-1 represented as None if at root).
pub fn get_current_parent_index() -> Option<usize> {
    PARENT_STACK.with(|stack| {
        let stack = stack.borrow();
        stack.last().copied()
    })
}

/// Push a parent index onto the stack.
pub fn push_parent_context(index: usize) {
    PARENT_STACK.with(|stack| {
        stack.borrow_mut().push(index);
    })
}

/// Pop a parent index from the stack.
pub fn pop_parent_context() {
    PARENT_STACK.with(|stack| {
        stack.borrow_mut().pop();
    })
}

// =============================================================================
// Index Allocation
// =============================================================================

/// Allocate an index for a new component.
///
/// # Arguments
/// * `id` - Optional component ID. If not provided, one is generated.
///
/// # Returns
/// The allocated index.
pub fn allocate_index(id: Option<&str>) -> usize {
    // Generate ID if not provided
    let component_id = match id {
        Some(id) => id.to_string(),
        None => {
            ID_COUNTER.with(|counter| {
                let mut counter = counter.borrow_mut();
                let id = format!("c{}", *counter);
                *counter += 1;
                id
            })
        }
    };

    // Check if already allocated
    let existing = ID_TO_INDEX.with(|map| {
        map.borrow().get(&component_id).copied()
    });
    if let Some(index) = existing {
        return index;
    }

    // Reuse free index or allocate new
    let index = FREE_INDICES.with(|free| {
        let mut free = free.borrow_mut();
        if let Some(index) = free.pop() {
            index
        } else {
            NEXT_INDEX.with(|next| {
                let mut next = next.borrow_mut();
                let index = *next;
                *next += 1;
                index
            })
        }
    });

    // Register mappings
    ID_TO_INDEX.with(|map| {
        map.borrow_mut().insert(component_id.clone(), index);
    });
    INDEX_TO_ID.with(|map| {
        map.borrow_mut().insert(index, component_id);
    });
    ALLOCATED_INDICES.with(|set| {
        set.insert(index);
    });

    // Ensure arrays have capacity for this index
    arrays::ensure_all_capacity(index);

    index
}

/// Release an index back to the pool.
///
/// Also recursively releases all children!
pub fn release_index(index: usize) {
    let id = INDEX_TO_ID.with(|map| {
        map.borrow().get(&index).cloned()
    });
    let Some(id) = id else { return };

    // FIRST: Find and release all children (recursive!)
    // We collect children first to avoid modifying while iterating
    let children: Vec<usize> = ALLOCATED_INDICES.with(|set| {
        set.iter()
            .into_iter()
            .filter(|&child_index| {
                arrays::core::get_parent_index(child_index) == Some(index)
            })
            .collect()
    });

    // Release children recursively
    for child_index in children {
        release_index(child_index);
    }

    // Run destroy callbacks before cleanup
    run_destroy_callbacks(index);

    // Destroy FlexNode if it exists
    flex_node_registry::destroy_flex_node(index);

    // Clean up mappings
    ID_TO_INDEX.with(|map| {
        map.borrow_mut().remove(&id);
    });
    INDEX_TO_ID.with(|map| {
        map.borrow_mut().remove(&index);
    });
    ALLOCATED_INDICES.with(|set| {
        set.remove(&index);
    });

    // Clear all array values at this index
    arrays::clear_all_at_index(index);

    // Return to pool for reuse
    FREE_INDICES.with(|free| {
        free.borrow_mut().push(index);
    });

    // AUTO-CLEANUP: When all components destroyed, reset all arrays to free memory
    let is_empty = ALLOCATED_INDICES.with(|set| set.is_empty());
    if is_empty {
        arrays::reset_all_arrays();
        flex_node_registry::reset_flex_nodes();
        FREE_INDICES.with(|free| {
            free.borrow_mut().clear();
        });
        NEXT_INDEX.with(|next| {
            *next.borrow_mut() = 0;
        });
    }
}

// =============================================================================
// Destroy Callbacks
// =============================================================================

/// Register a callback to run when the component at `index` is destroyed.
pub fn on_destroy(index: usize, callback: impl FnOnce() + 'static) {
    DESTROY_CALLBACKS.with(|callbacks| {
        callbacks
            .borrow_mut()
            .entry(index)
            .or_default()
            .push(Box::new(callback));
    });
}

/// Run and clear destroy callbacks for an index.
fn run_destroy_callbacks(index: usize) {
    let callbacks = DESTROY_CALLBACKS.with(|callbacks| {
        callbacks.borrow_mut().remove(&index)
    });
    if let Some(callbacks) = callbacks {
        for callback in callbacks {
            callback();
        }
    }
}

// =============================================================================
// Lookups
// =============================================================================

/// Get index for a component ID.
pub fn get_index(id: &str) -> Option<usize> {
    ID_TO_INDEX.with(|map| map.borrow().get(id).copied())
}

/// Get ID for an index.
pub fn get_id(index: usize) -> Option<String> {
    INDEX_TO_ID.with(|map| map.borrow().get(&index).cloned())
}

/// Get all currently allocated indices.
///
/// Note: This creates a reactive dependency when called from a derived/effect.
pub fn get_allocated_indices() -> Vec<usize> {
    ALLOCATED_INDICES.with(|set| set.iter())
}

/// Check if an index is currently allocated.
pub fn is_allocated(index: usize) -> bool {
    ALLOCATED_INDICES.with(|set| set.contains(&index))
}

/// Get the current capacity (highest index that would be allocated next).
pub fn get_capacity() -> usize {
    NEXT_INDEX.with(|next| *next.borrow())
}

/// Get the count of currently allocated components.
pub fn get_allocated_count() -> usize {
    ALLOCATED_INDICES.with(|set| set.len())
}

// =============================================================================
// Reset (for testing)
// =============================================================================

/// Reset all registry state (for testing).
pub fn reset_registry() {
    ID_TO_INDEX.with(|map| map.borrow_mut().clear());
    INDEX_TO_ID.with(|map| map.borrow_mut().clear());
    ALLOCATED_INDICES.with(|set| set.clear());
    FREE_INDICES.with(|free| free.borrow_mut().clear());
    NEXT_INDEX.with(|next| *next.borrow_mut() = 0);
    ID_COUNTER.with(|counter| *counter.borrow_mut() = 0);
    PARENT_STACK.with(|stack| stack.borrow_mut().clear());
    DESTROY_CALLBACKS.with(|callbacks| callbacks.borrow_mut().clear());
    flex_node_registry::reset_flex_nodes();
    arrays::reset_all_arrays();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocate_index() {
        reset_registry();

        let idx1 = allocate_index(None);
        let idx2 = allocate_index(None);
        let idx3 = allocate_index(Some("my_box"));

        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        assert_eq!(idx3, 2);

        assert!(is_allocated(0));
        assert!(is_allocated(1));
        assert!(is_allocated(2));
        assert!(!is_allocated(3));

        assert_eq!(get_allocated_count(), 3);
    }

    #[test]
    fn test_release_and_reuse() {
        reset_registry();

        let idx1 = allocate_index(None);
        let idx2 = allocate_index(None);

        release_index(idx1);
        assert!(!is_allocated(idx1));
        assert!(is_allocated(idx2));

        // Should reuse the freed index
        let idx3 = allocate_index(None);
        assert_eq!(idx3, idx1);
    }

    #[test]
    fn test_id_mapping() {
        reset_registry();

        let idx = allocate_index(Some("test_component"));
        assert_eq!(get_index("test_component"), Some(idx));
        assert_eq!(get_id(idx), Some("test_component".to_string()));
    }

    #[test]
    fn test_parent_context() {
        reset_registry();

        assert_eq!(get_current_parent_index(), None);

        push_parent_context(5);
        assert_eq!(get_current_parent_index(), Some(5));

        push_parent_context(10);
        assert_eq!(get_current_parent_index(), Some(10));

        pop_parent_context();
        assert_eq!(get_current_parent_index(), Some(5));

        pop_parent_context();
        assert_eq!(get_current_parent_index(), None);
    }

    #[test]
    fn test_destroy_callback() {
        use std::cell::Cell;
        use std::rc::Rc;

        reset_registry();

        let called = Rc::new(Cell::new(false));
        let called_clone = called.clone();

        let idx = allocate_index(None);
        on_destroy(idx, move || {
            called_clone.set(true);
        });

        assert!(!called.get());
        release_index(idx);
        assert!(called.get());
    }
}
