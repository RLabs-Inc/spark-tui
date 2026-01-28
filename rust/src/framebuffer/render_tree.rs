//! Component tree rendering from SharedBuffer to FrameBuffer.
//!
//! Reads layout output, visual properties, text content, and interaction state
//! from the SharedBuffer. Renders each component to the 2D cell grid.
//!
//! # Traversal Order
//!
//! 1. Build child map from hierarchy section
//! 2. Sort children by z-index
//! 3. DFS traversal: background → border → content → children → focus indicator

use crate::renderer::{FrameBuffer, BorderSides, BorderColors};
use crate::shared_buffer::SharedBuffer;
use crate::types::{Attr, BorderStyle, ClipRect, Rgba};
use crate::layout::{string_width, truncate_text, wrap_text_word};
use super::inheritance::{get_inherited_fg, get_inherited_bg, get_effective_opacity, apply_opacity};

// =============================================================================
// Types
// =============================================================================

/// A hit region for mouse interaction detection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HitRegion {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub component_index: usize,
}

// Component type constants (matching META_COMPONENT_TYPE values)
const COMP_NONE: u8 = 0;
const COMP_BOX: u8 = 1;
const COMP_TEXT: u8 = 2;
const COMP_INPUT: u8 = 3;
const COMP_SELECT: u8 = 4;
const COMP_PROGRESS: u8 = 5;

// Text alignment constants (matching META_TEXT_ALIGN values)
const ALIGN_LEFT: u8 = 0;
const ALIGN_CENTER: u8 = 1;
const ALIGN_RIGHT: u8 = 2;

// Text wrap constants (matching META_TEXT_WRAP values)
const WRAP_NOWRAP: u8 = 0;
const WRAP_WRAP: u8 = 1;
const WRAP_TRUNCATE: u8 = 2;

// =============================================================================
// Entry Point
// =============================================================================

/// Compute the framebuffer from SharedBuffer data.
///
/// Reads layout output + visual + text + interaction sections.
/// Returns the filled FrameBuffer and collected hit regions.
pub fn compute_framebuffer(
    buf: &SharedBuffer,
    width: u16,
    height: u16,
) -> (FrameBuffer, Vec<HitRegion>) {
    let mut buffer = FrameBuffer::new(width, height);
    let mut hit_regions = Vec::new();

    let node_count = buf.node_count();
    if node_count == 0 {
        return (buffer, hit_regions);
    }

    // Build child map: parent_index → Vec<child_index>
    let mut child_map: Vec<Vec<usize>> = vec![Vec::new(); node_count];
    let mut roots: Vec<usize> = Vec::new();

    for i in 0..node_count {
        let comp_type = buf.component_type(i);
        if comp_type == COMP_NONE || !buf.visible(i) {
            continue;
        }

        match buf.parent_index(i) {
            Some(parent) if parent < node_count => {
                child_map[parent].push(i);
            }
            _ => {
                roots.push(i);
            }
        }
    }

    // Sort roots and children by z-index
    roots.sort_by_key(|&idx| buf.z_index(idx));
    for children in child_map.iter_mut() {
        if children.len() > 1 {
            children.sort_by_key(|&idx| buf.z_index(idx));
        }
    }

    // Render each root and its subtree
    for root_idx in &roots {
        render_component(
            &mut buffer,
            buf,
            *root_idx,
            &child_map,
            &mut hit_regions,
            None,  // no parent clip
            0, 0,  // no parent scroll
            0, 0,  // parent absolute position
        );
    }

    (buffer, hit_regions)
}

// =============================================================================
// Component Rendering
// =============================================================================

