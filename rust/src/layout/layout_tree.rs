//! Layout Tree - Taffy 0.9 Low-Level API on SharedBuffer
//!
//! Zero-copy integration: NodeStyle reads directly from SharedBuffer bytes.
//! No intermediate structs, no translations, no allocations for style access.
//!
//! # Architecture
//!
//! ```text
//! SharedBuffer (1024 bytes/node)
//!     │
//!     ├── NodeStyle<'a> ──► Implements CoreStyle + Flexbox + Grid traits
//!     │   (zero-copy)       Reads from buffer on each method call
//!     │
//!     └── LayoutTree<'a> ──► Implements all 6 Taffy layout traits
//!         (wrapper)          Dispatches flex/grid/hidden based on display
//! ```
//!
//! # Taffy 0.9 Traits Implemented
//!
//! - TraversePartialTree: child iteration
//! - TraverseTree: marker for recursive access
//! - LayoutPartialTree: core layout dispatch
//! - LayoutFlexboxContainer: flexbox styles
//! - LayoutGridContainer: grid styles
//! - CacheTree: per-node layout caching
//! - RoundTree: pixel snapping
//! - PrintTree: debug output

use std::cell::RefCell;
use std::sync::Arc;

use taffy::prelude::*;
use taffy::style::GenericGridTemplateComponent;
use taffy::{
    compute_cached_layout, compute_flexbox_layout, compute_grid_layout, compute_hidden_layout,
    compute_leaf_layout, compute_root_layout, round_layout, Cache, CacheTree, Layout,
    LayoutFlexboxContainer, LayoutGridContainer, LayoutInput, LayoutOutput, LayoutPartialTree,
    NodeId, PrintTree, RoundTree, TraversePartialTree, TraverseTree,
};

use crate::shared_buffer::{
    SharedBuffer, RenderMode, COMPONENT_BOX, COMPONENT_INPUT, COMPONENT_NONE, COMPONENT_TEXT, DIRTY_LAYOUT,
};

use super::text_measure::{measure_text_height, string_width};

// =============================================================================
// CONSTANTS
// =============================================================================

const DISPLAY_NONE: u8 = 0;
const DISPLAY_FLEX: u8 = 1;
const DISPLAY_GRID: u8 = 2;

// =============================================================================
// LAYOUT CONTEXT (thread-local, reused across frames)
// =============================================================================

