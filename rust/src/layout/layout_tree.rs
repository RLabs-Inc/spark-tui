//! Low-level Taffy integration via trait implementation on SharedBuffer.
//!
//! Instead of using TaffyTree (which owns a SlotMap + HashMap<usize, NodeId>),
//! this module implements Taffy's layout traits directly on a `LayoutTree`
//! wrapper around SharedBuffer. NodeId IS the component index — zero translation.
//!
//! # What this eliminates
//!
//! - TaffyTree's internal SlotMap (duplicate node storage)
//! - HashMap<usize, NodeId> index translation
//! - build_taffy_tree / reuse_cached_tree code paths
//! - ~300 lines of bridge boilerplate
//!
//! # What this enables
//!
//! - Zero-copy layout: Taffy reads directly from SharedBuffer arrays
//! - Zero-translation: NodeId IS the component index
//! - Proper inset support for absolute positioning

use taffy::prelude::*;
use taffy::{
    compute_cached_layout, compute_flexbox_layout, compute_hidden_layout, compute_leaf_layout,
    compute_root_layout, round_layout, Cache, CacheTree, Layout, LayoutFlexboxContainer,
    LayoutInput, LayoutOutput, LayoutPartialTree, NodeId, Overflow as TaffyOverflow, RoundTree,
    RunMode, TraversePartialTree, TraverseTree,
};

use crate::shared_buffer::SharedBuffer;

use super::text_measure::{measure_text_height, string_width};

// =============================================================================
// CHILD ITERATOR
// =============================================================================

/// Iterator that yields `NodeId` from a slice of `usize` children indices.
pub struct ChildIter<'a> {
    inner: std::slice::Iter<'a, usize>,
}

impl<'a> Iterator for ChildIter<'a> {
    type Item = NodeId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|&idx| NodeId::from(idx))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl ExactSizeIterator for ChildIter<'_> {}

// =============================================================================
// LAYOUT TREE
// =============================================================================

/// A layout tree that wraps SharedBuffer and provides Taffy's trait API.
///
/// NodeId convention: `NodeId::from(component_index)`, so
/// `usize::from(node_id) == component_index`. Zero translation.
pub struct LayoutTree<'a> {
    buf: &'a SharedBuffer,
    /// Parent → children map, built from hierarchy section.
    children: Vec<Vec<usize>>,
    /// Per-node Taffy layout cache.
    cache: Vec<Cache>,
    /// Intermediate unrounded layouts (written by Taffy during computation).
    unrounded: Vec<Layout>,
    /// Final pixel-snapped layouts (written by round_layout).
    final_layout: Vec<Layout>,
    /// Root node indices (nodes with parent_index == -1 and allocated).
    pub roots: Vec<usize>,
    /// Number of active nodes.
    node_count: usize,
}

impl<'a> LayoutTree<'a> {
    /// Build a LayoutTree from a SharedBuffer.
    ///
    /// 1. Reads node_count from header
    /// 2. Builds children map from hierarchy section (O(N))
    /// 3. Allocates cache, unrounded, final_layout arrays
    /// 4. Identifies root nodes
    pub fn new(buf: &'a SharedBuffer) -> Self {
        let node_count = buf.node_count();

        let mut children: Vec<Vec<usize>> = vec![Vec::new(); node_count];
        let mut roots: Vec<usize> = Vec::new();

        // Build children map from hierarchy (parent indices).
        // A node is considered active if component_type != 0 and visible.
        for i in 0..node_count {
            let comp_type = buf.component_type(i);
            if comp_type == 0 || !buf.visible(i) {
                continue;
            }

            match buf.parent_index(i) {
                Some(parent_idx) if parent_idx < node_count => {
                    // Only add as child if parent is also allocated and visible
                    let parent_type = buf.component_type(parent_idx);
                    if parent_type != 0 && buf.visible(parent_idx) {
                        children[parent_idx].push(i);
                    } else {
                        roots.push(i);
                    }
                }
                _ => {
                    // No parent or invalid parent — this is a root
                    roots.push(i);
                }
            }
        }

        let cache = vec![Cache::new(); node_count];
        let unrounded = vec![Layout::with_order(0); node_count];
        let final_layout = vec![Layout::with_order(0); node_count];

        Self {
            buf,
            children,
            cache,
            unrounded,
            final_layout,
            roots,
            node_count,
        }
    }

