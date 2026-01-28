//! Taffy Bridge - Integration with Taffy layout engine
//!
//! Converts FlexNode properties to Taffy styles, runs layout computation,
//! and extracts results back to ComputedLayout.
//!
//! This replaces the manual TITAN layout engine with Taffy's W3C-compliant
//! flexbox implementation.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use taffy::{
    AvailableSpace, Dimension as TaffyDimension, Display, FlexDirection as TaffyFlexDirection,
    FlexWrap as TaffyFlexWrap, JustifyContent as TaffyJustifyContent,
    AlignItems as TaffyAlignItems, AlignContent as TaffyAlignContent,
    AlignSelf as TaffyAlignSelf, LengthPercentage, LengthPercentageAuto, NodeId,
    Overflow as TaffyOverflow, Position as TaffyPosition, Rect, Size, Style, TaffyTree,
};

use crate::engine::arrays::core;
use crate::engine::arrays::text;
use crate::engine::{get_allocated_indices, get_flex_node};
use crate::types::{
    AlignContent, AlignItems, AlignSelf, ComponentType, Dimension, FlexDirection,
    FlexWrap, JustifyContent, Overflow,
};

use super::text_measure::{measure_text_height, string_width};
use super::types::ComputedLayout;

// =============================================================================
// TAFFY CACHE
// =============================================================================
//
// Caching strategy:
// 1. Pre-allocate HashMap with expected capacity to avoid per-layout allocation
// 2. Track expected capacity based on historical component counts
// 3. Dirty flag to invalidate when component structure changes
// 4. When clean: reuse cached tree with style updates (fast path)
// 5. When dirty: rebuild tree from scratch

thread_local! {
    /// Cached TaffyTree - reused across layout computations when structure is stable.
    static TAFFY_TREE_CACHE: RefCell<Option<TaffyTree<usize>>> = const { RefCell::new(None) };

    /// Cached mapping from component index to Taffy NodeId.
    /// Valid only when TAFFY_CACHE_DIRTY is false.
    static INDEX_TO_NODE_CACHE: RefCell<HashMap<usize, NodeId>> = RefCell::new(HashMap::new());

    /// Cached roots list.
    static ROOTS_CACHE: RefCell<Vec<usize>> = RefCell::new(Vec::new());

    /// Dirty flag - set when component structure changes.
    /// When true, the cache must be rebuilt.
    static TAFFY_CACHE_DIRTY: Cell<bool> = const { Cell::new(true) };

    /// Expected capacity hint for HashMap pre-allocation.
    /// Updated based on actual component counts.
    static EXPECTED_CAPACITY: Cell<usize> = const { Cell::new(64) };
}

/// Invalidate the Taffy cache.
///
/// Call this when the component tree structure changes:
/// - Component added (allocate_index)
/// - Component removed (release_index)
/// - Parent-child relationship changed
/// - Visibility changed
///
/// This forces the next layout computation to rebuild the Taffy tree.
pub fn invalidate_taffy_cache() {
    TAFFY_CACHE_DIRTY.with(|d| d.set(true));
}

/// Check if the Taffy cache is dirty.
pub fn is_taffy_cache_dirty() -> bool {
    TAFFY_CACHE_DIRTY.with(|d| d.get())
}

/// Reset the Taffy cache completely.
/// Used by reset_registry to ensure clean state.
pub fn reset_taffy_cache() {
    TAFFY_TREE_CACHE.with(|cache| {
        *cache.borrow_mut() = None;
    });
    INDEX_TO_NODE_CACHE.with(|cache| {
        cache.borrow_mut().clear();
    });
    ROOTS_CACHE.with(|cache| {
        cache.borrow_mut().clear();
    });
    TAFFY_CACHE_DIRTY.with(|d| d.set(true));
    EXPECTED_CAPACITY.with(|c| c.set(64));
}

// =============================================================================
// DIMENSION CONVERSION
// =============================================================================

