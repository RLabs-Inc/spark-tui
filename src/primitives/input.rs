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
use crate::state::{mouse, keyboard, focus, clipboard};
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
// Selection Helpers
// =============================================================================

/// Check if the component has an active selection.
fn has_selection(index: usize) -> bool {
    interaction::has_selection(index)
}

/// Get the selected text from a value string based on selection state.
/// Returns empty string if no selection.
fn get_selected_text(index: usize, text: &str) -> String {
    let start = interaction::get_selection_start(index) as usize;
    let end = interaction::get_selection_end(index) as usize;

    if start >= end {
        return String::new();
    }

    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();

    let actual_start = start.min(len);
    let actual_end = end.min(len);

    if actual_start >= actual_end {
        return String::new();
    }

    chars[actual_start..actual_end].iter().collect()
}

/// Clear the selection (set both start and end to 0).
fn clear_selection(index: usize) {
    interaction::clear_selection(index);
}

/// Delete the selected text from the value string.
/// Returns the new string and the position where cursor should be placed.
fn delete_selection(index: usize, text: &str) -> (String, usize) {
    let start = interaction::get_selection_start(index) as usize;
    let end = interaction::get_selection_end(index) as usize;

    if start >= end {
        return (text.to_string(), start);
    }

    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();

    let actual_start = start.min(len);
    let actual_end = end.min(len);

    let mut new_chars: Vec<char> = Vec::with_capacity(len - (actual_end - actual_start));
    new_chars.extend(&chars[..actual_start]);
    new_chars.extend(&chars[actual_end..]);

    (new_chars.into_iter().collect(), actual_start)
}

/// Get the normalized selection range (min, max).
fn get_selection_range(index: usize) -> (usize, usize) {
    let start = interaction::get_selection_start(index) as usize;
    let end = interaction::get_selection_end(index) as usize;
    (start.min(end), start.max(end))
}

// =============================================================================
// Scroll Offset Helpers
// =============================================================================