    /// Build a Taffy `Style` from SharedBuffer data for the given node index.
    fn build_style(&self, idx: usize) -> Style {
        let buf = self.buf;
        let comp_type = buf.component_type(idx);

        // Determine gap: use row_gap/column_gap if set, fall back to gap
        let gap_val = buf.gap(idx);
        let row_gap_val = buf.row_gap(idx);
        let col_gap_val = buf.column_gap(idx);
        let effective_row_gap = if row_gap_val != 0.0 { row_gap_val } else { gap_val };
        let effective_col_gap = if col_gap_val != 0.0 { col_gap_val } else { gap_val };

        let mut style = Style {
            display: Display::Flex,
            position: to_position(buf.position(idx)),

            flex_direction: to_flex_direction(buf.flex_direction(idx)),
            flex_wrap: to_flex_wrap(buf.flex_wrap(idx)),
            justify_content: to_justify(buf.justify_content(idx)),
            align_items: to_align_items(buf.align_items(idx)),
            align_content: to_align_content(buf.align_content(idx)),

            flex_grow: buf.grow(idx),
            flex_shrink: buf.shrink(idx),
            flex_basis: to_taffy_dim(buf.basis(idx)),
            align_self: to_align_self(buf.align_self(idx)),

            size: Size {
                width: to_taffy_dim(buf.width(idx)),
                height: to_taffy_dim(buf.height(idx)),
            },
            min_size: Size {
                width: to_taffy_dim(buf.min_width(idx)),
                height: to_taffy_dim(buf.min_height(idx)),
            },
            max_size: Size {
                width: to_taffy_dim(buf.max_width(idx)),
                height: to_taffy_dim(buf.max_height(idx)),
            },

            margin: Rect {
                top: LengthPercentageAuto::Length(buf.margin_top(idx)),
                right: LengthPercentageAuto::Length(buf.margin_right(idx)),
                bottom: LengthPercentageAuto::Length(buf.margin_bottom(idx)),
                left: LengthPercentageAuto::Length(buf.margin_left(idx)),
            },

            padding: Rect {
                top: LengthPercentage::Length(buf.padding_top(idx)),
                right: LengthPercentage::Length(buf.padding_right(idx)),
                bottom: LengthPercentage::Length(buf.padding_bottom(idx)),
                left: LengthPercentage::Length(buf.padding_left(idx)),
            },

            border: Rect {
                top: LengthPercentage::Length(if buf.border_top(idx) > 0 { 1.0 } else { 0.0 }),
                right: LengthPercentage::Length(if buf.border_right(idx) > 0 { 1.0 } else { 0.0 }),
                bottom: LengthPercentage::Length(if buf.border_bottom(idx) > 0 { 1.0 } else { 0.0 }),
                left: LengthPercentage::Length(if buf.border_left(idx) > 0 { 1.0 } else { 0.0 }),
            },

            gap: Size {
                width: LengthPercentage::Length(effective_col_gap),
                height: LengthPercentage::Length(effective_row_gap),
            },

            inset: Rect {
                top: to_inset(buf.inset_top(idx)),
                right: to_inset(buf.inset_right(idx)),
                bottom: to_inset(buf.inset_bottom(idx)),
                left: to_inset(buf.inset_left(idx)),
            },

            overflow: taffy::Point {
                x: to_overflow(buf.overflow(idx)),
                y: to_overflow(buf.overflow(idx)),
            },

            ..Default::default()
        };

        // Text/Input nodes use measure function, not explicit size
        if comp_type == 2 || comp_type == 3 {
            style.size = Size::auto();
        }

        style
    }

    /// Write final layouts back to SharedBuffer output section.
    pub fn write_output(&self) {
        for i in 0..self.node_count {
            let comp_type = self.buf.component_type(i);
            if comp_type == 0 || !self.buf.visible(i) {
                continue;
            }

            let layout = &self.final_layout[i];
            self.buf.set_output(
                i,
                layout.location.x,
                layout.location.y,
                layout.size.width,
                layout.size.height,
            );

            // Check scrollability
            let overflow_val = self.buf.overflow(i);
            if overflow_val == 2 || overflow_val == 3 {
                // Scroll or Auto
                let content_w = layout.content_size.width;
                let content_h = layout.content_size.height;
                let max_scroll_x = (content_w - layout.size.width).max(0.0);
                let max_scroll_y = (content_h - layout.size.height).max(0.0);
                let scrollable = max_scroll_x > 0.0 || max_scroll_y > 0.0 || overflow_val == 2;
                self.buf.set_output_scroll(i, scrollable, max_scroll_x, max_scroll_y);
            }
        }
    }
}

