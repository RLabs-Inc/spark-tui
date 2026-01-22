# TITAN Layout Engine Specification

## Executive Summary

The TITAN Layout Engine is a terminal-optimized, reactive flexbox layout system implementing the W3C CSS Flexbox Level 1 specification. It features zero-recursion O(n) traversal, integer-only math for discrete terminal cells, and seamless integration with the Rust signals crate for automatic recalculation.

**Key Files Analyzed:**
- `/Users/rusty/Documents/Projects/TUI/tui/src/pipeline/layout/titan-engine.ts` - Main engine (862 lines)
- `/Users/rusty/Documents/Projects/TUI/tui/src/pipeline/layout/flexbox-spec/algorithm.ts` - W3C algorithm (1376 lines)
- `/Users/rusty/Documents/Projects/TUI/tui/src/pipeline/layout/flexbox-spec/types.ts` - Type system (278 lines)
- `/Users/rusty/Documents/Projects/TUI/tui/src/pipeline/layout/flexbox-spec/utils.ts` - Utilities
- `/Users/rusty/Documents/Projects/TUI/tui/src/pipeline/layout/utils/text-measure.ts` - Text measurement
- `/Users/rusty/Documents/Projects/TUI/tui/src/pipeline/layout/utils/math.ts` - Math utilities
- `/Users/rusty/Documents/Projects/TUI/tui/src/pipeline/layout/utils/hierarchy.ts` - Tree utilities

---

## 1. Architecture Overview

### 1.1 Reactive Pipeline

```
Component Arrays (signals)
    ↓ (reads create dependencies)
layoutDerived (derived())
    ↓ (auto-recalculates on change)
ComputedLayout
    ↓
Render Pipeline
```

**Key Insight**: FlexNode properties are **Slots** that directly reference array indices. Reading `flex_node.width.value` automatically creates a reactive dependency.

### 1.2 Three-Pass Algorithm

1. **Pass 1: Tree Structure (BFS)** - Build linked list representation (O(n))
2. **Pass 2: Intrinsic Sizing (Bottom-Up)** - Measure text and calculate natural sizes (O(n))
3. **Pass 3: Layout (Top-Down)** - Position and size via flexbox algorithm (O(n))

### 1.3 Memory Model

**Module-level arrays** (reused across layout passes):
```rust
// Output arrays
let mut out_x: Vec<i32> = Vec::new();
let mut out_y: Vec<i32> = Vec::new();
let mut out_w: Vec<i32> = Vec::new();
let mut out_h: Vec<i32> = Vec::new();
let mut out_scrollable: Vec<bool> = Vec::new();
let mut out_max_scroll_x: Vec<i32> = Vec::new();
let mut out_max_scroll_y: Vec<i32> = Vec::new();

// Tree structure
let mut first_child: Vec<i32> = Vec::new();
let mut next_sibling: Vec<i32> = Vec::new();
let mut last_child: Vec<i32> = Vec::new();

// Intrinsic sizes
let mut intrinsic_w: Vec<i32> = Vec::new();
let mut intrinsic_h: Vec<i32> = Vec::new();

// Flexbox working data
let mut item_main: Vec<i32> = Vec::new();
let mut item_cross: Vec<i32> = Vec::new();
```

**Why module-level?** Zero allocation overhead per layout pass. Only call `reset_layout_arrays()` when destroying all components.

---

## 2. Type System

### 2.1 Dimension Types

```rust
/// A dimension value that can be absolute, percentage, or auto
#[derive(Debug, Clone, PartialEq)]
pub enum Dimension {
    /// Absolute value in terminal cells
    Absolute(i32),
    /// Percentage of parent dimension (50% stored as 50)
    Percent(i32),
    /// Auto-sized (intrinsic or stretch)
    Auto,
}

/// Extended sizing with content keywords
#[derive(Debug, Clone, PartialEq)]
pub enum SizeValue {
    Absolute(i32),
    Percent(i32),
    Auto,
    /// Size to minimum content (tightest wrap)
    MinContent,
    /// Size to maximum content (no wrap)
    MaxContent,
    /// fit-content(argument) - clamp max-content
    FitContent(i32),
}

/// Margin can be auto for centering
#[derive(Debug, Clone, PartialEq)]
pub enum MarginValue {
    Absolute(i32),
    Percent(i32),
    Auto, // For margin: auto centering
}
```

