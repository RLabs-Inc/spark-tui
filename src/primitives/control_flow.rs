//! Control Flow Primitives - Conditional and list rendering.
//!
//! This module provides control flow primitives for dynamic UI:
//! - [`show`] - Conditional rendering based on reactive conditions
//! - [`each`] - List rendering with fine-grained updates
//! - [`when`] - Async handling with pending/then/catch states (TODO: Plan 03)
//!
//! # Pattern: EffectScope-based Cleanup
//!
//! All control flow primitives use spark-signals' EffectScope for cleanup:
//! 1. Create an EffectScope to manage the lifetime of child effects/components
//! 2. Run rendering logic inside `scope.run()`
//! 3. Register cleanup with `on_scope_dispose()`
//! 4. Return `Box::new(move || scope.stop())` as the Cleanup
//!
//! # Pattern: Parent Context Restoration
//!
//! When components are created inside control flow, the parent context must
//! be correct. The `show()` function captures the parent index at creation
//! time and restores it via `push_parent_context()` before rendering children.
//!
//! ```ignore
//! // Parent context is captured when show() is called
//! box_primitive(BoxProps {
//!     children: Some(Box::new(|| {
//!         // show() called here - captures parent = this box
//!         show(
//!             move || condition.get(),
//!             || text(TextProps { content: "Visible!".into(), ..Default::default() }),
//!             None::<fn() -> Cleanup>,
//!         );
//!     })),
//!     ..Default::default()
//! });
//! ```
//!
//! # Component Lifecycle
//!
//! ## show()
//! - When condition becomes true: `then_fn` is called, component created
//! - When condition becomes false: previous cleanup runs, component destroyed
//! - If `else_fn` provided: it renders when condition is false
//! - On show() cleanup: current branch cleaned up, scope stopped
//!
//! ## each()
//! - Items tracked by key (from `key_fn`)
//! - New keys: create signal + render component
//! - Existing keys: update signal (NO component recreation!)
//! - Removed keys: cleanup + destroy component
//! - On each() cleanup: all items cleaned up, scope stopped

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::rc::Rc;

use spark_signals::{effect, effect_scope, on_scope_dispose, signal, Signal};

use crate::engine::{get_current_parent_index, pop_parent_context, push_parent_context};
use crate::primitives::Cleanup;

/// Conditionally render components based on a reactive condition.
///
/// Creates and destroys components when the condition changes. The condition
/// getter establishes a reactive dependency, so the UI automatically updates.
///
/// # Arguments
///
/// * `condition` - Getter that returns boolean (creates reactive dependency)
/// * `then_fn` - Function to render when condition is true (returns cleanup)
/// * `else_fn` - Optional function to render when condition is false
///
/// # Returns
///
/// A cleanup function that destroys the current branch and stops tracking.
///
/// # Example
///
/// ```ignore
/// use spark_tui::primitives::{show, text, TextProps, Cleanup};
/// use spark_signals::signal;
///
/// let is_visible = signal(true);
/// let is_visible_clone = is_visible.clone();
///
/// let cleanup = show(
///     move || is_visible_clone.get(),
///     || text(TextProps { content: "Visible!".into(), ..Default::default() }),
///     Some(|| text(TextProps { content: "Hidden replacement".into(), ..Default::default() })),
/// );
///
/// // Toggle visibility
/// is_visible.set(false); // "Visible!" destroyed, "Hidden replacement" created
///
/// // Cleanup everything
/// cleanup();
/// ```
///
/// # Without else branch
///
/// ```ignore
/// let cleanup = show(
///     move || condition.get(),
///     || create_component(),
///     None::<fn() -> Cleanup>, // Type hint needed for None
/// );
/// ```
pub fn show<ThenF, ElseF, ThenR, ElseR>(
    condition: impl Fn() -> bool + 'static,
    then_fn: ThenF,
    else_fn: Option<ElseF>,
) -> Cleanup
where
    ThenF: Fn() -> ThenR + 'static,
    ElseF: Fn() -> ElseR + 'static,
    ThenR: Into<Cleanup>,
    ElseR: Into<Cleanup>,
{
    // Capture parent index at creation time - this ensures components
    // created inside show() have the correct parent
    let parent_index = get_current_parent_index();

    // Storage for current cleanup and condition state
    let cleanup: Rc<RefCell<Option<Cleanup>>> = Rc::new(RefCell::new(None));
    let was_true: Rc<Cell<Option<bool>>> = Rc::new(Cell::new(None));

    // Create scope for cleanup management
    let scope = effect_scope();

    // Clone Rcs for move into closures
    let cleanup_for_update = cleanup.clone();
    let cleanup_for_dispose = cleanup.clone();

    // Update function - runs when condition changes
    let update = move |new_condition: bool| {
        let previous = was_true.get();

        // Skip if condition unchanged
        if previous == Some(new_condition) {
            return;
        }
        was_true.set(Some(new_condition));

        // Cleanup previous branch
        if let Some(prev_cleanup) = cleanup_for_update.borrow_mut().take() {
            prev_cleanup();
        }

        // Render new branch with correct parent context
        if let Some(parent) = parent_index {
            push_parent_context(parent);
        }

        let new_cleanup = if new_condition {
            Some(then_fn().into())
        } else {
            else_fn.as_ref().map(|f| f().into())
        };

        if parent_index.is_some() {
            pop_parent_context();
        }

        *cleanup_for_update.borrow_mut() = new_cleanup;
    };

    // Run inside scope
    scope.run(move || {
        // Effect reads condition to establish dependency
        // Initial render happens on first effect run
        // Note: The effect is registered with the scope and will be cleaned up
        // when scope.stop() is called. We use _ to suppress the unused warning.
        let _effect_cleanup = effect(move || {
            let current = condition();
            update(current);
        });

        // Cleanup when scope is disposed
        on_scope_dispose(move || {
            if let Some(cleanup_fn) = cleanup_for_dispose.borrow_mut().take() {
                cleanup_fn();
            }
        });
    });

    // Return cleanup that stops the scope
    Box::new(move || {
        scope.stop();
    })
}