/// Convert our Dimension to Taffy's Dimension.
fn to_taffy_dimension(dim: Dimension) -> TaffyDimension {
    match dim {
        Dimension::Auto => TaffyDimension::Auto,
        Dimension::Cells(n) => TaffyDimension::Length(n as f32),
        Dimension::Percent(p) => TaffyDimension::Percent(p / 100.0),
    }
}


// =============================================================================
// ENUM CONVERSIONS
// =============================================================================

fn to_taffy_flex_direction(dir: FlexDirection) -> TaffyFlexDirection {
    match dir {
        FlexDirection::Column => TaffyFlexDirection::Column,
        FlexDirection::Row => TaffyFlexDirection::Row,
        FlexDirection::ColumnReverse => TaffyFlexDirection::ColumnReverse,
        FlexDirection::RowReverse => TaffyFlexDirection::RowReverse,
    }
}

fn to_taffy_flex_wrap(wrap: FlexWrap) -> TaffyFlexWrap {
    match wrap {
        FlexWrap::NoWrap => TaffyFlexWrap::NoWrap,
        FlexWrap::Wrap => TaffyFlexWrap::Wrap,
        FlexWrap::WrapReverse => TaffyFlexWrap::WrapReverse,
    }
}

fn to_taffy_justify_content(justify: JustifyContent) -> Option<TaffyJustifyContent> {
    Some(match justify {
        JustifyContent::FlexStart => TaffyJustifyContent::FlexStart,
        JustifyContent::Center => TaffyJustifyContent::Center,
        JustifyContent::FlexEnd => TaffyJustifyContent::FlexEnd,
        JustifyContent::SpaceBetween => TaffyJustifyContent::SpaceBetween,
        JustifyContent::SpaceAround => TaffyJustifyContent::SpaceAround,
        JustifyContent::SpaceEvenly => TaffyJustifyContent::SpaceEvenly,
    })
}

fn to_taffy_align_items(align: AlignItems) -> Option<TaffyAlignItems> {
    Some(match align {
        AlignItems::Stretch => TaffyAlignItems::Stretch,
        AlignItems::FlexStart => TaffyAlignItems::FlexStart,
        AlignItems::Center => TaffyAlignItems::Center,
        AlignItems::FlexEnd => TaffyAlignItems::FlexEnd,
        AlignItems::Baseline => TaffyAlignItems::Baseline,
    })
}

fn to_taffy_align_content(align: AlignContent) -> Option<TaffyAlignContent> {
    Some(match align {
        AlignContent::Stretch => TaffyAlignContent::Stretch,
        AlignContent::FlexStart => TaffyAlignContent::FlexStart,
        AlignContent::Center => TaffyAlignContent::Center,
        AlignContent::FlexEnd => TaffyAlignContent::FlexEnd,
        AlignContent::SpaceBetween => TaffyAlignContent::SpaceBetween,
        AlignContent::SpaceAround => TaffyAlignContent::SpaceAround,
    })
}

fn to_taffy_align_self(align: AlignSelf) -> Option<TaffyAlignSelf> {
    match align {
        AlignSelf::Auto => None, // inherit from parent
        AlignSelf::Stretch => Some(TaffyAlignSelf::Stretch),
        AlignSelf::FlexStart => Some(TaffyAlignSelf::FlexStart),
        AlignSelf::Center => Some(TaffyAlignSelf::Center),
        AlignSelf::FlexEnd => Some(TaffyAlignSelf::FlexEnd),
        AlignSelf::Baseline => Some(TaffyAlignSelf::Baseline),
    }
}

fn to_taffy_overflow(overflow: Overflow) -> TaffyOverflow {
    match overflow {
        Overflow::Visible => TaffyOverflow::Visible,
        Overflow::Hidden => TaffyOverflow::Clip,
        Overflow::Scroll => TaffyOverflow::Scroll,
        Overflow::Auto => TaffyOverflow::Scroll, // Auto acts like scroll when content overflows
    }
}

fn to_taffy_position(pos_val: u8) -> TaffyPosition {
    match pos_val {
        1 => TaffyPosition::Absolute,
        _ => TaffyPosition::Relative,
    }
}