pub struct LayoutContext {
    children: Vec<Vec<usize>>,
    cache: Vec<Cache>,
    unrounded: Vec<Layout>,
    final_layout: Vec<Layout>,
    roots: Vec<usize>,
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
            self.children.resize_with(count, Vec::new);
            self.cache.resize_with(count, Cache::new);
            self.unrounded.resize_with(count, || Layout::with_order(0));
            self.final_layout.resize_with(count, || Layout::with_order(0));
        }
    }

    /// Clear cache ONLY for nodes with DIRTY_LAYOUT flag set.
    /// This is the key to using Taffy's caching properly with reactive dirty tracking.
    fn clear_dirty_caches(&mut self, _buf: &SharedBuffer, node_count: usize) {
        // Clear ALL caches when layout runs.
        // This is needed because text size changes affect ancestor layouts.
        // TODO: Optimize by walking parent chain for dirty nodes only.
        for i in 0..node_count.min(self.cache.len()) {
            self.cache[i].clear();
        }
    }

    /// Build children lists from parent indices.
    fn rebuild_hierarchy(&mut self, buf: &SharedBuffer, node_count: usize) {
        self.roots.clear();
        for children in self.children.iter_mut().take(node_count) {
            children.clear();
        }

        for i in 0..node_count {
            let comp_type = buf.component_type(i);
            if comp_type == COMPONENT_NONE || !buf.visible(i) {
                continue;
            }

            match buf.parent_index(i) {
                Some(parent) if parent < node_count => {
                    if buf.component_type(parent) != COMPONENT_NONE && buf.visible(parent) {
                        self.children[parent].push(i);
                    } else {
                        self.roots.push(i);
                    }
                }
                _ => self.roots.push(i),
            }
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

impl ExactSizeIterator for ChildIter<'_> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

// =============================================================================
// EMPTY LINE NAMES (for Grid - we don't use named lines)
// =============================================================================

/// Empty line names using Taffy's built-in Map implementation.
/// Generic over lifetime 'a to match trait requirements.
/// Since we use an empty static slice, 'static coerces to any 'a.
pub type EmptyLineNames<'a> = core::iter::Map<
    core::slice::Iter<'a, Vec<Arc<str>>>,
    fn(&Vec<Arc<str>>) -> core::slice::Iter<'_, Arc<str>>,
>;

/// Get empty line names iterator.
/// The 'static slice coerces to 'a due to variance.
fn empty_line_names<'a>() -> EmptyLineNames<'a> {
    static EMPTY: &[Vec<Arc<str>>] = &[];
    EMPTY.iter().map((|v| v.iter()) as fn(&Vec<Arc<str>>) -> core::slice::Iter<'_, Arc<str>>)
}

// =============================================================================
// GRID TRACK ITERATORS
// =============================================================================

/// Never-instantiated repetition type (we only use Single tracks).
#[derive(Clone)]
pub struct NeverRepetition;

impl taffy::style::GenericRepetition for NeverRepetition {
    type CustomIdent = Arc<str>;
    type RepetitionTrackList<'a> = std::iter::Empty<TrackSizingFunction>;
    type TemplateLineNames<'a> = EmptyLineNames<'a>;

    fn count(&self) -> taffy::style::RepetitionCount {
        unreachable!()
    }
    fn tracks(&self) -> Self::RepetitionTrackList<'_> {
        std::iter::empty()
    }
    fn lines_names(&self) -> Self::TemplateLineNames<'_> {
        empty_line_names()
    }
}

/// Iterator over grid template tracks.
#[derive(Clone)]
pub struct TemplateTrackIter {
    tracks: Vec<TrackSizingFunction>,
    index: usize,
}

impl Iterator for TemplateTrackIter {
    type Item = GenericGridTemplateComponent<Arc<str>, NeverRepetition>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.tracks.len() {
            let track = self.tracks[self.index];
            self.index += 1;
            Some(GenericGridTemplateComponent::Single(track))
        } else {
            None
        }
    }
}

impl ExactSizeIterator for TemplateTrackIter {
    fn len(&self) -> usize {
        self.tracks.len() - self.index
    }
}

/// Iterator over auto track sizes.
#[derive(Clone)]
pub struct AutoTrackIter(Option<TrackSizingFunction>);

impl Iterator for AutoTrackIter {
    type Item = TrackSizingFunction;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.take()
    }
}

impl ExactSizeIterator for AutoTrackIter {
    fn len(&self) -> usize {
        if self.0.is_some() { 1 } else { 0 }
    }
}

// =============================================================================
// NODE STYLE (zero-copy wrapper)
// =============================================================================

/// Zero-copy style that reads directly from SharedBuffer.
#[derive(Clone, Copy)]
pub struct NodeStyle<'a> {
    buf: &'a SharedBuffer,
    idx: usize,
}

impl<'a> NodeStyle<'a> {
    #[inline]
    fn new(buf: &'a SharedBuffer, idx: usize) -> Self {
        Self { buf, idx }
    }

    /// f32 → Dimension: NaN=auto, negative=percent, positive=length
    #[inline]
    fn to_dim(val: f32) -> Dimension {
        if val.is_nan() || val == f32::MAX {
            Dimension::auto()
        } else if val < 0.0 {
            Dimension::percent(-val / 100.0)
        } else {
            Dimension::length(val)
        }
    }

    /// f32 → LengthPercentageAuto
    #[inline]
    fn to_lpa(val: f32) -> LengthPercentageAuto {
        if val.is_nan() || val == f32::MAX {
            LengthPercentageAuto::auto()
        } else if val < 0.0 {
            LengthPercentageAuto::percent(-val / 100.0)
        } else {
            LengthPercentageAuto::length(val)
        }
    }