/// Render a component and its children recursively.
#[allow(clippy::too_many_arguments)]
fn render_component(
    buffer: &mut FrameBuffer,
    buf: &SharedBuffer,
    index: usize,
    child_map: &[Vec<usize>],
    hit_regions: &mut Vec<HitRegion>,
    parent_clip: Option<&ClipRect>,
    parent_scroll_x: i32,
    parent_scroll_y: i32,
    parent_abs_x: i32,
    parent_abs_y: i32,
) {
    // Visibility check
    if !buf.visible(index) || buf.component_type(index) == COMP_NONE {
        return;
    }

    // Read computed layout from output section
    let rel_x = buf.output_x(index) as i32;
    let rel_y = buf.output_y(index) as i32;
    let w = buf.output_width(index) as u16;
    let h = buf.output_height(index) as u16;

    if w == 0 || h == 0 {
        return;
    }

    // Absolute position: parent absolute + relative - scroll offset
    let abs_x = parent_abs_x + rel_x - parent_scroll_x;
    let abs_y = parent_abs_y + rel_y - parent_scroll_y;

    let x = abs_x.max(0) as u16;
    let y = abs_y.max(0) as u16;

    // Component bounds
    let component_bounds = ClipRect::new(x, y, w, h);

    // Effective clip (intersection with parent)
    let effective_clip = if let Some(parent) = parent_clip {
        match component_bounds.intersect(parent) {
            Some(clip) => clip,
            None => return, // completely clipped
        }
    } else {
        component_bounds
    };

    // Color inheritance + opacity
    let fg = get_inherited_fg(buf, index);
    let bg = get_inherited_bg(buf, index);
    let opacity = get_effective_opacity(buf, index);
    let effective_fg = apply_opacity(fg, opacity);
    let effective_bg = apply_opacity(bg, opacity);

    // Background fill
    if effective_bg.a > 0 && !effective_bg.is_terminal_default() {
        buffer.fill_rect(x, y, w, h, effective_bg, Some(&effective_clip));
    }

    // Collect hit region
    hit_regions.push(HitRegion {
        x,
        y,
        width: w,
        height: h,
        component_index: index,
    });

    // Render borders
    render_borders(buffer, buf, index, x, y, w, h, &effective_clip);

    // Calculate content area (inside borders + padding)
    let border_t = if buf.border_top_width(index) > 0 { 1u16 } else { 0 };
    let border_r = if buf.border_right_width(index) > 0 { 1u16 } else { 0 };
    let border_b = if buf.border_bottom_width(index) > 0 { 1u16 } else { 0 };
    let border_l = if buf.border_left_width(index) > 0 { 1u16 } else { 0 };

    let pad_top = buf.padding_top(index) as u16;
    let pad_right = buf.padding_right(index) as u16;
    let pad_bottom = buf.padding_bottom(index) as u16;
    let pad_left = buf.padding_left(index) as u16;

    let total_top = pad_top.saturating_add(border_t);
    let total_right = pad_right.saturating_add(border_r);
    let total_bottom = pad_bottom.saturating_add(border_b);
    let total_left = pad_left.saturating_add(border_l);

    let content_x = x.saturating_add(total_left);
    let content_y = y.saturating_add(total_top);
    let content_w = w.saturating_sub(total_left).saturating_sub(total_right);
    let content_h = h.saturating_sub(total_top).saturating_sub(total_bottom);

    if content_w == 0 || content_h == 0 {
        render_children(buffer, buf, index, child_map, hit_regions, &effective_clip, 0, 0, abs_x, abs_y);
        return;
    }

    let content_bounds = ClipRect::new(content_x, content_y, content_w, content_h);
    let content_clip = match content_bounds.intersect(&effective_clip) {
        Some(clip) => clip,
        None => {
            render_children(buffer, buf, index, child_map, hit_regions, &effective_clip, 0, 0, abs_x, abs_y);
            return;
        }
    };

    // Type dispatch
    let comp_type = buf.component_type(index);
    match comp_type {
        COMP_BOX => {
            // Background and borders already rendered
        }
        COMP_TEXT => {
            render_text(buffer, buf, index, content_x, content_y, content_w, content_h, effective_fg, &content_clip);
        }
        COMP_INPUT => {
            render_input(buffer, buf, index, content_x, content_y, content_w, content_h, effective_fg, effective_bg, &content_clip);
        }
        COMP_PROGRESS => {
            render_progress(buffer, buf, index, content_x, content_y, content_w, content_h, effective_fg, &content_clip);
        }
        COMP_SELECT => {
            render_select(buffer, buf, index, content_x, content_y, content_w, effective_fg, &content_clip);
        }
        _ => {}
    }

    // Render children
    render_children(buffer, buf, index, child_map, hit_regions, &content_clip, 0, 0, abs_x, abs_y);

    // Focus indicator: draw '*' at top-right if focused + focusable
    render_focus_indicator(buffer, buf, index, x, y, w, comp_type, &effective_clip, effective_fg);

    // Scrollbar
    if buf.output_scrollable(index) {
        let scrollbar_x = x.saturating_add(w).saturating_sub(1).saturating_sub(border_r);
        let scrollbar_y = y.saturating_add(border_t);
        let scrollbar_h = h.saturating_sub(border_t).saturating_sub(border_b);
        render_scrollbar(buffer, buf, index, scrollbar_x, scrollbar_y, scrollbar_h, effective_fg, &effective_clip);
    }
}