// =============================================================================
// STYLE BUILDING
// =============================================================================

/// Build a Taffy Style from a FlexNode.
fn build_style(idx: usize) -> Style {
    let Some(node) = get_flex_node(idx) else {
        return Style::default();
    };

    let comp_type = core::get_component_type(idx);

    // Base style from FlexNode properties
    // Note: TrackedSlot.get() returns Option<T>, so we use unwrap_or() for defaults
    let mut style = Style {
        display: Display::Flex,
        position: to_taffy_position(node.position.get().unwrap_or(0)),

        // Flex container properties
        flex_direction: to_taffy_flex_direction(FlexDirection::from(node.flex_direction.get().unwrap_or(0))),
        flex_wrap: to_taffy_flex_wrap(FlexWrap::from(node.flex_wrap.get().unwrap_or(0))),
        justify_content: to_taffy_justify_content(JustifyContent::from(node.justify_content.get().unwrap_or(0))),
        align_items: to_taffy_align_items(AlignItems::from(node.align_items.get().unwrap_or(0))),
        align_content: to_taffy_align_content(AlignContent::from(node.align_content.get().unwrap_or(0))),

        // Flex item properties
        flex_grow: node.flex_grow.get().unwrap_or(0.0),
        flex_shrink: node.flex_shrink.get().unwrap_or(1.0),
        flex_basis: to_taffy_dimension(node.flex_basis.get().unwrap_or(Dimension::Auto)),
        align_self: to_taffy_align_self(AlignSelf::from(node.align_self.get().unwrap_or(0))),

        // Dimensions
        size: Size {
            width: to_taffy_dimension(node.width.get().unwrap_or(Dimension::Auto)),
            height: to_taffy_dimension(node.height.get().unwrap_or(Dimension::Auto)),
        },
        min_size: Size {
            width: to_taffy_dimension(node.min_width.get().unwrap_or(Dimension::Auto)),
            height: to_taffy_dimension(node.min_height.get().unwrap_or(Dimension::Auto)),
        },
        max_size: Size {
            width: to_taffy_dimension(node.max_width.get().unwrap_or(Dimension::Auto)),
            height: to_taffy_dimension(node.max_height.get().unwrap_or(Dimension::Auto)),
        },

        // Margins
        margin: Rect {
            top: LengthPercentageAuto::Length(node.margin_top.get().unwrap_or(0) as f32),
            right: LengthPercentageAuto::Length(node.margin_right.get().unwrap_or(0) as f32),
            bottom: LengthPercentageAuto::Length(node.margin_bottom.get().unwrap_or(0) as f32),
            left: LengthPercentageAuto::Length(node.margin_left.get().unwrap_or(0) as f32),
        },

        // Padding (uses LengthPercentage, not LengthPercentageAuto)
        padding: Rect {
            top: LengthPercentage::Length(node.padding_top.get().unwrap_or(0) as f32),
            right: LengthPercentage::Length(node.padding_right.get().unwrap_or(0) as f32),
            bottom: LengthPercentage::Length(node.padding_bottom.get().unwrap_or(0) as f32),
            left: LengthPercentage::Length(node.padding_left.get().unwrap_or(0) as f32),
        },

        // Border (uses LengthPercentage, just the width not style/color)
        border: Rect {
            top: LengthPercentage::Length(if node.border_top.get().unwrap_or(0) > 0 { 1.0 } else { 0.0 }),
            right: LengthPercentage::Length(if node.border_right.get().unwrap_or(0) > 0 { 1.0 } else { 0.0 }),
            bottom: LengthPercentage::Length(if node.border_bottom.get().unwrap_or(0) > 0 { 1.0 } else { 0.0 }),
            left: LengthPercentage::Length(if node.border_left.get().unwrap_or(0) > 0 { 1.0 } else { 0.0 }),
        },

        // Gap (uses LengthPercentage)
        gap: Size {
            width: LengthPercentage::Length(node.column_gap.get().unwrap_or(0).max(node.gap.get().unwrap_or(0)) as f32),
            height: LengthPercentage::Length(node.row_gap.get().unwrap_or(0).max(node.gap.get().unwrap_or(0)) as f32),
        },

        // Overflow
        overflow: taffy::Point {
            x: to_taffy_overflow(Overflow::from(node.overflow.get().unwrap_or(0))),
            y: to_taffy_overflow(Overflow::from(node.overflow.get().unwrap_or(0))),
        },

        ..Default::default()
    };

    // Text nodes have special handling - they use intrinsic sizing
    if comp_type == ComponentType::Text {
        // Text uses measure function, don't set explicit size
        style.size = Size::auto();
    }

    style
}

