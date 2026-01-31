//! Component tree rendering from SharedBuffer to FrameBuffer.
//!
//! Reads layout output, visual properties, text content, and interaction state
//! from the SharedBuffer. Renders each component to the 2D cell grid.
//!
//! # Coordinate System
//!
//! Taffy computes layout positions relative to parent's content box. To get
//! screen position, we transform through the parent chain:
//!
//! ```text
//! screen_position = parent_screen + layout_position - parent_scroll
//! ```
//!
//! Positions can be negative (scrolled out of view). We use i32 throughout
//! and only clamp to screen coordinates at final render time.
//!
//! # Traversal Order
//!
//! 1. Build child map from hierarchy section
//! 2. Sort children by z-index
//! 3. DFS traversal: background → border → content → children → focus indicator

use crate::renderer::FrameBuffer;
use crate::shared_buffer::{SharedBuffer, BorderStyle, COMPONENT_BOX, COMPONENT_TEXT, COMPONENT_INPUT};
use crate::utils::{Attr, ClipRect, Rgba};
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

// Component types (from SharedBuffer constants)
const COMP_NONE: u8 = 0;
const COMP_BOX: u8 = COMPONENT_BOX;
const COMP_TEXT: u8 = COMPONENT_TEXT;
const COMP_INPUT: u8 = COMPONENT_INPUT;
const COMP_SELECT: u8 = 4;
const COMP_PROGRESS: u8 = 5;

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

    // Screen bounds (root clip rect)
    let screen_clip = ClipRect::new(0, 0, width, height);

    // Render each root and its subtree
    for root_idx in &roots {
        render_component(
            &mut buffer,
            buf,
            *root_idx,
            &child_map,
            &mut hit_regions,
            &screen_clip,
            0, 0,  // parent screen position
        );
    }

    (buffer, hit_regions)
}

// =============================================================================
// Component Rendering
// =============================================================================

