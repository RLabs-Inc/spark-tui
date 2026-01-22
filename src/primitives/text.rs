//! Text Primitive - Display text with styling and wrapping.
//!
//! A pure display component for text content. Cannot have children.
//!
//! # Reactivity
//!
//! Content can be a static string, signal, or getter. When the content
//! source changes, the display updates automatically.
//!
//! # Example
//!
//! ```ignore
//! use spark_tui::primitives::{text, TextProps};
//! use spark_signals::signal;
//!
//! // Static text
//! text(TextProps {
//!     content: "Hello, World!".to_string().into(),
//!     ..Default::default()
//! });
//!
//! // Reactive text
//! let count = signal(0);
//! let count_clone = count.clone();
//! text(TextProps {
//!     content: PropValue::Getter(Rc::new(move || format!("Count: {}", count_clone.get()))),
//!     attrs: Some(Attr::BOLD.into()),
//!     ..Default::default()
//! });
//!
//! // Update count - text updates automatically
//! count.set(42);
//! ```

use crate::engine::{
    allocate_index, release_index, create_flex_node,
    get_current_parent_index,
};
use crate::engine::arrays::{core, visual, text as text_arrays};
use crate::state::mouse;
use crate::types::ComponentType;
use super::types::{TextProps, PropValue, Cleanup};

// =============================================================================
// Text Component
// =============================================================================

