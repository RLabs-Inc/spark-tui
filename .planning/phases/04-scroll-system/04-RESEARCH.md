# Phase 4: Scroll System - Research

**Researched:** 2026-01-22
**Domain:** Scroll state management, overflow handling, input event routing
**Confidence:** HIGH

## Summary

The scroll system requires coordinating several existing subsystems: the layout engine (which computes scrollability and max scroll values), the interaction arrays (which store scroll offsets), the frame buffer renderer (which already uses scroll offsets when positioning children), and the input dispatch (keyboard and mouse).

The TypeScript reference implementation at `/Users/rusty/Documents/Projects/TUI/tui/src/state/scroll.ts` provides a clean pattern: scroll state (offsets) lives in interaction arrays, computed values (scrollable, maxScrollX/Y) come from layout, and operations clamp to valid ranges. This separation is already established in the Rust codebase.

**Primary recommendation:** Build a scroll module (`src/state/scroll.rs`) that provides scroll operations and queries, wire keyboard/mouse event handlers into the existing dispatch system, and add scrollbar rendering to the frame buffer derived.

## Standard Stack

### Core (Already Present)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| spark-signals | local | Reactive primitives | Project foundation |
| taffy | 0.6.x | Flexbox layout (computes overflow) | Already computing scrollable/maxScroll |
| crossterm | 0.28.x | Terminal I/O, mouse events | Already handling input |

### New Components to Build
| Component | Purpose | Pattern |
|-----------|---------|---------|
| `scroll` module | Scroll operations, queries, handlers | Mirror TypeScript scroll.ts |
| Scrollbar renderer | Visual indicator of scroll position | Part of frame_buffer_derived |

### No External Dependencies Needed

The scroll system is self-contained using existing infrastructure:
- Scroll state: interaction arrays (`scroll_offset_x`, `scroll_offset_y`)
- Computed values: ComputedLayout (`scrollable`, `max_scroll_x`, `max_scroll_y`)
- Event routing: keyboard and mouse dispatch already exist
- Rendering: frame_buffer_derived already applies scroll offsets to children

## Architecture Patterns

### Recommended Module Structure
```
src/state/
    mod.rs           # Add scroll export
    scroll.rs        # NEW: Scroll operations and handlers
```

### Pattern 1: Scroll State Split (User vs Computed)

**What:** User scroll state (offsets) in interaction arrays, computed state (scrollable, maxScroll) from layout derived.

**When to use:** Always. This is the established pattern from TypeScript.

**Example:**
```rust
// Source: TypeScript scroll.ts pattern

/// User state - stored in interaction arrays
pub fn get_scroll_offset(index: usize) -> (u16, u16) {
    (
        interaction::get_scroll_offset_x(index),
        interaction::get_scroll_offset_y(index),
    )
}

/// Computed state - from layout derived
pub fn is_scrollable(layout: &ComputedLayout, index: usize) -> bool {
    layout.scrollable.get(index).copied().unwrap_or(0) == 1
}

pub fn get_max_scroll(layout: &ComputedLayout, index: usize) -> (u16, u16) {
    (
        layout.max_scroll_x.get(index).copied().unwrap_or(0),
        layout.max_scroll_y.get(index).copied().unwrap_or(0),
    )
}
```

**Why this pattern:**
- Layout already computes content size vs container size (via Taffy)
- User offsets can change independently without re-layout
- Changes to offset trigger re-render but not re-layout

### Pattern 2: Scroll-by with Clamping and Return Value

**What:** scroll_by returns bool indicating if scroll actually occurred. Enables scroll chaining.

**When to use:** All scroll operations should use this for uniform behavior.