/// Render children of a component.
#[allow(clippy::too_many_arguments)]
fn render_children(
    buffer: &mut FrameBuffer,
    buf: &SharedBuffer,
    index: usize,
    child_map: &[Vec<usize>],
    hit_regions: &mut Vec<HitRegion>,
    clip: &ClipRect,
    parent_scroll_x: i32,
    parent_scroll_y: i32,
    parent_abs_x: i32,
    parent_abs_y: i32,
) {
    if index >= child_map.len() {
        return;
    }

    let children = &child_map[index];
    if children.is_empty() {
        return;
    }

    // Accumulate scroll offsets
    let scroll_x = if buf.output_scrollable(index) {
        buf.scroll_offset_x(index)
    } else {
        0
    };
    let scroll_y = if buf.output_scrollable(index) {
        buf.scroll_offset_y(index)
    } else {
        0
    };

    let child_scroll_x = parent_scroll_x + scroll_x;
    let child_scroll_y = parent_scroll_y + scroll_y;

    for &child_idx in children {
        render_component(
            buffer,
            buf,
            child_idx,
            child_map,
            hit_regions,
            Some(clip),
            child_scroll_x,
            child_scroll_y,
            parent_abs_x,
            parent_abs_y,
        );
    }
}

// =============================================================================
// Border Rendering
// =============================================================================

fn render_borders(
    buffer: &mut FrameBuffer,
    buf: &SharedBuffer,
    index: usize,
    x: u16,
    y: u16,
    w: u16,
    h: u16,
    clip: &ClipRect,
) {
    // Read per-side border styles
    let base_style = buf.border_style(index);
    let style_top = buf.border_style_top(index);
    let style_right = buf.border_style_right(index);
    let style_bottom = buf.border_style_bottom(index);
    let style_left = buf.border_style_left(index);

    // If per-side styles are all 0, use the base style
    let top = if style_top != 0 { BorderStyle::from(style_top) } else { BorderStyle::from(base_style) };
    let right = if style_right != 0 { BorderStyle::from(style_right) } else { BorderStyle::from(base_style) };
    let bottom = if style_bottom != 0 { BorderStyle::from(style_bottom) } else { BorderStyle::from(base_style) };
    let left = if style_left != 0 { BorderStyle::from(style_left) } else { BorderStyle::from(base_style) };

    // Check if any borders exist (by width)
    let has_top = buf.border_top_width(index) > 0 && top != BorderStyle::None;
    let has_right = buf.border_right_width(index) > 0 && right != BorderStyle::None;
    let has_bottom = buf.border_bottom_width(index) > 0 && bottom != BorderStyle::None;
    let has_left = buf.border_left_width(index) > 0 && left != BorderStyle::None;

    if !has_top && !has_right && !has_bottom && !has_left {
        return;
    }

    // Read per-side border colors
    let color_top = buf.border_color_top_rgba(index);
    let color_right = buf.border_color_right_rgba(index);
    let color_bottom = buf.border_color_bottom_rgba(index);
    let color_left = buf.border_color_left_rgba(index);

    // If all sides have same style, use simple draw_border
    if top == right && right == bottom && bottom == left
        && color_top == color_right && color_right == color_bottom && color_bottom == color_left
    {
        buffer.draw_border(x, y, w, h, top, color_top, None, Some(clip));
    } else {
        buffer.draw_border_sides(
            x, y, w, h,
            BorderSides {
                top: if has_top { top } else { BorderStyle::None },
                right: if has_right { right } else { BorderStyle::None },
                bottom: if has_bottom { bottom } else { BorderStyle::None },
                left: if has_left { left } else { BorderStyle::None },
            },
            BorderColors {
                top: color_top,
                right: color_right,
                bottom: color_bottom,
                left: color_left,
            },
            None,
            Some(clip),
        );
    }
}

