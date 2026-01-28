//! Layout Derived - Reactive layout computation.
//!
//! Creates a Derived that computes layout whenever:
//! - Terminal size changes
//! - Render mode changes
//! - Any FlexNode slot changes
//! - Components are added/removed
//!
//! # Global Layout Accessor
//!
//! The `get_layout()` function provides global access to the current computed layout.
//! This mirrors TypeScript's `layoutDerived.value` pattern and is used by:
//! - Scroll handlers (keyboard and mouse wheel)
//! - stick_to_bottom effects
//! - Any code that needs layout data outside the render effect

use std::cell::RefCell;
use std::rc::Rc;

use spark_signals::{derived, Derived};

use crate::layout::{compute_layout, ComputedLayout};
use super::terminal::{terminal_width_signal, terminal_height_signal, render_mode_signal, RenderMode};

// =============================================================================
// GLOBAL LAYOUT ACCESSOR
// =============================================================================

thread_local! {
    /// Cached layout - updated by mount()'s render effect after each layout computation.
    /// This provides global access to layout data for scroll handlers and effects.
    /// Wrapped in Rc for zero-cost sharing (cloning Rc is O(1) reference count increment).
    static CURRENT_LAYOUT: RefCell<Option<Rc<ComputedLayout>>> = const { RefCell::new(None) };
}

/// Update the cached layout. Called by render effect after layout computation.
///
/// This is an internal API used by the render pipeline to expose layout data
/// to scroll handlers and other systems.
pub fn set_layout(layout: ComputedLayout) {
    CURRENT_LAYOUT.with(|l| *l.borrow_mut() = Some(Rc::new(layout)));
}

/// Get the current computed layout.
///
/// This is the primary way scroll handlers access layout data.
/// Mirrors TypeScript's `layoutDerived.value` pattern.
///
/// Returns an `Rc<ComputedLayout>` for zero-cost sharing. Cloning the Rc
/// is O(1) (just a reference count increment), unlike cloning the layout
/// which would copy all 7 internal Vec arrays.
///
/// # Panics
///
/// Panics if called before mount() initializes the layout derived.
/// Use `try_get_layout()` if you need to handle the uninitialized case.
pub fn get_layout() -> Rc<ComputedLayout> {
    CURRENT_LAYOUT.with(|l| {
        l.borrow().clone().expect("get_layout() called before layout computed")
    })
}

/// Try to get the current computed layout.
///
/// Returns `None` if layout hasn't been computed yet (before mount() or during
/// component initialization). Useful in effects that may run before the render
/// pipeline is fully set up.
///
/// Returns an `Rc<ComputedLayout>` for zero-cost sharing.
pub fn try_get_layout() -> Option<Rc<ComputedLayout>> {
    CURRENT_LAYOUT.with(|l| l.borrow().clone())
}

/// Clear the cached layout (for unmount/testing).
pub fn clear_layout() {
    CURRENT_LAYOUT.with(|l| *l.borrow_mut() = None);
}

/// Create the layout derived.
///
/// Returns a Derived that computes layout and automatically re-runs when
/// any dependency changes (terminal size, render mode, FlexNode slots, etc.)
pub fn create_layout_derived() -> Derived<ComputedLayout> {
    let tw_signal = terminal_width_signal();
    let th_signal = terminal_height_signal();
    let mode_signal = render_mode_signal();

    derived(move || {
        // Read terminal dimensions (creates reactive dependency)
        let tw = tw_signal.get();
        let th = th_signal.get();

        // Read render mode (creates reactive dependency)
        let mode = mode_signal.get();

        // Constrain height only in fullscreen mode
        let constrain_height = mode == RenderMode::Fullscreen;

        // Compute layout using Taffy
        // Note: compute_layout internally reads from FlexNode slots,
        // which creates dependencies on those slots via the reactive system.
        compute_layout(tw, th, constrain_height)
    })
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{allocate_index, create_flex_node, reset_registry};
    use crate::engine::arrays::core as core_arrays;
    use crate::types::{ComponentType, Dimension};
    use crate::pipeline::terminal::set_terminal_size;

    fn setup() {
        reset_registry();
        set_terminal_size(80, 24);
    }

    #[test]
    fn test_layout_derived_empty() {
        setup();

        let layout_derived = create_layout_derived();
        let layout = layout_derived.get();

        assert_eq!(layout.content_width, 0);
        assert_eq!(layout.content_height, 0);
    }

    #[test]
    fn test_layout_derived_with_component() {
        setup();

        // Create a component
        let idx = allocate_index(None);
        core_arrays::set_component_type(idx, ComponentType::Box);
        core_arrays::set_visible(idx, true);
        let node = create_flex_node(idx);
        node.width.set_value(Dimension::Cells(40));
        node.height.set_value(Dimension::Cells(10));

        let layout_derived = create_layout_derived();
        let layout = layout_derived.get();

        assert_eq!(layout.width[idx], 40);
        assert_eq!(layout.height[idx], 10);
    }

    #[test]
    fn test_layout_derived_reacts_to_terminal_resize() {
        setup();

        let idx = allocate_index(None);
        core_arrays::set_component_type(idx, ComponentType::Box);
        core_arrays::set_visible(idx, true);
        let node = create_flex_node(idx);
        node.width.set_value(Dimension::Percent(100.0));
        node.height.set_value(Dimension::Percent(100.0));

        let layout_derived = create_layout_derived();

        // Initial layout
        let layout1 = layout_derived.get();
        assert_eq!(layout1.width[idx], 80);
        assert_eq!(layout1.height[idx], 24);

        // Resize terminal
        set_terminal_size(120, 40);

        // Layout should update
        let layout2 = layout_derived.get();
        assert_eq!(layout2.width[idx], 120);
        assert_eq!(layout2.height[idx], 40);
    }
}
