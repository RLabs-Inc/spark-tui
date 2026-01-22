//! FrameBuffer Derived - Reactive frame buffer computation.
//!
//! Creates a Derived that renders all visible components to a FrameBuffer
//! whenever the layout or visual properties change.

use std::collections::HashMap;

use spark_signals::{derived, Derived};

use crate::engine::arrays::{core, visual, text, interaction};
use crate::engine::{get_allocated_indices, get_flex_node};
use crate::layout::ComputedLayout;
use crate::renderer::FrameBuffer;
use crate::types::{Attr, BorderStyle, ClipRect, ComponentType, Overflow, Rgba, TextAlign, TextWrap};
use super::inheritance::{get_inherited_fg, get_inherited_bg, get_effective_opacity, apply_opacity};
use super::terminal::{terminal_width_signal, terminal_height_signal, render_mode_signal, RenderMode};

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

/// Result of frame buffer computation.
#[derive(Debug, Clone, PartialEq)]
pub struct FrameBufferResult {
    /// The rendered frame buffer.
    pub buffer: FrameBuffer,
    /// Hit regions for mouse interaction (collected as data, not side effects).
    pub hit_regions: Vec<HitRegion>,
    /// Terminal size at time of render.
    pub terminal_size: (u16, u16),
}

// =============================================================================
// FrameBuffer Derived Factory
// =============================================================================

/// Create the frame buffer derived.
///
/// Takes the layout derived as input and returns a Derived that produces
/// the rendered FrameBuffer whenever layout or visual properties change.
pub fn create_frame_buffer_derived<F>(
    layout_derived: Derived<ComputedLayout, F>,
) -> Derived<FrameBufferResult, impl Fn() -> FrameBufferResult>
where
    F: Fn() -> ComputedLayout + 'static,
{
    let tw_signal = terminal_width_signal();
    let th_signal = terminal_height_signal();
    let mode_signal = render_mode_signal();

    derived(move || {
        // Read terminal dimensions
        let tw = tw_signal.get();
        let th = th_signal.get();
        let mode = mode_signal.get();

        // Read layout (creates dependency on layoutDerived)
        let computed_layout = layout_derived.get();

        // Determine buffer height based on render mode
        let buffer_height = match mode {
            RenderMode::Fullscreen => th,
            RenderMode::Inline | RenderMode::Append => {
                computed_layout.content_height.max(1)
            }
        };

        // Create frame buffer
        let mut buffer = FrameBuffer::new(tw, buffer_height);

        // Collect hit regions
        let mut hit_regions = Vec::new();

        // Get all allocated indices and sort them
        let mut indices = get_allocated_indices();
        if indices.is_empty() {
            return FrameBufferResult {
                buffer,
                hit_regions,
                terminal_size: (tw, th),
            };
        }
        indices.sort_unstable();

        // Build parent-child map
        let mut child_map: HashMap<usize, Vec<usize>> = HashMap::new();
        let mut roots: Vec<usize> = Vec::new();

        for &idx in &indices {
            if !core::get_visible(idx) {
                continue;
            }

            if let Some(parent_idx) = core::get_parent_index(idx) {
                child_map.entry(parent_idx).or_default().push(idx);
            } else {
                roots.push(idx);
            }
        }

        // Sort roots and children by z-index
        roots.sort_by_key(|&idx| visual::get_z_index(idx));
        for children in child_map.values_mut() {
            children.sort_by_key(|&idx| visual::get_z_index(idx));
        }

        // Render each root and its children
        for root_idx in roots {
            render_component(
                &mut buffer,
                root_idx,
                &computed_layout,
                &child_map,
                &mut hit_regions,
                None,  // No parent clip
                0,     // No parent scroll Y
                0,     // No parent scroll X
                0,     // Parent absolute X
                0,     // Parent absolute Y
            );
        }

        FrameBufferResult {
            buffer,
            hit_regions,
            terminal_size: (tw, th),
        }
    })
}

// =============================================================================
// Component Rendering
// =============================================================================