// =============================================================================
// Text Rendering
// =============================================================================

fn render_text(
    buffer: &mut FrameBuffer,
    buf: &SharedBuffer,
    index: usize,
    x: u16,
    y: u16,
    w: u16,
    h: u16,
    fg: Rgba,
    clip: &ClipRect,
) {
    let content = buf.text_content(index);
    if content.is_empty() {
        return;
    }

    let attrs = Attr::from_bits_truncate(buf.text_attrs(index));
    let align = buf.text_align(index);
    let wrap = buf.text_wrap(index);

    // Handle text wrapping
    let lines: Vec<String> = match wrap {
        WRAP_WRAP => {
            wrap_text_word(content, w as usize)
                .into_iter()
                .map(|s| s.to_string())
                .collect()
        }
        WRAP_TRUNCATE => {
            // Single line, truncated
            let text_w = string_width(content);
            if text_w > w as usize {
                vec![truncate_text(content, w as usize, "…")]
            } else {
                vec![content.to_string()]
            }
        }
        _ => {
            // NoWrap: just split on newlines
            content.lines().map(|s| s.to_string()).collect()
        }
    };

    for (line_idx, line) in lines.iter().enumerate() {
        let line_y = y + line_idx as u16;
        if line_y >= y + h {
            break;
        }

        let text_width = string_width(line) as u16;

        // Alignment
        let draw_x = match align {
            ALIGN_CENTER => x + w.saturating_sub(text_width) / 2,
            ALIGN_RIGHT => x + w.saturating_sub(text_width),
            _ => x, // Left
        };

        buffer.draw_text(draw_x, line_y, line, fg, None, attrs, Some(clip));
    }
}

// =============================================================================
// Input Rendering
// =============================================================================

#[allow(clippy::too_many_arguments)]
fn render_input(
    buffer: &mut FrameBuffer,
    buf: &SharedBuffer,
    index: usize,
    x: u16,
    y: u16,
    w: u16,
    _h: u16,
    fg: Rgba,
    bg: Rgba,
    clip: &ClipRect,
) {
    let content = buf.text_content(index);
    let attrs = Attr::from_bits_truncate(buf.text_attrs(index));

    // Horizontal scroll offset
    let scroll_x = buf.scroll_offset_x(index) as usize;

    // Visible text after scroll
    let chars: Vec<char> = content.chars().collect();
    let visible_start = scroll_x.min(chars.len());
    let visible_chars: String = chars.iter().skip(visible_start).collect();

    // Truncate to fit width
    let display_text = if string_width(&visible_chars) > w as usize {
        truncate_text(&visible_chars, w as usize, "…")
    } else {
        visible_chars
    };

    // Draw text
    buffer.draw_text(x, y, &display_text, fg, None, attrs, Some(clip));

    // Render selection highlighting
    render_input_selection(buffer, buf, index, x, y, w, &chars, fg, bg, scroll_x, clip);

    // Render cursor
    render_input_cursor(buffer, buf, index, x, y, w, &chars, fg, bg, scroll_x, clip);
}

