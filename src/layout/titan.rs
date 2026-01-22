//! TITAN Layout Engine
//!
//! Flexbox layout computation using parallel arrays pattern.
//!
//! # Algorithm
//!
//! 1. **Pass 1**: Build tree structure from parent relationships
//! 2. **Pass 2**: BFS traversal to get parents-before-children order
//! 3. **Pass 3**: Measure intrinsic sizes (bottom-up, leaf → root)
//! 4. **Pass 4**: Layout (top-down, root → leaf) with flex distribution
//! 5. **Pass 5**: Absolute positioning
//!
//! # Reactivity
//!
//! The layout reads from FlexNode Slots, which creates reactive dependencies
//! when called from within a derived. When any layout property changes,
//! the layout derived re-runs.

use std::cell::RefCell;

use crate::engine::arrays::core;
use crate::engine::arrays::text;
use crate::engine::{get_flex_node, get_allocated_indices};
use crate::types::{ComponentType, Dimension};

use super::text_measure::{measure_text_height, string_width};
use super::types::{ComputedLayout, Overflow};

// =============================================================================
// FLEX ENUMS (numeric values matching FlexNode Slot values)
// =============================================================================

const _FLEX_COLUMN: u8 = 0;  // Used as default
const FLEX_ROW: u8 = 1;
const FLEX_COLUMN_REVERSE: u8 = 2;
const FLEX_ROW_REVERSE: u8 = 3;

const WRAP_NOWRAP: u8 = 0;
const _WRAP_WRAP: u8 = 1;  // Used in matching
// const WRAP_REVERSE: u8 = 2;

const _JUSTIFY_START: u8 = 0;  // Used as default in match fallthrough
const JUSTIFY_CENTER: u8 = 1;
const JUSTIFY_END: u8 = 2;
const JUSTIFY_BETWEEN: u8 = 3;
const JUSTIFY_AROUND: u8 = 4;
const JUSTIFY_EVENLY: u8 = 5;

const ALIGN_STRETCH: u8 = 0;
// const ALIGN_START: u8 = 1;
const ALIGN_CENTER: u8 = 2;
const ALIGN_END: u8 = 3;

const POS_RELATIVE: u8 = 0;
const POS_ABSOLUTE: u8 = 1;

const ALIGN_SELF_AUTO: u8 = 0;

// =============================================================================
// WORKING ARRAYS (thread-local for reuse)
// =============================================================================

thread_local! {
    // Tree structure
    static FIRST_CHILD: RefCell<Vec<i32>> = RefCell::new(Vec::new());
    static NEXT_SIBLING: RefCell<Vec<i32>> = RefCell::new(Vec::new());
    static LAST_CHILD: RefCell<Vec<i32>> = RefCell::new(Vec::new());

    // Intrinsic sizes
    static INTRINSIC_W: RefCell<Vec<u16>> = RefCell::new(Vec::new());
    static INTRINSIC_H: RefCell<Vec<u16>> = RefCell::new(Vec::new());

    // Flex item sizes (after grow/shrink)
    static ITEM_MAIN: RefCell<Vec<u16>> = RefCell::new(Vec::new());
    static ITEM_CROSS: RefCell<Vec<u16>> = RefCell::new(Vec::new());
}

/// Reset all working arrays.
///
/// Call this after destroying all components to free memory.
pub fn reset_titan_arrays() {
    FIRST_CHILD.with(|v| v.borrow_mut().clear());
    NEXT_SIBLING.with(|v| v.borrow_mut().clear());
    LAST_CHILD.with(|v| v.borrow_mut().clear());
    INTRINSIC_W.with(|v| v.borrow_mut().clear());
    INTRINSIC_H.with(|v| v.borrow_mut().clear());
    ITEM_MAIN.with(|v| v.borrow_mut().clear());
    ITEM_CROSS.with(|v| v.borrow_mut().clear());
}

// =============================================================================
// DIMENSION RESOLUTION
// =============================================================================

/// Resolve a Dimension to an absolute value.
fn resolve_dimension(dim: Dimension, parent_size: u16) -> u16 {
    match dim {
        Dimension::Auto => 0,
        Dimension::Cells(n) => n,
        Dimension::Percent(p) => (parent_size as f32 * p / 100.0).floor() as u16,
    }
}

/// Apply min/max constraints.
fn clamp_dimension(value: u16, min_dim: Dimension, max_dim: Dimension, parent_size: u16) -> u16 {
    let min = resolve_dimension(min_dim, parent_size);
    let max = resolve_dimension(max_dim, parent_size);

    let mut result = value;
    if min > 0 && result < min {
        result = min;
    }
    if max > 0 && result > max {
        result = max;
    }
    result
}

