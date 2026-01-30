//! AoS Layout Tree - Taffy integration with AoS SharedBuffer
//!
//! This module implements Taffy's low-level trait API using the AoS buffer layout.
//! All properties for each node are contiguous in memory, enabling cache-friendly reads.

use taffy::prelude::*;
use taffy::{
    compute_cached_layout, compute_flexbox_layout, compute_hidden_layout, compute_leaf_layout,
    compute_root_layout, round_layout, Cache, CacheTree, Layout, LayoutFlexboxContainer,
    LayoutInput, LayoutOutput, LayoutPartialTree, NodeId, Overflow as TaffyOverflow, PrintTree,
    RoundTree, TraversePartialTree, TraverseTree,
};

use crate::shared_buffer_aos::{
    AoSBuffer, COMPONENT_BOX, COMPONENT_INPUT, COMPONENT_NONE, COMPONENT_TEXT,
};

use super::text_measure::{measure_text_height, string_width};

// =============================================================================
// LAYOUT CONTEXT (persistent, reused across frames)
// =============================================================================

use std::cell::RefCell;

pub struct LayoutContext {
    pub children: Vec<Vec<usize>>,
    pub cache: Vec<Cache>,
    pub unrounded: Vec<Layout>,
    pub final_layout: Vec<Layout>,
    pub roots: Vec<usize>,
}

impl LayoutContext {
    fn new() -> Self {
        Self {
            children: Vec::new(),
            cache: Vec::new(),
            unrounded: Vec::new(),
            final_layout: Vec::new(),
            roots: Vec::new(),
        }
    }

    fn ensure_capacity(&mut self, count: usize) {
        if count > self.cache.len() {
            self.children.resize(count, Vec::new());
            self.cache.resize(count, Cache::new());
            self.unrounded.resize(count, Layout::with_order(0));
            self.final_layout.resize(count, Layout::with_order(0));
        }
    }
}

thread_local! {
    static LAYOUT_CONTEXT: RefCell<LayoutContext> = RefCell::new(LayoutContext::new());
}

// =============================================================================
// CHILD ITERATOR
// =============================================================================

pub struct ChildIter<'a> {
    inner: std::slice::Iter<'a, usize>,
}

impl Iterator for ChildIter<'_> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|&idx| NodeId::from(idx))
    }
}

impl ExactSizeIterator for ChildIter<'_> {}

// =============================================================================
// LAYOUT TREE (AoS)
// =============================================================================

pub struct LayoutTreeAoS<'a> {
    pub buf: &'a AoSBuffer,
    pub ctx: &'a mut LayoutContext,
}

impl<'a> LayoutTreeAoS<'a> {
    /// Build a Taffy Style from the AoS buffer.
    ///
    /// ALL reads are from contiguous memory for this node - cache friendly!
    fn build_style(&self, idx: usize) -> Style {
        let buf = self.buf;
        let comp_type = buf.component_type(idx);

        // Read gap values
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

            flex_grow: buf.flex_grow(idx),
            flex_shrink: buf.flex_shrink(idx),
            flex_basis: to_taffy_dim(buf.flex_basis(idx)),
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
        if comp_type == COMPONENT_TEXT || comp_type == COMPONENT_INPUT {
            style.size = Size {
                width: Dimension::Auto,
                height: Dimension::Auto,
            };
        }

        style
    }