/// Render a component and its children recursively.
#[allow(clippy::too_many_arguments)]
fn render_component(
    buffer: &mut FrameBuffer,
    index: usize,
    layout: &ComputedLayout,
    child_map: &HashMap<usize, Vec<usize>>,
    hit_regions: &mut Vec<HitRegion>,
    parent_clip: Option<&ClipRect>,
    parent_scroll_y: i32,
    parent_scroll_x: i32,
    parent_abs_x: i32,
    parent_abs_y: i32,
) {
    // Check visibility
    if !core::get_visible(index) {
        return;
    }

    // Get computed position (relative to parent) and size
    let rel_x = layout.x.get(index).copied().unwrap_or(0) as i32;
    let rel_y = layout.y.get(index).copied().unwrap_or(0) as i32;
    let w = layout.width.get(index).copied().unwrap_or(0);
    let h = layout.height.get(index).copied().unwrap_or(0);

    if w == 0 || h == 0 {
        return;
    }

    // Calculate absolute position: parent absolute + relative + scroll offset
    let abs_x = parent_abs_x + rel_x - parent_scroll_x;
    let abs_y = parent_abs_y + rel_y - parent_scroll_y;

    let x = abs_x.max(0) as u16;
    let y = abs_y.max(0) as u16;

    // Create component bounds
    let component_bounds = ClipRect::new(x, y, w, h);

    // Compute effective clip (intersection with parent)
    let effective_clip = if let Some(parent) = parent_clip {
        match component_bounds.intersect(parent) {
            Some(clip) => clip,
            None => return, // Completely clipped
        }
    } else {
        component_bounds
    };

    // Get colors with inheritance
    let fg = get_inherited_fg(index);
    let bg = get_inherited_bg(index);
    let opacity = get_effective_opacity(index);

    // Apply opacity
    let effective_fg = apply_opacity(fg, opacity);
    let effective_bg = apply_opacity(bg, opacity);

    // Render background
    if effective_bg.a > 0 && !effective_bg.is_terminal_default() {
        buffer.fill_rect(x, y, w, h, effective_bg, Some(&effective_clip));
    }

    // Collect hit region (as data, not side effect!)
    hit_regions.push(HitRegion {
        x,
        y,
        width: w,
        height: h,
        component_index: index,
    });

    // Render borders
    render_borders(buffer, index, x, y, w, h, &effective_clip);

    // Get padding and border widths from FlexNode
    let (pad_top, pad_right, pad_bottom, pad_left, border_t, border_r, border_b, border_l) =
        if let Some(node) = get_flex_node(index) {
            (
                node.padding_top.get(),
                node.padding_right.get(),
                node.padding_bottom.get(),
                node.padding_left.get(),
                if node.border_top.get() > 0 { 1u16 } else { 0 },
                if node.border_right.get() > 0 { 1u16 } else { 0 },
                if node.border_bottom.get() > 0 { 1u16 } else { 0 },
                if node.border_left.get() > 0 { 1u16 } else { 0 },
            )
        } else {
            (0, 0, 0, 0, 0, 0, 0, 0)
        };

    // Calculate content area (inside borders and padding)
    let total_top = pad_top.saturating_add(border_t);
    let total_right = pad_right.saturating_add(border_r);
    let total_bottom = pad_bottom.saturating_add(border_b);
    let total_left = pad_left.saturating_add(border_l);

    let content_x = x.saturating_add(total_left);
    let content_y = y.saturating_add(total_top);
    let content_w = w.saturating_sub(total_left).saturating_sub(total_right);
    let content_h = h.saturating_sub(total_top).saturating_sub(total_bottom);

    if content_w == 0 || content_h == 0 {
        // Still render children even if no content area
        render_children(buffer, index, layout, child_map, hit_regions, &effective_clip, parent_scroll_y, parent_scroll_x, abs_x, abs_y);
        return;
    }

    let content_bounds = ClipRect::new(content_x, content_y, content_w, content_h);
    let content_clip = match content_bounds.intersect(&effective_clip) {
        Some(clip) => clip,
        None => {
            render_children(buffer, index, layout, child_map, hit_regions, &effective_clip, parent_scroll_y, parent_scroll_x, abs_x, abs_y);
            return;
        }
    };

    // Render based on component type
    let comp_type = core::get_component_type(index);
    match comp_type {
        ComponentType::Box => {
            // Background and borders already rendered
        }
        ComponentType::Text => {
            render_text(buffer, index, content_x, content_y, content_w, content_h, effective_fg, &content_clip);
        }
        ComponentType::Input => {
            render_input(buffer, index, content_x, content_y, content_w, content_h, effective_fg, &content_clip);
        }
        ComponentType::Progress => {
            render_progress(buffer, index, content_x, content_y, content_w, content_h, effective_fg, &content_clip);
        }
        ComponentType::Select => {
            render_select(buffer, index, content_x, content_y, content_w, content_h, effective_fg, &content_clip);
        }
        _ => {}
    }

    // Render children - pass this component's absolute position
    render_children(buffer, index, layout, child_map, hit_regions, &content_clip, parent_scroll_y, parent_scroll_x, abs_x, abs_y);

    // Render scrollbar (on right edge of content area, overlays content)
    // Place scrollbar at the right edge of the box, inside borders
    let scrollbar_x = x.saturating_add(w).saturating_sub(1).saturating_sub(border_r);
    let scrollbar_y = y.saturating_add(border_t);
    let scrollbar_h = h.saturating_sub(border_t).saturating_sub(border_b);
    render_scrollbar(buffer, layout, index, scrollbar_x, scrollbar_y, scrollbar_h, &effective_clip);
}

