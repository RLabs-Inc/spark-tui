# Scroll & Overflow Specification

## Overview

This specification documents overflow handling, scroll state management, keyboard/mouse scrolling, scroll-into-view, and rendering integration.

**Source Files Analyzed:**
- `/Users/rusty/Documents/Projects/TUI/tui/src/state/scroll.ts`
- `/Users/rusty/Documents/Projects/TUI/tui/src/primitives/box.ts` (overflow props)
- `/Users/rusty/Documents/Projects/TUI/tui/src/pipeline/layout/titan-engine.ts` (scroll computation)
- `/Users/rusty/Documents/Projects/TUI/tui/src/pipeline/frameBuffer.ts` (rendering)

---

## 1. Overflow Modes

### 1.1 Overflow Enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum Overflow {
    /// Content can overflow bounds (no clipping)
    #[default]
    Visible = 0,
    /// Content clipped, no scrolling
    Hidden = 1,
    /// Always scrollable (even if content fits)
    Scroll = 2,
    /// Scrollable only if content exceeds bounds
    Auto = 3,
}
```

### 1.2 Overflow Behavior Matrix

| Mode | Clips | Scrollable | When Scrollable |
|------|-------|------------|-----------------|
| visible | No | No | Never |
| hidden | Yes | No | Never |
| scroll | Yes | Yes | Always |
| auto | Yes | Yes | Content > viewport |

---

## 2. Scroll State

### 2.1 State Storage

Scroll state is split between two locations:

**User State (Interaction Arrays):**
```rust
// Current scroll offset (reactive)
INTERACTION.scroll_offset_x: SlotArray<i32>  // Default: 0
INTERACTION.scroll_offset_y: SlotArray<i32>  // Default: 0
```

**Computed State (Layout Output):**
```rust
// Computed by layout engine
pub struct ComputedLayout {
    // ... other fields ...
    pub scrollable: Vec<bool>,     // Is component scrollable?
    pub max_scroll_x: Vec<i32>,    // Maximum X scroll
    pub max_scroll_y: Vec<i32>,    // Maximum Y scroll
}
```

### 2.2 Scroll Manager

```rust
pub struct ScrollManager {
    interaction: Arc<InteractionArrays>,
    layout: Signal<ComputedLayout>,
}

impl ScrollManager {
    /// Check if component is scrollable
    pub fn is_scrollable(&self, index: usize) -> bool {
        self.layout.get().scrollable.get(index).copied().unwrap_or(false)
    }

    /// Get current scroll offset
    pub fn get_scroll_offset(&self, index: usize) -> (i32, i32) {
        (
            self.interaction.scroll_offset_x.get(index),
            self.interaction.scroll_offset_y.get(index),
        )
    }

    /// Get max scroll bounds
    pub fn get_max_scroll(&self, index: usize) -> (i32, i32) {
        let layout = self.layout.get();
        (
            layout.max_scroll_x.get(index).copied().unwrap_or(0),
            layout.max_scroll_y.get(index).copied().unwrap_or(0),
        )
    }

    /// Set scroll offset (clamped)
    pub fn set_scroll_offset(&self, index: usize, x: i32, y: i32) {
        if !self.is_scrollable(index) {
            return;
        }

        let (max_x, max_y) = self.get_max_scroll(index);
        let clamped_x = x.clamp(0, max_x);
        let clamped_y = y.clamp(0, max_y);

        self.interaction.scroll_offset_x.set(index, clamped_x);
        self.interaction.scroll_offset_y.set(index, clamped_y);
    }