// =============================================================================
// MAIN ENTRY POINT
// =============================================================================

/// Compute layout for all allocated components.
///
/// # Arguments
///
/// * `terminal_width` - Available width in terminal columns
/// * `terminal_height` - Available height in terminal rows
/// * `constrain_height` - If true, constrain to terminal height (fullscreen mode)
///
/// # Returns
///
/// Computed layout with positions and sizes for all components.
pub fn compute_layout(
    terminal_width: u16,
    terminal_height: u16,
    constrain_height: bool,
) -> ComputedLayout {
    let mut indices = get_allocated_indices();

    if indices.is_empty() {
        return ComputedLayout::new();
    }

    // Sort indices for consistent child ordering
    indices.sort_unstable();

    // Find max index for array sizing
    let max_index = indices.iter().max().copied().unwrap_or(0);
    let array_size = max_index + 1;

    // Initialize output
    let mut out_x: Vec<u16> = vec![0; array_size];
    let mut out_y: Vec<u16> = vec![0; array_size];
    let mut out_w: Vec<u16> = vec![0; array_size];
    let mut out_h: Vec<u16> = vec![0; array_size];
    let mut out_scrollable: Vec<u8> = vec![0; array_size];
    let mut out_max_scroll_x: Vec<u16> = vec![0; array_size];
    let mut out_max_scroll_y: Vec<u16> = vec![0; array_size];

    // Initialize working arrays
    FIRST_CHILD.with(|v| {
        let mut v = v.borrow_mut();
        v.clear();
        v.resize(array_size, -1);
    });
    NEXT_SIBLING.with(|v| {
        let mut v = v.borrow_mut();
        v.clear();
        v.resize(array_size, -1);
    });
    LAST_CHILD.with(|v| {
        let mut v = v.borrow_mut();
        v.clear();
        v.resize(array_size, -1);
    });
    INTRINSIC_W.with(|v| {
        let mut v = v.borrow_mut();
        v.clear();
        v.resize(array_size, 0);
    });
    INTRINSIC_H.with(|v| {
        let mut v = v.borrow_mut();
        v.clear();
        v.resize(array_size, 0);
    });
    ITEM_MAIN.with(|v| {
        let mut v = v.borrow_mut();
        v.clear();
        v.resize(array_size, 0);
    });
    ITEM_CROSS.with(|v| {
        let mut v = v.borrow_mut();
        v.clear();
        v.resize(array_size, 0);
    });

    // =========================================================================
    // PASS 1: Build tree structure
    // =========================================================================

    let mut bfs_queue: Vec<usize> = Vec::new();
    let mut root_count = 0;

    for &i in &indices {
        // Skip invisible components
        if !core::get_visible(i) {
            continue;
        }

        let parent = core::get_parent_index(i);

        if let Some(parent_idx) = parent {
            if indices.contains(&parent_idx) {
                // Has a valid parent - add to parent's child list
                FIRST_CHILD.with(|first| {
                    LAST_CHILD.with(|last| {
                        NEXT_SIBLING.with(|next| {
                            let mut first = first.borrow_mut();
                            let mut last = last.borrow_mut();
                            let mut next = next.borrow_mut();

                            if first[parent_idx] == -1 {
                                first[parent_idx] = i as i32;
                            } else if last[parent_idx] >= 0 {
                                next[last[parent_idx] as usize] = i as i32;
                            }
                            last[parent_idx] = i as i32;
                        });
                    });
                });
            } else {
                // Parent not in indices - treat as root
                bfs_queue.push(i);
                root_count += 1;
            }
        } else {
            // No parent - this is a root
            bfs_queue.push(i);
            root_count += 1;
        }
    }

    // =========================================================================
    // PASS 2: BFS traversal to get parents-before-children order
    // =========================================================================

    let mut head = 0;
    while head < bfs_queue.len() {
        let parent = bfs_queue[head];
        head += 1;

        FIRST_CHILD.with(|first| {
            NEXT_SIBLING.with(|next| {
                let first = first.borrow();
                let next = next.borrow();

                let mut child = first[parent];
                while child >= 0 {
                    bfs_queue.push(child as usize);
                    child = next[child as usize];
                }
            });
        });
    }

    // =========================================================================
    // PASS 3: Measure intrinsic sizes (bottom-up)
    // =========================================================================

    for &i in bfs_queue.iter().rev() {
        let comp_type = core::get_component_type(i);

        if comp_type == ComponentType::Text {
            // TEXT: intrinsic size from content
            let content = text::get_text_content(i);
            if !content.is_empty() {
                INTRINSIC_W.with(|w| {
                    INTRINSIC_H.with(|h| {
                        let mut w = w.borrow_mut();
                        let mut h = h.borrow_mut();

                        // Get parent width for wrapping calculation
                        let parent = core::get_parent_index(i);
                        let avail_w = if let Some(p) = parent {
                            if let Some(node) = get_flex_node(p) {
                                let parent_w = resolve_dimension(node.width.get(), terminal_width);
                                let pad_l = node.padding_left.get();
                                let pad_r = node.padding_right.get();
                                let bord_l = if node.border_left.get() > 0 { 1 } else { 0 };
                                let bord_r = if node.border_right.get() > 0 { 1 } else { 0 };
                                if parent_w > 0 {
                                    parent_w.saturating_sub(pad_l + pad_r + bord_l + bord_r)
                                } else {
                                    terminal_width
                                }
                            } else {
                                terminal_width
                            }
                        } else {
                            terminal_width
                        };

                        w[i] = string_width(&content);
                        h[i] = measure_text_height(&content, avail_w.max(1));
                    });
                });
            }
        } else if comp_type == ComponentType::Input {
            // INPUT: single line, intrinsic width from content
            let content = text::get_text_content(i);
            if let Some(node) = get_flex_node(i) {
                let pad_l = node.padding_left.get();
                let pad_r = node.padding_right.get();
                let pad_t = node.padding_top.get();
                let pad_b = node.padding_bottom.get();
                let bord_l = if node.border_left.get() > 0 { 1 } else { 0 };
                let bord_r = if node.border_right.get() > 0 { 1 } else { 0 };
                let bord_t = if node.border_top.get() > 0 { 1 } else { 0 };
                let bord_b = if node.border_bottom.get() > 0 { 1 } else { 0 };

                INTRINSIC_W.with(|w| {
                    INTRINSIC_H.with(|h| {
                        let mut w = w.borrow_mut();
                        let mut h = h.borrow_mut();
                        w[i] = string_width(&content) + pad_l + pad_r + bord_l + bord_r;
                        h[i] = 1 + pad_t + pad_b + bord_t + bord_b;
                    });
                });
            }
        } else if comp_type == ComponentType::Box {
            // BOX: intrinsic size from children
            if let Some(node) = get_flex_node(i) {
                let overflow = Overflow::from(node.overflow.get());
                let is_scrollable = matches!(overflow, Overflow::Scroll | Overflow::Auto);

                FIRST_CHILD.with(|first| {
                    NEXT_SIBLING.with(|next| {
                        INTRINSIC_W.with(|intr_w| {
                            INTRINSIC_H.with(|intr_h| {
                                let first = first.borrow();
                                let next = next.borrow();
                                let mut intr_w = intr_w.borrow_mut();
                                let mut intr_h = intr_h.borrow_mut();

                                let mut kid = first[i];
                                if kid != -1 && !is_scrollable {
                                    let dir = node.flex_direction.get();
                                    let is_row = dir == FLEX_ROW || dir == FLEX_ROW_REVERSE;
                                    let gap = node.gap.get();

                                    let mut sum_main: u16 = 0;
                                    let mut max_cross: u16 = 0;
                                    let mut child_count: u16 = 0;

                                    while kid >= 0 {
                                        let k = kid as usize;
                                        child_count += 1;

                                        // Child size from explicit or intrinsic
                                        let kid_w;
                                        let kid_h;
                                        if let Some(kid_node) = get_flex_node(k) {
                                            let explicit_w = resolve_dimension(kid_node.width.get(), terminal_width);
                                            let explicit_h = resolve_dimension(kid_node.height.get(), terminal_height);
                                            kid_w = if explicit_w > 0 { explicit_w } else { intr_w[k] };
                                            kid_h = if explicit_h > 0 { explicit_h } else { intr_h[k] };
                                        } else {
                                            kid_w = intr_w[k];
                                            kid_h = intr_h[k];
                                        }

                                        // Include margins
                                        let margin_main = if let Some(kid_node) = get_flex_node(k) {
                                            if is_row {
                                                kid_node.margin_left.get() + kid_node.margin_right.get()
                                            } else {
                                                kid_node.margin_top.get() + kid_node.margin_bottom.get()
                                            }
                                        } else {
                                            0
                                        };

                                        if is_row {
                                            sum_main = sum_main.saturating_add(kid_w + margin_main + gap);
                                            max_cross = max_cross.max(kid_h);
                                        } else {
                                            sum_main = sum_main.saturating_add(kid_h + margin_main + gap);
                                            max_cross = max_cross.max(kid_w);
                                        }

                                        kid = next[k];
                                    }

                                    if child_count > 0 {
                                        sum_main = sum_main.saturating_sub(gap);
                                    }

                                    // Add padding and borders
                                    let pad_t = node.padding_top.get();
                                    let pad_r = node.padding_right.get();
                                    let pad_b = node.padding_bottom.get();
                                    let pad_l = node.padding_left.get();
                                    let bord_t = if node.border_top.get() > 0 { 1 } else { 0 };
                                    let bord_r = if node.border_right.get() > 0 { 1 } else { 0 };
                                    let bord_b = if node.border_bottom.get() > 0 { 1 } else { 0 };
                                    let bord_l = if node.border_left.get() > 0 { 1 } else { 0 };

                                    let extra_w = pad_l + pad_r + bord_l + bord_r;
                                    let extra_h = pad_t + pad_b + bord_t + bord_b;

                                    if is_row {
                                        intr_w[i] = sum_main + extra_w;
                                        intr_h[i] = max_cross + extra_h;
                                    } else {
                                        intr_w[i] = max_cross + extra_w;
                                        intr_h[i] = sum_main + extra_h;
                                    }
                                } else if is_scrollable {
                                    // Scrollable: minimal intrinsic
                                    let pad_t = node.padding_top.get();
                                    let pad_r = node.padding_right.get();
                                    let pad_b = node.padding_bottom.get();
                                    let pad_l = node.padding_left.get();
                                    let bord_t = if node.border_top.get() > 0 { 1 } else { 0 };
                                    let bord_r = if node.border_right.get() > 0 { 1 } else { 0 };
                                    let bord_b = if node.border_bottom.get() > 0 { 1 } else { 0 };
                                    let bord_l = if node.border_left.get() > 0 { 1 } else { 0 };

                                    intr_w[i] = pad_l + pad_r + bord_l + bord_r;
                                    intr_h[i] = pad_t + pad_b + bord_t + bord_b;
                                }
                            });
                        });
                    });
                });
            }
        }
    }

    // =========================================================================
    // PASS 4: Layout (top-down)
    // =========================================================================

    // First, position all roots
    for &root in bfs_queue.iter().take(root_count) {
        if let Some(node) = get_flex_node(root) {
            let ew = resolve_dimension(node.width.get(), terminal_width);
            let eh = resolve_dimension(node.height.get(), terminal_height);

            out_x[root] = 0;
            out_y[root] = 0;
            out_w[root] = if ew > 0 { ew } else { terminal_width };

            if eh > 0 {
                out_h[root] = eh;
            } else if constrain_height {
                out_h[root] = terminal_height;
            } else {
                out_h[root] = INTRINSIC_H.with(|h| h.borrow()[root]).max(1);
            }
        }
    }

    // Layout children of each node in BFS order
    for &parent in &bfs_queue {
        layout_children(
            parent,
            terminal_width,
            terminal_height,
            constrain_height,
            &mut out_x,
            &mut out_y,
            &mut out_w,
            &mut out_h,
            &mut out_scrollable,
            &mut out_max_scroll_x,
            &mut out_max_scroll_y,
        );
    }

    // =========================================================================
    // PASS 5: Absolute positioning
    // =========================================================================

    for &i in &indices {
        if let Some(node) = get_flex_node(i) {
            if node.position.get() == POS_ABSOLUTE {
                layout_absolute(
                    i,
                    &indices,
                    terminal_width,
                    terminal_height,
                    &mut out_x,
                    &mut out_y,
                    &mut out_w,
                    &mut out_h,
                );
            }
        }
    }

    // =========================================================================
    // Compute content bounds
    // =========================================================================

    let content_width = out_w.get(bfs_queue.first().copied().unwrap_or(0)).copied().unwrap_or(0);
    let content_height = out_h.get(bfs_queue.first().copied().unwrap_or(0)).copied().unwrap_or(0);

    ComputedLayout {
        x: out_x,
        y: out_y,
        width: out_w,
        height: out_h,
        scrollable: out_scrollable,
        max_scroll_x: out_max_scroll_x,
        max_scroll_y: out_max_scroll_y,
        content_width,
        content_height,
    }
}