// =============================================================================
// each() - List rendering with fine-grained reactivity
// =============================================================================

/// Render a list of components reactively with fine-grained updates.
///
/// Creates one component per item, tracked by unique keys. When the list changes:
/// - New items: create signal + component
/// - Existing items: update signal only (NO component recreation!)
/// - Removed items: cleanup + destroy component
///
/// # Arguments
///
/// * `items_getter` - Getter that returns the items (creates reactive dependency)
/// * `render_fn` - Function receiving (getItem getter, key) that renders one item
/// * `key_fn` - Function to extract unique key from each item
///
/// # Returns
///
/// A cleanup function that destroys all components and stops tracking.
///
/// # Example
///
/// ```ignore
/// use spark_tui::primitives::{each, text, TextProps, Cleanup};
/// use spark_signals::signal;
///
/// let items = signal(vec!["apple", "banana", "cherry"]);
/// let items_clone = items.clone();
///
/// let cleanup = each(
///     move || items_clone.get(),
///     |get_item, key| {
///         text(TextProps {
///             content: (move || get_item().to_string()).into(),
///             ..Default::default()
///         })
///     },
///     |item| item.to_string(),
/// );
///
/// // Add item - only creates 1 new component
/// items.set(vec!["apple", "banana", "cherry", "date"]);
///
/// // Remove item - only destroys 1 component
/// items.set(vec!["apple", "cherry", "date"]);
///
/// // Reorder - no component recreation (same keys)
/// items.set(vec!["date", "apple", "cherry"]);
///
/// // Cleanup everything
/// cleanup();
/// ```
///
/// # Fine-grained Item Updates
///
/// Each item gets its own signal. When an item's value changes (but key stays same),
/// only that item's signal is updated - components using `get_item()` will react.
///
/// ```ignore
/// #[derive(Clone, PartialEq)]
/// struct Todo { id: i32, text: String, done: bool }
///
/// let todos = signal(vec![
///     Todo { id: 1, text: "First".into(), done: false },
/// ]);
///
/// let cleanup = each(
///     move || todos.get(),
///     |get_item, _key| {
///         // get_item() returns current Todo value
///         // When todo changes, this getter returns updated value
///         text(TextProps {
///             content: (move || get_item().text.clone()).into(),
///             ..Default::default()
///         })
///     },
///     |todo| todo.id.to_string(),
/// );
///
/// // Update todo text - component NOT recreated, signal updated
/// todos.set(vec![
///     Todo { id: 1, text: "Updated".into(), done: true },
/// ]);
/// ```
///
/// # Duplicate Key Handling
///
/// Duplicate keys are warned but don't crash. Only the first occurrence is tracked.
pub fn each<T, K, RenderF, R>(
    items_getter: impl Fn() -> Vec<T> + 'static,
    render_fn: RenderF,
    key_fn: impl Fn(&T) -> K + 'static,
) -> Cleanup
where
    T: Clone + PartialEq + 'static,
    K: Clone + Eq + Hash + std::fmt::Debug + 'static,
    RenderF: Fn(Rc<dyn Fn() -> T>, K) -> R + Clone + 'static,
    R: Into<Cleanup>,
{
    // Capture parent index at creation time
    let parent_index = get_current_parent_index();

    // Create scope for cleanup management
    let scope = effect_scope();

    // Maps for tracking items by key
    // Key -> Cleanup for destroying component
    let cleanups: Rc<RefCell<HashMap<K, Cleanup>>> = Rc::new(RefCell::new(HashMap::new()));
    // Key -> Signal<T> for fine-grained updates
    let item_signals: Rc<RefCell<HashMap<K, Signal<T>>>> = Rc::new(RefCell::new(HashMap::new()));

    // Clones for move into effect
    let cleanups_effect = cleanups.clone();
    let item_signals_effect = item_signals.clone();

    // Clones for move into dispose callback
    let cleanups_dispose = cleanups.clone();
    let item_signals_dispose = item_signals.clone();

    scope.run(move || {
        // Effect establishes reactive dependency on items_getter
        let _effect_cleanup = effect(move || {
            let items = items_getter();
            let mut current_keys = HashSet::new();

            // Push parent context for component creation
            if let Some(parent) = parent_index {
                push_parent_context(parent);
            }

            // Process each item
            for item in items.iter() {
                let key = key_fn(item);

                // Check for duplicate keys
                if current_keys.contains(&key) {
                    eprintln!(
                        "[spark-tui each()] Duplicate key detected: {:?}. \
                        Keys must be unique. This may cause unexpected behavior.",
                        key
                    );
                    continue; // Skip duplicate
                }
                current_keys.insert(key.clone());

                let mut signals = item_signals_effect.borrow_mut();
                let mut cleanup_map = cleanups_effect.borrow_mut();

                if signals.contains_key(&key) {
                    // EXISTING item - just update the signal (fine-grained!)
                    if let Some(sig) = signals.get(&key) {
                        sig.set(item.clone());
                    }
                } else {
                    // NEW item - create signal and component
                    let item_signal = signal(item.clone());
                    signals.insert(key.clone(), item_signal.clone());

                    // Create getter closure that reads from signal
                    let getter: Rc<dyn Fn() -> T> = Rc::new(move || item_signal.get());

                    // Render and store cleanup
                    let render_fn_clone = render_fn.clone();
                    let cleanup = render_fn_clone(getter, key.clone()).into();
                    cleanup_map.insert(key.clone(), cleanup);
                }
            }

            // Pop parent context
            if parent_index.is_some() {
                pop_parent_context();
            }

            // Cleanup removed items
            let mut signals = item_signals_effect.borrow_mut();
            let mut cleanup_map = cleanups_effect.borrow_mut();

            // Collect keys to remove (can't modify while iterating)
            let keys_to_remove: Vec<K> = cleanup_map
                .keys()
                .filter(|k| !current_keys.contains(*k))
                .cloned()
                .collect();

            for key in keys_to_remove {
                if let Some(cleanup) = cleanup_map.remove(&key) {
                    cleanup();
                }
                signals.remove(&key);
            }
        });

        // Cleanup all items when scope is disposed
        on_scope_dispose(move || {
            let mut cleanup_map = cleanups_dispose.borrow_mut();
            let mut signals = item_signals_dispose.borrow_mut();

            for cleanup in cleanup_map.drain().map(|(_, c)| c) {
                cleanup();
            }
            signals.clear();
        });
    });

    // Return cleanup that stops the scope
    Box::new(move || {
        scope.stop();
    })
}