    /// Write computed layout to output section of AoS buffer.
    ///
    /// Also detects scroll overflow: if content_size > layout.size and overflow
    /// mode allows scrolling, the node becomes scrollable with calculated
    /// max_scroll values.
    fn write_output(&self, node_count: usize) {
        for idx in 0..node_count {
            let comp_type = self.buf.component_type(idx);
            if comp_type == COMPONENT_NONE {
                continue;
            }

            let layout = self.ctx.final_layout[idx];
            self.buf.set_computed_x(idx, layout.location.x);
            self.buf.set_computed_y(idx, layout.location.y);
            self.buf.set_computed_width(idx, layout.size.width);
            self.buf.set_computed_height(idx, layout.size.height);

            // Auto-scroll detection based on computed sizes
            // Rule: if children computed size > node computed size → node becomes scrollable
            // This works for any container (box, future components) - text has no children so excluded
            //
            // overflow values: 0=visible (default), 1=hidden (opt-out), 2=scroll, 3=auto
            let has_children = !self.ctx.children[idx].is_empty();
            let overflow = self.buf.overflow(idx);

            // Calculate overflow from computed sizes
            let content_w = layout.content_size.width;
            let content_h = layout.content_size.height;
            let max_scroll_x = (content_w - layout.size.width).max(0.0);
            let max_scroll_y = (content_h - layout.size.height).max(0.0);
            let content_overflows = max_scroll_x > 0.0 || max_scroll_y > 0.0;

            // Determine scrollability:
            // - overflow: hidden (1) → opt-out, never scrollable
            // - overflow: scroll/auto (2,3) → always enable scrollable flag
            // - default/visible (0) → auto-scroll when has_children AND content overflows
            let is_scrollable = match overflow {
                1 => false, // hidden: explicit opt-out
                2 | 3 => true, // scroll/auto: always scrollable (even if no overflow yet)
                _ => has_children && content_overflows, // auto-scroll on overflow
            };

            self.buf.set_output_scroll(idx, is_scrollable, max_scroll_x, max_scroll_y);
        }
    }
}

// =============================================================================
// TAFFY TRAIT IMPLEMENTATIONS
// =============================================================================

impl TraversePartialTree for LayoutTreeAoS<'_> {
    type ChildIter<'a> = ChildIter<'a> where Self: 'a;

    fn child_ids(&self, parent_node_id: NodeId) -> Self::ChildIter<'_> {
        let idx = usize::from(parent_node_id);
        let children = if idx < self.ctx.children.len() {
            &self.ctx.children[idx]
        } else {
            &[] as &[usize]
        };
        ChildIter {
            inner: children.iter(),
        }
    }

    fn child_count(&self, parent_node_id: NodeId) -> usize {
        let idx = usize::from(parent_node_id);
        if idx < self.ctx.children.len() {
            self.ctx.children[idx].len()
        } else {
            0
        }
    }

    fn get_child_id(&self, parent_node_id: NodeId, child_index: usize) -> NodeId {
        let idx = usize::from(parent_node_id);
        NodeId::from(self.ctx.children[idx][child_index])
    }
}

impl TraverseTree for LayoutTreeAoS<'_> {}

impl PrintTree for LayoutTreeAoS<'_> {
    fn get_debug_label(&self, node_id: NodeId) -> &'static str {
        let idx = usize::from(node_id);
        match self.buf.component_type(idx) {
            COMPONENT_BOX => "Box",
            COMPONENT_TEXT => "Text",
            COMPONENT_INPUT => "Input",
            _ => "Unknown",
        }
    }

    fn get_final_layout(&self, node_id: NodeId) -> &Layout {
        &self.ctx.final_layout[usize::from(node_id)]
    }
}

impl LayoutPartialTree for LayoutTreeAoS<'_> {
    type CoreContainerStyle<'a> = Style where Self: 'a;

    fn get_core_container_style(&self, node_id: NodeId) -> Self::CoreContainerStyle<'_> {
        self.build_style(usize::from(node_id))
    }

    fn set_unrounded_layout(&mut self, node_id: NodeId, layout: &Layout) {
        let idx = usize::from(node_id);
        if idx < self.ctx.unrounded.len() {
            self.ctx.unrounded[idx] = *layout;
        }
    }

    fn compute_child_layout(
        &mut self,
        node_id: NodeId,
        inputs: LayoutInput,
    ) -> LayoutOutput {
        compute_cached_layout(self, node_id, inputs, |tree, node, inputs| {
            let idx = usize::from(node);
            let comp_type = tree.buf.component_type(idx);

            // Hidden or unallocated nodes
            if comp_type == COMPONENT_NONE || !tree.buf.visible(idx) {
                return compute_hidden_layout(tree, node);
            }

            match comp_type {
                // Box (1): flexbox container
                COMPONENT_BOX => compute_flexbox_layout(tree, node, inputs),

                // Text (2), Input (3): leaf nodes with measure function
                COMPONENT_TEXT | COMPONENT_INPUT => {
                    let style = tree.build_style(idx);
                    compute_leaf_layout(inputs, &style, |known, available| {
                        let text = tree.buf.text(idx);
                        if text.is_empty() {
                            return Size::ZERO;
                        }

                        let max_width = match known.width {
                            Some(w) => w as usize,
                            None => match available.width {
                                AvailableSpace::Definite(w) => w as usize,
                                AvailableSpace::MinContent => 1,
                                AvailableSpace::MaxContent => usize::MAX,
                            },
                        };

                        let text_w = string_width(text);
                        let text_h = measure_text_height(text, max_width);

                        Size {
                            width: text_w as f32,
                            height: text_h as f32,
                        }
                    })
                }

                // Unknown: treat as hidden
                _ => compute_hidden_layout(tree, node),
            }
        })
    }
}