// =============================================================================
// TRAIT IMPLEMENTATIONS
// =============================================================================

impl TraversePartialTree for LayoutTree<'_> {
    type ChildIter<'a> = ChildIter<'a> where Self: 'a;

    fn child_ids(&self, parent_node_id: NodeId) -> Self::ChildIter<'_> {
        let idx = usize::from(parent_node_id);
        let children = if idx < self.children.len() {
            &self.children[idx]
        } else {
            &[] as &[usize]
        };
        ChildIter {
            inner: children.iter(),
        }
    }

    fn child_count(&self, parent_node_id: NodeId) -> usize {
        let idx = usize::from(parent_node_id);
        if idx < self.children.len() {
            self.children[idx].len()
        } else {
            0
        }
    }

    fn get_child_id(&self, parent_node_id: NodeId, child_index: usize) -> NodeId {
        let idx = usize::from(parent_node_id);
        NodeId::from(self.children[idx][child_index])
    }
}

impl TraverseTree for LayoutTree<'_> {}

impl LayoutPartialTree for LayoutTree<'_> {
    type CoreContainerStyle<'a> = Style where Self: 'a;

    fn get_core_container_style(&self, node_id: NodeId) -> Self::CoreContainerStyle<'_> {
        self.build_style(usize::from(node_id))
    }

    fn set_unrounded_layout(&mut self, node_id: NodeId, layout: &Layout) {
        let idx = usize::from(node_id);
        if idx < self.unrounded.len() {
            self.unrounded[idx] = *layout;
        }
    }

    fn compute_child_layout(&mut self, node_id: NodeId, inputs: LayoutInput) -> LayoutOutput {
        compute_cached_layout(self, node_id, inputs, |tree, node_id, inputs| {
            let idx = usize::from(node_id);
            let comp_type = tree.buf.component_type(idx);

            // Hidden or unallocated nodes
            if comp_type == 0 || !tree.buf.visible(idx) {
                return compute_hidden_layout(tree, node_id);
            }

            match comp_type {
                // Box (1): flexbox container
                1 => compute_flexbox_layout(tree, node_id, inputs),

                // Text (2), Input (3): leaf nodes with measure function
                2 | 3 => {
                    let style = tree.build_style(idx);
                    compute_leaf_layout(inputs, &style, |known, avail| {
                        // We need to read from buf through raw pointer since tree is
                        // borrowed mutably by compute_cached_layout. The SharedBuffer
                        // is read-only during layout so this is safe.
                        let content = tree.buf.text_content(idx);
                        if content.is_empty() {
                            return Size::ZERO;
                        }

                        if comp_type == 3 {
                            // Input: single-line
                            let w = string_width(content).max(1) as f32;
                            return Size {
                                width: known.width.unwrap_or(w),
                                height: known.height.unwrap_or(1.0),
                            };
                        }

                        // Text: measure with wrapping
                        let avail_width = match avail.width {
                            AvailableSpace::Definite(w) => w as usize,
                            AvailableSpace::MinContent => string_width(content),
                            AvailableSpace::MaxContent => usize::MAX,
                        };

                        let text_w = string_width(content) as f32;
                        let text_h = measure_text_height(content, avail_width.max(1)) as f32;

                        Size {
                            width: known.width.unwrap_or(text_w),
                            height: known.height.unwrap_or(text_h),
                        }
                    })
                }

                // Unknown: hidden
                _ => compute_hidden_layout(tree, node_id),
            }
        })
    }
}

impl CacheTree for LayoutTree<'_> {
    fn cache_get(
        &self,
        node_id: NodeId,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
        run_mode: RunMode,
    ) -> Option<LayoutOutput> {
        let idx = usize::from(node_id);
        if idx < self.cache.len() {
            self.cache[idx].get(known_dimensions, available_space, run_mode)
        } else {
            None
        }
    }

    fn cache_store(
        &mut self,
        node_id: NodeId,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
        run_mode: RunMode,
        layout_output: LayoutOutput,
    ) {
        let idx = usize::from(node_id);
        if idx < self.cache.len() {
            self.cache[idx].store(known_dimensions, available_space, run_mode, layout_output);
        }
    }

    fn cache_clear(&mut self, node_id: NodeId) {
        let idx = usize::from(node_id);
        if idx < self.cache.len() {
            self.cache[idx].clear();
        }
    }
}