// =============================================================================
// Placeholder types for future plans
// =============================================================================

/// State for async `when()` primitive.
///
/// Tracks the current state of an async operation.
#[derive(Debug, Clone, PartialEq)]
pub enum AsyncState<T, E> {
    /// Operation is pending.
    Pending,
    /// Operation completed successfully with value.
    Resolved(T),
    /// Operation failed with error.
    Rejected(E),
}

/// Options for the `when()` primitive.
///
/// Provides render functions for each async state.
pub struct WhenOptions<T, E, PendingF, ThenF, CatchF>
where
    PendingF: Fn() -> Cleanup,
    ThenF: Fn(&T) -> Cleanup,
    CatchF: Fn(&E) -> Cleanup,
{
    /// Render while pending.
    pub pending: Option<PendingF>,
    /// Render on success (receives resolved value).
    pub then: ThenF,
    /// Render on error (receives error).
    pub catch: Option<CatchF>,
    /// Marker for T type.
    pub _t: std::marker::PhantomData<T>,
    /// Marker for E type.
    pub _e: std::marker::PhantomData<E>,
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{allocate_index, get_allocated_count, release_index, reset_registry};
    use spark_signals::signal;

    /// Helper to create a test component that tracks allocation.
    fn create_test_component() -> Cleanup {
        let index = allocate_index(None);
        Box::new(move || release_index(index))
    }

    #[test]
    fn test_show_renders_then_when_true() {
        reset_registry();

        let condition = signal(true);
        let cond_clone = condition.clone();

        let initial_count = get_allocated_count();

        let _cleanup = show(
            move || cond_clone.get(),
            || create_test_component(),
            None::<fn() -> Cleanup>,
        );

        // Component should be created
        assert_eq!(
            get_allocated_count(),
            initial_count + 1,
            "then branch should create one component"
        );
    }

    #[test]
    fn test_show_renders_else_when_false() {
        reset_registry();

        let condition = signal(false);
        let cond_clone = condition.clone();

        let initial_count = get_allocated_count();

        let _cleanup = show(
            move || cond_clone.get(),
            || create_test_component(),
            Some(|| create_test_component()),
        );

        // Else branch component should be created
        assert_eq!(
            get_allocated_count(),
            initial_count + 1,
            "else branch should create one component"
        );
    }

    #[test]
    fn test_show_toggles_components() {
        reset_registry();

        let condition = signal(true);
        let cond_clone = condition.clone();

        let initial_count = get_allocated_count();

        let _cleanup = show(
            move || cond_clone.get(),
            || create_test_component(),
            Some(|| create_test_component()),
        );

        // Then branch active
        assert_eq!(get_allocated_count(), initial_count + 1);

        // Toggle to false - then destroyed, else created
        condition.set(false);
        assert_eq!(
            get_allocated_count(),
            initial_count + 1,
            "should still have exactly one component after toggle"
        );

        // Toggle back to true - else destroyed, then created
        condition.set(true);
        assert_eq!(
            get_allocated_count(),
            initial_count + 1,
            "should still have exactly one component after toggle back"
        );
    }

    #[test]
    fn test_show_cleanup_destroys_all() {
        reset_registry();

        let condition = signal(true);
        let cond_clone = condition.clone();

        let initial_count = get_allocated_count();

        let cleanup = show(
            move || cond_clone.get(),
            || create_test_component(),
            None::<fn() -> Cleanup>,
        );

        assert_eq!(get_allocated_count(), initial_count + 1);

        // Cleanup should destroy component
        cleanup();
        assert_eq!(
            get_allocated_count(),
            initial_count,
            "cleanup should destroy the component"
        );
    }

    #[test]
    fn test_show_no_change_no_recreate() {
        reset_registry();

        let condition = signal(true);
        let cond_clone = condition.clone();

        // Track how many times then_fn is called
        let call_count = Rc::new(Cell::new(0));
        let call_count_clone = call_count.clone();

        let _cleanup = show(
            move || cond_clone.get(),
            move || {
                call_count_clone.set(call_count_clone.get() + 1);
                create_test_component()
            },
            None::<fn() -> Cleanup>,
        );

        // Initial render
        assert_eq!(call_count.get(), 1);

        // Set to same value - should NOT re-render
        condition.set(true);
        assert_eq!(
            call_count.get(),
            1,
            "setting same value should not recreate component"
        );

        // Set to different value - should re-render
        condition.set(false);
        condition.set(true);
        assert_eq!(
            call_count.get(),
            2,
            "toggling should recreate component"
        );
    }

    #[test]
    fn test_show_nested_parent_context() {
        use crate::engine::arrays::core::set_parent_index;

        reset_registry();

        // Create a parent component
        let parent_index = allocate_index(Some("parent"));

        // Push parent context as if we're inside parent's children
        push_parent_context(parent_index);

        let condition = signal(true);
        let cond_clone = condition.clone();

        // Track the created component's parent
        let created_parent: Rc<Cell<Option<usize>>> = Rc::new(Cell::new(None));
        let created_parent_clone = created_parent.clone();

        let _cleanup = show(
            move || cond_clone.get(),
            move || {
                let index = allocate_index(None);
                // Get current parent and store it
                let current_parent = get_current_parent_index();
                created_parent_clone.set(current_parent);
                // Set parent in arrays
                if let Some(p) = current_parent {
                    set_parent_index(index, Some(p));
                }
                Box::new(move || release_index(index)) as Cleanup
            },
            None::<fn() -> Cleanup>,
        );

        pop_parent_context();

        // Verify the created component has correct parent
        assert_eq!(
            created_parent.get(),
            Some(parent_index),
            "component created inside show() should have correct parent"
        );

        // Clean up parent
        release_index(parent_index);
    }

    #[test]
    fn test_show_no_else() {
        reset_registry();

        let condition = signal(true);
        let cond_clone = condition.clone();

        let initial_count = get_allocated_count();

        let _cleanup = show(
            move || cond_clone.get(),
            || create_test_component(),
            None::<fn() -> Cleanup>,
        );

        // Then branch active
        assert_eq!(get_allocated_count(), initial_count + 1);

        // Toggle to false - then destroyed, nothing created (no else)
        condition.set(false);
        assert_eq!(
            get_allocated_count(),
            initial_count,
            "with no else, false condition should have no component"
        );

        // Toggle back to true - then created again
        condition.set(true);
        assert_eq!(
            get_allocated_count(),
            initial_count + 1,
            "toggling back should recreate then branch"
        );
    }
}