/// Render a component and its children recursively.
///
/// Arguments:
/// - `parent_clip`: The clipping rectangle from the parent
/// - `parent_screen_x/y`: Parent's absolute screen position (i32, can be negative)
#[allow(clippy::too_many_arguments)]
fn render_component(
    buffer: &mut FrameBuffer,
    buf: &SharedBuffer,
    index: usize,
    child_map: &[Vec<usize>],
    hit_regions: &mut Vec<HitRegion>,
    parent_clip: &ClipRect,
    parent_screen_x: i32,
    parent_screen_y: i32,
) {
    // Visibility check
    if !buf.visible(index) || buf.component_type(index) == COMP_NONE {
        return;
    }

    // Read computed layout from output section
    // These are positions relative to parent's content box
    let rel_x = buf.computed_x(index) as i32;
    let rel_y = buf.computed_y(index) as i32;
    let w = buf.computed_width(index) as u16;
    let h = buf.computed_height(index) as u16;

    if w == 0 || h == 0 {
        return;
    }

    // Read parent's scroll offset (if parent is scrollable)
    let parent_scroll_x = if let Some(parent_idx) = buf.parent_index(index) {
        if buf.is_scrollable(parent_idx) { buf.scroll_x(parent_idx) } else { 0 }
    } else {
        0
    };
    let parent_scroll_y = if let Some(parent_idx) = buf.parent_index(index) {
        if buf.is_scrollable(parent_idx) { buf.scroll_y(parent_idx) } else { 0 }
    } else {
        0
    };

    // Calculate screen position (can be negative if scrolled out of view)
    let screen_x = parent_screen_x + rel_x - parent_scroll_x;
    let screen_y = parent_screen_y + rel_y - parent_scroll_y;

    // Create component bounds (with signed x/y)
    let component_bounds = ClipRect::new(screen_x, screen_y, w, h);

    // Intersect with parent clip (handles negative positions correctly)
    let effective_clip = match component_bounds.intersect(parent_clip) {
        Some(clip) => clip,
        None => return, // Completely clipped out
    };

    // Get visible screen region (clamped to non-negative)
    let visible = match effective_clip.visible_on_screen() {
        Some(v) => v,
        None => return, // Nothing visible on screen
    };
    let (vis_x, vis_y, vis_w, vis_h) = visible;

    // Color inheritance + opacity
    let fg = get_inherited_fg(buf, index);
    let bg = get_inherited_bg(buf, index);
    let opacity = get_effective_opacity(buf, index);
    let effective_fg = apply_opacity(fg, opacity);
    let effective_bg = apply_opacity(bg, opacity);

    // Background fill (at screen coordinates)
    if effective_bg.a > 0 && !effective_bg.is_terminal_default() {
        buffer.fill_rect(vis_x, vis_y, vis_w, vis_h, effective_bg, Some(&effective_clip));
    }

    // Collect hit region (use visible coordinates)
    hit_regions.push(HitRegion {
        x: vis_x,
        y: vis_y,
        width: vis_w,
        height: vis_h,
        component_index: index,
    });

    // Render borders
    render_borders(buffer, buf, index, screen_x, screen_y, w, h, &effective_clip);

    // Calculate content area (inside borders + padding)
    let border_t = if buf.border_top(index) > 0 { 1i32 } else { 0 };
    let border_r = if buf.border_right(index) > 0 { 1i32 } else { 0 };
    let border_b = if buf.border_bottom(index) > 0 { 1i32 } else { 0 };
    let border_l = if buf.border_left(index) > 0 { 1i32 } else { 0 };

    let pad_top = buf.padding_top(index) as i32;
    let pad_right = buf.padding_right(index) as i32;
    let pad_bottom = buf.padding_bottom(index) as i32;
    let pad_left = buf.padding_left(index) as i32;

    let total_top = pad_top + border_t;
    let total_right = pad_right + border_r;
    let total_bottom = pad_bottom + border_b;
    let total_left = pad_left + border_l;

    let content_x = screen_x + total_left;
    let content_y = screen_y + total_top;
    let content_w = (w as i32 - total_left - total_right).max(0) as u16;
    let content_h = (h as i32 - total_top - total_bottom).max(0) as u16;

    if content_w == 0 || content_h == 0 {
        render_children(buffer, buf, index, child_map, hit_regions, &effective_clip, content_x, content_y);
        return;
    }

    let content_bounds = ClipRect::new(content_x, content_y, content_w, content_h);
    let content_clip = match content_bounds.intersect(&effective_clip) {
        Some(clip) => clip,
        None => {
            render_children(buffer, buf, index, child_map, hit_regions, &effective_clip, content_x, content_y);
            return;
        }
    };

    // Type dispatch for content rendering
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

    // Render children (pass content area as their parent screen position)
    render_children(buffer, buf, index, child_map, hit_regions, &content_clip, content_x, content_y);

    // Focus indicator
    render_focus_indicator(buffer, buf, index, screen_x, screen_y, w, comp_type, &effective_clip, effective_fg);

    // Scrollbar
    if buf.is_scrollable(index) {
        let scrollbar_x = (screen_x + w as i32 - 1 - border_r).max(0);
        let scrollbar_y = screen_y + border_t;
        let scrollbar_h = (h as i32 - border_t - border_b).max(0) as u16;
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
    parent_screen_x: i32,
    parent_screen_y: i32,
) {
    if index >= child_map.len() {
        return;
    }

    let children = &child_map[index];
    if children.is_empty() {
        return;
    }

    for &child_idx in children {
        render_component(
            buffer,
            buf,
            child_idx,
            child_map,
            hit_regions,
            clip,
            parent_screen_x,
            parent_screen_y,
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
    screen_x: i32,
    screen_y: i32,
    w: u16,
    h: u16,
    clip: &ClipRect,
) {
    // Check if any borders exist
    let has_top = buf.border_top(index) > 0;
    let has_right = buf.border_right(index) > 0;
    let has_bottom = buf.border_bottom(index) > 0;
    let has_left = buf.border_left(index) > 0;

    if !has_top && !has_right && !has_bottom && !has_left {
        return;
    }

    // Get border style and characters
    let style = buf.border_style(index);
    if style == BorderStyle::None {
        return;
    }

    let (h_char, v_char, tl_char, tr_char, bl_char, br_char) = buf.border_chars(index);

    // Get border color (convert from packed u32 to utils::Rgba)
    let border_color = Rgba::from_u32(buf.border_color(index));

    // Early return if nothing visible on screen
    if clip.visible_on_screen().is_none() {
        return;
    }

    // Draw borders (only if visible on screen)
    // We need to check each position against the clip rect

    // Top border
    if has_top && screen_y >= 0 {
        let y = screen_y as u16;
        if clip.contains(0, y) || screen_y >= clip.y {
            // Top-left corner
            if has_left && screen_x >= 0 && clip.contains_signed(screen_x, screen_y) {
                buffer.draw_char(screen_x.max(0) as u16, y, tl_char, border_color, None, Attr::NONE, Some(clip));
            }
            // Top edge
            let start_x = if has_left { screen_x + 1 } else { screen_x };
            let end_x = if has_right { screen_x + w as i32 - 1 } else { screen_x + w as i32 };
            for x in start_x..end_x {
                if x >= 0 && clip.contains_signed(x, screen_y) {
                    buffer.draw_char(x as u16, y, h_char, border_color, None, Attr::NONE, Some(clip));
                }
            }
            // Top-right corner
            if has_right && screen_x + w as i32 - 1 >= 0 && clip.contains_signed(screen_x + w as i32 - 1, screen_y) {
                buffer.draw_char((screen_x + w as i32 - 1).max(0) as u16, y, tr_char, border_color, None, Attr::NONE, Some(clip));
            }
        }
    }

    // Bottom border
    let bottom_y = screen_y + h as i32 - 1;
    if has_bottom && bottom_y >= 0 {
        let y = bottom_y as u16;
        // Bottom-left corner
        if has_left && screen_x >= 0 && clip.contains_signed(screen_x, bottom_y) {
            buffer.draw_char(screen_x.max(0) as u16, y, bl_char, border_color, None, Attr::NONE, Some(clip));
        }
        // Bottom edge
        let start_x = if has_left { screen_x + 1 } else { screen_x };
        let end_x = if has_right { screen_x + w as i32 - 1 } else { screen_x + w as i32 };
        for x in start_x..end_x {
            if x >= 0 && clip.contains_signed(x, bottom_y) {
                buffer.draw_char(x as u16, y, h_char, border_color, None, Attr::NONE, Some(clip));
            }
        }
        // Bottom-right corner
        if has_right && screen_x + w as i32 - 1 >= 0 && clip.contains_signed(screen_x + w as i32 - 1, bottom_y) {
            buffer.draw_char((screen_x + w as i32 - 1).max(0) as u16, y, br_char, border_color, None, Attr::NONE, Some(clip));
        }
    }

    // Left border (excluding corners)
    if has_left && screen_x >= 0 {
        let x = screen_x as u16;
        let start_y = if has_top { screen_y + 1 } else { screen_y };
        let end_y = if has_bottom { screen_y + h as i32 - 1 } else { screen_y + h as i32 };
        for y in start_y..end_y {
            if y >= 0 && clip.contains_signed(screen_x, y) {
                buffer.draw_char(x, y as u16, v_char, border_color, None, Attr::NONE, Some(clip));
            }
        }
    }

    // Right border (excluding corners)
    let right_x = screen_x + w as i32 - 1;
    if has_right && right_x >= 0 {
        let x = right_x as u16;
        let start_y = if has_top { screen_y + 1 } else { screen_y };
        let end_y = if has_bottom { screen_y + h as i32 - 1 } else { screen_y + h as i32 };
        for y in start_y..end_y {
            if y >= 0 && clip.contains_signed(right_x, y) {
                buffer.draw_char(x, y as u16, v_char, border_color, None, Attr::NONE, Some(clip));
            }
        }
    }
}

// =============================================================================
// Text Rendering
// =============================================================================

fn render_text(
    buffer: &mut FrameBuffer,
    buf: &SharedBuffer,
    index: usize,
    content_x: i32,
    content_y: i32,
    content_w: u16,
    content_h: u16,
    fg: Rgba,
    clip: &ClipRect,
) {
    let content = buf.text(index);
    if content.is_empty() {
        return;
    }

    let attrs = Attr::from_bits_truncate(buf.text_attrs(index));
    let align = buf.text_align(index);
    let wrap = buf.text_wrap(index);

    // Handle text wrapping
    let lines: Vec<String> = match wrap {
        crate::shared_buffer::TextWrap::Wrap => {
            wrap_text_word(content, content_w as usize)
                .into_iter()
                .map(|s| s.to_string())
                .collect()
        }
        crate::shared_buffer::TextWrap::Truncate => {
            let text_w = string_width(content);
            if text_w > content_w as usize {
                vec![truncate_text(content, content_w as usize, "...")]
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
        let line_y = content_y + line_idx as i32;
        if line_y >= content_y + content_h as i32 {
            break;
        }
        if line_y < 0 {
            continue;
        }

        let text_width = string_width(line) as u16;

        // Alignment
        let draw_x = match align {
            crate::shared_buffer::TextAlign::Center => content_x + (content_w.saturating_sub(text_width) / 2) as i32,
            crate::shared_buffer::TextAlign::Right => content_x + content_w.saturating_sub(text_width) as i32,
            _ => content_x, // Left
        };

        if draw_x >= 0 {
            buffer.draw_text(draw_x as u16, line_y as u16, line, fg, None, attrs, Some(clip));
        }
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
    content_x: i32,
    content_y: i32,
    content_w: u16,
    _content_h: u16,
    fg: Rgba,
    bg: Rgba,
    clip: &ClipRect,
) {
    if content_x < 0 || content_y < 0 {
        return; // Off screen
    }

    let x = content_x as u16;
    let y = content_y as u16;

    let content = buf.text(index);
    let attrs = Attr::from_bits_truncate(buf.text_attrs(index));

    // Horizontal scroll offset
    let scroll_x = buf.scroll_x(index) as usize;

    // Visible text after scroll
    let chars: Vec<char> = content.chars().collect();
    let visible_start = scroll_x.min(chars.len());
    let visible_chars: String = chars.iter().skip(visible_start).collect();

    // Truncate to fit width
    let display_text = if string_width(&visible_chars) > content_w as usize {
        truncate_text(&visible_chars, content_w as usize, "...")
    } else {
        visible_chars
    };

    // Draw text
    buffer.draw_text(x, y, &display_text, fg, None, attrs, Some(clip));

    // Render selection highlighting
    render_input_selection(buffer, buf, index, x, y, content_w, &chars, fg, bg, scroll_x, clip);

    // Render cursor
    render_input_cursor(buffer, buf, index, x, y, content_w, &chars, fg, bg, scroll_x, clip);
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
        // Blink off phase: show alt char if set
        let alt_char = buf.cursor_alt_char(index);
        if alt_char > 0 {
            if let Some(ch) = char::from_u32(alt_char) {
                buffer.set_cell(render_x, content_y, ch as u32, fg, bg, Attr::NONE, Some(clip));
            }
        }
        return;
    }

    // Cursor visible
    let cursor_fg = Rgba::from_u32(buf.cursor_fg_color(index));
    let cursor_bg = Rgba::from_u32(buf.cursor_bg_color(index));

    if cursor_char == 0 {
        // Block cursor: inverse
        let effective_fg = if cursor_fg.is_terminal_default() { bg } else { cursor_fg };
        let effective_bg = if cursor_bg.is_terminal_default() { fg } else { cursor_bg };
        buffer.set_cell(render_x, content_y, char_at_cursor as u32, effective_fg, effective_bg, Attr::NONE, Some(clip));
    } else {
        // Custom cursor char with cursor colors
        let effective_fg = if cursor_fg.is_terminal_default() { fg } else { cursor_fg };
        let effective_bg = if cursor_bg.is_terminal_default() { bg } else { cursor_bg };
        buffer.set_cell(render_x, content_y, cursor_char, effective_fg, effective_bg, Attr::NONE, Some(clip));
    }
}

// =============================================================================
// Focus Indicator
// =============================================================================

fn render_focus_indicator(
    buffer: &mut FrameBuffer,
    buf: &SharedBuffer,
    index: usize,
    screen_x: i32,
    screen_y: i32,
    w: u16,
    comp_type: u8,
    clip: &ClipRect,
    fg: Rgba,
) {
    if !buf.focusable(index) || !buf.focus_indicator_enabled(index) {
        return;
    }

    // Only show indicator on the CURRENTLY focused element
    let focused = buf.focused_index();
    if focused < 0 || focused as usize != index {
        return;
    }

    let indicator_char = buf.focus_indicator_char(index);

    // Position: top-right corner
    let indicator_x = screen_x + w as i32 - 1;
    let indicator_y = screen_y;

    if indicator_x < 0 || indicator_y < 0 {
        return;
    }

    match comp_type {
        COMP_BOX | COMP_TEXT => {
            if clip.contains_signed(indicator_x, indicator_y) {
                buffer.draw_char(indicator_x as u16, indicator_y as u16, indicator_char, fg, None, Attr::BOLD, Some(clip));
            }
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
    content_x: i32,
    content_y: i32,
    content_w: u16,
    content_h: u16,
    fg: Rgba,
    clip: &ClipRect,
) {
    if content_x < 0 || content_y < 0 {
        return;
    }

    let content = buf.text(index);
    let progress: f32 = content.parse::<f32>().unwrap_or(0.0).clamp(0.0, 1.0);
    let bar_y = content_y + (content_h / 2) as i32;

    if bar_y >= 0 {
        buffer.draw_progress(
            content_x as u16, bar_y as u16, content_w,
            progress, '█', '░', fg, Rgba::GRAY, None, Some(clip),
        );
    }
}

// =============================================================================
// Select Dropdown
// =============================================================================

fn render_select(
    buffer: &mut FrameBuffer,
    buf: &SharedBuffer,
    index: usize,
    content_x: i32,
    content_y: i32,
    content_w: u16,
    fg: Rgba,
    clip: &ClipRect,
) {
    if content_x < 0 || content_y < 0 {
        return;
    }

    let x = content_x as u16;
    let y = content_y as u16;

    let content = buf.text(index);
    let attrs = Attr::from_bits_truncate(buf.text_attrs(index));

    let indicator = " \u{25BC}"; // Down arrow
    let indicator_width: u16 = 2;
    let text_width = content_w.saturating_sub(indicator_width);

    let display_text = if string_width(content) > text_width as usize {
        truncate_text(content, text_width as usize, "...")
    } else {
        content.to_string()
    };

    buffer.draw_text(x, y, &display_text, fg, None, attrs, Some(clip));
    buffer.draw_text(x + content_w - indicator_width, y, indicator, fg, None, Attr::NONE, Some(clip));
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
    x: i32,
    y: i32,
    h: u16,
    fg: Rgba,
    clip: &ClipRect,
) {
    let max_scroll_y = buf.max_scroll_y(index);
    if max_scroll_y <= 0.0 || h == 0 || x < 0 || y < 0 {
        return;
    }

    let x = x as u16;
    let scroll_y = buf.scroll_y(index) as f32;
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
        let draw_y = y + row as i32;
        if draw_y >= 0 && clip.contains(x, draw_y as u16) {
            buffer.draw_char(x, draw_y as u16, SCROLLBAR_TRACK, scrollbar_color.dim(0.3), None, Attr::NONE, Some(clip));
        }
    }

    // Draw thumb
    for row in thumb_pos..(thumb_pos + thumb_height).min(h) {
        let draw_y = y + row as i32;
        if draw_y >= 0 && clip.contains(x, draw_y as u16) {
            buffer.draw_char(x, draw_y as u16, SCROLLBAR_THUMB, scrollbar_color, None, Attr::NONE, Some(clip));
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