impl LayoutFlexboxContainer for LayoutTree<'_> {
    type FlexboxContainerStyle<'a> = Style where Self: 'a;
    type FlexboxItemStyle<'a> = Style where Self: 'a;

    fn get_flexbox_container_style(&self, node_id: NodeId) -> Self::FlexboxContainerStyle<'_> {
        self.build_style(usize::from(node_id))
    }

    fn get_flexbox_child_style(&self, child_node_id: NodeId) -> Self::FlexboxItemStyle<'_> {
        self.build_style(usize::from(child_node_id))
    }
}

impl RoundTree for LayoutTree<'_> {
    fn get_unrounded_layout(&self, node_id: NodeId) -> &Layout {
        &self.unrounded[usize::from(node_id)]
    }

    fn set_final_layout(&mut self, node_id: NodeId, layout: &Layout) {
        let idx = usize::from(node_id);
        if idx < self.final_layout.len() {
            self.final_layout[idx] = *layout;
        }
    }
}

// =============================================================================
// DIMENSION / ENUM CONVERSIONS
// =============================================================================

/// Convert a SharedBuffer float to Taffy dimension.
/// 0.0 = Auto, 0.0 < val <= 1.0 = Percent, val > 1.0 = Length (cells).
fn to_taffy_dim(val: f32) -> Dimension {
    if val.is_nan() {
        Dimension::Auto
    } else if val > 0.0 && val <= 1.0 {
        Dimension::Percent(val)
    } else if val > 1.0 {
        Dimension::Length(val)
    } else {
        Dimension::Auto
    }
}

/// Convert a SharedBuffer float to LengthPercentageAuto for insets.
fn to_inset(val: f32) -> LengthPercentageAuto {
    if val.is_nan() {
        LengthPercentageAuto::Auto
    } else if val > 0.0 && val <= 1.0 {
        LengthPercentageAuto::Percent(val)
    } else {
        LengthPercentageAuto::Length(val)
    }
}

fn to_flex_direction(val: u8) -> FlexDirection {
    match val {
        0 => FlexDirection::Column,
        1 => FlexDirection::Row,
        2 => FlexDirection::ColumnReverse,
        3 => FlexDirection::RowReverse,
        _ => FlexDirection::Column,
    }
}

fn to_flex_wrap(val: u8) -> FlexWrap {
    match val {
        0 => FlexWrap::NoWrap,
        1 => FlexWrap::Wrap,
        2 => FlexWrap::WrapReverse,
        _ => FlexWrap::NoWrap,
    }
}

fn to_justify(val: u8) -> Option<JustifyContent> {
    Some(match val {
        0 => JustifyContent::FlexStart,
        1 => JustifyContent::Center,
        2 => JustifyContent::FlexEnd,
        3 => JustifyContent::SpaceBetween,
        4 => JustifyContent::SpaceAround,
        5 => JustifyContent::SpaceEvenly,
        _ => JustifyContent::FlexStart,
    })
}

fn to_align_items(val: u8) -> Option<AlignItems> {
    Some(match val {
        0 => AlignItems::Stretch,
        1 => AlignItems::FlexStart,
        2 => AlignItems::Center,
        3 => AlignItems::FlexEnd,
        4 => AlignItems::Baseline,
        _ => AlignItems::Stretch,
    })
}

fn to_align_self(val: u8) -> Option<AlignSelf> {
    match val {
        0 => None, // auto = inherit
        1 => Some(AlignSelf::Stretch),
        2 => Some(AlignSelf::FlexStart),
        3 => Some(AlignSelf::Center),
        4 => Some(AlignSelf::FlexEnd),
        5 => Some(AlignSelf::Baseline),
        _ => None,
    }
}

fn to_align_content(val: u8) -> Option<AlignContent> {
    Some(match val {
        0 => AlignContent::Stretch,
        1 => AlignContent::FlexStart,
        2 => AlignContent::Center,
        3 => AlignContent::FlexEnd,
        4 => AlignContent::SpaceBetween,
        5 => AlignContent::SpaceAround,
        _ => AlignContent::Stretch,
    })
}

fn to_overflow(val: u8) -> TaffyOverflow {
    match val {
        0 => TaffyOverflow::Visible,
        1 => TaffyOverflow::Clip,
        2 => TaffyOverflow::Scroll,
        3 => TaffyOverflow::Scroll, // Auto → Scroll
        _ => TaffyOverflow::Visible,
    }
}