**Example:**
```rust
// Source: TypeScript scroll.ts lines 87-103

/// Scroll by delta, clamping to valid range.
/// Returns true if scroll position changed (for chaining).
pub fn scroll_by(
    layout: &ComputedLayout,
    index: usize,
    delta_x: i32,
    delta_y: i32,
) -> bool {
    if !is_scrollable(layout, index) {
        return false;
    }

    let (current_x, current_y) = get_scroll_offset(index);
    let (max_x, max_y) = get_max_scroll(layout, index);

    let new_x = (current_x as i32 + delta_x).clamp(0, max_x as i32) as u16;
    let new_y = (current_y as i32 + delta_y).clamp(0, max_y as i32) as u16;

    if new_x == current_x && new_y == current_y {
        return false; // At boundary
    }

    interaction::set_scroll_offset(index, new_x, new_y);
    true
}
```

### Pattern 3: Scroll Chaining (Mouse Wheel Only)

**What:** When scrollable hits boundary, propagate to parent scrollable.

**When to use:** Mouse wheel events only (per CONTEXT.md decision).

**Example:**
```rust
// Source: TypeScript scroll.ts lines 133-153

/// Scroll with parent chaining. Returns true if any scrolling occurred.
pub fn scroll_by_with_chaining(
    layout: &ComputedLayout,
    index: usize,
    delta_x: i32,
    delta_y: i32,
) -> bool {
    // Try to scroll this component
    if scroll_by(layout, index, delta_x, delta_y) {
        return true;
    }

    // At boundary - try parent
    if let Some(parent_idx) = core::get_parent_index(index) {
        if is_scrollable(layout, parent_idx) {
            return scroll_by_with_chaining(layout, parent_idx, delta_x, delta_y);
        }
    }

    false
}
```

### Pattern 4: Focus-Triggered scrollIntoView

**What:** When focus changes to element inside scrollable, auto-scroll to reveal it.

**When to use:** On every focus change, check if new focus is within a scrollable ancestor.

**Example:**
```rust
// Source: TypeScript scroll.ts lines 271-301

/// Scroll to make child visible within scrollable parent.
/// Uses minimal scroll - aligns to nearest edge.
pub fn scroll_into_view(
    layout: &ComputedLayout,
    child_index: usize,
    scrollable_index: usize,
) {
    if !is_scrollable(layout, scrollable_index) {
        return;
    }

    // Get child position relative to scrollable content
    let child_y = layout.y.get(child_index).copied().unwrap_or(0);
    let child_height = layout.height.get(child_index).copied().unwrap_or(0);
    let viewport_height = layout.height.get(scrollable_index).copied().unwrap_or(0);

    let (_, current_scroll_y) = get_scroll_offset(scrollable_index);
    let viewport_top = current_scroll_y;
    let viewport_bottom = viewport_top + viewport_height;

    let child_top = child_y;
    let child_bottom = child_y + child_height;

    // Already visible
    if child_top >= viewport_top && child_bottom <= viewport_bottom {
        return;
    }

    // Scroll to make visible (minimal scroll)
    if child_top < viewport_top {
        // Child above viewport
        interaction::set_scroll_offset(
            scrollable_index,
            interaction::get_scroll_offset_x(scrollable_index),
            child_top,
        );
    } else if child_bottom > viewport_bottom {
        // Child below viewport
        interaction::set_scroll_offset(
            scrollable_index,
            interaction::get_scroll_offset_x(scrollable_index),
            child_bottom.saturating_sub(viewport_height),
        );
    }
}
```

### Pattern 5: Stick-to-Bottom Auto-Follow

**What:** Container prop that auto-scrolls to bottom when new content added, unless user scrolled up.

**When to use:** Logs, chat views, terminal output.

**Implementation approach:**
```rust
// Track stick-to-bottom state in a new interaction array
// STICK_TO_BOTTOM: TrackedSlotArray<bool> = default true
// ON_SCROLL_CHANGE: If user scrolls up, set stick_to_bottom = false
// ON_SCROLL_CHANGE: If user scrolls to bottom, set stick_to_bottom = true
// ON_CONTENT_CHANGE: If stick_to_bottom && max_scroll_y increased, scroll to bottom
```