    /// f32 → LengthPercentage (no auto variant)
    #[inline]
    fn to_lp(val: f32) -> LengthPercentage {
        if val < 0.0 {
            LengthPercentage::percent(-val / 100.0)
        } else {
            LengthPercentage::length(val)
        }
    }
}

// -----------------------------------------------------------------------------
// CoreStyle
// -----------------------------------------------------------------------------

impl taffy::CoreStyle for NodeStyle<'_> {
    type CustomIdent = Arc<str>;

    fn box_generation_mode(&self) -> taffy::BoxGenerationMode {
        if self.buf.display(self.idx) == DISPLAY_NONE {
            taffy::BoxGenerationMode::None
        } else {
            taffy::BoxGenerationMode::Normal
        }
    }

    fn is_block(&self) -> bool { false }

    fn box_sizing(&self) -> taffy::BoxSizing {
        if self.buf.box_sizing(self.idx) == 1 {
            taffy::BoxSizing::ContentBox
        } else {
            taffy::BoxSizing::BorderBox
        }
    }

    fn overflow(&self) -> taffy::Point<taffy::Overflow> {
        let o = match self.buf.overflow(self.idx) {
            1 => taffy::Overflow::Clip,
            2 | 3 => taffy::Overflow::Scroll,
            _ => taffy::Overflow::Visible,
        };
        taffy::Point { x: o, y: o }
    }

    fn scrollbar_width(&self) -> f32 { 0.0 }

    fn position(&self) -> taffy::Position {
        if self.buf.position(self.idx) == 1 {
            taffy::Position::Absolute
        } else {
            taffy::Position::Relative
        }
    }

    fn inset(&self) -> taffy::Rect<LengthPercentageAuto> {
        taffy::Rect {
            top: Self::to_lpa(self.buf.inset_top(self.idx)),
            right: Self::to_lpa(self.buf.inset_right(self.idx)),
            bottom: Self::to_lpa(self.buf.inset_bottom(self.idx)),
            left: Self::to_lpa(self.buf.inset_left(self.idx)),
        }
    }

    fn size(&self) -> taffy::Size<Dimension> {
        let ct = self.buf.component_type(self.idx);
        if ct == COMPONENT_TEXT || ct == COMPONENT_INPUT {
            return taffy::Size { width: Dimension::auto(), height: Dimension::auto() };
        }
        taffy::Size {
            width: Self::to_dim(self.buf.width(self.idx)),
            height: Self::to_dim(self.buf.height(self.idx)),
        }
    }

    fn min_size(&self) -> taffy::Size<Dimension> {
        taffy::Size {
            width: Self::to_dim(self.buf.min_width(self.idx)),
            height: Self::to_dim(self.buf.min_height(self.idx)),
        }
    }

    fn max_size(&self) -> taffy::Size<Dimension> {
        taffy::Size {
            width: Self::to_dim(self.buf.max_width(self.idx)),
            height: Self::to_dim(self.buf.max_height(self.idx)),
        }
    }

    fn aspect_ratio(&self) -> Option<f32> {
        let v = self.buf.aspect_ratio(self.idx);
        if v.is_nan() || v <= 0.0 { None } else { Some(v) }
    }

    fn margin(&self) -> taffy::Rect<LengthPercentageAuto> {
        taffy::Rect {
            top: Self::to_lpa(self.buf.margin_top(self.idx)),
            right: Self::to_lpa(self.buf.margin_right(self.idx)),
            bottom: Self::to_lpa(self.buf.margin_bottom(self.idx)),
            left: Self::to_lpa(self.buf.margin_left(self.idx)),
        }
    }

    fn padding(&self) -> taffy::Rect<LengthPercentage> {
        taffy::Rect {
            top: Self::to_lp(self.buf.padding_top(self.idx)),
            right: Self::to_lp(self.buf.padding_right(self.idx)),
            bottom: Self::to_lp(self.buf.padding_bottom(self.idx)),
            left: Self::to_lp(self.buf.padding_left(self.idx)),
        }
    }

    fn border(&self) -> taffy::Rect<LengthPercentage> {
        taffy::Rect {
            top: LengthPercentage::length(if self.buf.border_top(self.idx) > 0 { 1.0 } else { 0.0 }),
            right: LengthPercentage::length(if self.buf.border_right(self.idx) > 0 { 1.0 } else { 0.0 }),
            bottom: LengthPercentage::length(if self.buf.border_bottom(self.idx) > 0 { 1.0 } else { 0.0 }),
            left: LengthPercentage::length(if self.buf.border_left(self.idx) > 0 { 1.0 } else { 0.0 }),
        }
    }
}