/// Render children of a component.
#[allow(clippy::too_many_arguments)]
fn render_children(
    buffer: &mut FrameBuffer,
    index: usize,
    layout: &ComputedLayout,
    child_map: &HashMap<usize, Vec<usize>>,
    hit_regions: &mut Vec<HitRegion>,
    clip: &ClipRect,
    parent_scroll_y: i32,
    parent_scroll_x: i32,
    parent_abs_x: i32,
    parent_abs_y: i32,
) {
    if let Some(children) = child_map.get(&index) {
        let is_scrollable = layout.scrollable.get(index).copied().unwrap_or(0) == 1;
        let scroll_y = if is_scrollable {
            interaction::get_scroll_offset_y(index) as i32
        } else {
            0
        };
        let scroll_x = if is_scrollable {
            interaction::get_scroll_offset_x(index) as i32
        } else {
            0
        };

        let child_scroll_y = parent_scroll_y + scroll_y;
        let child_scroll_x = parent_scroll_x + scroll_x;

        for &child_idx in children {
            render_component(
                buffer,
                child_idx,
                layout,
                child_map,
                hit_regions,
                Some(clip),
                child_scroll_y,
                child_scroll_x,
                parent_abs_x,
                parent_abs_y,
            );
        }
    }
}

// =============================================================================
// Component-Specific Renderers
// =============================================================================

/// Render borders for a component.
fn render_borders(
    buffer: &mut FrameBuffer,
    index: usize,
    x: u16,
    y: u16,
    w: u16,
    h: u16,
    clip: &ClipRect,
) {
    let style = visual::get_border_style(index);
    if style == BorderStyle::None {
        return;
    }

    let color = visual::get_border_color(index);

    // Use the simple draw_border API with single style/color
    buffer.draw_border(x, y, w, h, style, color, None, Some(clip));
}

// =============================================================================
// Scrollbar Rendering
// =============================================================================

/// Scrollbar character constants.
const SCROLLBAR_TRACK: char = '░';
const SCROLLBAR_THUMB: char = '█';
const SCROLL_INDICATOR: char = '▐';

/// Render scrollbar for a scrollable component.
///
/// For overflow:scroll - renders full scrollbar (track + thumb).
/// For overflow:auto - renders minimal scroll indicator.
fn render_scrollbar(
    buffer: &mut FrameBuffer,
    layout: &ComputedLayout,
    index: usize,
    x: u16,
    y: u16,
    h: u16,
    clip: &ClipRect,
) {
    // Check if scrollable
    let is_scrollable = layout.scrollable.get(index).copied().unwrap_or(0) == 1;
    if !is_scrollable {
        return;
    }

    // Get overflow mode from FlexNode
    let overflow_value = if let Some(node) = get_flex_node(index) {
        node.overflow.get()
    } else {
        0
    };
    let overflow = Overflow::from(overflow_value);

    // Get scroll metrics
    let scroll_y = interaction::get_scroll_offset_y(index);
    let max_scroll_y = layout.max_scroll_y.get(index).copied().unwrap_or(0);

    // Don't render if no scrollable content
    if max_scroll_y == 0 {
        return;
    }

    // Get colors (use border color or dim fg)
    let fg = get_inherited_fg(index);
    let scrollbar_color = fg.dim(0.5);

    match overflow {
        Overflow::Scroll => {
            // Full scrollbar with track and thumb
            render_full_scrollbar(buffer, x, y, h, scroll_y, max_scroll_y, scrollbar_color, clip);
        }
        Overflow::Auto => {
            // Minimal scroll indicator
            render_scroll_indicator(buffer, x, y, h, scroll_y, max_scroll_y, scrollbar_color, clip);
        }
        _ => {}
    }
}