/// Adjust scroll offset to keep cursor visible within the visible width.
///
/// This ensures the cursor is always on-screen when text overflows.
/// The visible_width is typically the input's content width.
///
/// # Arguments
/// * `cursor_pos` - Current cursor position in characters
/// * `_text_len` - Total text length (unused but kept for future use)
/// * `scroll_offset` - Current horizontal scroll offset
/// * `visible_width` - Width of visible area in characters (0 = use default 40)
///
/// # Returns
/// New scroll offset that keeps cursor visible
pub fn ensure_cursor_visible(
    cursor_pos: usize,
    _text_len: usize,
    scroll_offset: u16,
    visible_width: u16,
) -> u16 {
    // Default visible width if not provided (will be refined when integrated with layout)
    let visible_width = if visible_width == 0 { 40 } else { visible_width };

    let cursor_in_view_start = scroll_offset as usize;
    let cursor_in_view_end = cursor_in_view_start + visible_width as usize;

    if cursor_pos < cursor_in_view_start {
        // Cursor is before visible area - scroll left
        cursor_pos as u16
    } else if cursor_pos >= cursor_in_view_end {
        // Cursor is after visible area - scroll right
        (cursor_pos.saturating_sub(visible_width as usize) + 1) as u16
    } else {
        // Cursor is visible - no change
        scroll_offset
    }
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

    // Scroll offset for text that extends beyond visible width
    // Stored in interaction arrays for renderer to use
    let scroll_offset = signal(0u16);

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

    // Initialize scroll offset in interaction arrays
    // This will be used by the renderer to show overflow indicators
    interaction::set_scroll_offset(index, 0, 0);

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
    let scroll_offset_for_key = scroll_offset.clone();
    let value_for_key = value.clone();
    let max_length = props.max_length.unwrap_or(0);
    let on_change = props.on_change.clone();
    let on_submit = props.on_submit.clone();
    let on_cancel = props.on_cancel.clone();
    let history = props.history.clone();

    let key_cleanup = keyboard::on_focused(index, move |event| {
        let val = value_for_key.get();
        let char_count = val.chars().count();
        // Clamp cursor position to value length (handles external value changes)
        let pos = cursor_pos_for_key.get().min(char_count as u16) as usize;
        let current_scroll = scroll_offset_for_key.get();

        // Helper: Update selection during Shift+navigation
        let update_selection = |new_pos: usize| {
            if has_selection(index) {
                // Extend/shrink existing selection
                let _sel_start = interaction::get_selection_start(index) as usize;
                let sel_end = interaction::get_selection_end(index) as usize;

                // The anchor is the end that's NOT at cursor position
                // Move the end that was at cursor position
                if pos == sel_end {
                    // Cursor was at end, move end
                    interaction::set_selection_end(index, new_pos as u16);
                } else {
                    // Cursor was at start, move start
                    interaction::set_selection_start(index, new_pos as u16);
                }
            } else {
                // Start new selection from current position to new position
                if new_pos < pos {
                    interaction::set_selection(index, new_pos as u16, pos as u16);
                } else {
                    interaction::set_selection(index, pos as u16, new_pos as u16);
                }
            }
        };

        // Handle Shift+Ctrl combinations (word selection)
        if event.modifiers.shift && event.modifiers.ctrl {
            match event.key.as_str() {
                "ArrowLeft" => {
                    let new_pos = find_word_start(&val, pos);
                    update_selection(new_pos);
                    cursor_pos_for_key.set(new_pos as u16);
                    true
                }
                "ArrowRight" => {
                    let new_pos = find_word_end(&val, pos);
                    update_selection(new_pos);
                    cursor_pos_for_key.set(new_pos as u16);
                    true
                }
                _ => false
            }
        }
        // Handle Shift combinations (character selection)
        else if event.modifiers.shift && !event.modifiers.ctrl {
            match event.key.as_str() {
                "ArrowLeft" => {
                    if pos > 0 {
                        let new_pos = pos - 1;
                        update_selection(new_pos);
                        cursor_pos_for_key.set(new_pos as u16);
                    }
                    true
                }
                "ArrowRight" => {
                    if pos < char_count {
                        let new_pos = pos + 1;
                        update_selection(new_pos);
                        cursor_pos_for_key.set(new_pos as u16);
                    }
                    true
                }
                "Home" => {
                    update_selection(0);
                    cursor_pos_for_key.set(0);
                    true
                }
                "End" => {
                    update_selection(char_count);
                    cursor_pos_for_key.set(char_count as u16);
                    true
                }
                _ => false
            }
        }
        // Handle Ctrl+key combinations (no shift)
        else if event.modifiers.ctrl {
            match event.key.as_str() {
                // Word navigation (clears selection)
                "ArrowLeft" => {
                    clear_selection(index);
                    let new_pos = find_word_start(&val, pos);
                    cursor_pos_for_key.set(new_pos as u16);
                    true
                }
                "ArrowRight" => {
                    clear_selection(index);
                    let new_pos = find_word_end(&val, pos);
                    cursor_pos_for_key.set(new_pos as u16);
                    true
                }

                // Word deletion
                "Backspace" => {
                    // If there's a selection, delete that first
                    if has_selection(index) {
                        let (new_val, new_pos) = delete_selection(index, &val);
                        value_for_key.set(new_val.clone());
                        cursor_pos_for_key.set(new_pos as u16);
                        clear_selection(index);
                        if let Some(ref cb) = on_change {
                            cb(&new_val);
                        }
                    } else if pos > 0 {
                        let word_start = find_word_start(&val, pos);
                        let mut chars: Vec<char> = val.chars().collect();
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
                    // If there's a selection, delete that first
                    if has_selection(index) {
                        let (new_val, new_pos) = delete_selection(index, &val);
                        value_for_key.set(new_val.clone());
                        cursor_pos_for_key.set(new_pos as u16);
                        clear_selection(index);
                        if let Some(ref cb) = on_change {
                            cb(&new_val);
                        }
                    } else if pos < char_count {
                        let word_end = find_word_end(&val, pos);
                        let mut chars: Vec<char> = val.chars().collect();
                        chars.drain(pos..word_end);
                        let new_val: String = chars.into_iter().collect();
                        value_for_key.set(new_val.clone());
                        if let Some(ref cb) = on_change {
                            cb(&new_val);
                        }
                    }
                    true
                }

                // Select all
                "a" | "A" => {
                    interaction::set_selection(index, 0, char_count as u16);
                    cursor_pos_for_key.set(char_count as u16);
                    true
                }

                // Clipboard: Copy
                "c" | "C" => {
                    if has_selection(index) {
                        let selected = get_selected_text(index, &val);
                        clipboard::copy(&selected);
                    }
                    true
                }

                // Clipboard: Cut
                "x" | "X" => {
                    if has_selection(index) {
                        let selected = get_selected_text(index, &val);
                        clipboard::copy(&selected);
                        let (new_val, new_pos) = delete_selection(index, &val);
                        value_for_key.set(new_val.clone());
                        cursor_pos_for_key.set(new_pos as u16);
                        clear_selection(index);
                        if let Some(ref cb) = on_change {
                            cb(&new_val);
                        }
                    }
                    true
                }

                // Clipboard: Paste
                "v" | "V" => {
                    if let Some(pasted) = clipboard::paste() {
                        // If there's a selection, replace it
                        let (base_val, insert_pos) = if has_selection(index) {
                            let result = delete_selection(index, &val);
                            clear_selection(index);
                            result
                        } else {
                            (val.clone(), pos)
                        };

                        // Check max length
                        let pasted_chars: Vec<char> = pasted.chars().collect();
                        let base_chars: Vec<char> = base_val.chars().collect();
                        let mut pasted_len = pasted_chars.len();

                        if max_length > 0 {
                            let available = max_length.saturating_sub(base_chars.len());
                            pasted_len = pasted_len.min(available);
                        }

                        if pasted_len > 0 {
                            let mut new_chars: Vec<char> = Vec::with_capacity(base_chars.len() + pasted_len);
                            new_chars.extend(&base_chars[..insert_pos.min(base_chars.len())]);
                            new_chars.extend(&pasted_chars[..pasted_len]);
                            if insert_pos < base_chars.len() {
                                new_chars.extend(&base_chars[insert_pos..]);
                            }
                            let new_val: String = new_chars.into_iter().collect();
                            value_for_key.set(new_val.clone());
                            cursor_pos_for_key.set((insert_pos + pasted_len) as u16);
                            // Reset history position on edit
                            if let Some(ref hist) = history {
                                hist.borrow_mut().reset_position();
                            }
                            if let Some(ref cb) = on_change {
                                cb(&new_val);
                            }
                        }
                    }
                    true
                }

                _ => false
            }
        }
        // Regular keys (no Ctrl, no Shift or Shift on non-navigation)
        else {
            match event.key.as_str() {
                // Navigation (clears selection)
                "ArrowLeft" => {
                    let new_pos = if has_selection(index) {
                        // Move to start of selection
                        let (sel_min, _) = get_selection_range(index);
                        clear_selection(index);
                        sel_min
                    } else if pos > 0 {
                        pos - 1
                    } else {
                        pos
                    };
                    cursor_pos_for_key.set(new_pos as u16);
                    // Update scroll offset to keep cursor visible
                    let new_scroll = ensure_cursor_visible(new_pos, char_count, current_scroll, 0);
                    if new_scroll != current_scroll {
                        scroll_offset_for_key.set(new_scroll);
                        interaction::set_scroll_offset(index, new_scroll, 0);
                    }
                    true
                }
                "ArrowRight" => {
                    let new_pos = if has_selection(index) {
                        // Move to end of selection
                        let (_, sel_max) = get_selection_range(index);
                        clear_selection(index);
                        sel_max
                    } else if pos < char_count {
                        pos + 1
                    } else {
                        pos
                    };
                    cursor_pos_for_key.set(new_pos as u16);
                    // Update scroll offset to keep cursor visible
                    let new_scroll = ensure_cursor_visible(new_pos, char_count, current_scroll, 0);
                    if new_scroll != current_scroll {
                        scroll_offset_for_key.set(new_scroll);
                        interaction::set_scroll_offset(index, new_scroll, 0);
                    }
                    true
                }
                // History navigation
                "ArrowUp" => {
                    if let Some(ref hist) = history {
                        if let Some(entry) = hist.borrow_mut().up(&val) {
                            let entry_owned = entry.to_string();
                            value_for_key.set(entry_owned.clone());
                            // Move cursor to end
                            cursor_pos_for_key.set(entry_owned.chars().count() as u16);
                            clear_selection(index);
                        }
                    }
                    true
                }
                "ArrowDown" => {
                    if let Some(ref hist) = history {
                        if let Some(entry) = hist.borrow_mut().down() {
                            value_for_key.set(entry.clone());
                            // Move cursor to end
                            cursor_pos_for_key.set(entry.chars().count() as u16);
                            clear_selection(index);
                        }
                    }
                    true
                }
                "Home" => {
                    clear_selection(index);
                    cursor_pos_for_key.set(0);
                    // Reset scroll to start
                    scroll_offset_for_key.set(0);
                    interaction::set_scroll_offset(index, 0, 0);
                    true
                }
                "End" => {
                    clear_selection(index);
                    cursor_pos_for_key.set(char_count as u16);
                    // Update scroll offset to show end of text
                    let new_scroll = ensure_cursor_visible(char_count, char_count, current_scroll, 0);
                    scroll_offset_for_key.set(new_scroll);
                    interaction::set_scroll_offset(index, new_scroll, 0);
                    true
                }

                // Deletion
                "Backspace" => {
                    if has_selection(index) {
                        let (new_val, new_pos) = delete_selection(index, &val);
                        value_for_key.set(new_val.clone());
                        cursor_pos_for_key.set(new_pos as u16);
                        clear_selection(index);
                        if let Some(ref cb) = on_change {
                            cb(&new_val);
                        }
                    } else if pos > 0 {
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
                    if has_selection(index) {
                        let (new_val, new_pos) = delete_selection(index, &val);
                        value_for_key.set(new_val.clone());
                        cursor_pos_for_key.set(new_pos as u16);
                        clear_selection(index);
                        if let Some(ref cb) = on_change {
                            cb(&new_val);
                        }
                    } else if pos < char_count {
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
                    // Add to history if tracking
                    if let Some(ref hist) = history {
                        hist.borrow_mut().push(val.clone());
                    }
                    if let Some(ref cb) = on_submit {
                        cb(&val);
                    }
                    true
                }

                // Cancel
                "Escape" => {
                    clear_selection(index);
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
                        // If there's a selection, replace it
                        let (base_val, insert_pos) = if has_selection(index) {
                            let result = delete_selection(index, &val);
                            clear_selection(index);
                            result
                        } else {
                            (val.clone(), pos)
                        };

                        let base_char_count = base_val.chars().count();

                        // Check max length
                        if max_length > 0 && base_char_count >= max_length {
                            return true;
                        }

                        let ch = key.chars().next().unwrap();
                        let mut chars: Vec<char> = base_val.chars().collect();
                        chars.insert(insert_pos.min(chars.len()), ch);
                        let new_val: String = chars.into_iter().collect();
                        value_for_key.set(new_val.clone());
                        cursor_pos_for_key.set((insert_pos + 1) as u16);
                        // Reset history position on edit
                        if let Some(ref hist) = history {
                            hist.borrow_mut().reset_position();
                        }
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

    // =========================================================================
    // Word Boundary Helper Tests
    // =========================================================================

    #[test]
    fn test_find_word_start_basic() {
        // "hello world" - from position 8 (middle of "world") should go to 6 (start of "world")
        assert_eq!(find_word_start("hello world", 8), 6);

        // From end of "world" (11) should go to start of "world" (6)
        assert_eq!(find_word_start("hello world", 11), 6);

        // From middle of "hello" (3) should go to 0
        assert_eq!(find_word_start("hello world", 3), 0);
    }

    #[test]
    fn test_find_word_start_at_word_boundary() {
        // At position 6 (start of "world"), should skip back over space to "hello"
        assert_eq!(find_word_start("hello world", 6), 0);

        // At position 5 (the space), should go to start of "hello"
        assert_eq!(find_word_start("hello world", 5), 0);
    }

    #[test]
    fn test_find_word_start_edge_cases() {
        // At position 0 - stays at 0
        assert_eq!(find_word_start("hello", 0), 0);

        // Empty string
        assert_eq!(find_word_start("", 0), 0);

        // Single word - from end should go to 0
        assert_eq!(find_word_start("hello", 5), 0);

        // Multiple spaces: "hello   world" - from position 10 should go to 8 (start of "world")
        assert_eq!(find_word_start("hello   world", 10), 8);
    }

    #[test]
    fn test_find_word_start_with_punctuation() {
        // "hello, world" - punctuation is non-alphanumeric, treated like space
        // From position 9 (middle of "world"), should go to 7
        assert_eq!(find_word_start("hello, world", 9), 7);

        // From position 7 (start of "world"), skips ", " and goes to start of "hello"
        assert_eq!(find_word_start("hello, world", 7), 0);
    }

    #[test]
    fn test_find_word_end_basic() {
        // "hello world" - from position 0 should go to 5 (end of "hello")
        assert_eq!(find_word_end("hello world", 0), 5);

        // From position 6 (start of "world") should go to 11 (end)
        assert_eq!(find_word_end("hello world", 6), 11);

        // From middle of "hello" (2) should go to 5
        assert_eq!(find_word_end("hello world", 2), 5);
    }

    #[test]
    fn test_find_word_end_at_word_boundary() {
        // At position 5 (end of "hello", before space), should skip space and go to end of "world"
        assert_eq!(find_word_end("hello world", 5), 11);
    }

    #[test]
    fn test_find_word_end_edge_cases() {
        // At end of string - stays at end
        assert_eq!(find_word_end("hello", 5), 5);

        // Empty string
        assert_eq!(find_word_end("", 0), 0);

        // Beyond string length
        assert_eq!(find_word_end("hello", 10), 5);

        // Multiple spaces: "hello   world" - from position 5 should skip spaces to end of "world"
        assert_eq!(find_word_end("hello   world", 5), 13);
    }

    #[test]
    fn test_find_word_end_with_punctuation() {
        // "hello, world" - from position 0 should go to 5 (end of "hello")
        assert_eq!(find_word_end("hello, world", 0), 5);

        // From position 5 (end of "hello"), skips ", " and ends at "world"
        assert_eq!(find_word_end("hello, world", 5), 12);
    }

    // =========================================================================
    // Selection Tests (using interaction arrays directly)
    // =========================================================================

    #[test]
    fn test_selection_getters_setters() {
        setup();

        // Default selection is 0,0
        assert_eq!(interaction::get_selection_start(0), 0);
        assert_eq!(interaction::get_selection_end(0), 0);

        // Set selection
        interaction::set_selection(0, 5, 10);
        assert_eq!(interaction::get_selection_start(0), 5);
        assert_eq!(interaction::get_selection_end(0), 10);

        // Has selection
        assert!(interaction::has_selection(0));

        // Clear selection
        interaction::clear_selection(0);
        assert_eq!(interaction::get_selection_start(0), 0);
        assert_eq!(interaction::get_selection_end(0), 0);
        assert!(!interaction::has_selection(0));
    }

    #[test]
    fn test_selection_individual_setters() {
        setup();

        interaction::set_selection_start(0, 3);
        interaction::set_selection_end(0, 7);

        assert_eq!(interaction::get_selection_start(0), 3);
        assert_eq!(interaction::get_selection_end(0), 7);
    }

    // =========================================================================
    // Selection Helper Tests
    // =========================================================================

    #[test]
    fn test_has_selection() {
        setup();

        // No selection
        assert!(!has_selection(0));

        // With selection
        interaction::set_selection(0, 2, 5);
        assert!(has_selection(0));

        // Same start and end = no selection
        interaction::set_selection(0, 3, 3);
        assert!(!has_selection(0));
    }

    #[test]
    fn test_get_selected_text() {
        setup();

        let text = "Hello World";

        // No selection
        interaction::set_selection(0, 0, 0);
        assert_eq!(get_selected_text(0, text), "");

        // Select "Hello"
        interaction::set_selection(0, 0, 5);
        assert_eq!(get_selected_text(0, text), "Hello");

        // Select "World"
        interaction::set_selection(0, 6, 11);
        assert_eq!(get_selected_text(0, text), "World");

        // Select middle
        interaction::set_selection(0, 3, 8);
        assert_eq!(get_selected_text(0, text), "lo Wo");

        // Selection beyond text length is clamped
        interaction::set_selection(0, 8, 20);
        assert_eq!(get_selected_text(0, text), "rld");
    }

    #[test]
    fn test_get_selected_text_unicode() {
        setup();

        let text = "Hello 世界";

        // Select unicode
        interaction::set_selection(0, 6, 8);
        assert_eq!(get_selected_text(0, text), "世界");

        // Select mixed
        interaction::set_selection(0, 4, 7);
        assert_eq!(get_selected_text(0, text), "o 世");
    }

    #[test]
    fn test_delete_selection() {
        setup();

        let text = "Hello World";

        // No selection - returns original
        interaction::set_selection(0, 0, 0);
        let (result, pos) = delete_selection(0, text);
        assert_eq!(result, "Hello World");
        assert_eq!(pos, 0);

        // Delete "Hello "
        interaction::set_selection(0, 0, 6);
        let (result, pos) = delete_selection(0, text);
        assert_eq!(result, "World");
        assert_eq!(pos, 0);

        // Delete "World"
        interaction::set_selection(0, 6, 11);
        let (result, pos) = delete_selection(0, text);
        assert_eq!(result, "Hello ");
        assert_eq!(pos, 6);

        // Delete middle
        interaction::set_selection(0, 2, 9);
        let (result, pos) = delete_selection(0, text);
        assert_eq!(result, "Held");
        assert_eq!(pos, 2);
    }

    #[test]
    fn test_get_selection_range() {
        setup();

        // Normal order
        interaction::set_selection(0, 2, 7);
        assert_eq!(get_selection_range(0), (2, 7));

        // Same position
        interaction::set_selection(0, 5, 5);
        assert_eq!(get_selection_range(0), (5, 5));
    }

    #[test]
    fn test_clear_selection() {
        setup();

        interaction::set_selection(0, 5, 10);
        assert!(has_selection(0));

        clear_selection(0);
        assert!(!has_selection(0));
        assert_eq!(interaction::get_selection_start(0), 0);
        assert_eq!(interaction::get_selection_end(0), 0);
    }

    // =========================================================================
    // Clipboard Integration Tests
    // =========================================================================

    #[test]
    fn test_clipboard_copy() {
        setup();
        clipboard::clear();

        // Set up selection
        interaction::set_selection(0, 0, 5);
        let text = "Hello World";

        // Get selected and copy
        let selected = get_selected_text(0, text);
        clipboard::copy(&selected);

        // Verify clipboard
        assert_eq!(clipboard::paste(), Some("Hello".to_string()));
    }

    #[test]
    fn test_clipboard_cut() {
        setup();
        clipboard::clear();

        let text = "Hello World";

        // Select "World"
        interaction::set_selection(0, 6, 11);

        // Cut operation
        let selected = get_selected_text(0, text);
        clipboard::copy(&selected);
        let (new_text, new_pos) = delete_selection(0, text);
        clear_selection(0);

        // Verify
        assert_eq!(clipboard::paste(), Some("World".to_string()));
        assert_eq!(new_text, "Hello ");
        assert_eq!(new_pos, 6);
        assert!(!has_selection(0));
    }

    #[test]
    fn test_clipboard_paste_no_selection() {
        setup();
        clipboard::clear();

        clipboard::copy("inserted");

        let text = "Hello World";
        let pos = 6; // After "Hello "

        // Simulate paste at position
        if let Some(pasted) = clipboard::paste() {
            let chars: Vec<char> = text.chars().collect();
            let pasted_chars: Vec<char> = pasted.chars().collect();
            let mut new_chars: Vec<char> = Vec::new();
            new_chars.extend(&chars[..pos]);
            new_chars.extend(&pasted_chars);
            new_chars.extend(&chars[pos..]);
            let new_text: String = new_chars.into_iter().collect();

            assert_eq!(new_text, "Hello insertedWorld");
        }
    }

    #[test]
    fn test_clipboard_paste_replaces_selection() {
        setup();
        clipboard::clear();

        clipboard::copy("REPLACED");

        let text = "Hello World";

        // Select "World"
        interaction::set_selection(0, 6, 11);

        // Delete selection first
        let (base_text, insert_pos) = delete_selection(0, text);
        clear_selection(0);

        // Then paste
        if let Some(pasted) = clipboard::paste() {
            let chars: Vec<char> = base_text.chars().collect();
            let pasted_chars: Vec<char> = pasted.chars().collect();
            let mut new_chars: Vec<char> = Vec::new();
            new_chars.extend(&chars[..insert_pos]);
            new_chars.extend(&pasted_chars);
            let new_text: String = new_chars.into_iter().collect();

            assert_eq!(new_text, "Hello REPLACED");
        }
    }

    #[test]
    fn test_typing_replaces_selection() {
        setup();

        let text = "Hello World";

        // Select "World"
        interaction::set_selection(0, 6, 11);

        // Type a character - should replace selection
        let (base_text, insert_pos) = delete_selection(0, text);
        clear_selection(0);

        let ch = 'X';
        let mut chars: Vec<char> = base_text.chars().collect();
        chars.insert(insert_pos, ch);
        let new_text: String = chars.into_iter().collect();

        assert_eq!(new_text, "Hello X");
        assert!(!has_selection(0));
    }

    #[test]
    fn test_selection_with_unicode() {
        setup();

        let text = "Hello 世界 Test";

        // Select "世界"
        interaction::set_selection(0, 6, 8);

        let selected = get_selected_text(0, text);
        assert_eq!(selected, "世界");

        // Delete selection
        let (new_text, pos) = delete_selection(0, text);
        assert_eq!(new_text, "Hello  Test");
        assert_eq!(pos, 6);
    }

    #[test]
    fn test_empty_selection_operations() {
        setup();
        clipboard::clear();

        let text = "Hello";

        // No selection
        interaction::set_selection(0, 0, 0);

        // get_selected_text returns empty
        assert_eq!(get_selected_text(0, text), "");

        // delete_selection returns original
        let (result, pos) = delete_selection(0, text);
        assert_eq!(result, "Hello");
        assert_eq!(pos, 0);

        // Copy empty does nothing
        let selected = get_selected_text(0, text);
        clipboard::copy(&selected);
        assert!(!clipboard::has_content());
    }

    // =========================================================================
    // InputHistory Tests
    // =========================================================================

    use super::super::types::InputHistory;

    #[test]
    fn test_input_history_navigation() {
        let mut history = InputHistory::new(vec![
            "first".to_string(),
            "second".to_string(),
            "third".to_string(),
        ]);

        // Navigate up (older entries)
        assert_eq!(history.up("current"), Some("third"));
        assert_eq!(history.up("current"), Some("second"));
        assert_eq!(history.up("current"), Some("first"));
        assert_eq!(history.up("current"), None); // At boundary

        // Navigate down (newer entries)
        assert_eq!(history.down(), Some("second".to_string()));
        assert_eq!(history.down(), Some("third".to_string()));
        assert_eq!(history.down(), Some("current".to_string())); // Back to editing value
        assert_eq!(history.down(), None); // Not in history
    }

    #[test]
    fn test_input_history_push() {
        let mut history = InputHistory::default();

        history.push("first".to_string());
        history.push("second".to_string());

        assert_eq!(history.entries, vec!["first", "second"]);

        // Don't add duplicates of last entry
        history.push("second".to_string());
        assert_eq!(history.entries, vec!["first", "second"]);

        // Don't add empty
        history.push(String::new());
        assert_eq!(history.entries, vec!["first", "second"]);
    }

    #[test]
    fn test_input_history_reset() {
        let mut history = InputHistory::new(vec!["old".to_string()]);

        // Enter history
        history.up("current");
        assert!(history.is_browsing());

        // Reset
        history.reset_position();
        assert!(!history.is_browsing());
    }

    #[test]
    fn test_input_history_max_entries() {
        let mut history = InputHistory::default();
        history.max_entries = 3;

        history.push("a".to_string());
        history.push("b".to_string());
        history.push("c".to_string());
        history.push("d".to_string());

        // Should only keep the last 3
        assert_eq!(history.entries.len(), 3);
        assert_eq!(history.entries, vec!["b", "c", "d"]);
    }

    #[test]
    fn test_input_history_preserves_editing_value() {
        let mut history = InputHistory::new(vec!["old".to_string()]);

        // Start editing "current", then go to history
        history.up("current");
        assert_eq!(history.editing_value, Some("current".to_string()));

        // Return from history
        let restored = history.down();
        assert_eq!(restored, Some("current".to_string()));
    }

    #[test]
    fn test_input_history_empty() {
        let mut history = InputHistory::default();

        // Can't navigate empty history
        assert_eq!(history.up("current"), None);
        assert_eq!(history.down(), None);
        assert!(!history.is_browsing());
    }

    // =========================================================================
    // Scroll Offset / ensure_cursor_visible Tests
    // =========================================================================

    #[test]
    fn test_ensure_cursor_visible_at_start() {
        // Cursor at start, no scroll needed
        assert_eq!(ensure_cursor_visible(0, 100, 0, 40), 0);
    }

    #[test]
    fn test_ensure_cursor_visible_in_view() {
        // Cursor in view, no change
        assert_eq!(ensure_cursor_visible(20, 100, 0, 40), 0);
        assert_eq!(ensure_cursor_visible(39, 100, 0, 40), 0);
    }

    #[test]
    fn test_ensure_cursor_visible_past_end() {
        // Cursor past end of view, scroll right
        assert_eq!(ensure_cursor_visible(50, 100, 0, 40), 11);
        assert_eq!(ensure_cursor_visible(40, 100, 0, 40), 1);
    }

    #[test]
    fn test_ensure_cursor_visible_before_view() {
        // Cursor before view, scroll left
        assert_eq!(ensure_cursor_visible(5, 100, 20, 40), 5);
        assert_eq!(ensure_cursor_visible(0, 100, 10, 40), 0);
    }

    #[test]
    fn test_ensure_cursor_visible_default_width() {
        // With width=0, should use default of 40
        assert_eq!(ensure_cursor_visible(50, 100, 0, 0), 11);
    }

    #[test]
    fn test_scroll_offset_interaction_array() {
        setup();

        // Scroll offset should be stored in interaction arrays
        interaction::set_scroll_offset(0, 10, 5);
        assert_eq!(interaction::get_scroll_offset_x(0), 10);
        assert_eq!(interaction::get_scroll_offset_y(0), 5);
    }

    // =========================================================================
    // Input with History Integration Test
    // =========================================================================

    #[test]
    fn test_input_with_history_prop() {
        setup();

        use std::rc::Rc;
        use std::cell::RefCell;

        let history = Rc::new(RefCell::new(InputHistory::default()));
        let value = signal(String::new());

        let mut props = InputProps::new(value.clone());
        props.history = Some(history.clone());

        let _cleanup = input(props);

        // Add some history entries
        history.borrow_mut().push("command1".to_string());
        history.borrow_mut().push("command2".to_string());

        // History should have 2 entries
        assert_eq!(history.borrow().entries.len(), 2);
    }
}