// =============================================================================
// TEXT MEASUREMENT
// =============================================================================

/// Measure function for text content.
fn measure_text(
    idx: usize,
    known_dimensions: Size<Option<f32>>,
    available_space: Size<AvailableSpace>,
) -> Size<f32> {
    let comp_type = core::get_component_type(idx);

    if comp_type == ComponentType::Text {
        let content = text::get_text_content(idx);
        if content.is_empty() {
            return Size::ZERO;
        }

        // Get available width for wrapping
        let avail_width = match available_space.width {
            AvailableSpace::Definite(w) => w as u16,
            AvailableSpace::MinContent => string_width(&content),
            AvailableSpace::MaxContent => u16::MAX,
        };

        let text_width = string_width(&content);
        let text_height = measure_text_height(&content, avail_width.max(1));

        Size {
            width: known_dimensions.width.unwrap_or(text_width as f32),
            height: known_dimensions.height.unwrap_or(text_height as f32),
        }
    } else if comp_type == ComponentType::Input {
        // Input is single-line, measure content width
        let content = text::get_text_content(idx);
        let text_width = string_width(&content).max(1);

        Size {
            width: known_dimensions.width.unwrap_or(text_width as f32),
            height: known_dimensions.height.unwrap_or(1.0),
        }
    } else {
        Size::ZERO
    }
}

// =============================================================================
// MAIN ENTRY POINT
// =============================================================================

