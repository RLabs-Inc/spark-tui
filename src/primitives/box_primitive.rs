//! Box Primitive - Container component with flexbox layout.
//!
//! The fundamental container component. Can have children, borders,
//! backgrounds, and handles events.
//!
//! # Reactivity
//!
//! Props are bound directly to FlexNode slots, preserving reactive connections.
//! When a signal changes, the layout or visual update happens automatically.
//!
//! # Example
//!
//! ```ignore
//! use spark_tui::primitives::{box_primitive, BoxProps};
//! use spark_signals::signal;
//!
//! let width = signal(Dimension::Cells(40));
//!
//! let cleanup = box_primitive(BoxProps {
//!     width: Some(width.clone().into()),
//!     height: Some(10.into()),
//!     border: Some(BorderStyle::Single.into()),
//!     children: Some(Box::new(|| {
//!         text(TextProps {
//!             content: "Hello!".to_string().into(),
//!             ..Default::default()
//!         });
//!     })),
//!     ..Default::default()
//! });
//!
//! // Update width - UI reacts automatically
//! width.set(Dimension::Cells(80));
//! ```

use std::rc::Rc;

use crate::engine::{
    allocate_index, release_index, create_flex_node,
    get_current_parent_index, push_parent_context, pop_parent_context,
};
use crate::engine::arrays::{core, visual, interaction};
use crate::state::{mouse, keyboard};
use crate::types::{ComponentType, BorderStyle};
use super::types::{BoxProps, PropValue, Cleanup};

// =============================================================================
// Helper: Bind PropValue to Slot
// =============================================================================

/// Bind a PropValue to a FlexNode Slot.
///
/// This preserves reactivity:
/// - Static values are set directly
/// - Signals stay connected
/// - Getters are wrapped
macro_rules! bind_slot {
    ($slot:expr, $prop:expr) => {
        match $prop {
            PropValue::Static(v) => $slot.set_value(v),
            PropValue::Signal(s) => $slot.set_signal(s),
            PropValue::Getter(g) => $slot.set_signal_readonly(move || g()),
        }
    };
}


// =============================================================================
// Box Component
// =============================================================================