    /// Scroll by delta (returns true if scrolled)
    pub fn scroll_by(&self, index: usize, delta_x: i32, delta_y: i32) -> bool {
        if !self.is_scrollable(index) {
            return false;
        }

        let (cur_x, cur_y) = self.get_scroll_offset(index);
        let (max_x, max_y) = self.get_max_scroll(index);

        let new_x = (cur_x + delta_x).clamp(0, max_x);
        let new_y = (cur_y + delta_y).clamp(0, max_y);

        if new_x == cur_x && new_y == cur_y {
            return false; // At boundary
        }

        self.set_scroll_offset(index, new_x, new_y);
        true
    }
}
```

---

## 3. Layout Integration

### 3.1 Minimal Intrinsic Size

Scrollable containers don't expand to fit content:

```rust
// In TITAN layout engine
if overflow == Overflow::Scroll || overflow == Overflow::Auto {
    // DON'T include children in intrinsic size
    intrinsic_w[i] = padding_left + padding_right + border_l + border_r;
    intrinsic_h[i] = padding_top + padding_bottom + border_t + border_b;
}
```

### 3.2 No Flex Shrink for Scrollable

Children of scrollable containers maintain natural size:

```rust
// Children don't shrink when parent is scrollable
if free_space < 0 && total_shrink > 0 && !is_scrollable_parent {
    // Only shrink if parent is NOT scrollable
    kid_main += (flex_shrink / total_shrink) * free_space;
}
```

### 3.3 Max Scroll Calculation

```rust
fn compute_scroll_bounds(
    parent: usize,
    children_max_x: i32,
    children_max_y: i32,
    content_width: i32,
    content_height: i32,
    overflow: Overflow,
    out: &mut ComputedLayout,
) {
    let scroll_range_x = (children_max_x - content_width).max(0);
    let scroll_range_y = (children_max_y - content_height).max(0);

    let should_scroll = match overflow {
        Overflow::Scroll => true,
        Overflow::Auto => scroll_range_x > 0 || scroll_range_y > 0,
        _ => false,
    };

    out.scrollable[parent] = should_scroll;
    out.max_scroll_x[parent] = if should_scroll { scroll_range_x } else { 0 };
    out.max_scroll_y[parent] = if should_scroll { scroll_range_y } else { 0 };
}
```

---

## 4. Scroll Operations

### 4.1 Constants

```rust
pub const LINE_SCROLL: i32 = 1;      // Arrow key scroll amount
pub const WHEEL_SCROLL: i32 = 3;     // Mouse wheel scroll amount
pub const PAGE_SCROLL_FACTOR: f32 = 0.9;  // Page Up/Down = 90% viewport
```

### 4.2 Basic Operations

```rust
/// Scroll to top
pub fn scroll_to_top(&self, index: usize) {
    self.set_scroll_offset(index, 0, 0);
}

/// Scroll to bottom
pub fn scroll_to_bottom(&self, index: usize) {
    let (_, max_y) = self.get_max_scroll(index);
    let (cur_x, _) = self.get_scroll_offset(index);
    self.set_scroll_offset(index, cur_x, max_y);
}

/// Scroll to start (left)
pub fn scroll_to_start(&self, index: usize) {
    let (_, cur_y) = self.get_scroll_offset(index);
    self.set_scroll_offset(index, 0, cur_y);
}

/// Scroll to end (right)
pub fn scroll_to_end(&self, index: usize) {
    let (max_x, _) = self.get_max_scroll(index);
    let (_, cur_y) = self.get_scroll_offset(index);
    self.set_scroll_offset(index, max_x, cur_y);
}
```

---

## 5. Input Handlers

### 5.1 Keyboard Scrolling

```rust
pub fn handle_arrow_scroll(direction: &str) -> bool {
    let focused = get_focused_index();
    if focused < 0 {
        return false;
    }

    let scrollable = find_scrollable_ancestor(focused as usize);
    if scrollable.is_none() {
        return false;
    }

    let index = scrollable.unwrap();
    let scroll = get_scroll_manager();

    match direction {
        "ArrowUp" => scroll.scroll_by(index, 0, -LINE_SCROLL),
        "ArrowDown" => scroll.scroll_by(index, 0, LINE_SCROLL),
        "ArrowLeft" => scroll.scroll_by(index, -LINE_SCROLL, 0),
        "ArrowRight" => scroll.scroll_by(index, LINE_SCROLL, 0),
        _ => false,
    }
}

pub fn handle_page_scroll(direction: &str) -> bool {
    let focused = get_focused_index();
    if focused < 0 {
        return false;
    }

    let scrollable = find_scrollable_ancestor(focused as usize);
    if scrollable.is_none() {
        return false;
    }

    let index = scrollable.unwrap();
    let scroll = get_scroll_manager();
    let layout = get_layout();

    let viewport_height = layout.height[index];
    let page_amount = (viewport_height as f32 * PAGE_SCROLL_FACTOR) as i32;

    match direction {
        "PageUp" => scroll.scroll_by(index, 0, -page_amount),
        "PageDown" => scroll.scroll_by(index, 0, page_amount),
        _ => false,
    }
}