/// Render full scrollbar with track and thumb for overflow:scroll.
fn render_full_scrollbar(
    buffer: &mut FrameBuffer,
    x: u16,
    y: u16,
    height: u16,
    scroll_y: u16,
    max_scroll_y: u16,
    color: Rgba,
    clip: &ClipRect,
) {
    if height == 0 {
        return;
    }

    // Calculate thumb size and position
    let total_content = max_scroll_y + height;
    let thumb_height = ((height as f32 / total_content as f32) * height as f32).max(1.0) as u16;
    let thumb_pos = if max_scroll_y > 0 {
        ((scroll_y as f32 / max_scroll_y as f32) * (height - thumb_height) as f32) as u16
    } else {
        0
    };

    // Render track
    for row in 0..height {
        let draw_y = y + row;
        if clip.contains(x, draw_y) {
            buffer.draw_char(x, draw_y, SCROLLBAR_TRACK, color.dim(0.3), None, Attr::NONE, Some(clip));
        }
    }

    // Render thumb
    for row in thumb_pos..(thumb_pos + thumb_height).min(height) {
        let draw_y = y + row;
        if clip.contains(x, draw_y) {
            buffer.draw_char(x, draw_y, SCROLLBAR_THUMB, color, None, Attr::NONE, Some(clip));
        }
    }
}

/// Render minimal scroll indicator for overflow:auto.
fn render_scroll_indicator(
    buffer: &mut FrameBuffer,
    x: u16,
    y: u16,
    height: u16,
    scroll_y: u16,
    max_scroll_y: u16,
    color: Rgba,
    clip: &ClipRect,
) {
    if height == 0 || max_scroll_y == 0 {
        return;
    }

    // Calculate indicator position
    let indicator_pos = if max_scroll_y > 0 {
        ((scroll_y as f32 / max_scroll_y as f32) * (height - 1) as f32) as u16
    } else {
        0
    };

    let draw_y = y + indicator_pos;
    if clip.contains(x, draw_y) {
        buffer.draw_char(x, draw_y, SCROLL_INDICATOR, color, None, Attr::NONE, Some(clip));
    }
}

/// Render text content.
fn render_text(
    buffer: &mut FrameBuffer,
    index: usize,
    x: u16,
    y: u16,
    w: u16,
    h: u16,
    fg: Rgba,
    clip: &ClipRect,
) {
    let content = text::get_text_content(index);
    if content.is_empty() {
        return;
    }

    let attrs = text::get_text_attrs(index);
    let align = text::get_text_align(index);
    let wrap = text::get_text_wrap(index);

    // Handle text wrapping
    let lines: Vec<&str> = if wrap == TextWrap::Wrap {
        // Simple line splitting for now
        content.lines().collect()
    } else {
        vec![content.as_str()]
    };

    for (line_idx, line) in lines.iter().enumerate() {
        let line_y = y + line_idx as u16;
        if line_y >= y + h {
            break;
        }

        let text_to_draw = if wrap == TextWrap::Truncate && crate::layout::string_width(line) > w {
            crate::layout::truncate_text(line, w)
        } else {
            line.to_string()
        };

        let text_width = crate::layout::string_width(&text_to_draw);

        // Calculate x position based on alignment
        let draw_x = match align {
            TextAlign::Left => x,
            TextAlign::Center => x + (w.saturating_sub(text_width)) / 2,
            TextAlign::Right => x + w.saturating_sub(text_width),
        };

        buffer.draw_text(draw_x, line_y, &text_to_draw, fg, None, attrs, Some(clip));
    }
}

