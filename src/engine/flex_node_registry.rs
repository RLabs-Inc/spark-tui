//! FlexNode Registry - Manages FlexNode lifecycle (creation/destruction).
//!
//! Each component gets one persistent FlexNode that lives for the component's
//! entire lifetime. This registry tracks the index → FlexNode mapping.
//!
//! FlexNodes are created by primitives (box, text, input) after `allocate_index()`,
//! and destroyed by `release_index()` cleanup.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::flex_node::FlexNode;

// =============================================================================
// Registry State
// =============================================================================

thread_local! {
    /// Map component index → FlexNode
    static FLEX_NODES: RefCell<HashMap<usize, Rc<FlexNode>>> = RefCell::new(HashMap::new());
}

// =============================================================================
// FlexNode Lifecycle
// =============================================================================

/// Create a FlexNode for the given component index.
///
/// Called by primitives (box, text, input) after `allocate_index()`.
///
/// # Arguments
/// * `index` - Component index from `allocate_index()`
///
/// # Returns
/// The created (or existing) FlexNode wrapped in Rc for shared access.
pub fn create_flex_node(index: usize) -> Rc<FlexNode> {
    FLEX_NODES.with(|nodes| {
        let mut nodes = nodes.borrow_mut();

        // Check if already exists (shouldn't happen, but defensive)
        if let Some(node) = nodes.get(&index) {
            return node.clone();
        }

        // Create new node
        let node = Rc::new(FlexNode::new(index));
        nodes.insert(index, node.clone());
        node
    })
}

/// Destroy FlexNode and clean up.
///
/// Called by `release_index()` cleanup in registry.
/// This disconnects all slot sources and removes the node from the registry.
///
/// # Arguments
/// * `index` - Component index being released
pub fn destroy_flex_node(index: usize) {
    FLEX_NODES.with(|nodes| {
        if let Some(node) = nodes.borrow_mut().remove(&index) {
            // Disconnect all slot sources (breaks reactive connections)
            node.disconnect();
        }
    });
}

/// Get FlexNode for a component index.
///
/// # Arguments
/// * `index` - Component index
///
/// # Returns
/// FlexNode if it exists, None otherwise
pub fn get_flex_node(index: usize) -> Option<Rc<FlexNode>> {
    FLEX_NODES.with(|nodes| nodes.borrow().get(&index).cloned())
}

/// Get all FlexNodes (for iteration during layout).
///
/// Note: Use `get_allocated_indices()` from registry to get the reactive set
/// of allocated component indices. This just returns FlexNodes for those indices.
///
/// # Returns
/// Vector of (index, FlexNode) pairs
pub fn get_all_flex_nodes() -> Vec<(usize, Rc<FlexNode>)> {
    FLEX_NODES.with(|nodes| {
        nodes.borrow()
            .iter()
            .map(|(&index, node)| (index, node.clone()))
            .collect()
    })
}

/// Reset all FlexNodes (for testing).
///
/// Disconnects all nodes and clears the registry.
pub fn reset_flex_nodes() {
    FLEX_NODES.with(|nodes| {
        let mut nodes = nodes.borrow_mut();

        // Disconnect all nodes first
        for node in nodes.values() {
            node.disconnect();
        }

        nodes.clear();
    });
}

/// Get the number of FlexNodes currently in the registry.
pub fn flex_node_count() -> usize {
    FLEX_NODES.with(|nodes| nodes.borrow().len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Dimension;

    #[test]
    fn test_create_flex_node() {
        reset_flex_nodes();

        let node = create_flex_node(0);
        assert_eq!(node.index, 0);
        assert_eq!(flex_node_count(), 1);

        // Creating again should return the same node
        let node2 = create_flex_node(0);
        assert_eq!(Rc::as_ptr(&node), Rc::as_ptr(&node2));
    }

    #[test]
    fn test_get_flex_node() {
        reset_flex_nodes();

        assert!(get_flex_node(0).is_none());

        let node = create_flex_node(0);
        let retrieved = get_flex_node(0).unwrap();
        assert_eq!(Rc::as_ptr(&node), Rc::as_ptr(&retrieved));
    }

    #[test]
    fn test_destroy_flex_node() {
        reset_flex_nodes();

        let node = create_flex_node(0);
        node.width.set_value(Dimension::Cells(100));

        assert_eq!(flex_node_count(), 1);
        destroy_flex_node(0);
        assert_eq!(flex_node_count(), 0);
        assert!(get_flex_node(0).is_none());
    }

    #[test]
    fn test_get_all_flex_nodes() {
        reset_flex_nodes();

        create_flex_node(0);
        create_flex_node(5);
        create_flex_node(10);

        let all = get_all_flex_nodes();
        assert_eq!(all.len(), 3);

        let indices: Vec<usize> = all.iter().map(|(i, _)| *i).collect();
        assert!(indices.contains(&0));
        assert!(indices.contains(&5));
        assert!(indices.contains(&10));
    }
}