/// Compute layout for all allocated components using Taffy.
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
pub fn compute_layout_taffy(
    terminal_width: u16,
    terminal_height: u16,
    constrain_height: bool,
) -> ComputedLayout {
    let mut indices = get_allocated_indices();

    if indices.is_empty() {
        return ComputedLayout::new();
    }

    // Sort indices for consistent child ordering in flex layout
    indices.sort_unstable();

    // Find max index for array sizing
    let max_index = indices.iter().max().copied().unwrap_or(0);
    let array_size = max_index + 1;

    // Initialize output
    let mut result = ComputedLayout {
        x: vec![0; array_size],
        y: vec![0; array_size],
        width: vec![0; array_size],
        height: vec![0; array_size],
        scrollable: vec![0; array_size],
        max_scroll_x: vec![0; array_size],
        max_scroll_y: vec![0; array_size],
        content_width: 0,
        content_height: 0,
    };

    // Check cache state and get capacity hint
    let cache_dirty = TAFFY_CACHE_DIRTY.with(|d| d.get());
    let expected_capacity = EXPECTED_CAPACITY.with(|c| c.get());

    // Update expected capacity based on actual usage
    let visible_count = indices.iter().filter(|&&idx| core::get_visible(idx)).count();
    if visible_count > expected_capacity {
        EXPECTED_CAPACITY.with(|c| c.set(visible_count));
    }

    // Determine if we can reuse the cached tree
    let can_reuse_cache = !cache_dirty && TAFFY_TREE_CACHE.with(|cache| cache.borrow().is_some());

    let (mut tree, index_to_node, roots) = if can_reuse_cache {
        // Fast path: reuse cached tree with style updates
        reuse_cached_tree(&indices)
    } else {
        // Slow path: rebuild tree from scratch
        build_taffy_tree(&indices, expected_capacity.max(visible_count))
    };

    // Compute layout for each root with measure function
    let available = Size {
        width: AvailableSpace::Definite(terminal_width as f32),
        height: if constrain_height {
            AvailableSpace::Definite(terminal_height as f32)
        } else {
            AvailableSpace::MaxContent
        },
    };

    // Define measure function for text measurement
    let mut measure_fn = |known_dimensions: Size<Option<f32>>,
                          available_space: Size<AvailableSpace>,
                          _node_id: NodeId,
                          context: Option<&mut usize>,
                          _style: &Style| {
        if let Some(&mut idx) = context {
            measure_text(idx, known_dimensions, available_space)
        } else {
            Size::ZERO
        }
    };

    for &root_idx in &roots {
        if let Some(&root_node) = index_to_node.get(&root_idx) {
            let _ = tree.compute_layout_with_measure(root_node, available, &mut measure_fn);
        }
    }

    // Extract results
    for &idx in &indices {
        if let Some(&node_id) = index_to_node.get(&idx) {
            if let Ok(layout) = tree.layout(node_id) {
                result.x[idx] = layout.location.x.round() as u16;
                result.y[idx] = layout.location.y.round() as u16;
                result.width[idx] = layout.size.width.round() as u16;
                result.height[idx] = layout.size.height.round() as u16;

                // Check scrollability
                if let Some(node) = get_flex_node(idx) {
                    let overflow = Overflow::from(node.overflow.get().unwrap_or(0));
                    if matches!(overflow, Overflow::Scroll | Overflow::Auto) {
                        result.scrollable[idx] = 1;
                        result.max_scroll_x[idx] = (layout.content_size.width.round() as u16)
                            .saturating_sub(layout.size.width.round() as u16);
                        result.max_scroll_y[idx] = (layout.content_size.height.round() as u16)
                            .saturating_sub(layout.size.height.round() as u16);
                    }
                }
            }
        }
    }

    // Store tree and mapping in cache for potential reuse
    TAFFY_TREE_CACHE.with(|cache| {
        *cache.borrow_mut() = Some(tree);
    });
    INDEX_TO_NODE_CACHE.with(|cache| {
        *cache.borrow_mut() = index_to_node;
    });
    ROOTS_CACHE.with(|cache| {
        *cache.borrow_mut() = roots.clone();
    });
    TAFFY_CACHE_DIRTY.with(|d| d.set(false));

    // Calculate content bounds from first root
    if let Some(&first_root) = roots.first() {
        result.content_width = result.width.get(first_root).copied().unwrap_or(0);
        result.content_height = result.height.get(first_root).copied().unwrap_or(0);
    }

    result
}

/// Build a new Taffy tree from scratch.
///
/// This is the slow path, used when the cache is dirty or empty.
fn build_taffy_tree(
    indices: &[usize],
    capacity_hint: usize,
) -> (TaffyTree<usize>, HashMap<usize, NodeId>, Vec<usize>) {
    let mut tree: TaffyTree<usize> = TaffyTree::new();

    // Pre-allocate HashMap with expected capacity to avoid reallocation
    let mut index_to_node: HashMap<usize, NodeId> = HashMap::with_capacity(capacity_hint);

    // First pass: Create all nodes (without children)
    for &idx in indices {
        if !core::get_visible(idx) {
            continue;
        }

        let style = build_style(idx);
        let comp_type = core::get_component_type(idx);

        // Create node with measure function for text/input
        let node_id = if comp_type == ComponentType::Text || comp_type == ComponentType::Input {
            tree.new_leaf_with_context(style, idx).unwrap()
        } else {
            tree.new_leaf(style).unwrap()
        };

        index_to_node.insert(idx, node_id);
    }

    // Second pass: Build parent-child relationships
    let mut roots: Vec<usize> = Vec::new();

    for &idx in indices {
        if !core::get_visible(idx) {
            continue;
        }

        let parent = core::get_parent_index(idx);

        if let Some(parent_idx) = parent {
            if let (Some(&parent_node), Some(&child_node)) =
                (index_to_node.get(&parent_idx), index_to_node.get(&idx))
            {
                // Add as child
                let _ = tree.add_child(parent_node, child_node);
            } else {
                // Parent not in tree - this is a root
                roots.push(idx);
            }
        } else {
            // No parent - this is a root
            roots.push(idx);
        }
    }

    (tree, index_to_node, roots)
}