### Anti-Patterns to Avoid

- **Direct scroll offset modification without clamping:** Always go through scroll_by or set_scroll_offset which clamps
- **Keyboard scroll chaining:** Would conflict with focus management (Tab should change focus, not chain scroll)
- **Storing scrollable state in arrays:** It's computed from layout, not user state
- **Horizontal scroll implementation:** Deferred per CONTEXT.md

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Content overflow detection | Manual size comparison | Taffy/ComputedLayout | Already computed as `scrollable`, `max_scroll_y` |
| Scroll offset storage | New signal/state | interaction arrays | `scroll_offset_x/y` already exist |
| Scroll offset rendering | Manual position calc | frame_buffer_derived | Already applies scroll when rendering children |
| Hit testing for scroll target | Manual coordinate math | HitGrid + mouse module | Already provides component at coordinates |
| Parent chain traversal | Manual loop | core::get_parent_index | Existing function for hierarchy navigation |

**Key insight:** The scroll system primarily wires together existing components. Taffy computes overflow, interaction arrays store offsets, frame_buffer_derived applies them, and input dispatch routes events.

## Common Pitfalls

### Pitfall 1: Forgetting Layout is Required for Scroll Queries

**What goes wrong:** Calling `is_scrollable(index)` without access to current layout.

**Why it happens:** Scrollable status is computed, not stored. Without the layout result, you can't query it.

**How to avoid:** Scroll functions that need scrollable/maxScroll must take `&ComputedLayout` parameter. Use a layout derived or pass layout through event handlers.

**Warning signs:** Functions that check `scrollable` without layout parameter.

### Pitfall 2: Scroll Chaining Infinite Loop

**What goes wrong:** Parent-child cycles in scroll chaining cause stack overflow.

**Why it happens:** Malformed tree or incorrect parent lookup.

**How to avoid:** Use `core::get_parent_index` which returns `None` for roots. Add recursion depth limit (10 levels is reasonable max).

**Warning signs:** Stack overflow in scroll_by_with_chaining.

### Pitfall 3: Scroll Offset Not Triggering Re-render

**What goes wrong:** Changing scroll offset doesn't update display.

**Why it happens:** Scroll offset in interaction arrays must trigger frame_buffer_derived re-run.

**How to avoid:** frame_buffer_derived already reads scroll offsets via `interaction::get_scroll_offset_y(index)`. Ensure interaction array updates fire signal notifications.

**Warning signs:** Visual doesn't update after scroll, but offset value changes.

### Pitfall 4: Mouse Scroll on Wrong Component

**What goes wrong:** Mouse wheel scrolls component under cursor even when it's not scrollable.

**Why it happens:** HitGrid returns topmost component, which may not be scrollable.

**How to avoid:** Walk up parent chain to find nearest scrollable ancestor. Fall back to focused scrollable if none found under cursor.

**Warning signs:** Mouse wheel does nothing on child elements inside scrollable container.

### Pitfall 5: scrollIntoView with Wrong Coordinate System

**What goes wrong:** scrollIntoView calculation off by padding/border or scroll offset.

**Why it happens:** Mixing absolute vs relative vs content coordinates.

**How to avoid:** Child position from layout is relative to parent's content area. Account for padding and existing scroll offset.

**Warning signs:** Focus jumps to wrong position, overshoots, or undershoots.

## Code Examples