impl CacheTree for LayoutTreeAoS<'_> {
    fn cache_get(
        &self,
        node_id: NodeId,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
        run_mode: taffy::RunMode,
    ) -> Option<taffy::LayoutOutput> {
        let idx = usize::from(node_id);
        if idx < self.ctx.cache.len() {
            self.ctx.cache[idx].get(known_dimensions, available_space, run_mode)
        } else {
            None
        }
    }

    fn cache_store(
        &mut self,
        node_id: NodeId,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
        run_mode: taffy::RunMode,
        layout_output: taffy::LayoutOutput,
    ) {
        let idx = usize::from(node_id);
        if idx < self.ctx.cache.len() {
            self.ctx.cache[idx].store(known_dimensions, available_space, run_mode, layout_output);
        }
    }

    fn cache_clear(&mut self, node_id: NodeId) {
        let idx = usize::from(node_id);
        if idx < self.ctx.cache.len() {
            self.ctx.cache[idx].clear();
        }
    }
}

impl LayoutFlexboxContainer for LayoutTreeAoS<'_> {
    type FlexboxContainerStyle<'a> = Style where Self: 'a;
    type FlexboxItemStyle<'a> = Style where Self: 'a;

    fn get_flexbox_container_style(&self, node_id: NodeId) -> Self::FlexboxContainerStyle<'_> {
        self.build_style(usize::from(node_id))
    }

    fn get_flexbox_child_style(&self, child_node_id: NodeId) -> Self::FlexboxItemStyle<'_> {
        self.build_style(usize::from(child_node_id))
    }
}

impl RoundTree for LayoutTreeAoS<'_> {
    fn get_unrounded_layout(&self, node_id: NodeId) -> &Layout {
        &self.ctx.unrounded[usize::from(node_id)]
    }

    fn set_final_layout(&mut self, node_id: NodeId, layout: &Layout) {
        let idx = usize::from(node_id);
        if idx < self.ctx.final_layout.len() {
            self.ctx.final_layout[idx] = *layout;
        }
    }
}

// =============================================================================
// CONVERSION HELPERS
// =============================================================================

fn to_taffy_dim(val: f32) -> Dimension {
    if val.is_nan() || val == f32::MAX {
        // NaN or MAX = unset = Auto
        Dimension::Auto
    } else if val < 0.0 {
        // Negative = percent (encoded as -percent)
        Dimension::Percent(-val / 100.0)
    } else {
        // 0.0 or positive = explicit pixels
        Dimension::Length(val)
    }
}

fn to_inset(val: f32) -> LengthPercentageAuto {
    if val.is_nan() || val == f32::MAX {
        // NaN or MAX = unset = Auto
        LengthPercentageAuto::Auto
    } else {
        // 0.0 or positive = explicit value
        LengthPercentageAuto::Length(val)
    }
}