// -----------------------------------------------------------------------------
// FlexboxContainerStyle
// -----------------------------------------------------------------------------

impl taffy::FlexboxContainerStyle for NodeStyle<'_> {
    fn flex_direction(&self) -> FlexDirection {
        match self.buf.flex_direction(self.idx) {
            1 => FlexDirection::Column,
            2 => FlexDirection::RowReverse,
            3 => FlexDirection::ColumnReverse,
            _ => FlexDirection::Row,
        }
    }

    fn flex_wrap(&self) -> FlexWrap {
        match self.buf.flex_wrap(self.idx) {
            1 => FlexWrap::Wrap,
            2 => FlexWrap::WrapReverse,
            _ => FlexWrap::NoWrap,
        }
    }

    fn gap(&self) -> taffy::Size<LengthPercentage> {
        let g = self.buf.gap(self.idx);
        let rg = self.buf.row_gap(self.idx);
        let cg = self.buf.column_gap(self.idx);
        taffy::Size {
            width: LengthPercentage::length(if cg != 0.0 { cg } else { g }),
            height: LengthPercentage::length(if rg != 0.0 { rg } else { g }),
        }
    }

    fn align_content(&self) -> Option<AlignContent> {
        match self.buf.align_content(self.idx) {
            0 => Some(AlignContent::FlexStart),
            1 => Some(AlignContent::FlexEnd),
            2 => Some(AlignContent::Center),
            3 => Some(AlignContent::Stretch),
            4 => Some(AlignContent::SpaceBetween),
            5 => Some(AlignContent::SpaceAround),
            6 => Some(AlignContent::SpaceEvenly),
            _ => None,
        }
    }

    fn align_items(&self) -> Option<AlignItems> {
        match self.buf.align_items(self.idx) {
            0 => Some(AlignItems::FlexStart),
            1 => Some(AlignItems::FlexEnd),
            2 => Some(AlignItems::Center),
            3 => Some(AlignItems::Stretch),
            4 => Some(AlignItems::Baseline),
            _ => None,
        }
    }

    fn justify_content(&self) -> Option<JustifyContent> {
        match self.buf.justify_content(self.idx) {
            0 => Some(JustifyContent::FlexStart),
            1 => Some(JustifyContent::FlexEnd),
            2 => Some(JustifyContent::Center),
            3 => Some(JustifyContent::SpaceBetween),
            4 => Some(JustifyContent::SpaceAround),
            5 => Some(JustifyContent::SpaceEvenly),
            _ => None,
        }
    }
}

// -----------------------------------------------------------------------------
// FlexboxItemStyle
// -----------------------------------------------------------------------------

impl taffy::FlexboxItemStyle for NodeStyle<'_> {
    fn flex_basis(&self) -> Dimension {
        Self::to_dim(self.buf.flex_basis(self.idx))
    }

    fn flex_grow(&self) -> f32 {
        self.buf.flex_grow(self.idx)
    }

    fn flex_shrink(&self) -> f32 {
        self.buf.flex_shrink(self.idx)
    }

    fn align_self(&self) -> Option<AlignSelf> {
        match self.buf.align_self(self.idx) {
            1 => Some(AlignSelf::FlexStart),
            2 => Some(AlignSelf::FlexEnd),
            3 => Some(AlignSelf::Center),
            4 => Some(AlignSelf::Stretch),
            5 => Some(AlignSelf::Baseline),
            _ => None,
        }
    }
}

// -----------------------------------------------------------------------------
// GridContainerStyle
// -----------------------------------------------------------------------------