/// Layout children of a single parent.
#[allow(clippy::too_many_arguments)]
fn layout_children(
    parent: usize,
    _terminal_width: u16,
    _terminal_height: u16,
    constrain_height: bool,
    out_x: &mut [u16],
    out_y: &mut [u16],
    out_w: &mut [u16],
    out_h: &mut [u16],
    out_scrollable: &mut [u8],
    out_max_scroll_x: &mut [u16],
    out_max_scroll_y: &mut [u16],
) {
    let Some(parent_node) = get_flex_node(parent) else {
        return;
    };

    // Collect flow children (non-absolute)
    let mut flow_kids: Vec<usize> = Vec::new();
    FIRST_CHILD.with(|first| {
        NEXT_SIBLING.with(|next| {
            let first = first.borrow();
            let next = next.borrow();

            let mut kid = first[parent];
            while kid >= 0 {
                let k = kid as usize;
                if let Some(kid_node) = get_flex_node(k) {
                    if kid_node.position.get() != POS_ABSOLUTE {
                        flow_kids.push(k);
                    }
                } else {
                    flow_kids.push(k);
                }
                kid = next[k];
            }
        });
    });

    if flow_kids.is_empty() {
        return;
    }

    // Parent's content area
    let p_pad_t = parent_node.padding_top.get();
    let p_pad_r = parent_node.padding_right.get();
    let p_pad_b = parent_node.padding_bottom.get();
    let p_pad_l = parent_node.padding_left.get();
    let p_bord_t = if parent_node.border_top.get() > 0 { 1 } else { 0 };
    let p_bord_r = if parent_node.border_right.get() > 0 { 1 } else { 0 };
    let p_bord_b = if parent_node.border_bottom.get() > 0 { 1 } else { 0 };
    let p_bord_l = if parent_node.border_left.get() > 0 { 1 } else { 0 };

    let content_x = out_x[parent] + p_pad_l + p_bord_l;
    let content_y = out_y[parent] + p_pad_t + p_bord_t;
    let content_w = out_w[parent].saturating_sub(p_pad_l + p_pad_r + p_bord_l + p_bord_r);
    let content_h = out_h[parent].saturating_sub(p_pad_t + p_pad_b + p_bord_t + p_bord_b);

    // Flex properties
    let dir = parent_node.flex_direction.get();
    let wrap = parent_node.flex_wrap.get();
    let justify = parent_node.justify_content.get();
    let align_items = parent_node.align_items.get();
    let gap = parent_node.gap.get();
    let overflow = Overflow::from(parent_node.overflow.get());

    let is_row = dir == FLEX_ROW || dir == FLEX_ROW_REVERSE;
    let is_reverse = dir == FLEX_ROW_REVERSE || dir == FLEX_COLUMN_REVERSE;
    let is_root = core::get_parent_index(parent).is_none();
    let is_scrollable = matches!(overflow, Overflow::Scroll | Overflow::Auto) || (is_root && constrain_height);

    let main_size = if is_row { content_w } else { content_h };
    let cross_size = if is_row { content_h } else { content_w };

    // =========================================================================
    // Step 1: Collect items into flex lines
    // =========================================================================

    let mut line_starts: Vec<usize> = Vec::new();
    let mut line_ends: Vec<usize> = Vec::new();
    let mut line_main_used: Vec<u16> = Vec::new();

    let mut line_start = 0;
    let mut current_main: u16 = 0;

    for (fi, &kid) in flow_kids.iter().enumerate() {
        let kid_main = INTRINSIC_W.with(|w| {
            INTRINSIC_H.with(|h| {
                let w = w.borrow();
                let h = h.borrow();

                if let Some(kid_node) = get_flex_node(kid) {
                    let ew = resolve_dimension(kid_node.width.get(), content_w);
                    let eh = resolve_dimension(kid_node.height.get(), content_h);

                    if is_row {
                        if ew > 0 { ew } else { w[kid] }
                    } else {
                        if eh > 0 { eh } else { h[kid] }
                    }
                } else {
                    if is_row { w[kid] } else { h[kid] }
                }
            })
        });

        // Check for wrap
        if wrap != WRAP_NOWRAP && fi > line_start && current_main + kid_main + gap > main_size {
            line_starts.push(line_start);
            line_ends.push(fi - 1);
            line_main_used.push(current_main.saturating_sub(gap));
            line_start = fi;
            current_main = 0;
        }

        current_main = current_main.saturating_add(kid_main + gap);
    }

    // Final line
    if !flow_kids.is_empty() {
        line_starts.push(line_start);
        line_ends.push(flow_kids.len() - 1);
        line_main_used.push(current_main.saturating_sub(gap));
    }

    let line_count = line_starts.len();

    // =========================================================================
    // Step 2: Resolve flex grow/shrink per line
    // =========================================================================

    for li in 0..line_count {
        let l_start = line_starts[li];
        let l_end = line_ends[li];
        let free_space = main_size as i32 - line_main_used[li] as i32;

        let mut total_grow: f32 = 0.0;
        let mut total_shrink: f32 = 0.0;

        for fi in l_start..=l_end {
            let kid = flow_kids[fi];
            if let Some(kid_node) = get_flex_node(kid) {
                total_grow += kid_node.flex_grow.get();
                total_shrink += kid_node.flex_shrink.get();
            } else {
                total_shrink += 1.0; // default shrink
            }
        }

        for fi in l_start..=l_end {
            let kid = flow_kids[fi];

            let (kid_main, kid_cross) = INTRINSIC_W.with(|w| {
                INTRINSIC_H.with(|h| {
                    let w = w.borrow();
                    let h = h.borrow();

                    if let Some(kid_node) = get_flex_node(kid) {
                        let ew = resolve_dimension(kid_node.width.get(), content_w);
                        let eh = resolve_dimension(kid_node.height.get(), content_h);
                        let basis = resolve_dimension(kid_node.flex_basis.get(), if is_row { content_w } else { content_h });

                        let mut km = if basis > 0 {
                            basis
                        } else if is_row {
                            if ew > 0 { ew } else { w[kid] }
                        } else {
                            if eh > 0 { eh } else { h[kid] }
                        };

                        // Apply grow/shrink
                        if free_space > 0 && total_grow > 0.0 {
                            let grow = kid_node.flex_grow.get();
                            km = km.saturating_add(((grow / total_grow) * free_space as f32).floor() as u16);
                        } else if free_space < 0 && total_shrink > 0.0 && !is_scrollable {
                            let shrink = kid_node.flex_shrink.get();
                            let shrink_amount = ((shrink / total_shrink) * (-free_space) as f32).floor() as u16;
                            km = km.saturating_sub(shrink_amount);
                        }

                        // Apply min/max
                        km = clamp_dimension(
                            km,
                            if is_row { kid_node.min_width.get() } else { kid_node.min_height.get() },
                            if is_row { kid_node.max_width.get() } else { kid_node.max_height.get() },
                            if is_row { content_w } else { content_h },
                        );

                        let mut kc = if is_row {
                            if eh > 0 {
                                eh
                            } else if align_items == ALIGN_STRETCH {
                                cross_size / line_count.max(1) as u16
                            } else {
                                h[kid]
                            }
                        } else {
                            if ew > 0 {
                                ew
                            } else if align_items == ALIGN_STRETCH {
                                cross_size / line_count.max(1) as u16
                            } else {
                                w[kid]
                            }
                        };

                        // Apply cross min/max
                        kc = clamp_dimension(
                            kc,
                            if is_row { kid_node.min_height.get() } else { kid_node.min_width.get() },
                            if is_row { kid_node.max_height.get() } else { kid_node.max_width.get() },
                            if is_row { content_h } else { content_w },
                        );

                        (km, kc)
                    } else {
                        if is_row { (w[kid], h[kid]) } else { (h[kid], w[kid]) }
                    }
                })
            });

            ITEM_MAIN.with(|m| m.borrow_mut()[kid] = kid_main);
            ITEM_CROSS.with(|c| c.borrow_mut()[kid] = kid_cross);
        }
    }

    // =========================================================================
    // Step 3: Position items
    // =========================================================================

    let mut cross_offset: u16 = 0;
    let line_height = cross_size / line_count.max(1) as u16;

    let mut children_max_main: u16 = 0;
    let mut children_max_cross: u16 = 0;

    for li in 0..line_count {
        let line_idx = if is_reverse { line_count - 1 - li } else { li };
        let l_start = line_starts[line_idx];
        let l_end = line_ends[line_idx];

        // Calculate line main size
        let mut line_main: u16 = 0;
        for fi in l_start..=l_end {
            let kid = flow_kids[fi];
            let km = ITEM_MAIN.with(|m| m.borrow()[kid]);

            let m_main = if let Some(kid_node) = get_flex_node(kid) {
                if is_row {
                    kid_node.margin_left.get() + kid_node.margin_right.get()
                } else {
                    kid_node.margin_top.get() + kid_node.margin_bottom.get()
                }
            } else {
                0
            };

            line_main = line_main.saturating_add(km + m_main + gap);
        }
        line_main = line_main.saturating_sub(gap);

        let remaining_space = main_size.saturating_sub(line_main);
        let item_count = (l_end - l_start + 1) as u16;

        let (mut main_offset, item_gap) = match justify {
            JUSTIFY_CENTER => (remaining_space / 2, gap),
            JUSTIFY_END => (remaining_space, gap),
            JUSTIFY_BETWEEN => {
                if item_count > 1 {
                    (0, remaining_space / (item_count - 1) + gap)
                } else {
                    (0, gap)
                }
            }
            JUSTIFY_AROUND => {
                let around = remaining_space / item_count;
                (around / 2, around + gap)
            }
            JUSTIFY_EVENLY => {
                let even = remaining_space / (item_count + 1);
                (even, even + gap)
            }
            _ => (0, gap), // JUSTIFY_START
        };

        for fi in l_start..=l_end {
            let kid = flow_kids[fi];
            let size_main = ITEM_MAIN.with(|m| m.borrow()[kid]);
            let size_cross = ITEM_CROSS.with(|c| c.borrow()[kid]);

            let (m_top, m_right, m_bottom, m_left) = if let Some(kid_node) = get_flex_node(kid) {
                (
                    kid_node.margin_top.get(),
                    kid_node.margin_right.get(),
                    kid_node.margin_bottom.get(),
                    kid_node.margin_left.get(),
                )
            } else {
                (0, 0, 0, 0)
            };

            // Align-self
            let self_align = if let Some(kid_node) = get_flex_node(kid) {
                let a = kid_node.align_self.get();
                if a != ALIGN_SELF_AUTO { a - 1 } else { align_items }
            } else {
                align_items
            };

            let mut cross_pos = cross_offset;
            match self_align {
                ALIGN_CENTER => cross_pos += (line_height.saturating_sub(size_cross)) / 2,
                ALIGN_END => cross_pos += line_height.saturating_sub(size_cross),
                _ => {} // STRETCH or START
            }

            // Set position
            if is_row {
                if dir == FLEX_ROW_REVERSE {
                    out_x[kid] = content_x + content_w.saturating_sub(main_offset + size_main + m_right);
                } else {
                    out_x[kid] = content_x + main_offset + m_left;
                }
                out_y[kid] = content_y + cross_pos + m_top;
                out_w[kid] = size_main;
                out_h[kid] = size_cross;
            } else {
                out_x[kid] = content_x + cross_pos + m_left;
                if dir == FLEX_COLUMN_REVERSE {
                    out_y[kid] = content_y + content_h.saturating_sub(main_offset + size_main + m_bottom);
                } else {
                    out_y[kid] = content_y + main_offset + m_top;
                }
                out_w[kid] = size_cross;
                out_h[kid] = size_main;
            }

            // Update text height if needed
            if core::get_component_type(kid) == ComponentType::Text {
                let content = text::get_text_content(kid);
                if !content.is_empty() {
                    let wrapped_h = measure_text_height(&content, out_w[kid].max(1));
                    out_h[kid] = wrapped_h.max(1);
                }
            }

            // Track max extent
            if is_row {
                children_max_main = children_max_main.max(main_offset + m_left + out_w[kid] + m_right);
                children_max_cross = children_max_cross.max(cross_pos + m_top + out_h[kid] + m_bottom);
            } else {
                children_max_main = children_max_main.max(main_offset + m_top + out_h[kid] + m_bottom);
                children_max_cross = children_max_cross.max(cross_pos + m_left + out_w[kid] + m_right);
            }

            // Advance
            let main_margin = if is_row { m_left + m_right } else { m_top + m_bottom };
            main_offset = main_offset.saturating_add(if is_row { out_w[kid] } else { out_h[kid] } + main_margin + item_gap);
        }

        cross_offset = cross_offset.saturating_add(line_height);
    }

    // Scroll detection
    if is_scrollable {
        let (children_max_x, children_max_y) = if is_row {
            (children_max_main, children_max_cross)
        } else {
            (children_max_cross, children_max_main)
        };

        let scroll_range_x = children_max_x.saturating_sub(content_w);
        let scroll_range_y = children_max_y.saturating_sub(content_h);

        if matches!(overflow, Overflow::Scroll) || scroll_range_x > 0 || scroll_range_y > 0 {
            out_scrollable[parent] = 1;
            out_max_scroll_x[parent] = scroll_range_x;
            out_max_scroll_y[parent] = scroll_range_y;
        }
    }
}