### Scroll Module Structure
```rust
// src/state/scroll.rs
// Source: TypeScript scroll.ts structure

/// Constants
pub const LINE_SCROLL: u16 = 1;
pub const WHEEL_SCROLL: u16 = 3;
pub const PAGE_SCROLL_FACTOR: f32 = 0.9;

// State access
pub fn is_scrollable(layout: &ComputedLayout, index: usize) -> bool { ... }
pub fn get_scroll_offset(index: usize) -> (u16, u16) { ... }
pub fn get_max_scroll(layout: &ComputedLayout, index: usize) -> (u16, u16) { ... }

// Operations
pub fn set_scroll_offset(layout: &ComputedLayout, index: usize, x: u16, y: u16) { ... }
pub fn scroll_by(layout: &ComputedLayout, index: usize, dx: i32, dy: i32) -> bool { ... }
pub fn scroll_to_top(layout: &ComputedLayout, index: usize) { ... }
pub fn scroll_to_bottom(layout: &ComputedLayout, index: usize) { ... }
pub fn scroll_by_with_chaining(layout: &ComputedLayout, index: usize, dx: i32, dy: i32) -> bool { ... }

// Keyboard handlers
pub fn handle_arrow_scroll(layout: &ComputedLayout, direction: ScrollDirection) -> bool { ... }
pub fn handle_page_scroll(layout: &ComputedLayout, direction: ScrollDirection) -> bool { ... }
pub fn handle_home_end(layout: &ComputedLayout, key: HomeEnd) -> bool { ... }

// Mouse handlers
pub fn handle_wheel_scroll(layout: &ComputedLayout, x: u16, y: u16, direction: ScrollDirection) -> bool { ... }

// scrollIntoView
pub fn scroll_into_view(layout: &ComputedLayout, child_index: usize, scrollable_index: usize) { ... }

// Stick to bottom
pub fn get_stick_to_bottom(index: usize) -> bool { ... }
pub fn set_stick_to_bottom(index: usize, value: bool) { ... }
pub fn handle_content_change(layout: &ComputedLayout, index: usize) { ... }
```

### Wiring Keyboard Scroll
```rust
// Wire into global key handler
// Source: Project keyboard.rs pattern

pub fn setup_scroll_keyboard_handlers() {
    // Arrow keys for focused scrollable
    keyboard::on(|event| {
        let focused = focus::get_focused_index();
        if focused < 0 {
            return false;
        }

        // Get current layout (requires access to layout derived)
        let layout = get_current_layout(); // Implementation detail

        match event.key.as_str() {
            "ArrowUp" => scroll::handle_arrow_scroll(&layout, ScrollDirection::Up),
            "ArrowDown" => scroll::handle_arrow_scroll(&layout, ScrollDirection::Down),
            "PageUp" => scroll::handle_page_scroll(&layout, ScrollDirection::Up),
            "PageDown" => scroll::handle_page_scroll(&layout, ScrollDirection::Down),
            "Home" if event.modifiers.ctrl => scroll::handle_home_end(&layout, HomeEnd::Home),
            "End" if event.modifiers.ctrl => scroll::handle_home_end(&layout, HomeEnd::End),
            _ => false,
        }
    });
}
```

### Wiring Mouse Wheel
```rust
// Wire into mouse scroll handler
// Source: Project mouse.rs pattern

pub fn setup_scroll_mouse_handlers() {
    mouse::on_scroll(|event| {
        let layout = get_current_layout();

        if let Some(scroll_info) = &event.scroll {
            let direction = scroll_info.direction;

            // First try hovered component chain
            if let Some(hovered_idx) = event.component_index {
                if scroll::handle_wheel_scroll_at(
                    &layout,
                    hovered_idx,
                    direction,
                    true, // enable chaining
                ) {
                    return true;
                }
            }

            // Fallback to focused scrollable
            let focused = focus::get_focused_index();
            if focused >= 0 && scroll::is_scrollable(&layout, focused as usize) {
                return scroll::scroll_by(
                    &layout,
                    focused as usize,
                    0,
                    match direction {
                        ScrollDirection::Up => -(WHEEL_SCROLL as i32),
                        ScrollDirection::Down => WHEEL_SCROLL as i32,
                        _ => 0,
                    },
                );
            }
        }

        false
    });
}
```

