---
phase: 04-scroll-system
plan: 02
subsystem: state/scroll
tags: [keyboard, scroll, navigation, global-keys]
dependency-graph:
  requires: [04-01]
  provides: [keyboard-scroll-handlers, layout-accessor-pattern]
  affects: [04-03, 04-04]
tech-stack:
  added: []
  patterns: [thread-local-accessor, closure-based-handlers]
key-files:
  created: []
  modified:
    - src/state/scroll.rs
    - src/state/global_keys.rs
    - src/state/mod.rs
decisions:
  - id: layout-accessor-pattern
    choice: "Thread-local CURRENT_LAYOUT with set/with/clear functions"
    rationale: "Keyboard handlers need layout access but can't receive it as parameter"
  - id: keyboard-no-chaining
    choice: "Keyboard scroll does NOT chain to parent"
    rationale: "Would conflict with focus management; mouse wheel chains, keyboard doesn't"
  - id: arrow-key-conditions
    choice: "Arrow keys only scroll without Ctrl/Alt modifiers"
    rationale: "Ctrl+Arrow used for word navigation in inputs"
metrics:
  duration: "~8 minutes"
  completed: "2026-01-22"
---

# Phase 4 Plan 2: Keyboard Scroll Handlers Summary

**One-liner:** Keyboard scroll handlers (arrow/page/home-end) wired into global_keys with thread-local layout accessor.

## What Was Built

### 1. Layout Accessor Pattern (`scroll.rs`)

Thread-local storage for current layout, enabling scroll handlers to access layout without parameter passing:

```rust
thread_local! {
    static CURRENT_LAYOUT: RefCell<Option<ComputedLayout>> = RefCell::new(None);
}

pub fn set_current_layout(layout: ComputedLayout);
pub fn with_current_layout<F, R>(f: F) -> Option<R>;
pub fn clear_current_layout();
```

### 2. Keyboard Scroll Handlers (`scroll.rs`)

Four handler functions for keyboard-driven scrolling:

- **`get_focused_scrollable(layout) -> i32`** - Find focused scrollable, -1 if none
- **`handle_arrow_scroll(layout, direction) -> bool`** - Scroll by LINE_SCROLL (1)
- **`handle_page_scroll(layout, direction) -> bool`** - Scroll by viewport * 0.9
- **`handle_home_end(layout, to_top) -> bool`** - Scroll to boundaries

All return `true` if scroll occurred, `false` if at boundary or not scrollable.

### 3. Global Keys Integration (`global_keys.rs`)

Extended `setup_global_keys()` to register scroll keyboard handlers:

- **Arrow keys** (without Ctrl/Alt): Line scroll focused scrollable
- **PageUp/PageDown**: Page scroll (90% of viewport)
- **Ctrl+Home/End**: Scroll to top/bottom

Handlers use `with_current_layout()` closure pattern:

```rust
scroll_cleanup = keyboard::on(move |event| {
    match event.key.as_str() {
        "ArrowDown" if !event.modifiers.shift => {
            return scroll::with_current_layout(|layout| {
                scroll::handle_arrow_scroll(layout, ScrollDirection::Down)
            }).unwrap_or(false);
        }
        // ... other keys
    }
});
```

### 4. Additional Features

- **Mouse wheel handler** (`handle_wheel_scroll`) with chaining support
- **Scroll into view** (`scroll_into_view`) for focus visibility

## Key Design Decisions

### Keyboard Scroll Does NOT Chain

Per CONTEXT.md decision: "No keyboard chaining - would conflict with focus management"

Mouse wheel chains to parent at boundaries, keyboard scroll does not.

### Arrow Key Conditions

Arrow keys only scroll when:
1. No Ctrl modifier (Ctrl+Arrow = word navigation in inputs)
2. No Alt modifier
3. No Shift modifier (Shift+Arrow = selection in inputs)
4. Focused component is scrollable

### Layout Accessor Pattern

The thread-local pattern enables keyboard handlers (which are registered as closures) to access the current layout without parameter drilling. The render pipeline calls `set_current_layout()` after each layout computation.

## Files Modified

| File | Changes |
|------|---------|
| `src/state/scroll.rs` | +170 lines: layout accessor, keyboard handlers, wheel handler, scroll_into_view |
| `src/state/global_keys.rs` | +230 lines: scroll key handler registration, 6 new tests |
| `src/state/mod.rs` | +3 exports: set_current_layout, with_current_layout, clear_current_layout |

## Tests Added

### scroll.rs (14 tests)
- test_get_focused_scrollable
- test_handle_arrow_scroll_down
- test_handle_arrow_scroll_up
- test_handle_arrow_scroll_at_boundary
- test_handle_arrow_scroll_horizontal
- test_handle_page_scroll
- test_handle_page_scroll_up
- test_handle_home_end
- test_no_scroll_when_not_focused
- test_no_scroll_when_focused_not_scrollable
- test_scroll_into_view_above
- test_scroll_into_view_below
- test_scroll_into_view_already_visible
- (plus existing chaining tests)

### global_keys.rs (6 tests)
- test_arrow_down_scrolls_focused_scrollable
- test_page_down_scrolls_by_viewport
- test_ctrl_end_scrolls_to_bottom
- test_ctrl_home_scrolls_to_top
- test_arrow_keys_dont_scroll_without_layout
- test_scroll_only_affects_focused_scrollable

## Test Results

```
352 unit tests passed (was 319 at phase start)
+33 new tests in 04-01 and 04-02
```

## Commits

| Hash | Type | Description |
|------|------|-------------|
| fb5dcfd | feat | Add keyboard scroll handlers to scroll module |
| 6561706 | feat | Wire scroll handlers into global_keys |
| 0e25a34 | test | Add keyboard scroll tests for global_keys |

## Deviations from Plan

### Auto-fixed (Rule 3 - Blocking)

The plan assumed scroll.rs already existed from 04-01. Found that 04-01 core operations were already implemented but plan file showed as untracked. Proceeded with 04-02 keyboard handlers building on existing core.

## Next Steps

**04-03: Mouse Wheel Scroll**
- Wire `handle_wheel_scroll` into mouse dispatch
- Scroll chaining for nested scrollables
- Coordinate-based scroll target detection

**04-04: Scroll Into View**
- Automatic scroll on focus change
- Focus visibility within scrollable parents
- Integration with focus system

## Integration Notes

For scroll keys to work in a running app:
1. Render pipeline must call `scroll::set_current_layout(layout)` after layout computation
2. Component must be focused and scrollable (overflow: scroll)
3. Arrow keys won't fire if input component captures them first