/// Layout an absolutely positioned element.
#[allow(clippy::too_many_arguments)]
fn layout_absolute(
    i: usize,
    indices: &[usize],
    terminal_width: u16,
    terminal_height: u16,
    out_x: &mut [u16],
    out_y: &mut [u16],
    out_w: &mut [u16],
    out_h: &mut [u16],
) {
    // Find containing block (nearest positioned ancestor)
    let mut container = core::get_parent_index(i);
    while let Some(c) = container {
        if indices.contains(&c) {
            if let Some(node) = get_flex_node(c) {
                if node.position.get() != POS_RELATIVE {
                    break;
                }
            }
        }
        container = core::get_parent_index(c);
    }

    let (container_x, container_y, container_w, container_h) = if let Some(c) = container {
        (out_x[c], out_y[c], out_w[c], out_h[c])
    } else {
        (0, 0, out_w.first().copied().unwrap_or(terminal_width), out_h.first().copied().unwrap_or(terminal_height))
    };

    let Some(node) = get_flex_node(i) else { return };

    // Resolve dimensions
    let ew = resolve_dimension(node.width.get(), container_w);
    let eh = resolve_dimension(node.height.get(), container_h);
    let abs_w = clamp_dimension(
        INTRINSIC_W.with(|w| if ew > 0 { ew } else { w.borrow()[i] }),
        node.min_width.get(),
        node.max_width.get(),
        container_w,
    );
    let abs_h = clamp_dimension(
        INTRINSIC_H.with(|h| if eh > 0 { eh } else { h.borrow()[i] }),
        node.min_height.get(),
        node.max_height.get(),
        container_h,
    );

    out_w[i] = abs_w;
    out_h[i] = abs_h;

    // Position based on top/right/bottom/left
    // Note: We'd need to add these to FlexNode, using position offset for now
    out_x[i] = container_x;
    out_y[i] = container_y;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{allocate_index, reset_registry, create_flex_node};
    use crate::engine::arrays::core as core_arrays;

    fn setup() {
        reset_registry();
        reset_titan_arrays();
    }

    #[test]
    fn test_compute_layout_empty() {
        setup();

        let layout = compute_layout(80, 24, true);
        assert_eq!(layout.content_width, 0);
        assert_eq!(layout.content_height, 0);
    }

    #[test]
    fn test_compute_layout_single_root() {
        setup();

        let idx = allocate_index(None);
        core_arrays::set_component_type(idx, ComponentType::Box);
        core_arrays::set_visible(idx, true);
        let node = create_flex_node(idx);
        node.width.set_value(Dimension::Cells(40));
        node.height.set_value(Dimension::Cells(10));

        let layout = compute_layout(80, 24, true);

        assert_eq!(layout.x[idx], 0);
        assert_eq!(layout.y[idx], 0);
        assert_eq!(layout.width[idx], 40);
        assert_eq!(layout.height[idx], 10);
    }

    #[test]
    fn test_compute_layout_parent_child() {
        setup();

        // Parent box
        let parent = allocate_index(None);
        core_arrays::set_component_type(parent, ComponentType::Box);
        core_arrays::set_visible(parent, true);
        let parent_node = create_flex_node(parent);
        parent_node.width.set_value(Dimension::Cells(40));
        parent_node.height.set_value(Dimension::Cells(10));

        // Child box
        let child = allocate_index(None);
        core_arrays::set_component_type(child, ComponentType::Box);
        core_arrays::set_visible(child, true);
        core_arrays::set_parent_index(child, Some(parent));
        let child_node = create_flex_node(child);
        child_node.width.set_value(Dimension::Cells(20));
        child_node.height.set_value(Dimension::Cells(5));

        let layout = compute_layout(80, 24, true);

        // Parent at origin
        assert_eq!(layout.x[parent], 0);
        assert_eq!(layout.y[parent], 0);

        // Child inside parent
        assert_eq!(layout.x[child], 0);
        assert_eq!(layout.y[child], 0);
        assert_eq!(layout.width[child], 20);
        assert_eq!(layout.height[child], 5);
    }

    #[test]
    fn test_flex_row() {
        setup();

        // Parent with row direction
        let parent = allocate_index(None);
        core_arrays::set_component_type(parent, ComponentType::Box);
        core_arrays::set_visible(parent, true);
        let parent_node = create_flex_node(parent);
        parent_node.width.set_value(Dimension::Cells(40));
        parent_node.height.set_value(Dimension::Cells(10));
        parent_node.flex_direction.set_value(FLEX_ROW);

        // Two children
        let child1 = allocate_index(None);
        core_arrays::set_component_type(child1, ComponentType::Box);
        core_arrays::set_visible(child1, true);
        core_arrays::set_parent_index(child1, Some(parent));
        let child1_node = create_flex_node(child1);
        child1_node.width.set_value(Dimension::Cells(10));
        child1_node.height.set_value(Dimension::Cells(5));

        let child2 = allocate_index(None);
        core_arrays::set_component_type(child2, ComponentType::Box);
        core_arrays::set_visible(child2, true);
        core_arrays::set_parent_index(child2, Some(parent));
        let child2_node = create_flex_node(child2);
        child2_node.width.set_value(Dimension::Cells(10));
        child2_node.height.set_value(Dimension::Cells(5));

        let layout = compute_layout(80, 24, true);

        // Children should be side by side
        assert_eq!(layout.x[child1], 0);
        assert_eq!(layout.x[child2], 10);  // After first child
    }

    #[test]
    fn test_resolve_dimension() {
        assert_eq!(resolve_dimension(Dimension::Auto, 100), 0);
        assert_eq!(resolve_dimension(Dimension::Cells(50), 100), 50);
        assert_eq!(resolve_dimension(Dimension::Percent(50.0), 100), 50);
        assert_eq!(resolve_dimension(Dimension::Percent(100.0), 80), 80);
    }
}