impl taffy::GridContainerStyle for NodeStyle<'_> {
    type Repetition<'a> = NeverRepetition where Self: 'a;
    type TemplateTrackList<'a> = TemplateTrackIter where Self: 'a;
    type AutoTrackList<'a> = AutoTrackIter where Self: 'a;
    type TemplateLineNames<'a> = EmptyLineNames<'a> where Self: 'a;
    type GridTemplateAreas<'a> = std::iter::Empty<taffy::style::GridTemplateArea<Arc<str>>> where Self: 'a;

    fn grid_template_rows(&self) -> Option<Self::TemplateTrackList<'_>> {
        let tracks: Vec<_> = self.buf.grid_row_tracks(self.idx)
            .into_iter()
            .map(|t| track_to_taffy(t.track_type, t.value))
            .collect();
        if tracks.is_empty() { None } else { Some(TemplateTrackIter { tracks, index: 0 }) }
    }

    fn grid_template_columns(&self) -> Option<Self::TemplateTrackList<'_>> {
        let tracks: Vec<_> = self.buf.grid_column_tracks(self.idx)
            .into_iter()
            .map(|t| track_to_taffy(t.track_type, t.value))
            .collect();
        if tracks.is_empty() { None } else { Some(TemplateTrackIter { tracks, index: 0 }) }
    }

    fn grid_auto_rows(&self) -> Self::AutoTrackList<'_> {
        let tt = self.buf.grid_auto_rows_type(self.idx);
        let v = self.buf.grid_auto_rows_value(self.idx);
        AutoTrackIter(Some(track_to_taffy(tt, v)))
    }

    fn grid_auto_columns(&self) -> Self::AutoTrackList<'_> {
        let tt = self.buf.grid_auto_columns_type(self.idx);
        let v = self.buf.grid_auto_columns_value(self.idx);
        AutoTrackIter(Some(track_to_taffy(tt, v)))
    }

    fn grid_template_areas(&self) -> Option<Self::GridTemplateAreas<'_>> { None }
    fn grid_template_column_names(&self) -> Option<Self::TemplateLineNames<'_>> { None }
    fn grid_template_row_names(&self) -> Option<Self::TemplateLineNames<'_>> { None }

    fn grid_auto_flow(&self) -> GridAutoFlow {
        match self.buf.grid_auto_flow(self.idx) {
            crate::shared_buffer::GridAutoFlow::Column => GridAutoFlow::Column,
            crate::shared_buffer::GridAutoFlow::RowDense => GridAutoFlow::RowDense,
            crate::shared_buffer::GridAutoFlow::ColumnDense => GridAutoFlow::ColumnDense,
            _ => GridAutoFlow::Row,
        }
    }

    fn gap(&self) -> taffy::Size<LengthPercentage> {
        <Self as taffy::FlexboxContainerStyle>::gap(self)
    }

    fn align_content(&self) -> Option<AlignContent> {
        <Self as taffy::FlexboxContainerStyle>::align_content(self)
    }

    fn justify_content(&self) -> Option<JustifyContent> {
        <Self as taffy::FlexboxContainerStyle>::justify_content(self)
    }

    fn align_items(&self) -> Option<AlignItems> {
        <Self as taffy::FlexboxContainerStyle>::align_items(self)
    }

    fn justify_items(&self) -> Option<AlignItems> {
        match self.buf.justify_items(self.idx) {
            crate::shared_buffer::JustifyItems::End => Some(AlignItems::End),
            crate::shared_buffer::JustifyItems::Center => Some(AlignItems::Center),
            crate::shared_buffer::JustifyItems::Stretch => Some(AlignItems::Stretch),
            _ => Some(AlignItems::Start),
        }
    }
}

// -----------------------------------------------------------------------------
// GridItemStyle
// -----------------------------------------------------------------------------