pub fn handle_home_end_scroll(key: &str) -> bool {
    let focused = get_focused_index();
    if focused < 0 {
        return false;
    }

    let scrollable = find_scrollable_ancestor(focused as usize);
    if scrollable.is_none() {
        return false;
    }

    let index = scrollable.unwrap();
    let scroll = get_scroll_manager();

    match key {
        "Home" => {
            scroll.scroll_to_top(index);
            true
        }
        "End" => {
            scroll.scroll_to_bottom(index);
            true
        }
        _ => false,
    }
}
```

### 5.2 Mouse Wheel Scrolling

```rust
pub fn handle_wheel_scroll(
    x: usize,
    y: usize,
    direction: WheelDirection,
    hit_grid: &HitGrid,
) -> bool {
    // Try element under cursor
    let mut scrollable = None;

    if let Some(hit) = hit_grid.hit_test(x, y) {
        scrollable = find_scrollable_ancestor(hit);
    }

    // Fallback to focused scrollable
    if scrollable.is_none() {
        let focused = get_focused_index();
        if focused >= 0 {
            scrollable = find_scrollable_ancestor(focused as usize);
        }
    }

    if scrollable.is_none() {
        return false;
    }

    let index = scrollable.unwrap();
    let scroll = get_scroll_manager();

    match direction {
        WheelDirection::Up => scroll.scroll_by(index, 0, -WHEEL_SCROLL),
        WheelDirection::Down => scroll.scroll_by(index, 0, WHEEL_SCROLL),
        WheelDirection::Left => scroll.scroll_by(index, -WHEEL_SCROLL, 0),
        WheelDirection::Right => scroll.scroll_by(index, WHEEL_SCROLL, 0),
    }
}

#[derive(Debug, Clone, Copy)]
pub enum WheelDirection {
    Up,
    Down,
    Left,
    Right,
}
```

---

## 6. Scroll Into View

### 6.1 Algorithm

Minimal scroll to ensure element is visible:

```rust
pub fn scroll_into_view(
    scrollable_index: usize,
    child_y: i32,
    child_height: i32,
) {
    let scroll = get_scroll_manager();
    let layout = get_layout();

    let (_, cur_scroll_y) = scroll.get_scroll_offset(scrollable_index);
    let viewport_height = layout.height[scrollable_index];

    let viewport_top = cur_scroll_y;
    let viewport_bottom = cur_scroll_y + viewport_height;

    let child_top = child_y;
    let child_bottom = child_y + child_height;

    // Already visible?
    if child_top >= viewport_top && child_bottom <= viewport_bottom {
        return;
    }

    // Calculate minimal scroll
    let new_scroll_y = if child_top < viewport_top {
        // Above viewport - scroll up
        child_top
    } else if child_bottom > viewport_bottom {
        // Below viewport - scroll down
        child_bottom - viewport_height
    } else {
        cur_scroll_y
    };

    let (cur_scroll_x, _) = scroll.get_scroll_offset(scrollable_index);
    scroll.set_scroll_offset(scrollable_index, cur_scroll_x, new_scroll_y);
}
```

### 6.2 Integration with Focus

```rust
// Called when focus changes
fn on_focus_change(new_focus: usize) {
    // Find scrollable ancestor
    if let Some(scrollable) = find_scrollable_ancestor(new_focus) {
        let layout = get_layout();
        let child_y = layout.y[new_focus];
        let child_height = layout.height[new_focus];

        scroll_into_view(scrollable, child_y, child_height);
    }
}
```

---

## 7. Scroll Chaining

### 7.1 Nested Scrollable Containers

```rust
/// Scroll with chaining (try parent if at boundary)
pub fn scroll_by_with_chaining(index: usize, delta_x: i32, delta_y: i32) -> bool {
    let scroll = get_scroll_manager();

    // Try this component
    if scroll.scroll_by(index, delta_x, delta_y) {
        return true;
    }

    // At boundary - try parent
    let parent = LAYOUT.parent_index.get(index);
    if parent >= 0 {
        if let Some(parent_scrollable) = find_scrollable_ancestor(parent as usize) {
            return scroll_by_with_chaining(parent_scrollable, delta_x, delta_y);
        }
    }

    false
}
```

**Note:** Chaining is opt-in, not default behavior.

---

## 8. Rendering Integration

### 8.1 Scroll Offset Accumulation

```rust
fn render_component(
    buffer: &mut FrameBuffer,
    index: usize,
    layout: &ComputedLayout,
    parent_scroll_x: i32,
    parent_scroll_y: i32,
    // ...
) {
    // Apply parent scroll offset to position
    let x = (layout.x[index] - parent_scroll_x) as usize;
    let y = (layout.y[index] - parent_scroll_y) as usize;

    // ... render component ...

    // Get this component's scroll offset
    let is_scrollable = layout.scrollable.get(index).copied().unwrap_or(false);
    let (self_scroll_x, self_scroll_y) = if is_scrollable {
        get_scroll_offset(index)
    } else {
        (0, 0)
    };

    // Accumulate for children
    let child_scroll_x = parent_scroll_x + self_scroll_x;
    let child_scroll_y = parent_scroll_y + self_scroll_y;

    // Render children with accumulated scroll
    for child in children(index) {
        render_component(
            buffer,
            child,
            layout,
            child_scroll_x,
            child_scroll_y,
            // ...
        );
    }
}
```

### 8.2 Clipping

Children are clipped to scrollable container's content area:

```rust
// Content area = inside padding and border
let content_clip = ClipRect {
    x: x + padding_left + border_left,
    y: y + padding_top + border_top,
    w: width - padding_left - padding_right - border_left - border_right,
    h: height - padding_top - padding_bottom - border_top - border_bottom,
};