### Scrollbar Rendering
```rust
// Add to frame_buffer_derived render_component
// Source: CONTEXT.md scrollbar decision

fn render_scrollbar(
    buffer: &mut FrameBuffer,
    layout: &ComputedLayout,
    index: usize,
    x: u16,
    y: u16,
    height: u16,
) {
    let overflow = get_flex_node(index).map(|n| Overflow::from(n.overflow.get()));
    let (_, max_scroll_y) = scroll::get_max_scroll(layout, index);
    let (_, scroll_y) = scroll::get_scroll_offset(index);

    if max_scroll_y == 0 {
        return; // No scrolling needed
    }

    match overflow {
        Some(Overflow::Scroll) => {
            // Full scrollbar: track + thumb
            let track_x = x + layout.width.get(index).copied().unwrap_or(0) - 1;
            render_full_scrollbar(buffer, track_x, y, height, scroll_y, max_scroll_y);
        }
        Some(Overflow::Auto) => {
            // Minimal indicator: position marker only
            let indicator_x = x + layout.width.get(index).copied().unwrap_or(0) - 1;
            render_scroll_indicator(buffer, indicator_x, y, height, scroll_y, max_scroll_y);
        }
        _ => {}
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Manual overflow calculation | Taffy computes content_size vs size | Already in codebase | No custom overflow math needed |
| Separate scroll state storage | Interaction arrays | Already in codebase | scroll_offset_x/y exist |

**What's already done:**
- `ComputedLayout.scrollable` - computed by taffy_bridge.rs line 407-414
- `ComputedLayout.max_scroll_x/y` - computed by taffy_bridge.rs line 410-413
- `interaction::scroll_offset_x/y` - stored in interaction.rs
- frame_buffer_derived applies scroll offsets - lines 303-316

**What needs implementation:**
- scroll.rs module with operations and handlers
- Event handler wiring for keyboard and mouse
- Scrollbar visual rendering
- scrollIntoView integration with focus
- stick_to_bottom interaction array and logic

## Open Questions

1. **Layout Access in Scroll Handlers**
   - What we know: Scroll operations need `&ComputedLayout`
   - What's unclear: How to access current layout from event handlers
   - Recommendation: Pass layout through event dispatch context, or use a global derived accessor

2. **Scroll Position Persistence**
   - What we know: scroll_offset stored in interaction arrays
   - What's unclear: Should scroll position persist across component hide/show?
   - Recommendation: Keep current behavior (arrays persist until component released)

3. **Stick-to-Bottom Trigger**
   - What we know: Need to detect "new content added" to trigger auto-scroll
   - What's unclear: What exactly triggers "content changed"? Layout recompute? Specific child addition?
   - Recommendation: Track previous max_scroll_y, trigger on increase when stick_to_bottom=true

## Sources

### Primary (HIGH confidence)
- `/Users/rusty/Documents/Projects/TUI/tui/src/state/scroll.ts` - TypeScript reference implementation (339 lines)
- `/Users/rusty/Documents/Projects/TUI/tui/src/pipeline/layout/titan-engine.ts` - Layout scroll computation (lines 713-725)
- `/Users/rusty/Documents/Projects/TUI/tui-rust/crates/tui/src/layout/taffy_bridge.rs` - Rust scrollable computation (lines 406-415)
- `/Users/rusty/Documents/Projects/TUI/tui-rust/crates/tui/src/pipeline/frame_buffer_derived.rs` - Scroll offset application (lines 289-332)
- `/Users/rusty/Documents/Projects/TUI/tui-rust/crates/tui/src/engine/arrays/interaction.rs` - Scroll offset storage

### Secondary (MEDIUM confidence)
- `.planning/phases/04-scroll-system/04-CONTEXT.md` - User decisions on scroll behavior

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Existing codebase provides all infrastructure
- Architecture: HIGH - TypeScript reference + existing Rust patterns
- Pitfalls: HIGH - Based on actual TypeScript implementation and common issues

**Research date:** 2026-01-22
**Valid until:** Stable pattern, valid indefinitely unless layout engine changes