fn to_flex_direction(val: u8) -> FlexDirection {
    match val {
        0 => FlexDirection::Row,
        1 => FlexDirection::Column,
        2 => FlexDirection::RowReverse,
        3 => FlexDirection::ColumnReverse,
        _ => FlexDirection::Row,
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
    match val {
        0 => Some(JustifyContent::FlexStart),
        1 => Some(JustifyContent::FlexEnd),
        2 => Some(JustifyContent::Center),
        3 => Some(JustifyContent::SpaceBetween),
        4 => Some(JustifyContent::SpaceAround),
        5 => Some(JustifyContent::SpaceEvenly),
        _ => None,
    }
}

fn to_align_items(val: u8) -> Option<AlignItems> {
    match val {
        0 => Some(AlignItems::FlexStart),
        1 => Some(AlignItems::FlexEnd),
        2 => Some(AlignItems::Center),
        3 => Some(AlignItems::Stretch),
        4 => Some(AlignItems::Baseline),
        _ => None,
    }
}

fn to_align_content(val: u8) -> Option<AlignContent> {
    match val {
        0 => Some(AlignContent::FlexStart),
        1 => Some(AlignContent::FlexEnd),
        2 => Some(AlignContent::Center),
        3 => Some(AlignContent::Stretch),
        4 => Some(AlignContent::SpaceBetween),
        5 => Some(AlignContent::SpaceAround),
        _ => None,
    }
}

fn to_align_self(val: u8) -> Option<AlignSelf> {
    match val {
        0 => None, // Auto
        1 => Some(AlignSelf::FlexStart),
        2 => Some(AlignSelf::FlexEnd),
        3 => Some(AlignSelf::Center),
        4 => Some(AlignSelf::Stretch),
        5 => Some(AlignSelf::Baseline),
        _ => None,
    }
}

fn to_position(val: u8) -> Position {
    match val {
        1 => Position::Absolute,
        _ => Position::Relative,
    }
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

// =============================================================================
// PUBLIC API
// =============================================================================

/// Compute layout using AoS buffer.
/// Returns number of nodes processed.
pub fn compute_layout_aos(buf: &AoSBuffer) -> u32 {
    let node_count = buf.node_count();

    LAYOUT_CONTEXT.with(|ctx_cell| {
        let mut ctx = ctx_cell.borrow_mut();

        // 1. Ensure capacity
        ctx.ensure_capacity(node_count);

        // 2. Clear children map (reuse capacity)
        ctx.roots.clear();
        for i in 0..node_count {
            if i < ctx.children.len() {
                ctx.children[i].clear();
            }
            // Clear all caches for fresh computation (can optimize later with dirty flags)
            if i < ctx.cache.len() {
                ctx.cache[i].clear();
            }
        }

        // 3. Build hierarchy
        for i in 0..node_count {
            let comp_type = buf.component_type(i);
            if comp_type == COMPONENT_NONE || !buf.visible(i) {
                continue;
            }

            match buf.parent_index(i) {
                Some(parent_idx) if parent_idx < node_count => {
                    let parent_type = buf.component_type(parent_idx);
                    if parent_type != COMPONENT_NONE && buf.visible(parent_idx) {
                        ctx.children[parent_idx].push(i);
                    } else {
                        ctx.roots.push(i);
                    }
                }
                _ => {
                    ctx.roots.push(i);
                }
            }
        }

        // 4. Create tree wrapper
        let mut tree = LayoutTreeAoS {
            buf,
            ctx: &mut *ctx,
        };

        if tree.ctx.roots.is_empty() {
            return 0;
        }

        let tw = buf.terminal_width() as f32;
        let th = buf.terminal_height() as f32;

        // Available space = terminal dimensions (maximum constraint).
        // The root element's styles determine actual size:
        // - height: auto → content height
        // - height: 100% → terminal height
        // - height: 20 → explicit 20 rows
        let available = Size {
            width: AvailableSpace::Definite(tw),
            height: AvailableSpace::Definite(th),
        };

        // 5. Compute layout for each root
        let roots = tree.ctx.roots.clone();
        for &root_idx in &roots {
            compute_root_layout(&mut tree, NodeId::from(root_idx), available);
        }

        // 6. Round to pixel grid
        for &root_idx in &roots {
            round_layout(&mut tree, NodeId::from(root_idx));
        }

        // 7. Write results
        tree.write_output(node_count);

        node_count as u32
    })
}