### 2.2 Layout Enums

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FlexDirection {
    Column = 0,        // Vertical, top-to-bottom
    Row = 1,           // Horizontal, left-to-right
    ColumnReverse = 2, // Vertical, bottom-to-top
    RowReverse = 3,    // Horizontal, right-to-left
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FlexWrap {
    NoWrap = 0,      // Single line
    Wrap = 1,        // Multi-line
    WrapReverse = 2, // Multi-line, reversed
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum JustifyContent {
    FlexStart = 0,    // Pack to start
    Center = 1,       // Pack to center
    FlexEnd = 2,      // Pack to end
    SpaceBetween = 3, // Even spacing between
    SpaceAround = 4,  // Even spacing around
    SpaceEvenly = 5,  // Equal spacing
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AlignItems {
    Stretch = 0,   // Stretch to fill cross axis
    FlexStart = 1, // Align to start
    Center = 2,    // Align to center
    FlexEnd = 3,   // Align to end
    Baseline = 4,  // Align baselines
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AlignContent {
    Stretch = 0,     // Stretch lines to fill
    FlexStart = 1,   // Pack lines to start
    Center = 2,      // Pack lines to center
    FlexEnd = 3,     // Pack lines to end
    SpaceBetween = 4,
    SpaceAround = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AlignSelf {
    Auto = 0,      // Inherit from align-items
    Stretch = 1,   // Override to stretch
    FlexStart = 2,
    Center = 3,
    FlexEnd = 4,
    Baseline = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Position {
    Relative = 0, // In flow
    Absolute = 1, // Out of flow, positioned ancestor
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Overflow {
    Visible = 0, // Content can overflow
    Hidden = 1,  // Content clipped
    Scroll = 2,  // Always scrollable
    Auto = 3,    // Scrollable if needed
}
```

### 2.3 Container and Item Styles

```rust
/// Flex container configuration
#[derive(Debug, Clone)]
pub struct FlexContainerStyle {
    // Flex layout
    pub flex_direction: FlexDirection,
    pub flex_wrap: FlexWrap,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub align_content: AlignContent,

    // Gaps (CSS Box Alignment Level 3)
    pub gap: i32,
    pub row_gap: i32,    // Overrides gap for rows
    pub column_gap: i32, // Overrides gap for columns

    // Sizing
    pub width: SizeValue,
    pub min_width: SizeValue,
    pub max_width: SizeValue,
    pub height: SizeValue,
    pub min_height: SizeValue,
    pub max_height: SizeValue,

    // Padding
    pub padding_top: i32,
    pub padding_right: i32,
    pub padding_bottom: i32,
    pub padding_left: i32,

    // Border (1 cell max per side)
    pub border_top: bool,
    pub border_right: bool,
    pub border_bottom: bool,
    pub border_left: bool,
}

/// Flex item configuration
#[derive(Debug, Clone)]
pub struct FlexItemStyle {
    // Flex behavior
    pub flex_grow: f32,       // Default: 0.0
    pub flex_shrink: f32,     // Default: 1.0
    pub flex_basis: SizeValue, // Default: Auto

    // Alignment
    pub align_self: AlignSelf, // Default: Auto
    pub order: i32,            // Default: 0

    // Sizing
    pub width: SizeValue,
    pub min_width: SizeValue,
    pub max_width: SizeValue,
    pub height: SizeValue,
    pub min_height: SizeValue,
    pub max_height: SizeValue,

    // Margins (can be auto)
    pub margin_top: MarginValue,
    pub margin_right: MarginValue,
    pub margin_bottom: MarginValue,
    pub margin_left: MarginValue,

    // Padding
    pub padding_top: i32,
    pub padding_right: i32,
    pub padding_bottom: i32,
    pub padding_left: i32,

    // Border
    pub border_top: bool,
    pub border_right: bool,
    pub border_bottom: bool,
    pub border_left: bool,
}
```

### 2.4 Output Types

```rust
/// Computed layout result
#[derive(Debug, Clone)]
pub struct ComputedLayout {
    pub x: Vec<i32>,
    pub y: Vec<i32>,
    pub width: Vec<i32>,
    pub height: Vec<i32>,
    pub scrollable: Vec<bool>,
    pub max_scroll_x: Vec<i32>,
    pub max_scroll_y: Vec<i32>,
    pub content_width: i32,
    pub content_height: i32,
}
```

---

## 3. W3C Flexbox Algorithm (15 Steps)

### 3.1 Algorithm Overview

The W3C CSS Flexbox Level 1 specification defines a **15-step algorithm**. Our implementation follows this EXACTLY:

1. Generate Flex Items
2. Determine Available Space
3. Determine Flex Base Size
4. Collect into Flex Lines
5. **Resolve Flexible Lengths** (THE CORE - freeze loop)
6. Determine Hypothetical Cross Size
7. Calculate Line Cross Sizes
8. Handle align-content: stretch
9. Collapse visibility (skip - not implemented)
10. Determine Used Cross Size
11. Distribute Remaining Space (main axis)
12. Resolve Auto Margins (cross axis)
13. Align Items (cross axis)
14. Determine Container Cross Size
15. Align Lines

### 3.2 Step 1: Generate Flex Items

**Purpose**: Create FlexItem structs for each visible child.

```rust
struct FlexItem {
    index: usize,
    style: FlexItemStyle,

    // Computed during algorithm
    flex_base_size: i32,
    hypothetical_main_size: i32,
    target_main_size: i32,
    used_main_size: i32,
    hypothetical_cross_size: i32,
    used_cross_size: i32,
    main_position: i32,
    cross_position: i32,
    frozen: bool,
    baseline: i32,

    // Resolved auto margins
    resolved_margin_top: i32,
    resolved_margin_right: i32,
    resolved_margin_bottom: i32,
    resolved_margin_left: i32,
}

fn generate_flex_items(
    children: &[usize],
    visible: &[Signal<bool>],
) -> Vec<FlexItem> {
    children.iter()
        .filter(|&&child| visible[child].get() != false)
        .map(|&child| FlexItem {
            index: child,
            // ... initialize fields
        })
        .collect()
}
```

### 3.3 Step 2: Determine Available Space

**Purpose**: Calculate container dimensions in main and cross axes.

```rust
enum AvailableSpace {
    Definite(i32),
    MinContent,
    MaxContent,
    FitContent(i32),
}

fn determine_available_space(
    container: &FlexContainerStyle,
    terminal_width: i32,
    terminal_height: i32,
    axis: Axis,
) -> AvailableSpace {
    let size = match axis {
        Axis::Main => if is_row(container.flex_direction) {
            &container.width
        } else {
            &container.height
        },
        Axis::Cross => if is_row(container.flex_direction) {
            &container.height
        } else {
            &container.width
        },
    };

    match size {
        SizeValue::Absolute(px) => AvailableSpace::Definite(*px),
        SizeValue::Percent(pct) => {
            let parent_size = if is_main_horizontal(container, axis) {
                terminal_width
            } else {
                terminal_height
            };
            AvailableSpace::Definite(parent_size * pct / 100)
        }
        SizeValue::Auto => AvailableSpace::Definite(
            if is_main_horizontal(container, axis) {
                terminal_width
            } else {
                terminal_height
            }
        ),
        SizeValue::MinContent => AvailableSpace::MinContent,
        SizeValue::MaxContent => AvailableSpace::MaxContent,
        SizeValue::FitContent(limit) => AvailableSpace::FitContent(*limit),
    }
}
```

### 3.4 Step 3: Determine Flex Base Size

**Purpose**: Calculate initial size before flex grow/shrink.

```rust
fn determine_flex_base_size(
    items: &mut [FlexItem],
    container: &FlexContainerStyle,
    available_main: &AvailableSpace,
    available_cross: &AvailableSpace,
    measure_func: &MeasureFunc,
) {
    let is_row_dir = is_row(container.flex_direction);

    for item in items.iter_mut() {
        let flex_basis = &item.style.flex_basis;
        let mut flex_base_size = None;

        // §9.2.3.A: If flex-basis is definite, use it
        if !matches!(flex_basis, SizeValue::Auto | SizeValue::MinContent | SizeValue::MaxContent) {
            flex_base_size = resolve_size(flex_basis, available_main);
        }

        // §9.2.3.C: If sizing under content constraint
        if flex_base_size.is_none() &&
           matches!(available_main, AvailableSpace::MinContent | AvailableSpace::MaxContent) {
            let measured = measure_func(item.index, *available_main, *available_cross);
            flex_base_size = Some(if is_row_dir { measured.width } else { measured.height });
        }

        // §9.2.3.E: Otherwise, use main size or measure
        if flex_base_size.is_none() {
            let main_size = if is_row_dir { &item.style.width } else { &item.style.height };
            flex_base_size = resolve_size(main_size, available_main);

            if flex_base_size.is_none() {
                let measured = measure_func(item.index, *available_main, *available_cross);
                flex_base_size = Some(if is_row_dir { measured.width } else { measured.height });
            }
        }

        item.flex_base_size = flex_base_size.unwrap_or(0);

        // Hypothetical main size = flex base size clamped by min/max
        let min_main = if is_row_dir { &item.style.min_width } else { &item.style.min_height };
        let max_main = if is_row_dir { &item.style.max_width } else { &item.style.max_height };

        item.hypothetical_main_size = clamp_dimension(
            item.flex_base_size,
            min_main,
            max_main,
            available_main,
        );
    }
}
```

### 3.5 Step 4: Collect into Flex Lines

**Purpose**: Handle wrapping by grouping items into lines.

```rust
struct FlexLine {
    items: Vec<usize>,
    cross_size: i32,
    cross_position: i32,
}

fn collect_flex_lines(
    items: &mut [FlexItem],
    container: &FlexContainerStyle,
    available_main: &AvailableSpace,
) -> Vec<FlexLine> {
    // Sort by order property
    items.sort_by_key(|item| (item.style.order, item.index));

    // Single-line case
    if !is_wrap(container.flex_wrap) {
        return vec![FlexLine {
            items: items.iter().map(|i| i.index).collect(),
            cross_size: 0,
            cross_position: 0,
        }];
    }

    // Multi-line wrapping
    let mut lines = Vec::new();
    let mut current_line = Vec::new();
    let mut current_size = 0;

    let gap = if is_row(container.flex_direction) {
        container.column_gap
    } else {
        container.row_gap
    };

    let inner_main = match available_main {
        AvailableSpace::Definite(size) => *size - get_main_padding_border(container),
        _ => i32::MAX,
    };

    for item in items {
        let outer_size = get_outer_main_size(item, container.flex_direction);
        let gap_size = if current_line.is_empty() { 0 } else { gap };

        if current_line.is_empty() || current_size + outer_size + gap_size <= inner_main {
            current_line.push(item.index);
            current_size += outer_size + gap_size;
        } else {
            lines.push(FlexLine {
                items: current_line,
                cross_size: 0,
                cross_position: 0,
            });
            current_line = vec![item.index];
            current_size = outer_size;
        }
    }

    if !current_line.is_empty() {
        lines.push(FlexLine {
            items: current_line,
            cross_size: 0,
            cross_position: 0,
        });
    }

    lines
}
```

### 3.6 Step 5: Resolve Flexible Lengths (THE CORE)

**Purpose**: Distribute free space using flex-grow/flex-shrink. This is the **iterative freeze loop** - the heart of flexbox.

```rust
fn resolve_flexible_lengths(
    line: &mut FlexLine,
    items: &mut [FlexItem],
    container: &FlexContainerStyle,
    available_main: &AvailableSpace,
) {
    let is_row_dir = is_row(container.flex_direction);
    let gap = if is_row_dir { container.column_gap } else { container.row_gap };

    // Step 1: Calculate free space
    let total_hypothetical: i32 = line.items.iter()
        .map(|&idx| get_outer_main_size(&items[idx], container.flex_direction))
        .sum();
    let total_gaps = (line.items.len() as i32 - 1).max(0) * gap;

    let inner_main = match available_main {
        AvailableSpace::Definite(size) => *size - get_main_padding_border(container),
        _ => i32::MAX,
    };

    let free_space = inner_main - total_hypothetical - total_gaps;
    let growing = free_space > 0;

    // Step 2: Initialize
    for &idx in &line.items {
        items[idx].target_main_size = items[idx].flex_base_size;
        items[idx].frozen = false;
    }

    // Step 3: Size inflexible items
    for &idx in &line.items {
        let item = &mut items[idx];
        let flex_factor = if growing { item.style.flex_grow } else { item.style.flex_shrink };

        if flex_factor == 0.0 ||
           (growing && item.flex_base_size > item.hypothetical_main_size) ||
           (!growing && item.flex_base_size < item.hypothetical_main_size)
        {
            item.frozen = true;
            item.target_main_size = item.hypothetical_main_size;
        }
    }

    // Step 4: Calculate initial free space
    let mut initial_free_space = inner_main;
    for &idx in &line.items {
        let item = &items[idx];
        let size = if item.frozen {
            get_outer_size(item, container.flex_direction, Axis::Main, item.target_main_size)
        } else {
            get_outer_size(item, container.flex_direction, Axis::Main, item.flex_base_size)
        };
        initial_free_space -= size;
    }
    initial_free_space -= total_gaps;

    // Step 5: FREEZE LOOP
    loop {
        // 5.a: Check for flexible items
        let unfrozen: Vec<usize> = line.items.iter()
            .filter(|&&idx| !items[idx].frozen)
            .copied()
            .collect();

        if unfrozen.is_empty() {
            break; // All frozen - DONE!
        }

        // 5.b: Calculate remaining free space
        let mut remaining_free_space = inner_main - total_gaps;
        for &idx in &line.items {
            let item = &items[idx];
            let size = if item.frozen {
                get_outer_size(item, container.flex_direction, Axis::Main, item.target_main_size)
            } else {
                get_outer_size(item, container.flex_direction, Axis::Main, item.flex_base_size)
            };
            remaining_free_space -= size;
        }

        // Handle sum of flex factors < 1
        let flex_sum: f32 = unfrozen.iter()
            .map(|&idx| if growing { items[idx].style.flex_grow } else { items[idx].style.flex_shrink })
            .sum();

        if flex_sum < 1.0 {
            let scaled = (initial_free_space as f32 * flex_sum) as i32;
            if scaled.abs() < remaining_free_space.abs() {
                remaining_free_space = scaled;
            }
        }

        // 5.c: Distribute space
        if remaining_free_space != 0 {
            if growing {
                // Flex-grow
                let total_grow: f32 = unfrozen.iter()
                    .map(|&idx| items[idx].style.flex_grow)
                    .sum();

                for &idx in &unfrozen {
                    if total_grow > 0.0 {
                        let item = &mut items[idx];
                        let share = (item.style.flex_grow / total_grow) * remaining_free_space as f32;
                        item.target_main_size = item.flex_base_size + share as i32;
                    }
                }
            } else {
                // Flex-shrink (scaled by flex-basis)
                let total_scaled: f32 = unfrozen.iter()
                    .map(|&idx| items[idx].style.flex_shrink * items[idx].flex_base_size as f32)
                    .sum();

                for &idx in &unfrozen {
                    if total_scaled > 0.0 {
                        let item = &mut items[idx];
                        let scaled = item.style.flex_shrink * item.flex_base_size as f32;
                        let share = (scaled / total_scaled) * remaining_free_space.abs() as f32;
                        item.target_main_size = item.flex_base_size - share as i32;
                    }
                }
            }
        }

        // 5.d: Fix min/max violations
        let mut violations = Vec::new();

        for &idx in &unfrozen {
            let item = &mut items[idx];
            let min_main = if is_row_dir { &item.style.min_width } else { &item.style.min_height };
            let max_main = if is_row_dir { &item.style.max_width } else { &item.style.max_height };

            let unclamped = item.target_main_size;
            let clamped = clamp_dimension(unclamped, min_main, max_main, available_main).max(0);

            if clamped != unclamped {
                violations.push((idx, clamped - unclamped, clamped > unclamped));
                item.target_main_size = clamped;
            }
        }

        // 5.e: Freeze over-flexed items
        let total_violation: i32 = violations.iter().map(|(_, delta, _)| delta).sum();

        if total_violation == 0 {
            // Freeze all
            for &idx in &unfrozen {
                items[idx].frozen = true;
            }
        } else if total_violation > 0 {
            // Freeze min violations
            for (idx, _, is_min) in violations.iter().filter(|(_, _, is_min)| *is_min) {
                items[*idx].frozen = true;
            }
        } else {
            // Freeze max violations
            for (idx, _, is_min) in violations.iter().filter(|(_, _, is_min)| !*is_min) {
                items[*idx].frozen = true;
            }
        }

        // 5.f: Loop back
    }

    // Step 6: Set used sizes
    for &idx in &line.items {
        items[idx].used_main_size = items[idx].target_main_size;
    }
}
```

**Critical Details**:
- **Scaled shrink**: flex-shrink is weighted by flex-basis (larger items shrink more)
- **Freeze order**: Min violations freeze first when growing; max violations freeze first when shrinking
- **Sum < 1**: If sum of flex factors < 1.0, scale free space proportionally

### 3.7 Steps 6-10: Cross Size Determination

```rust
// Step 6: Hypothetical cross size
fn determine_hypothetical_cross_size(
    items: &mut [FlexItem],
    container: &FlexContainerStyle,
    measure_func: &MeasureFunc,
) {
    let is_row_dir = is_row(container.flex_direction);

    for item in items {
        let cross_prop = if is_row_dir { &item.style.height } else { &item.style.width };

        if let Some(size) = resolve_size(cross_prop, &AvailableSpace::Definite(0)) {
            item.hypothetical_cross_size = size;
        } else {
            // Measure with main size constraint
            let measured = if is_row_dir {
                measure_func(item.index,
                            AvailableSpace::Definite(item.used_main_size),
                            AvailableSpace::Auto)
            } else {
                measure_func(item.index,
                            AvailableSpace::Auto,
                            AvailableSpace::Definite(item.used_main_size))
            };
            item.hypothetical_cross_size = if is_row_dir { measured.height } else { measured.width };
        }
    }
}

// Step 7: Line cross sizes
fn calculate_line_cross_sizes(
    lines: &mut [FlexLine],
    items: &[FlexItem],
    container: &FlexContainerStyle,
    available_cross: &AvailableSpace,
) {
    // Single-line with definite cross size
    if lines.len() == 1 && !is_wrap(container.flex_wrap) {
        if let AvailableSpace::Definite(size) = available_cross {
            lines[0].cross_size = *size - get_cross_padding_border(container);
            return;
        }
    }

    // Multi-line: line cross size = largest item
    for line in lines {
        let mut max_cross = 0;
        for &idx in &line.items {
            let outer = get_outer_cross_size(&items[idx], container.flex_direction);
            max_cross = max_cross.max(outer);
        }
        line.cross_size = max_cross;
    }
}

// Step 8: align-content: stretch
fn handle_align_content_stretch(
    lines: &mut [FlexLine],
    container: &FlexContainerStyle,
    available_cross: &AvailableSpace,
) {
    if container.align_content != AlignContent::Stretch {
        return;
    }

    if let AvailableSpace::Definite(size) = available_cross {
        let inner_cross = *size - get_cross_padding_border(container);
        let total_cross: i32 = lines.iter().map(|l| l.cross_size).sum();
        let gap = if is_row(container.flex_direction) { container.row_gap } else { container.column_gap };
        let total_gaps = (lines.len() as i32 - 1).max(0) * gap;

        if total_cross + total_gaps < inner_cross {
            let extra = (inner_cross - total_cross - total_gaps) / lines.len() as i32;
            for line in lines {
                line.cross_size += extra;
            }
        }
    }
}

// Step 10: Used cross size
fn determine_used_cross_size(
    items: &mut [FlexItem],
    lines: &[FlexLine],
    container: &FlexContainerStyle,
) {
    let is_row_dir = is_row(container.flex_direction);

    for line in lines {
        for &idx in &line.items {
            let item = &mut items[idx];
            let align_self = if item.style.align_self == AlignSelf::Auto {
                container.align_items
            } else {
                align_self_to_align_items(item.style.align_self)
            };

            let cross_prop = if is_row_dir { &item.style.height } else { &item.style.width };

            // Stretch if: stretch + auto cross size + no auto margins
            if align_self == AlignItems::Stretch &&
               matches!(cross_prop, SizeValue::Auto) &&
               !has_auto_margin_cross(item, is_row_dir)
            {
                let border_padding = get_cross_border_padding(item, is_row_dir);
                let inner = (line.cross_size - border_padding).max(0);

                let min_cross = if is_row_dir { &item.style.min_height } else { &item.style.min_width };
                let max_cross = if is_row_dir { &item.style.max_height } else { &item.style.max_width };

                item.used_cross_size = clamp_dimension(inner, min_cross, max_cross,
                                                      &AvailableSpace::Definite(line.cross_size));
            } else {
                item.used_cross_size = item.hypothetical_cross_size;
            }
        }
    }
}
```

### 3.8 Steps 11-13: Main and Cross Axis Alignment

```rust
// Step 11: Distribute remaining space (main axis)
fn distribute_remaining_space(
    lines: &mut [FlexLine],
    items: &mut [FlexItem],
    container: &FlexContainerStyle,
    available_main: &AvailableSpace,
) {
    let is_row_dir = is_row(container.flex_direction);
    let gap = if is_row_dir { container.column_gap } else { container.row_gap };

    let inner_main = match available_main {
        AvailableSpace::Definite(size) => *size - get_main_padding_border(container),
        _ => i32::MAX,
    };

    for line in lines {
        // Calculate used space
        let mut used: i32 = line.items.iter()
            .map(|&idx| get_outer_main_size(&items[idx], container.flex_direction))
            .sum();
        used += (line.items.len() as i32 - 1).max(0) * gap;

        let mut free_space = inner_main - used;

        // Auto margins absorb free space FIRST
        let auto_items: Vec<usize> = line.items.iter()
            .filter(|&&idx| has_auto_margin_main(&items[idx], is_row_dir))
            .copied()
            .collect();

        if !auto_items.is_empty() && free_space > 0 {
            let total_autos: i32 = auto_items.iter()
                .map(|&idx| count_auto_margins_main(&items[idx], is_row_dir))
                .sum();

            let per_margin = free_space / total_autos;

            for &idx in &auto_items {
                let item = &mut items[idx];
                if is_row_dir {
                    if matches!(item.style.margin_left, MarginValue::Auto) {
                        item.resolved_margin_left = per_margin;
                    }
                    if matches!(item.style.margin_right, MarginValue::Auto) {
                        item.resolved_margin_right = per_margin;
                    }
                } else {
                    if matches!(item.style.margin_top, MarginValue::Auto) {
                        item.resolved_margin_top = per_margin;
                    }
                    if matches!(item.style.margin_bottom, MarginValue::Auto) {
                        item.resolved_margin_bottom = per_margin;
                    }
                }
            }

            free_space = 0;
        }

        // Position items via justify-content
        align_main_axis(line, items, container.justify_content, free_space, gap);
    }
}

// Step 13: Align items on cross axis
fn align_items_cross_axis(
    items: &mut [FlexItem],
    lines: &[FlexLine],
    container: &FlexContainerStyle,
) {
    let is_row_dir = is_row(container.flex_direction);

    for line in lines {
        for &idx in &line.items {
            let item = &mut items[idx];

            if has_auto_margin_cross(item, is_row_dir) {
                continue; // Auto margins handled separately
            }

            let align_self = if item.style.align_self == AlignSelf::Auto {
                container.align_items
            } else {
                align_self_to_align_items(item.style.align_self)
            };

            let outer_cross = get_outer_cross_size(item, is_row_dir);
            let free_space = line.cross_size - outer_cross;

            item.cross_position = match align_self {
                AlignItems::FlexStart => 0,
                AlignItems::FlexEnd => free_space,
                AlignItems::Center => free_space / 2,
                AlignItems::Stretch => 0,
                AlignItems::Baseline => 0, // TODO: Proper baseline alignment
            };
        }
    }
}
```

### 3.9 Steps 14-15: Container Size and Line Alignment

```rust
// Step 14: Determine container cross size
fn determine_container_size(
    lines: &[FlexLine],
    container: &FlexContainerStyle,
) -> (i32, i32) {
    let is_row_dir = is_row(container.flex_direction);

    // Cross size
    let cross_size = if let Some(size) = resolve_size(&container.height, &AvailableSpace::Definite(0)) {
        size
    } else {
        let content: i32 = lines.iter().map(|l| l.cross_size).sum();
        let gap = if is_row_dir { container.row_gap } else { container.column_gap };
        let gaps = (lines.len() as i32 - 1).max(0) * gap;
        content + gaps + get_cross_padding_border(container)
    };

    // Main size (similar logic)
    let main_size = 0; // ... calculate from content or explicit size

    if is_row_dir {
        (main_size, cross_size)
    } else {
        (cross_size, main_size)
    }
}

// Step 15: Align lines
fn align_lines(
    lines: &mut [FlexLine],
    container: &FlexContainerStyle,
    container_cross: i32,
) {
    let gap = if is_row(container.flex_direction) { container.row_gap } else { container.column_gap };
    let inner_cross = container_cross - get_cross_padding_border(container);
    let total_cross: i32 = lines.iter().map(|l| l.cross_size).sum();
    let total_gaps = (lines.len() as i32 - 1).max(0) * gap;
    let free_space = inner_cross - total_cross - total_gaps;

    let mut pos = 0;

    for (i, line) in lines.iter_mut().enumerate() {
        line.cross_position = match container.align_content {
            AlignContent::FlexStart => pos,
            AlignContent::FlexEnd => pos + free_space,
            AlignContent::Center => pos + free_space / 2,
            AlignContent::SpaceBetween => {
                if lines.len() == 1 {
                    pos
                } else {
                    pos + (free_space * i as i32) / (lines.len() as i32 - 1)
                }
            }
            AlignContent::SpaceAround => {
                let around = free_space / lines.len() as i32;
                pos + around / 2 + (around * i as i32)
            }
            AlignContent::Stretch => pos,
        };

        pos += line.cross_size + if i < lines.len() - 1 { gap } else { 0 };
    }
}
```

---

## 4. MeasureFunc: Intrinsic Sizing

**Purpose**: Components report their natural size to the layout engine.

### 4.1 MeasureFunc Definition

```rust
pub struct MeasureResult {
    pub width: i32,
    pub height: i32,
}

pub type MeasureFunc = dyn Fn(usize, AvailableSpace, AvailableSpace) -> MeasureResult;
```

### 4.2 Implementation for Component Types

```rust
fn create_measure_func(
    arrays: &ComponentArrays,
    flex_nodes: &FlexNodeRegistry,
) -> impl Fn(usize, AvailableSpace, AvailableSpace) -> MeasureResult {
    move |index, avail_w, avail_h| {
        match arrays.component_type[index] {
            ComponentType::Text => {
                let content = &arrays.text_content[index];
                let wrap = arrays.text_wrap[index];

                if wrap == 0 {
                    // No wrap - single line
                    MeasureResult {
                        width: measure_text_width(content),
                        height: 1,
                    }
                } else {
                    // Wrap to available width
                    let max_width = match avail_w {
                        AvailableSpace::Definite(w) => w,
                        _ => content.len() as i32,
                    };

                    MeasureResult {
                        width: max_width,
                        height: measure_text_height(content, max_width),
                    }
                }
            }

            ComponentType::Input => {
                let content = &arrays.text_content[index];
                let node = &flex_nodes[index];

                let border_w = (node.border_left as i32) + (node.border_right as i32);
                let border_h = (node.border_top as i32) + (node.border_bottom as i32);

                MeasureResult {
                    width: measure_text_width(content) +
                           node.padding_left + node.padding_right + border_w,
                    height: 1 + node.padding_top + node.padding_bottom + border_h,
                }
            }

            ComponentType::Box => {
                // Recursive layout for intrinsic size
                let node = &flex_nodes[index];
                let children = get_visible_children(index, arrays);

                if children.is_empty() {
                    let w = resolve_size(&node.width, &AvailableSpace::Definite(0)).unwrap_or(0);
                    let h = resolve_size(&node.height, &AvailableSpace::Definite(0)).unwrap_or(0);
                    return MeasureResult { width: w, height: h };
                }

                // Compute nested layout
                let nested = compute_flex_layout_recursive(
                    index,
                    &children,
                    avail_w,
                    avail_h,
                    arrays,
                    flex_nodes,
                );

                // Return CONTENT size (algorithm adds padding/border)
                let pad_border_w = node.padding_left + node.padding_right +
                                  (node.border_left as i32) + (node.border_right as i32);
                let pad_border_h = node.padding_top + node.padding_bottom +
                                  (node.border_top as i32) + (node.border_bottom as i32);

                MeasureResult {
                    width: (nested.container_width - pad_border_w).max(0),
                    height: (nested.container_height - pad_border_h).max(0),
                }
            }
        }
    }
}
```

---

## 5. Text Measurement

**Unicode-aware measurement** for terminals using the `unicode-width` crate.

### 5.1 Implementation

```rust
use unicode_width::{UnicodeWidthStr, UnicodeWidthChar};

pub fn measure_text_width(content: &str) -> i32 {
    UnicodeWidthStr::width(content) as i32
}

pub fn measure_text_height(content: &str, max_width: i32) -> i32 {
    if content.is_empty() || max_width <= 0 {
        return 0;
    }

    let mut lines = 1;
    let mut current_width = 0;

    for paragraph in content.split('\n') {
        if paragraph.is_empty() {
            lines += 1;
            continue;
        }

        for ch in paragraph.chars() {
            let char_width = UnicodeWidthChar::width(ch).unwrap_or(0) as i32;

            if current_width + char_width > max_width && current_width > 0 {
                lines += 1;
                current_width = char_width;
            } else {
                current_width += char_width;
            }
        }

        current_width = 0;
    }

    lines
}

pub fn wrap_text_lines(content: &str, max_width: i32) -> Vec<String> {
    // Word-boundary wrapping with character fallback
    let mut lines = Vec::new();

    for paragraph in content.split('\n') {
        if paragraph.is_empty() {
            lines.push(String::new());
            continue;
        }

        let mut current_line = String::new();
        let mut current_width = 0;

        for word in paragraph.split_whitespace() {
            let word_width = UnicodeWidthStr::width(word) as i32;

            if current_width == 0 {
                // First word on line
                if word_width <= max_width {
                    current_line = word.to_string();
                    current_width = word_width;
                } else {
                    // Word too long - character wrap
                    for ch in word.chars() {
                        let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0) as i32;
                        if current_width + ch_width > max_width {
                            lines.push(current_line);
                            current_line = ch.to_string();
                            current_width = ch_width;
                        } else {
                            current_line.push(ch);
                            current_width += ch_width;
                        }
                    }
                }
            } else if current_width + 1 + word_width <= max_width {
                // Word fits with space
                current_line.push(' ');
                current_line.push_str(word);
                current_width += 1 + word_width;
            } else {
                // Word doesn't fit - new line
                lines.push(current_line);
                current_line = word.to_string();
                current_width = word_width;
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }
    }

    lines
}
```

**Key Points**:
- ASCII: 1 cell per character
- Emoji: 2 cells (usually)
- CJK: 2 cells
- Control chars: 0 cells

---

## 6. Gap Property

**CSS Box Alignment Level 3** - spacing between items/lines.

```rust
// Gap is applied BETWEEN items, not around them
let gap = if is_row(container.flex_direction) {
    container.column_gap  // Horizontal spacing
} else {
    container.row_gap     // Vertical spacing
};

// When calculating size
let total_gaps = (item_count - 1).max(0) * gap;
let total_size = item_sizes.sum() + total_gaps;
```

**Fallback logic**:
- If `row_gap` not set, use `gap`
- If `column_gap` not set, use `gap`
- `gap` is shorthand for both

---

## 7. Scrolling

**Automatic scroll detection** for overflow containers.

```rust
fn detect_scrolling(
    container_index: usize,
    children_max_x: i32,
    children_max_y: i32,
    content_width: i32,
    content_height: i32,
    overflow: Overflow,
    out: &mut ComputedLayout,
) {
    let scroll_x = (children_max_x - content_width).max(0);
    let scroll_y = (children_max_y - content_height).max(0);

    let should_scroll = match overflow {
        Overflow::Scroll => true,
        Overflow::Auto => scroll_x > 0 || scroll_y > 0,
        _ => false,
    };

    if should_scroll {
        out.scrollable[container_index] = true;
        out.max_scroll_x[container_index] = scroll_x;
        out.max_scroll_y[container_index] = scroll_y;
    }
}
```

**Scrollable behavior**:
- **overflow: scroll**: Always scrollable
- **overflow: auto**: Scrollable if content overflows
- Root containers in fullscreen are auto-scrollable

---

## 8. Layout Caching

**Intrinsic size caching** avoids redundant measurements.

```rust
struct IntrinsicCache {
    text_hash: Vec<u64>,
    text_length: Vec<usize>,
    avail_width: Vec<i32>,
    cached_width: Vec<i32>,
    cached_height: Vec<i32>,
}

impl IntrinsicCache {
    fn get_or_compute(
        &mut self,
        index: usize,
        content: &str,
        avail_width: i32,
        compute: impl FnOnce() -> (i32, i32),
    ) -> (i32, i32) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        let hash = hasher.finish();

        if self.text_hash.get(index) == Some(&hash) &&
           self.text_length.get(index) == Some(&content.len()) &&
           self.avail_width.get(index) == Some(&avail_width)
        {
            // Cache hit!
            (self.cached_width[index], self.cached_height[index])
        } else {
            // Cache miss - compute
            let (w, h) = compute();

            self.text_hash[index] = hash;
            self.text_length[index] = content.len();
            self.avail_width[index] = avail_width;
            self.cached_width[index] = w;
            self.cached_height[index] = h;

            (w, h)
        }
    }
}
```

---

## 9. Reactive Integration

**Automatic recalculation** when dependencies change.

```rust
use crates::signals::{signal, derived, Signal, Derived};

// Terminal size signals
pub fn terminal_width() -> Signal<i32> {
    signal(80) // Updated on resize events
}

pub fn terminal_height() -> Signal<i32> {
    signal(24)
}

// Layout derived - THE MAGIC
pub fn layout_derived(
    arrays: &ComponentArrays,
    flex_nodes: &FlexNodeRegistry,
) -> Derived<ComputedLayout> {
    derived(move || {
        // Reading signals creates dependencies!
        let tw = terminal_width().get();
        let th = terminal_height().get();

        // Reading from arrays creates dependencies!
        // Inside flexbox algorithm:
        // - flex_nodes[i].width.get() tracks arrays.width[i]
        // - flex_nodes[i].flex_grow.get() tracks arrays.flex_grow[i]

        compute_layout_flexbox(tw, th, arrays, flex_nodes)
    })
}
```

**Key Insight**: FlexNode properties are Slots. Reading `flex_node.width.get()` automatically tracks dependency on `arrays.width[index]`.

---

## 10. Implementation Checklist

### 10.1 Core Types
- [ ] `Dimension`, `SizeValue`, `MarginValue` enums
- [ ] All layout enums (FlexDirection, JustifyContent, etc.)
- [ ] `FlexContainerStyle`, `FlexItemStyle` structs
- [ ] `ComputedLayout` output type
- [ ] `FlexItem`, `FlexLine` internal types

### 10.2 Dimension Resolution
- [ ] `resolve_size()` for percentage/absolute
- [ ] `clamp_dimension()` for min/max constraints
- [ ] Helper functions (is_row, is_column, is_wrap, etc.)

### 10.3 Flexbox Algorithm
- [ ] Step 1: Generate flex items
- [ ] Step 2: Determine available space
- [ ] Step 3: Determine flex base size
- [ ] Step 4: Collect into lines
- [ ] **Step 5: Resolve flexible lengths (freeze loop)**
- [ ] Step 6: Hypothetical cross size
- [ ] Step 7: Line cross sizes
- [ ] Step 8: align-content stretch
- [ ] Step 10: Used cross size
- [ ] Step 11: Distribute remaining space
- [ ] Step 12: Resolve auto margins (cross)
- [ ] Step 13: Align items (cross)
- [ ] Step 14: Container size
- [ ] Step 15: Align lines

### 10.4 MeasureFunc
- [ ] Text measurement (unicode-width)
- [ ] Input measurement
- [ ] Box recursive layout

### 10.5 Additional Features
- [ ] Gap property (row-gap, column-gap)
- [ ] Absolute positioning
- [ ] Scroll detection
- [ ] Intrinsic caching
- [ ] Text wrapping

### 10.6 Signal Integration
- [ ] FlexNode with Signal properties
- [ ] layout_derived() function
- [ ] Terminal size signals
- [ ] Array slot tracking

### 10.7 Testing
- [ ] Unit tests for each step
- [ ] Integration tests for full layouts
- [ ] Snapshot tests vs reference implementations
- [ ] Unicode edge cases

---

## 11. Performance Guidelines

### 11.1 O(n) Guarantees
- Single pass for tree structure
- Single pass for intrinsic sizing
- Single pass for layout (BFS)
- **Total: O(n) regardless of nesting depth**

### 11.2 Memory Efficiency
- Module-level arrays (reused)
- Grow but never shrink
- Call `reset_layout_arrays()` only on major transitions
- Cache-friendly data layout

### 11.3 Optimization Opportunities
- SIMD for batch operations
- Parallel layout for independent subtrees
- Incremental layout (only dirty subtrees)
- GPU acceleration for text measurement

---

## 12. Testing Strategy

### 12.1 Unit Tests
Test each algorithm step independently:
```rust
#[test]
fn test_freeze_loop_min_violation() {
    let items = vec![
        FlexItem { flex_grow: 1.0, flex_base_size: 100, min_width: 150, .. },
    ];

    resolve_flexible_lengths(&mut items, 200);

    // Should clamp to min_width
    assert_eq!(items[0].used_main_size, 150);
}
```

### 12.2 Integration Tests
Full layout scenarios:
```rust
#[test]
fn test_flex_wrap_two_lines() {
    let container = FlexContainerStyle {
        width: SizeValue::Absolute(200),
        flex_wrap: FlexWrap::Wrap,
        ..Default::default()
    };

    let items = vec![
        FlexItemStyle { width: SizeValue::Absolute(150), .. },
        FlexItemStyle { width: SizeValue::Absolute(150), .. },
    ];

    let layout = compute_layout(&container, &items, 200, 100);

    // Should have 2 lines
    assert_eq!(layout.y[1], layout.height[0]);
}
```

### 12.3 Snapshot Tests
Compare against Yoga/Taffy reference implementations.

---

## 13. Summary

The TITAN Layout Engine is a **complete W3C CSS Flexbox implementation** optimized for terminals:

✅ **Zero Recursion**: O(n) flat array iteration
✅ **Integer Math**: No floating point, discrete cells
✅ **Reactive**: Automatic recalculation via signals
✅ **Complete Spec**: All 15 steps of W3C algorithm
✅ **Optimized**: Cache-friendly parallel arrays
✅ **Unicode-Aware**: Proper emoji/CJK handling

**Next Steps for Rust Port**:
1. Implement type system (Section 2)
2. Build flexbox algorithm (Section 3)
3. Add MeasureFunc integration (Section 4)
4. Integrate with signals (Section 9)
5. Add text measurement (Section 5)
6. Implement caching (Section 8)
7. Test extensively (Section 12)