/// Render selection highlighting (inverse colors).
#[allow(clippy::too_many_arguments)]
fn render_input_selection(
    buffer: &mut FrameBuffer,
    buf: &SharedBuffer,
    index: usize,
    content_x: u16,
    content_y: u16,
    content_w: u16,
    chars: &[char],
    fg: Rgba,
    bg: Rgba,
    scroll_x: usize,
    clip: &ClipRect,
) {
    let sel_start = buf.selection_start(index) as usize;
    let sel_end = buf.selection_end(index) as usize;

    if sel_start >= sel_end {
        return;
    }

    for pos in sel_start..sel_end {
        let screen_pos = pos.saturating_sub(scroll_x);
        if screen_pos >= content_w as usize {
            break;
        }
        if pos < scroll_x {
            continue;
        }

        let render_x = content_x + screen_pos as u16;
        let ch = chars.get(pos).copied().unwrap_or(' ');

        // INVERSE for selection
        buffer.set_cell(render_x, content_y, ch as u32, bg, fg, Attr::INVERSE, Some(clip));
    }
}

/// Render cursor for input field.
///
/// Cursor styles:
/// - cursorChar == 0, visible → block (inverse fg/bg)
/// - cursorChar > 0, visible → custom char with cursor colors
/// - not visible → show alt char (blink off phase)
#[allow(clippy::too_many_arguments)]
fn render_input_cursor(
    buffer: &mut FrameBuffer,
    buf: &SharedBuffer,
    index: usize,
    content_x: u16,
    content_y: u16,
    content_w: u16,
    chars: &[char],
    fg: Rgba,
    bg: Rgba,
    scroll_x: usize,
    clip: &ClipRect,
) {
    // Only render when focused
    if !buf.focusable(index) {
        return;
    }
    // We check focus via the interaction section — focused_index is tracked
    // by the input system. For now, we use cursor_visible as a proxy:
    // if cursor_visible is set, it means this input is focused and cursor should show.

    let cursor_pos = buf.cursor_position(index) as usize;
    let screen_pos = cursor_pos.saturating_sub(scroll_x);
    if screen_pos >= content_w as usize {
        return;
    }

    let render_x = content_x + screen_pos as u16;
    let char_at_cursor = chars.get(cursor_pos).copied().unwrap_or(' ');

    let cursor_char = buf.cursor_char(index);
    let cursor_visible = buf.cursor_visible(index);

    if !cursor_visible {
        // Blink off phase: show alt char if set, otherwise show normal text
        let alt_char = buf.cursor_alt_char(index);
        if alt_char > 0 {
            if let Some(ch) = char::from_u32(alt_char as u32) {
                buffer.set_cell(render_x, content_y, ch as u32, fg, bg, Attr::NONE, Some(clip));
            }
        }
        return;
    }

    // Cursor visible
    let cursor_fg = buf.cursor_fg_rgba(index);
    let cursor_bg = buf.cursor_bg_rgba(index);

    if cursor_char == 0 {
        // Block cursor: inverse
        let effective_fg = if cursor_fg.is_terminal_default() { bg } else { cursor_fg };
        let effective_bg = if cursor_bg.is_terminal_default() { fg } else { cursor_bg };
        buffer.set_cell(render_x, content_y, char_at_cursor as u32, effective_fg, effective_bg, Attr::NONE, Some(clip));
    } else {
        // Custom cursor char with cursor colors
        let effective_fg = if cursor_fg.is_terminal_default() { fg } else { cursor_fg };
        let effective_bg = if cursor_bg.is_terminal_default() { bg } else { cursor_bg };
        buffer.set_cell(render_x, content_y, cursor_char as u32, effective_fg, effective_bg, Attr::NONE, Some(clip));
    }
}

// =============================================================================
// Focus Indicator
// =============================================================================

/// Render focus indicator: single '*' at top-right corner (box) or end of text (text).
fn render_focus_indicator(
    buffer: &mut FrameBuffer,
    buf: &SharedBuffer,
    index: usize,
    x: u16,
    y: u16,
    w: u16,
    comp_type: u8,
    clip: &ClipRect,
    fg: Rgba,
) {
    if !buf.focusable(index) || !buf.show_focus_ring(index) {
        return;
    }

    // Check if this node is currently focused
    // The focus system writes a focused_index; for now, check interaction arrays
    // The focus indicator only shows if the component has the focused state
    // We'll use hovered as proxy initially, but real focus uses a global focused_index
    // TODO: Read global focused_index from SharedBuffer header/interaction

    match comp_type {
        COMP_BOX => {
            // '*' at top-right corner
            let indicator_x = x.saturating_add(w).saturating_sub(1);
            buffer.draw_char(indicator_x, y, '*', fg, None, Attr::BOLD, Some(clip));
        }
        COMP_TEXT => {
            // '*' at end of text (we'd need text width, approximate with content area)
            // For simplicity, draw at top-right like box
            let indicator_x = x.saturating_add(w).saturating_sub(1);
            buffer.draw_char(indicator_x, y, '*', fg, None, Attr::BOLD, Some(clip));
        }
        _ => {}
    }
}

