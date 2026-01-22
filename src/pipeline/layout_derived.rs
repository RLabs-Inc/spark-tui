//! Layout Derived - Reactive layout computation.
//!
//! Creates a Derived that computes layout whenever:
//! - Terminal size changes
//! - Render mode changes
//! - Any FlexNode slot changes
//! - Components are added/removed

use spark_signals::{derived, Derived};

use crate::layout::{compute_layout, ComputedLayout};
use super::terminal::{terminal_width_signal, terminal_height_signal, render_mode_signal, RenderMode};

/// Create the layout derived.
///
/// Returns a Derived that computes layout and automatically re-runs when
/// any dependency changes (terminal size, render mode, FlexNode slots, etc.)
pub fn create_layout_derived() -> Derived<ComputedLayout, impl Fn() -> ComputedLayout> {
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