impl taffy::GridItemStyle for NodeStyle<'_> {
    fn grid_row(&self) -> taffy::Line<GridPlacement<Arc<str>>> {
        taffy::Line {
            start: to_placement(self.buf.grid_row_start(self.idx)),
            end: to_placement(self.buf.grid_row_end(self.idx)),
        }
    }

    fn grid_column(&self) -> taffy::Line<GridPlacement<Arc<str>>> {
        taffy::Line {
            start: to_placement(self.buf.grid_column_start(self.idx)),
            end: to_placement(self.buf.grid_column_end(self.idx)),
        }
    }

    fn align_self(&self) -> Option<AlignSelf> {
        <Self as taffy::FlexboxItemStyle>::align_self(self)
    }

    fn justify_self(&self) -> Option<AlignSelf> {
        match self.buf.justify_self(self.idx) {
            crate::shared_buffer::JustifySelf::Start => Some(AlignSelf::Start),
            crate::shared_buffer::JustifySelf::End => Some(AlignSelf::End),
            crate::shared_buffer::JustifySelf::Center => Some(AlignSelf::Center),
            crate::shared_buffer::JustifySelf::Stretch => Some(AlignSelf::Stretch),
            _ => None,
        }
    }
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

#[inline]
fn to_placement(val: i16) -> GridPlacement<Arc<str>> {
    if val == 0 {
        GridPlacement::Auto
    } else if val < 0 {
        GridPlacement::from_span((-val) as u16)
    } else {
        GridPlacement::from_line_index(val)
    }
}

#[inline]
fn track_to_taffy(tt: crate::shared_buffer::TrackType, value: f32) -> TrackSizingFunction {
    use crate::shared_buffer::TrackType;
    match tt {
        TrackType::Auto | TrackType::None => TrackSizingFunction::AUTO,
        TrackType::MinContent => TrackSizingFunction::MIN_CONTENT,
        TrackType::MaxContent => TrackSizingFunction::MAX_CONTENT,
        TrackType::Length => TrackSizingFunction::from_length(value),
        TrackType::Percent => TrackSizingFunction::from_percent(value),
        TrackType::Fr => TrackSizingFunction::from_fr(value),
        TrackType::FitContent => TrackSizingFunction::fit_content(LengthPercentage::length(value)),
    }
}

// =============================================================================
// LAYOUT TREE
// =============================================================================

pub struct LayoutTree<'a> {
    pub buf: &'a SharedBuffer,
    pub ctx: &'a mut LayoutContext,
}

impl<'a> LayoutTree<'a> {
    fn write_output(&self, node_count: usize) {
        for i in 0..node_count {
            if self.buf.component_type(i) == COMPONENT_NONE {
                continue;
            }

            let layout = self.ctx.final_layout[i];
            self.buf.set_computed_x(i, layout.location.x);
            self.buf.set_computed_y(i, layout.location.y);
            self.buf.set_computed_width(i, layout.size.width);
            self.buf.set_computed_height(i, layout.size.height);
            self.buf.set_content_width(i, layout.content_size.width);
            self.buf.set_content_height(i, layout.content_size.height);

            let has_children = !self.ctx.children[i].is_empty();
            let overflow = self.buf.overflow(i);
            let max_scroll_x = (layout.content_size.width - layout.size.width).max(0.0);
            let max_scroll_y = (layout.content_size.height - layout.size.height).max(0.0);
            let scrollable = match overflow {
                1 => false,                    // clip
                2 | 3 => true,                 // scroll/auto
                _ => has_children && (max_scroll_x > 0.0 || max_scroll_y > 0.0),
            };

            self.buf.set_output_scroll(i, scrollable, max_scroll_x, max_scroll_y);
            self.buf.clear_dirty(i);
        }
    }
}

// -----------------------------------------------------------------------------
// TraversePartialTree + TraverseTree
// -----------------------------------------------------------------------------

impl TraversePartialTree for LayoutTree<'_> {
    type ChildIter<'a> = ChildIter<'a> where Self: 'a;

    fn child_ids(&self, parent: NodeId) -> Self::ChildIter<'_> {
        let idx = usize::from(parent);
        let children = if idx < self.ctx.children.len() {
            &self.ctx.children[idx]
        } else {
            &[] as &[usize]
        };
        ChildIter { inner: children.iter() }
    }

    fn child_count(&self, parent: NodeId) -> usize {
        let idx = usize::from(parent);
        if idx < self.ctx.children.len() { self.ctx.children[idx].len() } else { 0 }
    }

    fn get_child_id(&self, parent: NodeId, child_index: usize) -> NodeId {
        NodeId::from(self.ctx.children[usize::from(parent)][child_index])
    }
}