// =============================================================================
// Progress Bar
// =============================================================================

fn render_progress(
    buffer: &mut FrameBuffer,
    buf: &SharedBuffer,
    index: usize,
    x: u16,
    y: u16,
    w: u16,
    h: u16,
    fg: Rgba,
    clip: &ClipRect,
) {
    let content = buf.text_content(index);
    let progress: f32 = content.parse::<f32>().unwrap_or(0.0).clamp(0.0, 1.0);
    let bar_y = y + h / 2;
    buffer.draw_progress(x, bar_y, w, progress, '█', '░', fg, Rgba::GRAY, None, Some(clip));
}

// =============================================================================
// Select Dropdown
// =============================================================================

fn render_select(
    buffer: &mut FrameBuffer,
    buf: &SharedBuffer,
    index: usize,
    x: u16,
    y: u16,
    w: u16,
    fg: Rgba,
    clip: &ClipRect,
) {
    let content = buf.text_content(index);
    let attrs = Attr::from_bits_truncate(buf.text_attrs(index));

    let indicator = " ▼";
    let indicator_width: u16 = 2;
    let text_width = w.saturating_sub(indicator_width);

    let display_text = if string_width(content) > text_width as usize {
        truncate_text(content, text_width as usize, "…")
    } else {
        content.to_string()
    };

    buffer.draw_text(x, y, &display_text, fg, None, attrs, Some(clip));
    buffer.draw_text(x + w - indicator_width, y, indicator, fg, None, Attr::NONE, Some(clip));
}

// =============================================================================
// Scrollbar
// =============================================================================

const SCROLLBAR_TRACK: char = '░';
const SCROLLBAR_THUMB: char = '█';

fn render_scrollbar(
    buffer: &mut FrameBuffer,
    buf: &SharedBuffer,
    index: usize,
    x: u16,
    y: u16,
    h: u16,
    fg: Rgba,
    clip: &ClipRect,
) {
    let max_scroll_y = buf.output_max_scroll_y(index);
    if max_scroll_y <= 0.0 || h == 0 {
        return;
    }

    let scroll_y = buf.scroll_offset_y(index) as f32;
    let scrollbar_color = fg.dim(0.5);

    // Calculate thumb size and position
    let total_content = max_scroll_y + h as f32;
    let thumb_height = ((h as f32 / total_content) * h as f32).max(1.0) as u16;
    let thumb_pos = if max_scroll_y > 0.0 {
        ((scroll_y / max_scroll_y) * (h - thumb_height) as f32) as u16
    } else {
        0
    };

    // Draw track
    for row in 0..h {
        let draw_y = y + row;
        if clip.contains(x, draw_y) {
            buffer.draw_char(x, draw_y, SCROLLBAR_TRACK, scrollbar_color.dim(0.3), None, Attr::NONE, Some(clip));
        }
    }

    // Draw thumb
    for row in thumb_pos..(thumb_pos + thumb_height).min(h) {
        let draw_y = y + row;
        if clip.contains(x, draw_y) {
            buffer.draw_char(x, draw_y, SCROLLBAR_THUMB, scrollbar_color, None, Attr::NONE, Some(clip));
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hit_region_struct() {
        let hr = HitRegion {
            x: 10,
            y: 20,
            width: 30,
            height: 40,
            component_index: 5,
        };
        assert_eq!(hr.x, 10);
        assert_eq!(hr.component_index, 5);
    }
}
