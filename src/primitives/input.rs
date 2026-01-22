//! Input Primitive - Single-line text input component.
//!
//! A text entry component with full editing capabilities.
//!
//! # Features
//!
//! - Two-way value binding via Signal
//! - Cursor navigation (arrows, home, end)
//! - Text editing (backspace, delete)
//! - Password mode with configurable mask
//! - Placeholder text
//! - Always focusable
//!
//! # Example
//!
//! ```ignore
//! use spark_tui::primitives::{input, InputProps};
//! use spark_signals::signal;
//!
//! let name = signal("".to_string());
//! let name_clone = name.clone();
//!
//! let cleanup = input(InputProps {
//!     value: name_clone,
//!     placeholder: Some("Enter your name...".to_string()),
//!     ..InputProps::new(signal("".to_string()))
//! });
//!
//! // Update value - input displays it automatically
//! name.set("Alice".to_string());
//! ```

use std::rc::Rc;
use spark_signals::signal;

use crate::engine::{
    allocate_index, release_index, create_flex_node,
    get_current_parent_index,
};
use crate::engine::arrays::{core, visual, text as text_arrays, interaction};
use crate::state::{mouse, keyboard, focus};
use crate::types::{ComponentType, BorderStyle};
use super::types::{InputProps, PropValue, Cleanup};

// =============================================================================
// Word Boundary Helpers
// =============================================================================

/// Find the start of the word before the given position.
/// A word is defined as a sequence of alphanumeric characters.
fn find_word_start(text: &str, pos: usize) -> usize {
    if pos == 0 {
        return 0;
    }

    let chars: Vec<char> = text.chars().collect();
    let mut i = pos;

    // Skip whitespace/punctuation going backward
    while i > 0 && !chars[i - 1].is_alphanumeric() {
        i -= 1;
    }

    // Skip word characters going backward
    while i > 0 && chars[i - 1].is_alphanumeric() {
        i -= 1;
    }

    i
}

/// Find the end of the word after the given position.
/// A word is defined as a sequence of alphanumeric characters.
fn find_word_end(text: &str, pos: usize) -> usize {
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();

    if pos >= len {
        return len;
    }

    let mut i = pos;

    // Skip whitespace/punctuation going forward
    while i < len && !chars[i].is_alphanumeric() {
        i += 1;
    }

    // Skip word characters going forward
    while i < len && chars[i].is_alphanumeric() {
        i += 1;
    }

    i
}

// =============================================================================
// Helper: Bind PropValue to Slot
// =============================================================================

/// Bind a PropValue to a FlexNode Slot.
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
// Input Component
// =============================================================================