fn to_position(val: u8) -> Position {
    match val {
        1 => Position::Absolute,
        _ => Position::Relative,
    }
}

// =============================================================================
// PUBLIC ENTRY POINT
// =============================================================================

/// Compute layout directly from SharedBuffer using Taffy's low-level trait API.
///
/// This replaces `compute_layout_from_buffer` in lib.rs with zero-copy,
/// zero-translation layout. NodeId IS the component index.
///
/// Returns the number of nodes laid out.
pub fn compute_layout_direct(buf: &SharedBuffer) -> u32 {
    let mut tree = LayoutTree::new(buf);

    if tree.roots.is_empty() {
        return 0;
    }

    let tw = buf.terminal_width() as f32;
    let th = buf.terminal_height() as f32;
    let available = Size {
        width: AvailableSpace::Definite(tw),
        height: AvailableSpace::Definite(th),
    };

    // Compute layout for each root
    let roots = tree.roots.clone();
    for &root_idx in &roots {
        compute_root_layout(&mut tree, NodeId::from(root_idx), available);
    }

    // Round to pixel grid
    for &root_idx in &roots {
        round_layout(&mut tree, NodeId::from(root_idx));
    }

    // Write results to SharedBuffer output section
    tree.write_output();

    tree.node_count as u32
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared_buffer::*;

    /// Create a test SharedBuffer backed by a Vec.
    fn make_test_buffer(node_count: usize) -> (Vec<u8>, *mut u8) {
        let mut data = vec![0u8; TOTAL_BUFFER_SIZE];
        let ptr = data.as_mut_ptr();

        // Write header
        let header = ptr as *mut u32;
        unsafe {
            header.add(HEADER_VERSION).write(2); // v2
            header.add(HEADER_NODE_COUNT).write(node_count as u32);
            header.add(HEADER_MAX_NODES).write(MAX_NODES as u32);
            header.add(HEADER_TERMINAL_WIDTH).write(80);
            header.add(HEADER_TERMINAL_HEIGHT).write(24);
        }

        (data, ptr)
    }

    /// Set metadata u8 for a node (SoA: each field is a contiguous array).
    fn set_meta(ptr: *mut u8, node: usize, field: usize, value: u8) {
        unsafe {
            *ptr.add(SECTION_U8_OFFSET + field * U8_FIELD_BYTES + node) = value;
        }
    }

    /// Set float for a node (SoA: each field is a contiguous f32 array).
    fn set_float(ptr: *mut u8, node: usize, field: usize, value: f32) {
        unsafe {
            let field_ptr = ptr.add(SECTION_F32_OFFSET + field * F32_FIELD_BYTES) as *mut f32;
            field_ptr.add(node).write(value);
        }
    }

    /// Set hierarchy (parent index) for a node (SoA: I32_PARENT_INDEX field).
    fn set_parent(ptr: *mut u8, node: usize, parent: i32) {
        unsafe {
            let field_ptr = ptr.add(SECTION_I32_OFFSET + I32_PARENT_INDEX * I32_FIELD_BYTES) as *mut i32;
            field_ptr.add(node).write(parent);
        }
    }

    /// Set text content for a node.
    fn set_text(ptr: *mut u8, node: usize, text: &str) {
        unsafe {
            // Write text to pool
            let pool_ptr = ptr.add(SECTION_TEXT_POOL_OFFSET);
            // Use node * 64 as a simple offset (each node gets 64 bytes)
            let offset = node * 64;
            std::ptr::copy_nonoverlapping(text.as_ptr(), pool_ptr.add(offset), text.len());

            // Write text index (SoA: separate arrays for offset and length)
            let offset_field = ptr.add(SECTION_U32_OFFSET + U32_TEXT_OFFSET * U32_FIELD_BYTES) as *mut u32;
            let length_field = ptr.add(SECTION_U32_OFFSET + U32_TEXT_LENGTH * U32_FIELD_BYTES) as *mut u32;
            offset_field.add(node).write(offset as u32);
            length_field.add(node).write(text.len() as u32);
        }
    }

    /// Read output for a node (SoA: each output field is its own f32 array).
    fn read_output(ptr: *mut u8, node: usize) -> (f32, f32, f32, f32) {
        unsafe {
            let x_ptr = ptr.add(SECTION_F32_OFFSET + F32_COMPUTED_X * F32_FIELD_BYTES) as *const f32;
            let y_ptr = ptr.add(SECTION_F32_OFFSET + F32_COMPUTED_Y * F32_FIELD_BYTES) as *const f32;
            let w_ptr = ptr.add(SECTION_F32_OFFSET + F32_COMPUTED_WIDTH * F32_FIELD_BYTES) as *const f32;
            let h_ptr = ptr.add(SECTION_F32_OFFSET + F32_COMPUTED_HEIGHT * F32_FIELD_BYTES) as *const f32;
            (
                x_ptr.add(node).read(),
                y_ptr.add(node).read(),
                w_ptr.add(node).read(),
                h_ptr.add(node).read(),
            )
        }
    }

    /// Set up a node as a visible box with given dimensions.
    fn setup_box(ptr: *mut u8, node: usize, width: f32, height: f32) {
        set_meta(ptr, node, META_COMPONENT_TYPE, 1); // Box
        set_meta(ptr, node, META_VISIBLE, 1);
        set_float(ptr, node, FLOAT_WIDTH, width);
        set_float(ptr, node, FLOAT_HEIGHT, height);
        set_float(ptr, node, FLOAT_SHRINK, 1.0); // default shrink
        set_parent(ptr, node, -1); // root by default
    }

    #[test]
    fn test_children_map_building() {
        let (data, ptr) = make_test_buffer(3);
        let buf = unsafe { SharedBuffer::from_ptr(ptr, data.len()) };

        setup_box(ptr, 0, 80.0, 24.0);
        setup_box(ptr, 1, 20.0, 10.0);
        set_parent(ptr, 1, 0);
        setup_box(ptr, 2, 20.0, 10.0);
        set_parent(ptr, 2, 0);

        let tree = LayoutTree::new(&buf);

        assert_eq!(tree.children[0], vec![1, 2]);
        assert!(tree.children[1].is_empty());
        assert!(tree.children[2].is_empty());
        assert_eq!(tree.roots, vec![0]);

        // Keep data alive
        let _ = &data;
    }

    #[test]
    fn test_single_root_layout() {
        let (data, ptr) = make_test_buffer(1);
        let buf = unsafe { SharedBuffer::from_ptr(ptr, data.len()) };

        setup_box(ptr, 0, 40.0, 10.0);

        compute_layout_direct(&buf);

        let (x, y, w, h) = read_output(ptr, 0);
        assert_eq!(x, 0.0);
        assert_eq!(y, 0.0);
        assert_eq!(w, 40.0);
        assert_eq!(h, 10.0);

        let _ = &data;
    }

    #[test]
    fn test_parent_child_layout() {
        let (data, ptr) = make_test_buffer(2);
        let buf = unsafe { SharedBuffer::from_ptr(ptr, data.len()) };

        // Parent: 40x10
        setup_box(ptr, 0, 40.0, 10.0);

        // Child: 20x5
        setup_box(ptr, 1, 20.0, 5.0);
        set_parent(ptr, 1, 0);

        compute_layout_direct(&buf);

        let (px, py, pw, ph) = read_output(ptr, 0);
        assert_eq!((px, py, pw, ph), (0.0, 0.0, 40.0, 10.0));

        let (cx, cy, cw, ch) = read_output(ptr, 1);
        assert_eq!((cx, cy), (0.0, 0.0));
        assert_eq!(cw, 20.0);
        assert_eq!(ch, 5.0);

        let _ = &data;
    }

    #[test]
    fn test_flex_row() {
        let (data, ptr) = make_test_buffer(3);
        let buf = unsafe { SharedBuffer::from_ptr(ptr, data.len()) };

        // Parent: 40x10, flex-direction: row
        setup_box(ptr, 0, 40.0, 10.0);
        set_meta(ptr, 0, META_FLEX_DIRECTION, 1); // Row

        // Child 1: 10x5
        setup_box(ptr, 1, 10.0, 5.0);
        set_parent(ptr, 1, 0);

        // Child 2: 10x5
        setup_box(ptr, 2, 10.0, 5.0);
        set_parent(ptr, 2, 0);

        compute_layout_direct(&buf);

        let (c1x, _, c1w, _) = read_output(ptr, 1);
        let (c2x, _, c2w, _) = read_output(ptr, 2);

        assert_eq!(c1x, 0.0);
        assert_eq!(c1w, 10.0);
        assert_eq!(c2x, 10.0); // After first child
        assert_eq!(c2w, 10.0);

        let _ = &data;
    }

    #[test]
    fn test_flex_grow() {
        let (data, ptr) = make_test_buffer(2);
        let buf = unsafe { SharedBuffer::from_ptr(ptr, data.len()) };

        // Parent: 100x10, row
        setup_box(ptr, 0, 100.0, 10.0);
        set_meta(ptr, 0, META_FLEX_DIRECTION, 1); // Row

        // Child: flex-grow=1
        setup_box(ptr, 1, 0.0, 5.0); // auto width
        set_float(ptr, 1, FLOAT_GROW, 1.0);
        set_parent(ptr, 1, 0);

        compute_layout_direct(&buf);

        let (_, _, cw, _) = read_output(ptr, 1);
        assert_eq!(cw, 100.0); // Should fill parent

        let _ = &data;
    }

    #[test]
    fn test_padding_and_border() {
        let (data, ptr) = make_test_buffer(2);
        let buf = unsafe { SharedBuffer::from_ptr(ptr, data.len()) };

        // Parent: 40x10, padding-left=2, padding-top=1, border-left=1
        setup_box(ptr, 0, 40.0, 10.0);
        set_float(ptr, 0, FLOAT_PADDING_LEFT, 2.0);
        set_float(ptr, 0, FLOAT_PADDING_TOP, 1.0);
        set_meta(ptr, 0, META_BORDER_LEFT_WIDTH, 1);

        // Child: 10x5
        setup_box(ptr, 1, 10.0, 5.0);
        set_parent(ptr, 1, 0);

        compute_layout_direct(&buf);

        let (cx, cy, _, _) = read_output(ptr, 1);
        assert_eq!(cx, 3.0); // 2 padding + 1 border
        assert_eq!(cy, 1.0); // 1 padding, no border top

        let _ = &data;
    }

    #[test]
    fn test_justify_center() {
        let (data, ptr) = make_test_buffer(2);
        let buf = unsafe { SharedBuffer::from_ptr(ptr, data.len()) };

        // Parent: 100x10, row, justify-content: center
        setup_box(ptr, 0, 100.0, 10.0);
        set_meta(ptr, 0, META_FLEX_DIRECTION, 1); // Row
        set_meta(ptr, 0, META_JUSTIFY_CONTENT, 1); // Center

        // Child: 20x5
        setup_box(ptr, 1, 20.0, 5.0);
        set_parent(ptr, 1, 0);

        compute_layout_direct(&buf);

        let (cx, _, _, _) = read_output(ptr, 1);
        assert_eq!(cx, 40.0); // (100 - 20) / 2 = 40

        let _ = &data;
    }

    #[test]
    fn test_dimension_encoding() {
        // Auto (0.0 or NaN)
        assert!(matches!(to_taffy_dim(0.0), Dimension::Auto));
        assert!(matches!(to_taffy_dim(f32::NAN), Dimension::Auto));

        // Percent (0.0 < val <= 1.0)
        if let Dimension::Percent(p) = to_taffy_dim(0.5) {
            assert!((p - 0.5).abs() < 0.001);
        } else {
            panic!("Expected Percent");
        }

        // Length (val > 1.0)
        if let Dimension::Length(l) = to_taffy_dim(40.0) {
            assert_eq!(l, 40.0);
        } else {
            panic!("Expected Length");
        }
    }

    #[test]
    fn test_text_measurement() {
        let (data, ptr) = make_test_buffer(2);
        let buf = unsafe { SharedBuffer::from_ptr(ptr, data.len()) };

        // Parent: 80x24, align_items: FlexStart so text gets intrinsic width
        setup_box(ptr, 0, 80.0, 24.0);
        set_meta(ptr, 0, META_ALIGN_ITEMS, 1); // FlexStart (no cross-axis stretch)

        // Text node
        set_meta(ptr, 1, META_COMPONENT_TYPE, 2); // Text
        set_meta(ptr, 1, META_VISIBLE, 1);
        set_float(ptr, 1, FLOAT_SHRINK, 1.0);
        set_parent(ptr, 1, 0);
        set_text(ptr, 1, "Hello");

        compute_layout_direct(&buf);

        let (_, _, tw, th) = read_output(ptr, 1);
        assert_eq!(tw, 5.0); // "Hello" = 5 chars
        assert_eq!(th, 1.0); // 1 line

        let _ = &data;
    }

    #[test]
    fn test_text_measurement_stretch() {
        let (data, ptr) = make_test_buffer(2);
        let buf = unsafe { SharedBuffer::from_ptr(ptr, data.len()) };

        // Parent: 80x24, default align_items (Stretch)
        setup_box(ptr, 0, 80.0, 24.0);

        // Text node stretches to parent width in column direction
        set_meta(ptr, 1, META_COMPONENT_TYPE, 2); // Text
        set_meta(ptr, 1, META_VISIBLE, 1);
        set_float(ptr, 1, FLOAT_SHRINK, 1.0);
        set_parent(ptr, 1, 0);
        set_text(ptr, 1, "Hello");

        compute_layout_direct(&buf);

        let (_, _, tw, th) = read_output(ptr, 1);
        // In column direction with align_items: Stretch, text width = parent width
        assert_eq!(tw, 80.0);
        assert_eq!(th, 1.0);

        let _ = &data;
    }

    #[test]
    fn test_input_measurement() {
        let (data, ptr) = make_test_buffer(2);
        let buf = unsafe { SharedBuffer::from_ptr(ptr, data.len()) };

        // Parent: 80x24, align_items: FlexStart so input gets intrinsic width
        setup_box(ptr, 0, 80.0, 24.0);
        set_meta(ptr, 0, META_ALIGN_ITEMS, 1); // FlexStart

        // Input node
        set_meta(ptr, 1, META_COMPONENT_TYPE, 3); // Input
        set_meta(ptr, 1, META_VISIBLE, 1);
        set_float(ptr, 1, FLOAT_SHRINK, 1.0);
        set_parent(ptr, 1, 0);
        set_text(ptr, 1, "user@host");

        compute_layout_direct(&buf);

        let (_, _, iw, ih) = read_output(ptr, 1);
        assert_eq!(iw, 9.0); // "user@host" = 9 chars
        assert_eq!(ih, 1.0); // Always 1 line for input

        let _ = &data;
    }

    #[test]
    fn test_absolute_positioning() {
        let (data, ptr) = make_test_buffer(2);
        let buf = unsafe { SharedBuffer::from_ptr(ptr, data.len()) };

        // Parent: 80x24
        setup_box(ptr, 0, 80.0, 24.0);

        // Absolute child: 20x10, top=5, left=10
        setup_box(ptr, 1, 20.0, 10.0);
        set_meta(ptr, 1, META_POSITION, 1); // Absolute
        set_float(ptr, 1, FLOAT_TOP, 5.0);
        set_float(ptr, 1, FLOAT_LEFT, 10.0);
        set_parent(ptr, 1, 0);

        compute_layout_direct(&buf);

        let (cx, cy, cw, ch) = read_output(ptr, 1);
        assert_eq!(cx, 10.0);
        assert_eq!(cy, 5.0);
        assert_eq!(cw, 20.0);
        assert_eq!(ch, 10.0);

        let _ = &data;
    }

    #[test]
    fn test_invisible_nodes_skipped() {
        let (data, ptr) = make_test_buffer(2);
        let buf = unsafe { SharedBuffer::from_ptr(ptr, data.len()) };

        // Visible root
        setup_box(ptr, 0, 80.0, 24.0);

        // Invisible child
        set_meta(ptr, 1, META_COMPONENT_TYPE, 1);
        set_meta(ptr, 1, META_VISIBLE, 0); // invisible
        set_float(ptr, 1, FLOAT_WIDTH, 20.0);
        set_float(ptr, 1, FLOAT_HEIGHT, 10.0);
        set_parent(ptr, 1, 0);

        let tree = LayoutTree::new(&buf);

        // Invisible child should not appear in children map
        assert!(tree.children[0].is_empty());

        let _ = &data;
    }

    #[test]
    fn test_multiple_roots() {
        let (data, ptr) = make_test_buffer(2);
        let buf = unsafe { SharedBuffer::from_ptr(ptr, data.len()) };

        // Two independent roots
        setup_box(ptr, 0, 40.0, 10.0);
        setup_box(ptr, 1, 30.0, 8.0);

        compute_layout_direct(&buf);

        let (_, _, w0, h0) = read_output(ptr, 0);
        let (_, _, w1, h1) = read_output(ptr, 1);
        assert_eq!(w0, 40.0);
        assert_eq!(h0, 10.0);
        assert_eq!(w1, 30.0);
        assert_eq!(h1, 8.0);

        let _ = &data;
    }
}