impl TraverseTree for LayoutTree<'_> {}

// -----------------------------------------------------------------------------
// PrintTree (debug)
// -----------------------------------------------------------------------------

impl PrintTree for LayoutTree<'_> {
    fn get_debug_label(&self, node: NodeId) -> &'static str {
        let idx = usize::from(node);
        match self.buf.component_type(idx) {
            COMPONENT_BOX => if self.buf.display(idx) == DISPLAY_GRID { "Grid" } else { "Flex" },
            COMPONENT_TEXT => "Text",
            COMPONENT_INPUT => "Input",
            _ => "?",
        }
    }

    fn get_final_layout(&self, node: NodeId) -> Layout {
        self.ctx.final_layout[usize::from(node)]
    }
}

// -----------------------------------------------------------------------------
// LayoutPartialTree (core layout dispatch)
// -----------------------------------------------------------------------------

impl LayoutPartialTree for LayoutTree<'_> {
    type CoreContainerStyle<'a> = NodeStyle<'a> where Self: 'a;
    type CustomIdent = Arc<str>;

    fn get_core_container_style(&self, node: NodeId) -> Self::CoreContainerStyle<'_> {
        NodeStyle::new(self.buf, usize::from(node))
    }

    fn set_unrounded_layout(&mut self, node: NodeId, layout: &Layout) {
        let idx = usize::from(node);
        if idx < self.ctx.unrounded.len() {
            self.ctx.unrounded[idx] = *layout;
        }
    }

    fn compute_child_layout(&mut self, node: NodeId, inputs: LayoutInput) -> LayoutOutput {
        compute_cached_layout(self, node, inputs, |tree, node, inputs| {
            let idx = usize::from(node);
            let comp = tree.buf.component_type(idx);

            if comp == COMPONENT_NONE || !tree.buf.visible(idx) {
                return compute_hidden_layout(tree, node);
            }

            match comp {
                COMPONENT_BOX => match tree.buf.display(idx) {
                    DISPLAY_NONE => compute_hidden_layout(tree, node),
                    DISPLAY_FLEX => compute_flexbox_layout(tree, node, inputs),
                    DISPLAY_GRID => compute_grid_layout(tree, node, inputs),
                    _ => compute_hidden_layout(tree, node), // Unknown = hidden (fail visible)
                },
                COMPONENT_TEXT | COMPONENT_INPUT => {
                    let style = NodeStyle::new(tree.buf, idx);
                    let text = tree.buf.text(idx);

                    compute_leaf_layout(
                        inputs,
                        &style,
                        |_, _| 0.0, // resolve_calc_value (no-op, we don't use calc())
                        |known, available| {
                            if text.is_empty() {
                                return taffy::Size::ZERO;
                            }
                            let max_w = match known.width {
                                Some(w) => w as usize,
                                None => match available.width {
                                    AvailableSpace::Definite(w) => w as usize,
                                    AvailableSpace::MinContent => 1,
                                    AvailableSpace::MaxContent => usize::MAX,
                                },
                            };
                            taffy::Size {
                                width: string_width(text) as f32,
                                height: measure_text_height(text, max_w) as f32,
                            }
                        },
                    )
                }
                _ => compute_hidden_layout(tree, node),
            }
        })
    }
}

// -----------------------------------------------------------------------------
// CacheTree
// -----------------------------------------------------------------------------