/// Create a single-line text input component.
///
/// The input is always focusable and handles keyboard events when focused.
/// Pass a Signal<String> for two-way value binding.
///
/// # Properties
///
/// - `value` - The current text value (required, two-way bound)
/// - `placeholder` - Text shown when value is empty
/// - `password` - Mask characters with mask_char
/// - `max_length` - Maximum input length (0 = unlimited)
/// - `cursor` - Cursor style, blink, and color configuration
///
/// Returns a cleanup function that releases resources when called.
pub fn input(props: InputProps) -> Cleanup {
    // 1. ALLOCATE INDEX
    let index = allocate_index(props.id.as_deref());

    // 2. CREATE FLEXNODE - Persistent layout object with reactive Slot properties
    let flex_node = create_flex_node(index);

    // 3. CORE SETUP - Type, parent
    core::set_component_type(index, ComponentType::Input);
    if let Some(parent) = get_current_parent_index() {
        core::set_parent_index(index, Some(parent));
    }

    // ==========================================================================
    // INTERNAL STATE
    // ==========================================================================

    // Cursor position within the text - local signal synced to slot array
    let cursor_pos = signal(0u16);

    // Value signal (cloned for various closures)
    let value = props.value.clone();

    // Password mask character
    let mask_char = props.mask_char.unwrap_or('•');
    let password = props.password;
    let placeholder = props.placeholder.clone();

    // ==========================================================================
    // TEXT CONTENT - Display via slot array
    // Syncs reactive value, handles password masking and placeholder
    // ==========================================================================

    // Create getter that produces display text
    let value_for_display = value.clone();
    let placeholder_for_display = placeholder.clone();
    text_arrays::set_text_content_getter(index, move || {
        let val = value_for_display.get();
        if val.is_empty() {
            if let Some(ref ph) = placeholder_for_display {
                return ph.clone();
            }
        }
        if password {
            mask_char.to_string().repeat(val.chars().count())
        } else {
            val
        }
    });

    // Text attributes (if specified)
    if let Some(attrs) = props.attrs {
        match attrs {
            PropValue::Static(v) => text_arrays::set_text_attrs(index, v),
            PropValue::Signal(s) => text_arrays::set_text_attrs_getter(index, move || s.get()),
            PropValue::Getter(g) => text_arrays::set_text_attrs_getter(index, move || g()),
        }
    }

    // ==========================================================================
    // VISIBILITY
    // ==========================================================================

    if let Some(visible) = props.visible {
        match visible {
            PropValue::Static(v) => core::set_visible(index, v),
            PropValue::Signal(s) => core::set_visible_signal(index, s),
            PropValue::Getter(g) => core::set_visible_getter(index, move || g()),
        }
    }

    // ==========================================================================
    // CURSOR POSITION - Synced to interaction arrays
    // ==========================================================================

    let cursor_pos_for_array = cursor_pos.clone();
    let value_for_cursor = value.clone();
    interaction::set_cursor_position_getter(index, move || {
        let pos = cursor_pos_for_array.get();
        let len = value_for_cursor.get().chars().count() as u16;
        // Clamp cursor position to value length
        pos.min(len)
    });

    // Cursor visible when focused (placeholder for focus integration)
    interaction::set_cursor_visible(index, true);

    // ==========================================================================
    // FLEXNODE BINDINGS - Layout properties
    // ==========================================================================

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

    // Spacing - Padding
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

    // Border widths for layout (0 or 1)
    if let Some(ref border) = props.border {
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

    // ==========================================================================
    // VISUAL ARRAYS
    // ==========================================================================

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
            PropValue::Signal(s) => visual::set_opacity(index, s.get()),
            PropValue::Getter(g) => visual::set_opacity(index, g()),
        }
    }

    // ==========================================================================
    // FOCUS - Inputs are always focusable
    // ==========================================================================

    interaction::set_focusable(index, true);
    if let Some(tab_idx) = props.tab_index {
        interaction::set_tab_index(index, tab_idx);
    }

    // ==========================================================================
    // KEYBOARD HANDLERS
    // ==========================================================================

    let cursor_pos_for_key = cursor_pos.clone();
    let value_for_key = value.clone();
    let max_length = props.max_length.unwrap_or(0);
    let on_change = props.on_change.clone();
    let on_submit = props.on_submit.clone();
    let on_cancel = props.on_cancel.clone();

    let key_cleanup = keyboard::on_focused(index, move |event| {
        let val = value_for_key.get();
        let char_count = val.chars().count();
        // Clamp cursor position to value length (handles external value changes)
        let pos = cursor_pos_for_key.get().min(char_count as u16) as usize;

        // Handle Ctrl+key combinations first
        if event.modifiers.ctrl {
            match event.key.as_str() {
                // Word navigation
                "ArrowLeft" => {
                    let new_pos = find_word_start(&val, pos);
                    cursor_pos_for_key.set(new_pos as u16);
                    true
                }
                "ArrowRight" => {
                    let new_pos = find_word_end(&val, pos);
                    cursor_pos_for_key.set(new_pos as u16);
                    true
                }

                // Word deletion
                "Backspace" => {
                    if pos > 0 {
                        let word_start = find_word_start(&val, pos);
                        let mut chars: Vec<char> = val.chars().collect();
                        // Remove characters from word_start to pos
                        chars.drain(word_start..pos);
                        let new_val: String = chars.into_iter().collect();
                        value_for_key.set(new_val.clone());
                        cursor_pos_for_key.set(word_start as u16);
                        if let Some(ref cb) = on_change {
                            cb(&new_val);
                        }
                    }
                    true
                }
                "Delete" => {
                    if pos < char_count {
                        let word_end = find_word_end(&val, pos);
                        let mut chars: Vec<char> = val.chars().collect();
                        // Remove characters from pos to word_end
                        chars.drain(pos..word_end);
                        let new_val: String = chars.into_iter().collect();
                        value_for_key.set(new_val.clone());
                        // Cursor stays at pos
                        if let Some(ref cb) = on_change {
                            cb(&new_val);
                        }
                    }
                    true
                }

                // Select all
                "a" | "A" => {
                    // Set selection to entire text
                    interaction::set_selection(index, 0, char_count as u16);
                    true
                }

                _ => false
            }
        } else {
            match event.key.as_str() {
                // Navigation
                "ArrowLeft" => {
                    if pos > 0 {
                        cursor_pos_for_key.set((pos - 1) as u16);
                    }
                    true
                }
                "ArrowRight" => {
                    if pos < char_count {
                        cursor_pos_for_key.set((pos + 1) as u16);
                    }
                    true
                }
                "Home" => {
                    cursor_pos_for_key.set(0);
                    true
                }
                "End" => {
                    cursor_pos_for_key.set(char_count as u16);
                    true
                }

                // Deletion
                "Backspace" => {
                    if pos > 0 {
                        let mut chars: Vec<char> = val.chars().collect();
                        chars.remove(pos - 1);
                        let new_val: String = chars.into_iter().collect();
                        value_for_key.set(new_val.clone());
                        cursor_pos_for_key.set((pos - 1) as u16);
                        if let Some(ref cb) = on_change {
                            cb(&new_val);
                        }
                    }
                    true
                }
                "Delete" => {
                    if pos < char_count {
                        let mut chars: Vec<char> = val.chars().collect();
                        chars.remove(pos);
                        let new_val: String = chars.into_iter().collect();
                        value_for_key.set(new_val.clone());
                        if let Some(ref cb) = on_change {
                            cb(&new_val);
                        }
                    }
                    true
                }

                // Submission
                "Enter" => {
                    if let Some(ref cb) = on_submit {
                        cb(&val);
                    }
                    true
                }

                // Cancel
                "Escape" => {
                    if let Some(ref cb) = on_cancel {
                        cb();
                    }
                    true
                }

                // Regular character input
                key => {
                    // Only single printable characters, no modifiers
                    if key.len() == 1
                        && !event.modifiers.alt
                        && !event.modifiers.meta
                    {
                        // Check max length
                        if max_length > 0 && char_count >= max_length {
                            return true;
                        }

                        let ch = key.chars().next().unwrap();
                        let mut chars: Vec<char> = val.chars().collect();
                        chars.insert(pos, ch);
                        let new_val: String = chars.into_iter().collect();
                        value_for_key.set(new_val.clone());
                        cursor_pos_for_key.set((pos + 1) as u16);
                        if let Some(ref cb) = on_change {
                            cb(&new_val);
                        }
                        true
                    } else {
                        false
                    }
                }
            }
        }
    });

    // ==========================================================================
    // MOUSE HANDLERS - Click to focus
    // ==========================================================================

    let user_on_click = props.on_click.clone();
    let click_handler: Rc<dyn Fn(&mouse::MouseEvent)> = Rc::new(move |event: &mouse::MouseEvent| {
        focus::focus(index);
        if let Some(ref handler) = user_on_click {
            handler(event);
        }
    });

    let handlers = mouse::MouseHandlers {
        on_mouse_down: props.on_mouse_down.clone(),
        on_mouse_up: props.on_mouse_up.clone(),
        on_click: Some(click_handler),
        on_mouse_enter: props.on_mouse_enter.clone(),
        on_mouse_leave: props.on_mouse_leave.clone(),
        on_scroll: props.on_scroll.clone(),
    };

    let mouse_cleanup = mouse::on_component(index, handlers);

    // ==========================================================================
    // AUTO FOCUS
    // ==========================================================================

    if props.auto_focus {
        focus::focus(index);
    }

    // ==========================================================================
    // CLEANUP
    // ==========================================================================

    Box::new(move || {
        // Clean up keyboard handlers
        key_cleanup();
        keyboard::cleanup_index(index);

        // Clean up mouse handlers
        mouse_cleanup();
        mouse::cleanup_index(index);

        // Clear cursor position array
        interaction::set_cursor_position(index, 0);

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
    use spark_signals::signal;

    fn setup() {
        reset_registry();
    }

    #[test]
    fn test_input_creation() {
        setup();

        let value = signal("".to_string());
        let cleanup = input(InputProps::new(value.clone()));

        // Check component was created
        assert_eq!(core::get_component_type(0), ComponentType::Input);
        assert!(interaction::get_focusable(0));

        // Cleanup
        cleanup();
        assert_eq!(core::get_component_type(0), ComponentType::None);
    }

    #[test]
    fn test_input_value_display() {
        setup();

        let value = signal("Hello".to_string());
        let _cleanup = input(InputProps::new(value.clone()));

        // Check text content displays value
        assert_eq!(text_arrays::get_text_content(0), "Hello");

        // Update value
        value.set("World".to_string());
        assert_eq!(text_arrays::get_text_content(0), "World");
    }

    #[test]
    fn test_input_placeholder() {
        setup();

        let value = signal("".to_string());
        let mut props = InputProps::new(value.clone());
        props.placeholder = Some("Enter text...".to_string());

        let _cleanup = input(props);

        // Should show placeholder when empty
        assert_eq!(text_arrays::get_text_content(0), "Enter text...");

        // Should show value when not empty
        value.set("Hi".to_string());
        assert_eq!(text_arrays::get_text_content(0), "Hi");

        // Should show placeholder again when cleared
        value.set("".to_string());
        assert_eq!(text_arrays::get_text_content(0), "Enter text...");
    }

    #[test]
    fn test_input_password_mode() {
        setup();

        let value = signal("secret".to_string());
        let mut props = InputProps::new(value.clone());
        props.password = true;

        let _cleanup = input(props);

        // Should mask characters
        assert_eq!(text_arrays::get_text_content(0), "••••••");

        // Update value
        value.set("hi".to_string());
        assert_eq!(text_arrays::get_text_content(0), "••");
    }

    #[test]
    fn test_input_custom_mask_char() {
        setup();

        let value = signal("abc".to_string());
        let mut props = InputProps::new(value.clone());
        props.password = true;
        props.mask_char = Some('*');

        let _cleanup = input(props);

        assert_eq!(text_arrays::get_text_content(0), "***");
    }

    #[test]
    fn test_input_always_focusable() {
        setup();

        let value = signal("".to_string());
        let _cleanup = input(InputProps::new(value));

        assert!(interaction::get_focusable(0));
    }

    #[test]
    fn test_input_tab_index() {
        setup();

        let value = signal("".to_string());
        let mut props = InputProps::new(value);
        props.tab_index = Some(5);

        let _cleanup = input(props);

        assert_eq!(interaction::get_tab_index(0), 5);
    }
}