/// Render input field.
fn render_input(
    buffer: &mut FrameBuffer,
    index: usize,
    x: u16,
    y: u16,
    w: u16,
    _h: u16,
    fg: Rgba,
    clip: &ClipRect,
) {
    let content = text::get_text_content(index);
    let attrs = text::get_text_attrs(index);

    // For now, simple single-line input
    let display_text = if crate::layout::string_width(&content) > w {
        crate::layout::truncate_text(&content, w)
    } else {
        content
    };

    buffer.draw_text(x, y, &display_text, fg, None, attrs, Some(clip));

    // TODO: Render cursor if focused
}

/// Render progress bar.
fn render_progress(
    buffer: &mut FrameBuffer,
    index: usize,
    x: u16,
    y: u16,
    w: u16,
    h: u16,
    fg: Rgba,
    clip: &ClipRect,
) {
    // Get progress value (stored in text content as a number string for simplicity)
    let content = text::get_text_content(index);
    let progress: f32 = content.parse().unwrap_or(0.0);
    let progress = progress.clamp(0.0, 1.0);

    // Use mid-height for progress bar
    let bar_y = y + h / 2;

    buffer.draw_progress(x, bar_y, w, progress, '█', '░', fg, Rgba::GRAY, None, Some(clip));
}

/// Render select dropdown.
fn render_select(
    buffer: &mut FrameBuffer,
    index: usize,
    x: u16,
    y: u16,
    w: u16,
    _h: u16,
    fg: Rgba,
    clip: &ClipRect,
) {
    let content = text::get_text_content(index);
    let attrs = text::get_text_attrs(index);

    // Reserve space for dropdown indicator
    let indicator = " ▼";
    let indicator_width = 2;
    let text_width = w.saturating_sub(indicator_width);

    let display_text = if crate::layout::string_width(&content) > text_width {
        crate::layout::truncate_text(&content, text_width)
    } else {
        content.clone()
    };

    buffer.draw_text(x, y, &display_text, fg, None, attrs, Some(clip));
    buffer.draw_text(x + w - indicator_width, y, indicator, fg, None, Attr::NONE, Some(clip));
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{allocate_index, create_flex_node, reset_registry};
    use crate::engine::arrays::core as core_arrays;
    use crate::types::Dimension;
    use crate::pipeline::terminal::set_terminal_size;
    use crate::pipeline::layout_derived::create_layout_derived;

    fn setup() {
        reset_registry();
        set_terminal_size(80, 24);
    }

    #[test]
    fn test_frame_buffer_derived_empty() {
        setup();

        let layout_derived = create_layout_derived();
        let fb_derived = create_frame_buffer_derived(layout_derived);
        let result = fb_derived.get();

        assert_eq!(result.buffer.width(), 80);
        assert!(result.hit_regions.is_empty());
    }

    #[test]
    fn test_frame_buffer_derived_with_box() {
        setup();

        let idx = allocate_index(None);
        core_arrays::set_component_type(idx, ComponentType::Box);
        core_arrays::set_visible(idx, true);
        let node = create_flex_node(idx);
        node.width.set_value(Dimension::Cells(20));
        node.height.set_value(Dimension::Cells(5));
        visual::set_bg_color(idx, Rgba::BLUE);

        let layout_derived = create_layout_derived();
        let fb_derived = create_frame_buffer_derived(layout_derived);
        let result = fb_derived.get();

        // Should have one hit region
        assert_eq!(result.hit_regions.len(), 1);
        assert_eq!(result.hit_regions[0].width, 20);
        assert_eq!(result.hit_regions[0].height, 5);

        // Background should be filled
        let cell = result.buffer.get(5, 2).unwrap();
        assert_eq!(cell.bg, Rgba::BLUE);
    }

    #[test]
    fn test_frame_buffer_derived_with_text() {
        setup();

        let idx = allocate_index(None);
        core_arrays::set_component_type(idx, ComponentType::Text);
        core_arrays::set_visible(idx, true);
        let node = create_flex_node(idx);
        node.width.set_value(Dimension::Cells(20));
        node.height.set_value(Dimension::Cells(1));
        visual::set_fg_color(idx, Rgba::WHITE);
        text::set_text_content(idx, "Hello".to_string());

        let layout_derived = create_layout_derived();
        let fb_derived = create_frame_buffer_derived(layout_derived);
        let result = fb_derived.get();

        // Text should be rendered
        let cell = result.buffer.get(0, 0).unwrap();
        assert_eq!(cell.char, 'H' as u32);
        assert_eq!(cell.fg, Rgba::WHITE);
    }
}