/// Reuse cached Taffy tree with updated styles.
///
/// This is the fast path, used when the tree structure hasn't changed.
/// Only style values are updated on existing nodes.
fn reuse_cached_tree(
    indices: &[usize],
) -> (TaffyTree<usize>, HashMap<usize, NodeId>, Vec<usize>) {
    // Take ownership of cached tree
    let mut tree = TAFFY_TREE_CACHE.with(|cache| {
        cache.borrow_mut().take().expect("cache should be present")
    });

    // Clone the cached mapping
    let index_to_node = INDEX_TO_NODE_CACHE.with(|cache| cache.borrow().clone());

    // Clone the cached roots
    let roots = ROOTS_CACHE.with(|cache| cache.borrow().clone());

    // Update styles for all visible nodes (styles may have changed reactively)
    for &idx in indices {
        if !core::get_visible(idx) {
            continue;
        }

        if let Some(&node_id) = index_to_node.get(&idx) {
            let style = build_style(idx);
            let _ = tree.set_style(node_id, style);
        }
    }

    (tree, index_to_node, roots)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{allocate_index, create_flex_node, reset_registry};
    use crate::engine::arrays::core as core_arrays;

    fn setup() {
        reset_registry();
        reset_taffy_cache();
    }

    #[test]
    fn test_compute_layout_empty() {
        setup();

        let layout = compute_layout_taffy(80, 24, true);
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

        let layout = compute_layout_taffy(80, 24, true);

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

        let layout = compute_layout_taffy(80, 24, true);

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
        parent_node.flex_direction.set_value(1); // Row

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

        let layout = compute_layout_taffy(80, 24, true);

        // Children should be side by side
        assert_eq!(layout.x[child1], 0);
        assert_eq!(layout.x[child2], 10); // After first child
    }

    #[test]
    fn test_flex_grow() {
        setup();

        // Parent
        let parent = allocate_index(None);
        core_arrays::set_component_type(parent, ComponentType::Box);
        core_arrays::set_visible(parent, true);
        let parent_node = create_flex_node(parent);
        parent_node.width.set_value(Dimension::Cells(100));
        parent_node.height.set_value(Dimension::Cells(10));
        parent_node.flex_direction.set_value(1); // Row

        // Child with flex-grow: 1
        let child = allocate_index(None);
        core_arrays::set_component_type(child, ComponentType::Box);
        core_arrays::set_visible(child, true);
        core_arrays::set_parent_index(child, Some(parent));
        let child_node = create_flex_node(child);
        child_node.flex_grow.set_value(1.0);
        child_node.height.set_value(Dimension::Cells(5));

        let layout = compute_layout_taffy(80, 24, true);

        // Child should grow to fill parent
        assert_eq!(layout.width[child], 100);
    }

    #[test]
    fn test_padding_and_border() {
        setup();

        // Parent with padding and border
        let parent = allocate_index(None);
        core_arrays::set_component_type(parent, ComponentType::Box);
        core_arrays::set_visible(parent, true);
        let parent_node = create_flex_node(parent);
        parent_node.width.set_value(Dimension::Cells(40));
        parent_node.height.set_value(Dimension::Cells(10));
        parent_node.padding_left.set_value(2);
        parent_node.padding_top.set_value(1);
        parent_node.border_left.set_value(1); // Will become 1 cell

        // Child
        let child = allocate_index(None);
        core_arrays::set_component_type(child, ComponentType::Box);
        core_arrays::set_visible(child, true);
        core_arrays::set_parent_index(child, Some(parent));
        let child_node = create_flex_node(child);
        child_node.width.set_value(Dimension::Cells(10));
        child_node.height.set_value(Dimension::Cells(5));

        let layout = compute_layout_taffy(80, 24, true);

        // Child should be offset by padding + border
        assert_eq!(layout.x[child], 3); // 2 padding + 1 border
        assert_eq!(layout.y[child], 1); // 1 padding, no border top
    }

    #[test]
    fn test_dimension_conversion() {
        assert!(matches!(to_taffy_dimension(Dimension::Auto), TaffyDimension::Auto));
        assert!(matches!(to_taffy_dimension(Dimension::Cells(50)), TaffyDimension::Length(50.0)));
        // Percent: 50% → 0.5
        if let TaffyDimension::Percent(p) = to_taffy_dimension(Dimension::Percent(50.0)) {
            assert!((p - 0.5).abs() < 0.001);
        } else {
            panic!("Expected Percent variant");
        }
    }

    #[test]
    fn test_justify_content_center() {
        setup();

        // Parent
        let parent = allocate_index(None);
        core_arrays::set_component_type(parent, ComponentType::Box);
        core_arrays::set_visible(parent, true);
        let parent_node = create_flex_node(parent);
        parent_node.width.set_value(Dimension::Cells(100));
        parent_node.height.set_value(Dimension::Cells(10));
        parent_node.flex_direction.set_value(1); // Row
        parent_node.justify_content.set_value(1); // Center

        // Child
        let child = allocate_index(None);
        core_arrays::set_component_type(child, ComponentType::Box);
        core_arrays::set_visible(child, true);
        core_arrays::set_parent_index(child, Some(parent));
        let child_node = create_flex_node(child);
        child_node.width.set_value(Dimension::Cells(20));
        child_node.height.set_value(Dimension::Cells(5));

        let layout = compute_layout_taffy(80, 24, true);

        // Child should be centered: (100 - 20) / 2 = 40
        assert_eq!(layout.x[child], 40);
    }

    #[test]
    fn test_cache_invalidation() {
        setup();

        // Create initial component
        let idx = allocate_index(None);
        core_arrays::set_component_type(idx, ComponentType::Box);
        core_arrays::set_visible(idx, true);
        let node = create_flex_node(idx);
        node.width.set_value(Dimension::Cells(40));
        node.height.set_value(Dimension::Cells(10));

        // First layout - builds cache
        let layout1 = compute_layout_taffy(80, 24, true);
        assert_eq!(layout1.width[idx], 40);

        // Cache should now be clean
        assert!(!is_taffy_cache_dirty());

        // Change style without structural change - cache should still work
        node.width.set_value(Dimension::Cells(60));
        let layout2 = compute_layout_taffy(80, 24, true);
        assert_eq!(layout2.width[idx], 60); // Style updated

        // Explicit invalidation
        invalidate_taffy_cache();
        assert!(is_taffy_cache_dirty());

        // Layout after invalidation
        let layout3 = compute_layout_taffy(80, 24, true);
        assert_eq!(layout3.width[idx], 60); // Still works
        assert!(!is_taffy_cache_dirty()); // Cache rebuilt
    }

    #[test]
    fn test_cache_reuse_with_style_changes() {
        setup();

        // Create parent and child
        let parent = allocate_index(None);
        core_arrays::set_component_type(parent, ComponentType::Box);
        core_arrays::set_visible(parent, true);
        let parent_node = create_flex_node(parent);
        parent_node.width.set_value(Dimension::Cells(100));
        parent_node.height.set_value(Dimension::Cells(20));
        parent_node.flex_direction.set_value(1); // Row

        let child = allocate_index(None);
        core_arrays::set_component_type(child, ComponentType::Box);
        core_arrays::set_visible(child, true);
        core_arrays::set_parent_index(child, Some(parent));
        let child_node = create_flex_node(child);
        child_node.width.set_value(Dimension::Cells(30));
        child_node.height.set_value(Dimension::Cells(10));

        // First layout
        let layout1 = compute_layout_taffy(80, 24, true);
        assert_eq!(layout1.width[child], 30);
        assert!(!is_taffy_cache_dirty());

        // Change child width - should use cache fast path
        child_node.width.set_value(Dimension::Cells(50));
        let layout2 = compute_layout_taffy(80, 24, true);
        assert_eq!(layout2.width[child], 50);
    }
}