impl CacheTree for LayoutTree<'_> {
    fn cache_get(
        &self,
        node: NodeId,
        kd: taffy::Size<Option<f32>>,
        avail: taffy::Size<AvailableSpace>,
        mode: taffy::RunMode,
    ) -> Option<LayoutOutput> {
        let idx = usize::from(node);
        if idx < self.ctx.cache.len() {
            self.ctx.cache[idx].get(kd, avail, mode)
        } else {
            None
        }
    }

    fn cache_store(
        &mut self,
        node: NodeId,
        kd: taffy::Size<Option<f32>>,
        avail: taffy::Size<AvailableSpace>,
        mode: taffy::RunMode,
        output: LayoutOutput,
    ) {
        let idx = usize::from(node);
        if idx < self.ctx.cache.len() {
            self.ctx.cache[idx].store(kd, avail, mode, output);
        }
    }

    fn cache_clear(&mut self, node: NodeId) {
        let idx = usize::from(node);
        if idx < self.ctx.cache.len() {
            self.ctx.cache[idx].clear();
        }
    }
}

// -----------------------------------------------------------------------------
// LayoutFlexboxContainer
// -----------------------------------------------------------------------------

impl LayoutFlexboxContainer for LayoutTree<'_> {
    type FlexboxContainerStyle<'a> = NodeStyle<'a> where Self: 'a;
    type FlexboxItemStyle<'a> = NodeStyle<'a> where Self: 'a;

    fn get_flexbox_container_style(&self, node: NodeId) -> Self::FlexboxContainerStyle<'_> {
        NodeStyle::new(self.buf, usize::from(node))
    }

    fn get_flexbox_child_style(&self, child: NodeId) -> Self::FlexboxItemStyle<'_> {
        NodeStyle::new(self.buf, usize::from(child))
    }
}

// -----------------------------------------------------------------------------
// LayoutGridContainer
// -----------------------------------------------------------------------------

impl LayoutGridContainer for LayoutTree<'_> {
    type GridContainerStyle<'a> = NodeStyle<'a> where Self: 'a;
    type GridItemStyle<'a> = NodeStyle<'a> where Self: 'a;

    fn get_grid_container_style(&self, node: NodeId) -> Self::GridContainerStyle<'_> {
        NodeStyle::new(self.buf, usize::from(node))
    }

    fn get_grid_child_style(&self, child: NodeId) -> Self::GridItemStyle<'_> {
        NodeStyle::new(self.buf, usize::from(child))
    }
}

// -----------------------------------------------------------------------------
// RoundTree
// -----------------------------------------------------------------------------

impl RoundTree for LayoutTree<'_> {
    fn get_unrounded_layout(&self, node: NodeId) -> Layout {
        self.ctx.unrounded[usize::from(node)]
    }

    fn set_final_layout(&mut self, node: NodeId, layout: &Layout) {
        let idx = usize::from(node);
        if idx < self.ctx.final_layout.len() {
            self.ctx.final_layout[idx] = *layout;
        }
    }
}

// =============================================================================
// PUBLIC API
// =============================================================================

/// Compute layout for all nodes in the SharedBuffer.
///
/// Returns the number of nodes processed.
pub fn compute_layout(buf: &SharedBuffer) -> u32 {
    let node_count = buf.node_count();

    LAYOUT_CONTEXT.with(|cell| {
        let mut ctx = cell.borrow_mut();
        ctx.ensure_capacity(node_count);
        ctx.clear_dirty_caches(buf, node_count);
        ctx.rebuild_hierarchy(buf, node_count);

        let mut tree = LayoutTree { buf, ctx: &mut *ctx };

        if tree.ctx.roots.is_empty() {
            return 0;
        }

        // Available space depends on render mode:
        // - Fullscreen: use terminal dimensions
        // - Inline/Append: width from terminal, height unbounded (content determines)
        let render_mode = buf.render_mode();
        let available = taffy::Size {
            width: AvailableSpace::Definite(buf.terminal_width() as f32),
            height: match render_mode {
                RenderMode::Diff => AvailableSpace::Definite(buf.terminal_height() as f32),
                RenderMode::Inline | RenderMode::Append => AvailableSpace::MaxContent,
            },
        };

        let roots = tree.ctx.roots.clone();
        for &root in &roots {
            compute_root_layout(&mut tree, NodeId::from(root), available);
        }
        for &root in &roots {
            round_layout(&mut tree, NodeId::from(root));
        }

        tree.write_output(node_count);
        buf.increment_layout_count();

        node_count as u32
    })
}

#[cfg(test)]
mod tests {
    // Tests go here
}