/// Create a box container component.
///
/// Boxes are the building blocks of layouts. They can:
/// - Have borders and backgrounds
/// - Use flexbox for child layout
/// - Contain other components as children
/// - Be focusable and handle keyboard events
///
/// Returns a cleanup function that releases resources when called.
pub fn box_primitive(props: BoxProps) -> Cleanup {
    // 1. ALLOCATE INDEX
    let index = allocate_index(props.id.as_deref());

    // 2. CREATE FLEXNODE - Persistent layout object with reactive Slot properties
    let flex_node = create_flex_node(index);

    // 3. CORE SETUP - Type, parent
    core::set_component_type(index, ComponentType::Box);
    if let Some(parent) = get_current_parent_index() {
        core::set_parent_index(index, Some(parent));
    }

    // 4. BIND VISIBILITY
    if let Some(visible) = props.visible {
        match visible {
            PropValue::Static(v) => core::set_visible(index, v),
            PropValue::Signal(s) => core::set_visible_signal(index, s),
            PropValue::Getter(g) => core::set_visible_getter(index, move || g()),
        }
    }

    // 5. BIND FLEXNODE SLOTS - Layout properties

    // Container properties
    if let Some(dir) = props.flex_direction {
        bind_slot!(flex_node.flex_direction, dir);
    }
    if let Some(wrap) = props.flex_wrap {
        bind_slot!(flex_node.flex_wrap, wrap);
    }
    if let Some(justify) = props.justify_content {
        bind_slot!(flex_node.justify_content, justify);
    }
    if let Some(align) = props.align_items {
        bind_slot!(flex_node.align_items, align);
    }
    if let Some(align_content) = props.align_content {
        bind_slot!(flex_node.align_content, align_content);
    }

    // Item properties
    if let Some(grow) = props.grow {
        bind_slot!(flex_node.flex_grow, grow);
    }
    if let Some(shrink) = props.shrink {
        bind_slot!(flex_node.flex_shrink, shrink);
    }
    if let Some(basis) = props.flex_basis {
        bind_slot!(flex_node.flex_basis, basis);
    }
    if let Some(align_self) = props.align_self {
        bind_slot!(flex_node.align_self, align_self);
    }
    if let Some(order) = props.order {
        bind_slot!(flex_node.order, order);
    }

    // Dimensions
    if let Some(w) = props.width {
        bind_slot!(flex_node.width, w);
    }
    if let Some(h) = props.height {
        bind_slot!(flex_node.height, h);
    }
    if let Some(min_w) = props.min_width {
        bind_slot!(flex_node.min_width, min_w);
    }
    if let Some(max_w) = props.max_width {
        bind_slot!(flex_node.max_width, max_w);
    }
    if let Some(min_h) = props.min_height {
        bind_slot!(flex_node.min_height, min_h);
    }
    if let Some(max_h) = props.max_height {
        bind_slot!(flex_node.max_height, max_h);
    }

    // Spacing - Margin
    if let Some(ref m) = props.margin {
        // Shorthand - apply to all sides unless individual is set
        if props.margin_top.is_none() {
            bind_slot!(flex_node.margin_top, m.clone());
        }
        if props.margin_right.is_none() {
            bind_slot!(flex_node.margin_right, m.clone());
        }
        if props.margin_bottom.is_none() {
            bind_slot!(flex_node.margin_bottom, m.clone());
        }
        if props.margin_left.is_none() {
            bind_slot!(flex_node.margin_left, m.clone());
        }
    }
    if let Some(mt) = props.margin_top {
        bind_slot!(flex_node.margin_top, mt);
    }
    if let Some(mr) = props.margin_right {
        bind_slot!(flex_node.margin_right, mr);
    }
    if let Some(mb) = props.margin_bottom {
        bind_slot!(flex_node.margin_bottom, mb);
    }
    if let Some(ml) = props.margin_left {
        bind_slot!(flex_node.margin_left, ml);
    }

    // Spacing - Padding (bind to FlexNode for layout)
    if let Some(ref p) = props.padding {
        if props.padding_top.is_none() {
            bind_slot!(flex_node.padding_top, p.clone());
        }
        if props.padding_right.is_none() {
            bind_slot!(flex_node.padding_right, p.clone());
        }
        if props.padding_bottom.is_none() {
            bind_slot!(flex_node.padding_bottom, p.clone());
        }
        if props.padding_left.is_none() {
            bind_slot!(flex_node.padding_left, p.clone());
        }
    }
    if let Some(pt) = props.padding_top {
        bind_slot!(flex_node.padding_top, pt);
    }
    if let Some(pr) = props.padding_right {
        bind_slot!(flex_node.padding_right, pr);
    }
    if let Some(pb) = props.padding_bottom {
        bind_slot!(flex_node.padding_bottom, pb);
    }
    if let Some(pl) = props.padding_left {
        bind_slot!(flex_node.padding_left, pl);
    }

    // Gap
    if let Some(g) = props.gap {
        bind_slot!(flex_node.gap, g.clone());
        if props.row_gap.is_none() {
            bind_slot!(flex_node.row_gap, g.clone());
        }
        if props.column_gap.is_none() {
            bind_slot!(flex_node.column_gap, g);
        }
    }
    if let Some(rg) = props.row_gap {
        bind_slot!(flex_node.row_gap, rg);
    }
    if let Some(cg) = props.column_gap {
        bind_slot!(flex_node.column_gap, cg);
    }

    // Overflow
    if let Some(overflow) = props.overflow {
        bind_slot!(flex_node.overflow, overflow);
    }

    // Stick to bottom (auto-scroll on content growth)
    if props.stick_to_bottom {
        interaction::set_stick_to_bottom(index, true);
    }

    // Position
    if let Some(position) = props.position {
        bind_slot!(flex_node.position, position);
    }

    // Border widths for layout (0 or 1)
    if let Some(ref border) = props.border {
        // Set border width to 1 if style is not None
        match border {
            PropValue::Static(style) => {
                let has_border = *style != BorderStyle::None;
                flex_node.border_top.set_value(if has_border { 1 } else { 0 });
                flex_node.border_right.set_value(if has_border { 1 } else { 0 });
                flex_node.border_bottom.set_value(if has_border { 1 } else { 0 });
                flex_node.border_left.set_value(if has_border { 1 } else { 0 });
            }
            PropValue::Signal(s) => {
                let s_clone = s.clone();
                flex_node.border_top.set_signal_readonly(move || {
                    if s_clone.get() != BorderStyle::None { 1 } else { 0 }
                });
                let s_clone = s.clone();
                flex_node.border_right.set_signal_readonly(move || {
                    if s_clone.get() != BorderStyle::None { 1 } else { 0 }
                });
                let s_clone = s.clone();
                flex_node.border_bottom.set_signal_readonly(move || {
                    if s_clone.get() != BorderStyle::None { 1 } else { 0 }
                });
                let s_clone = s.clone();
                flex_node.border_left.set_signal_readonly(move || {
                    if s_clone.get() != BorderStyle::None { 1 } else { 0 }
                });
            }
            PropValue::Getter(g) => {
                let g_clone = g.clone();
                flex_node.border_top.set_signal_readonly(move || {
                    if g_clone() != BorderStyle::None { 1 } else { 0 }
                });
                let g_clone = g.clone();
                flex_node.border_right.set_signal_readonly(move || {
                    if g_clone() != BorderStyle::None { 1 } else { 0 }
                });
                let g_clone = g.clone();
                flex_node.border_bottom.set_signal_readonly(move || {
                    if g_clone() != BorderStyle::None { 1 } else { 0 }
                });
                let g_clone = g.clone();
                flex_node.border_left.set_signal_readonly(move || {
                    if g_clone() != BorderStyle::None { 1 } else { 0 }
                });
            }
        }
    }

    // 6. BIND VISUAL ARRAYS

    // Border style (for rendering)
    if let Some(border) = props.border {
        match border {
            PropValue::Static(v) => visual::set_border_style(index, v),
            PropValue::Signal(s) => visual::set_border_style_getter(index, move || s.get()),
            PropValue::Getter(g) => visual::set_border_style_getter(index, move || g()),
        }
    }

    // Border color
    if let Some(bc) = props.border_color {
        match bc {
            PropValue::Static(v) => visual::set_border_color(index, v),
            PropValue::Signal(s) => visual::set_border_color_getter(index, move || s.get()),
            PropValue::Getter(g) => visual::set_border_color_getter(index, move || g()),
        }
    }

    // Foreground color
    if let Some(fg) = props.fg {
        match fg {
            PropValue::Static(v) => visual::set_fg_color(index, v),
            PropValue::Signal(s) => visual::set_fg_color_getter(index, move || s.get()),
            PropValue::Getter(g) => visual::set_fg_color_getter(index, move || g()),
        }
    }

    // Background color
    if let Some(bg) = props.bg {
        match bg {
            PropValue::Static(v) => visual::set_bg_color(index, v),
            PropValue::Signal(s) => visual::set_bg_color_getter(index, move || s.get()),
            PropValue::Getter(g) => visual::set_bg_color_getter(index, move || g()),
        }
    }

    // Opacity
    if let Some(opacity) = props.opacity {
        match opacity {
            PropValue::Static(v) => visual::set_opacity(index, v),
            PropValue::Signal(s) => visual::set_opacity(index, s.get()), // TODO: reactive opacity
            PropValue::Getter(g) => visual::set_opacity(index, g()), // TODO: reactive opacity
        }
    }

    // Z-index
    if let Some(z) = props.z_index {
        match z {
            PropValue::Static(v) => visual::set_z_index(index, v),
            PropValue::Signal(s) => visual::set_z_index(index, s.get()), // TODO: reactive z-index
            PropValue::Getter(g) => visual::set_z_index(index, g()), // TODO: reactive z-index
        }
    }

    // 7. BIND INTERACTION

    // Focusable
    let should_be_focusable = props.focusable.unwrap_or(false);
    if should_be_focusable {
        interaction::set_focusable(index, true);
        if let Some(tab_idx) = props.tab_index {
            interaction::set_tab_index(index, tab_idx);
        }
    }

    // 8. REGISTER MOUSE HANDLERS

    let has_mouse_handlers = props.on_click.is_some()
        || props.on_mouse_down.is_some()
        || props.on_mouse_up.is_some()
        || props.on_mouse_enter.is_some()
        || props.on_mouse_leave.is_some()
        || props.on_scroll.is_some();

    let mut mouse_cleanup: Option<Box<dyn FnOnce()>> = None;
    let mut key_cleanup: Option<Box<dyn FnOnce()>> = None;

    if should_be_focusable || has_mouse_handlers {
        // Clone Rc callbacks for use in closure (Rc::clone is cheap)
        let user_on_click = props.on_click.clone();

        // Build click handler that includes click-to-focus
        let click_handler: Option<Rc<dyn Fn(&mouse::MouseEvent)>> = if should_be_focusable {
            Some(Rc::new(move |event: &mouse::MouseEvent| {
                crate::state::focus::focus(index);
                if let Some(ref handler) = user_on_click {
                    handler(event);
                }
            }))
        } else {
            props.on_click.clone()
        };

        let handlers = mouse::MouseHandlers {
            on_mouse_down: props.on_mouse_down.clone(),
            on_mouse_up: props.on_mouse_up.clone(),
            on_click: click_handler,
            on_mouse_enter: props.on_mouse_enter.clone(),
            on_mouse_leave: props.on_mouse_leave.clone(),
            on_scroll: props.on_scroll.clone(),
        };

        let cleanup_fn = mouse::on_component(index, handlers);
        mouse_cleanup = Some(Box::new(cleanup_fn));
    }

    // 9. REGISTER KEYBOARD HANDLER (if focusable and has on_key)

    if should_be_focusable {
        if let Some(on_key) = props.on_key.clone() {
            let cleanup_fn = keyboard::on_focused(index, move |event| {
                on_key(event)
            });
            key_cleanup = Some(Box::new(cleanup_fn));
        }
    }

    // 10. RENDER CHILDREN
    if let Some(children) = props.children {
        push_parent_context(index);
        children();
        pop_parent_context();
    }

    // 11. RETURN CLEANUP
    Box::new(move || {
        // Clean up mouse handlers
        if let Some(cleanup) = mouse_cleanup {
            cleanup();
        }
        // Clean up keyboard handlers
        if let Some(cleanup) = key_cleanup {
            cleanup();
        }
        // Clean up component state in mouse/keyboard modules
        mouse::cleanup_index(index);
        keyboard::cleanup_index(index);
        // Release index
        release_index(index);
    })
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::reset_registry;
    use crate::types::Dimension;
    use spark_signals::signal;

    fn setup() {
        reset_registry();
    }

    #[test]
    fn test_box_creation() {
        setup();

        let cleanup = box_primitive(BoxProps {
            width: Some(PropValue::Static(Dimension::Cells(50))),
            height: Some(PropValue::Static(Dimension::Cells(20))),
            ..Default::default()
        });

        // Check component was created
        assert_eq!(core::get_component_type(0), ComponentType::Box);

        // Cleanup
        cleanup();
        assert_eq!(core::get_component_type(0), ComponentType::None);
    }

    #[test]
    fn test_box_with_children() {
        setup();

        let _cleanup = box_primitive(BoxProps {
            children: Some(Box::new(|| {
                box_primitive(BoxProps::default());
            })),
            ..Default::default()
        });

        // Parent should be index 0, child should be index 1
        assert_eq!(core::get_component_type(0), ComponentType::Box);
        assert_eq!(core::get_component_type(1), ComponentType::Box);
        assert_eq!(core::get_parent_index(1), Some(0));
    }

    #[test]
    fn test_box_reactive_width() {
        setup();

        let width_signal = signal(Dimension::Cells(40));
        let width_for_box = width_signal.clone();

        let _cleanup = box_primitive(BoxProps {
            width: Some(PropValue::Signal(width_for_box)),
            ..Default::default()
        });

        // Get FlexNode and check width
        let flex_node = crate::engine::get_flex_node(0).unwrap();
        assert_eq!(flex_node.width.get(), Dimension::Cells(40));

        // Update signal - FlexNode should reflect change
        width_signal.set(Dimension::Cells(80));
        assert_eq!(flex_node.width.get(), Dimension::Cells(80));
    }

    #[test]
    fn test_box_border() {
        setup();

        let _cleanup = box_primitive(BoxProps {
            border: Some(PropValue::Static(BorderStyle::Single)),
            ..Default::default()
        });

        // Visual array should have border style
        assert_eq!(visual::get_border_style(0), BorderStyle::Single);

        // FlexNode should have border width of 1
        let flex_node = crate::engine::get_flex_node(0).unwrap();
        assert_eq!(flex_node.border_top.get(), 1);
    }

    #[test]
    fn test_box_focusable() {
        setup();

        let _cleanup = box_primitive(BoxProps {
            focusable: Some(true),
            tab_index: Some(5),
            ..Default::default()
        });

        assert!(interaction::get_focusable(0));
        assert_eq!(interaction::get_tab_index(0), 5);
    }
}