// Pass to children
for child in children(index) {
    render_component(buffer, child, layout, Some(content_clip), ...);
}
```

---

## 9. Helper Functions

### 9.1 Find Scrollable Ancestor

```rust
pub fn find_scrollable_ancestor(index: usize) -> Option<usize> {
    let layout = get_layout();

    // Check self
    if layout.scrollable.get(index).copied().unwrap_or(false) {
        return Some(index);
    }

    // Walk parent chain
    let mut current = LAYOUT.parent_index.get(index);
    while current >= 0 {
        if layout.scrollable.get(current as usize).copied().unwrap_or(false) {
            return Some(current as usize);
        }
        current = LAYOUT.parent_index.get(current as usize);
    }

    None
}
```

### 9.2 Get Focused Scrollable

```rust
pub fn get_focused_scrollable() -> Option<usize> {
    let focused = get_focused_index();
    if focused < 0 {
        return None;
    }

    find_scrollable_ancestor(focused as usize)
}
```

---

## 10. Special Behaviors

### 10.1 Root Container Scrolling

In fullscreen mode, root container is implicitly scrollable:

```rust
// In TITAN layout
let is_root = parent_index == -1;
let is_scrollable = overflow == Overflow::Scroll
    || overflow == Overflow::Auto
    || (is_root && constrain_height); // Implicit scrolling for root
```

### 10.2 Auto-Focusable Scrollable

Scrollable containers are automatically focusable (for keyboard navigation):

```rust
// In Box implementation
let should_be_focusable = props.focusable.unwrap_or(false)
    || (props.overflow == Some(Overflow::Scroll) && props.focusable != Some(false));

if should_be_focusable {
    INTERACTION.focusable.set(index, true);
}
```

### 10.3 No Scroll Bars

The TypeScript implementation does not render scroll bars. Scroll position changes are the only visual feedback.

---

## 11. Edge Cases

### 11.1 Empty Scrollable Container
- `overflow: scroll` → `scrollable = true`, `maxScroll = 0`
- Can't scroll but is marked scrollable

### 11.2 Content Exactly Fits
- `overflow: visible` → not scrollable
- `overflow: scroll` → scrollable with range `[0, 0]`
- `overflow: auto` → not scrollable

### 11.3 Nested Scrollable
- Each has independent scroll state
- Offsets accumulate during rendering
- Mouse wheel scrolls element under cursor
- Keyboard scrolls focused scrollable

### 11.4 Focus and Scroll Persistence
- Scroll state cleared on component destroy
- Recreated component starts at `(0, 0)`
- No automatic persistence

---

## 12. Hidden Automatic Behaviors

### 12.1 Minimal Intrinsic Size
Scrollable containers don't expand to fit content.

### 12.2 Children Don't Shrink
In scrollable, children maintain natural size.

### 12.3 Auto-Focusable
`overflow: scroll` makes container focusable.

### 12.4 Scroll Offset Clamping
All offsets clamped to `[0, maxScroll]`.

### 12.5 Root Implicit Scroll
Fullscreen root auto-scrollable if content exceeds.

### 12.6 Scroll on Focus
Focused element auto-scrolls into view.

### 12.7 Mouse Wheel Priority
Element under cursor → focused scrollable.

### 12.8 Content Area Clipping
Children clipped to inside padding/border.

---

## 13. Module Structure

```
crates/tui/src/state/
├── scroll.rs          # ScrollManager, operations
└── scroll_handlers.rs # Keyboard/mouse handlers

crates/tui/src/engine/arrays/
└── interaction.rs     # scroll_offset_x/y arrays

crates/tui/src/pipeline/
├── layout.rs          # scrollable, max_scroll in output
└── frame_buffer.rs    # Scroll offset application
```

---

## 14. Summary

The scroll system provides:

✅ **4 Overflow Modes**: visible, hidden, scroll, auto
✅ **Split State**: User offsets + computed bounds
✅ **Layout Integration**: Minimal intrinsic, no shrink
✅ **Clamped Operations**: All scrolls respect bounds
✅ **Keyboard Scrolling**: Arrow, Page, Home/End
✅ **Mouse Wheel**: Element under cursor or focused
✅ **Scroll Into View**: Minimal scroll for visibility
✅ **Scroll Chaining**: Try parent at boundary (opt-in)
✅ **Rendering**: Accumulated offsets, content clipping
✅ **Auto-Focusable**: Scrollable = keyboard navigable