/// Create a text display component.
///
/// Text is used to display strings with optional styling (bold, italic, etc.),
/// alignment, and wrapping behavior.
///
/// # Properties
///
/// - `content` - The text to display (required)
/// - `attrs` - Text attributes like bold, italic, underline
/// - `align` - Text alignment: left, center, right
/// - `wrap` - Wrap mode: wrap, nowrap, truncate
///
/// Returns a cleanup function that releases resources when called.
pub fn text(props: TextProps) -> Cleanup {
    // 1. ALLOCATE INDEX
    let index = allocate_index(props.id.as_deref());

    // 2. CREATE FLEXNODE - Even text needs layout properties
    let flex_node = create_flex_node(index);

    // 3. CORE SETUP - Type, parent
    core::set_component_type(index, ComponentType::Text);
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

    // 5. BIND TEXT CONTENT
    match props.content {
        PropValue::Static(v) => text_arrays::set_text_content(index, v),
        PropValue::Signal(s) => text_arrays::set_text_content_signal(index, s),
        PropValue::Getter(g) => text_arrays::set_text_content_getter(index, move || g()),
    }

    // 6. BIND TEXT STYLING
    if let Some(attrs) = props.attrs {
        match attrs {
            PropValue::Static(v) => text_arrays::set_text_attrs(index, v),
            PropValue::Signal(s) => text_arrays::set_text_attrs_getter(index, move || s.get()),
            PropValue::Getter(g) => text_arrays::set_text_attrs_getter(index, move || g()),
        }
    }

    if let Some(align) = props.align {
        match align {
            PropValue::Static(v) => text_arrays::set_text_align(index, v),
            PropValue::Signal(s) => text_arrays::set_text_align_getter(index, move || s.get()),
            PropValue::Getter(g) => text_arrays::set_text_align_getter(index, move || g()),
        }
    }

    if let Some(wrap) = props.wrap {
        match wrap {
            PropValue::Static(v) => text_arrays::set_text_wrap(index, v),
            PropValue::Signal(s) => text_arrays::set_text_wrap_getter(index, move || s.get()),
            PropValue::Getter(g) => text_arrays::set_text_wrap_getter(index, move || g()),
        }
    }

    // 7. BIND FLEXNODE SLOTS - Layout properties

    // Item properties (text is always a flex item, never a container)
    if let Some(grow) = props.grow {
        match grow {
            PropValue::Static(v) => flex_node.flex_grow.set_value(v),
            PropValue::Signal(s) => flex_node.flex_grow.set_signal(s),
            PropValue::Getter(g) => flex_node.flex_grow.set_signal_readonly(move || g()),
        }
    }
    if let Some(shrink) = props.shrink {
        match shrink {
            PropValue::Static(v) => flex_node.flex_shrink.set_value(v),
            PropValue::Signal(s) => flex_node.flex_shrink.set_signal(s),
            PropValue::Getter(g) => flex_node.flex_shrink.set_signal_readonly(move || g()),
        }
    }
    if let Some(basis) = props.flex_basis {
        match basis {
            PropValue::Static(v) => flex_node.flex_basis.set_value(v),
            PropValue::Signal(s) => flex_node.flex_basis.set_signal(s),
            PropValue::Getter(g) => flex_node.flex_basis.set_signal_readonly(move || g()),
        }
    }
    if let Some(align_self) = props.align_self {
        match align_self {
            PropValue::Static(v) => flex_node.align_self.set_value(v),
            PropValue::Signal(s) => flex_node.align_self.set_signal(s),
            PropValue::Getter(g) => flex_node.align_self.set_signal_readonly(move || g()),
        }
    }

    // Dimensions
    if let Some(w) = props.width {
        match w {
            PropValue::Static(v) => flex_node.width.set_value(v),
            PropValue::Signal(s) => flex_node.width.set_signal(s),
            PropValue::Getter(g) => flex_node.width.set_signal_readonly(move || g()),
        }
    }
    if let Some(h) = props.height {
        match h {
            PropValue::Static(v) => flex_node.height.set_value(v),
            PropValue::Signal(s) => flex_node.height.set_signal(s),
            PropValue::Getter(g) => flex_node.height.set_signal_readonly(move || g()),
        }
    }
    if let Some(min_w) = props.min_width {
        match min_w {
            PropValue::Static(v) => flex_node.min_width.set_value(v),
            PropValue::Signal(s) => flex_node.min_width.set_signal(s),
            PropValue::Getter(g) => flex_node.min_width.set_signal_readonly(move || g()),
        }
    }
    if let Some(max_w) = props.max_width {
        match max_w {
            PropValue::Static(v) => flex_node.max_width.set_value(v),
            PropValue::Signal(s) => flex_node.max_width.set_signal(s),
            PropValue::Getter(g) => flex_node.max_width.set_signal_readonly(move || g()),
        }
    }
    if let Some(min_h) = props.min_height {
        match min_h {
            PropValue::Static(v) => flex_node.min_height.set_value(v),
            PropValue::Signal(s) => flex_node.min_height.set_signal(s),
            PropValue::Getter(g) => flex_node.min_height.set_signal_readonly(move || g()),
        }
    }
    if let Some(max_h) = props.max_height {
        match max_h {
            PropValue::Static(v) => flex_node.max_height.set_value(v),
            PropValue::Signal(s) => flex_node.max_height.set_signal(s),
            PropValue::Getter(g) => flex_node.max_height.set_signal_readonly(move || g()),
        }
    }

    // Spacing - Padding
    if let Some(ref p) = props.padding {
        if props.padding_top.is_none() {
            match p.clone() {
                PropValue::Static(v) => flex_node.padding_top.set_value(v),
                PropValue::Signal(s) => flex_node.padding_top.set_signal(s),
                PropValue::Getter(g) => flex_node.padding_top.set_signal_readonly(move || g()),
            }
        }
        if props.padding_right.is_none() {
            match p.clone() {
                PropValue::Static(v) => flex_node.padding_right.set_value(v),
                PropValue::Signal(s) => flex_node.padding_right.set_signal(s),
                PropValue::Getter(g) => flex_node.padding_right.set_signal_readonly(move || g()),
            }
        }
        if props.padding_bottom.is_none() {
            match p.clone() {
                PropValue::Static(v) => flex_node.padding_bottom.set_value(v),
                PropValue::Signal(s) => flex_node.padding_bottom.set_signal(s),
                PropValue::Getter(g) => flex_node.padding_bottom.set_signal_readonly(move || g()),
            }
        }
        if props.padding_left.is_none() {
            match p.clone() {
                PropValue::Static(v) => flex_node.padding_left.set_value(v),
                PropValue::Signal(s) => flex_node.padding_left.set_signal(s),
                PropValue::Getter(g) => flex_node.padding_left.set_signal_readonly(move || g()),
            }
        }
    }
    if let Some(pt) = props.padding_top {
        match pt {
            PropValue::Static(v) => flex_node.padding_top.set_value(v),
            PropValue::Signal(s) => flex_node.padding_top.set_signal(s),
            PropValue::Getter(g) => flex_node.padding_top.set_signal_readonly(move || g()),
        }
    }
    if let Some(pr) = props.padding_right {
        match pr {
            PropValue::Static(v) => flex_node.padding_right.set_value(v),
            PropValue::Signal(s) => flex_node.padding_right.set_signal(s),
            PropValue::Getter(g) => flex_node.padding_right.set_signal_readonly(move || g()),
        }
    }
    if let Some(pb) = props.padding_bottom {
        match pb {
            PropValue::Static(v) => flex_node.padding_bottom.set_value(v),
            PropValue::Signal(s) => flex_node.padding_bottom.set_signal(s),
            PropValue::Getter(g) => flex_node.padding_bottom.set_signal_readonly(move || g()),
        }
    }
    if let Some(pl) = props.padding_left {
        match pl {
            PropValue::Static(v) => flex_node.padding_left.set_value(v),
            PropValue::Signal(s) => flex_node.padding_left.set_signal(s),
            PropValue::Getter(g) => flex_node.padding_left.set_signal_readonly(move || g()),
        }
    }

    // 8. BIND VISUAL ARRAYS

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
            PropValue::Signal(s) => visual::set_opacity(index, s.get()),
            PropValue::Getter(g) => visual::set_opacity(index, g()),
        }
    }

    // 9. REGISTER MOUSE HANDLER (if on_click provided)
    let mut mouse_cleanup: Option<Box<dyn FnOnce()>> = None;

    if let Some(on_click) = props.on_click.clone() {
        let handlers = mouse::MouseHandlers {
            on_mouse_down: None,
            on_mouse_up: None,
            on_click: Some(on_click),
            on_mouse_enter: None,
            on_mouse_leave: None,
            on_scroll: None,
        };
        let cleanup_fn = mouse::on_component(index, handlers);
        mouse_cleanup = Some(Box::new(cleanup_fn));
    }

    // 10. RETURN CLEANUP
    Box::new(move || {
        // Clean up mouse handler
        if let Some(cleanup) = mouse_cleanup {
            cleanup();
        }
        // Clean up component state in mouse module
        mouse::cleanup_index(index);
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
    use crate::types::{Attr, TextAlign, TextWrap};
    use spark_signals::signal;

    fn setup() {
        reset_registry();
    }

    #[test]
    fn test_text_creation() {
        setup();

        let cleanup = text(TextProps {
            content: PropValue::Static("Hello, World!".to_string()),
            ..Default::default()
        });

        // Check component was created
        assert_eq!(core::get_component_type(0), ComponentType::Text);
        assert_eq!(text_arrays::get_text_content(0), "Hello, World!");

        // Cleanup
        cleanup();
        assert_eq!(core::get_component_type(0), ComponentType::None);
    }

    #[test]
    fn test_text_reactive_content() {
        setup();

        let content = signal("Initial".to_string());
        let content_for_text = content.clone();

        let _cleanup = text(TextProps {
            content: PropValue::Signal(content_for_text),
            ..Default::default()
        });

        assert_eq!(text_arrays::get_text_content(0), "Initial");

        // Update signal - text should update
        content.set("Updated".to_string());
        assert_eq!(text_arrays::get_text_content(0), "Updated");
    }

    #[test]
    fn test_text_with_attrs() {
        setup();

        let _cleanup = text(TextProps {
            content: PropValue::Static("Bold Text".to_string()),
            attrs: Some(PropValue::Static(Attr::BOLD | Attr::ITALIC)),
            ..Default::default()
        });

        assert_eq!(text_arrays::get_text_attrs(0), Attr::BOLD | Attr::ITALIC);
    }

    #[test]
    fn test_text_alignment() {
        setup();

        let _cleanup = text(TextProps {
            content: PropValue::Static("Centered".to_string()),
            align: Some(PropValue::Static(TextAlign::Center)),
            wrap: Some(PropValue::Static(TextWrap::NoWrap)),
            ..Default::default()
        });

        assert_eq!(text_arrays::get_text_align(0), TextAlign::Center);
        assert_eq!(text_arrays::get_text_wrap(0), TextWrap::NoWrap);
    }

    #[test]
    fn test_text_in_box() {
        setup();

        use super::super::{box_primitive, BoxProps};

        let _cleanup = box_primitive(BoxProps {
            children: Some(Box::new(|| {
                text(TextProps {
                    content: PropValue::Static("Child Text".to_string()),
                    ..Default::default()
                });
            })),
            ..Default::default()
        });

        // Parent box at index 0, text at index 1
        assert_eq!(core::get_component_type(0), ComponentType::Box);
        assert_eq!(core::get_component_type(1), ComponentType::Text);
        assert_eq!(core::get_parent_index(1), Some(0));
        assert_eq!(text_arrays::get_text_content(1), "Child Text");
    }
}
