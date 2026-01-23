---
phase: 04-scroll-system
plan: 05
subsystem: scroll-integration
tags: [scroll, layout, pipeline, reactive, effects]

dependency-graph:
  requires:
    - 04-01 (scroll core)
    - 04-02 (keyboard handlers)
    - 04-03 (mouse handlers)
    - 04-04 (scrollbar rendering + stick_to_bottom)
  provides:
    - Global layout accessor for scroll system
    - Reactive stick_to_bottom integration
    - Unified scroll handler API (no layout parameter)
  affects:
    - Any future code using keyboard/mouse scroll handlers
    - Components with stick_to_bottom prop

tech-stack:
  added: []
  patterns:
    - Global layout accessor via thread_local
    - Reactive effects for auto-scroll
    - Guard patterns for uninitialized state

file-tracking:
  created: []
  modified:
    - src/pipeline/layout_derived.rs (global layout accessor)
    - src/pipeline/mod.rs (re-exports)
    - src/pipeline/mount.rs (set_layout calls)
    - src/state/scroll.rs (removed old pattern, updated handlers)
    - src/state/mod.rs (updated re-exports)
    - src/state/global_keys.rs (use new scroll API)
    - src/state/mouse.rs (use new scroll API)
    - src/state/focus.rs (use try_get_layout)
    - src/primitives/box_primitive.rs (stick_to_bottom effect)

decisions:
  - id: global-layout-accessor
    decision: Thread-local cache with set/get/clear functions
    rationale: Scroll handlers need layout but can't receive it as parameter from keyboard/mouse dispatch

metrics:
  duration: ~15 minutes
  completed: 2026-01-22
---

# Phase 04 Plan 05: Scroll Pipeline Integration Summary

## One-liner

Global layout accessor for scroll handlers plus reactive stick_to_bottom effect in Box component.

## What Changed

### 1. Global Layout Accessor (`layout_derived.rs`)

Added thread-local CURRENT_LAYOUT cache with accessor functions:

```rust
// Set by render effect after layout computation
pub fn set_layout(layout: ComputedLayout)

// Used by scroll handlers (panics if not set)
pub fn get_layout() -> ComputedLayout

// Safe version for effects that may run before mount
pub fn try_get_layout() -> Option<ComputedLayout>

// Clear on unmount
pub fn clear_layout()
```

The render effect in `mount.rs` now calls `set_layout()` before processing hit regions and rendering.

### 2. Updated Scroll Handlers (`scroll.rs`)

Removed old imperative pattern:
- Deleted `CURRENT_LAYOUT` thread_local
- Deleted `set_current_layout()`, `with_current_layout()`, `clear_current_layout()`

Updated handlers to use global accessor:
```rust
// OLD: handle_arrow_scroll(layout: &ComputedLayout, direction) -> bool
// NEW: handle_arrow_scroll(direction: ScrollDirection) -> bool
pub fn handle_arrow_scroll(direction: ScrollDirection) -> bool {
    let layout = get_layout();
    // ... same logic
}
```

All handlers updated:
- `handle_arrow_scroll(direction)` - uses `get_layout()`
- `handle_page_scroll(direction)` - uses `get_layout()`
- `handle_home_end(to_top)` - uses `get_layout()`
- `handle_wheel_scroll(x, y, direction)` - uses `get_layout()`, now uses chaining for focused fallback

### 3. stick_to_bottom Reactive Effect (`box_primitive.rs`)

When `stick_to_bottom: true`, Box now creates a reactive effect:

```rust
if props.stick_to_bottom {
    interaction::set_stick_to_bottom(index, true);

    let effect_index = index;
    let stop_effect = effect(move || {
        // Guard: Layout may not be initialized yet
        if let Some(layout) = try_get_layout() {
            scroll::handle_stick_to_bottom(&layout, effect_index);
        }
    });

    stick_effect_cleanup = Some(Box::new(stop_effect));
}
```

The effect:
- Watches layout changes via `try_get_layout()`
- Calls `handle_stick_to_bottom()` to auto-scroll when content grows
- Cleans up when the Box component is destroyed

### 4. Updated Consumers

- `global_keys.rs` - Uses `try_get_layout()` guard before calling scroll handlers
- `mouse.rs` - Uses `try_get_layout()` guard before calling `handle_wheel_scroll()`
- `focus.rs` - Uses `try_get_layout()` for scroll_focused_into_view
- `state/mod.rs` - Updated re-exports

## Bugs Fixed

**BLOCKER 1: Keyboard scroll handlers couldn't access layout**
- Previous: `set_current_layout()` was never called by the pipeline
- Now: `set_layout()` is called by render effect in `mount.rs`

**BLOCKER 2: stick_to_bottom auto-scroll never triggered**
- Previous: `handle_stick_to_bottom()` was orphaned (never called)
- Now: Reactive effect in Box component calls it when layout changes

**BUG: Mouse wheel focused fallback didn't use chaining**
- Previous: When falling back to focused scrollable, used `scroll_by()` (no chaining)
- Now: Uses `scroll_by_with_chaining()` for consistent behavior

## Test Results

- 379 unit tests pass
- 19 doc tests pass
- All scroll tests updated to use `setup_with_layout()` helper

## Architecture Notes

The solution maintains reactive correctness:

1. **Layout derived** still computes layout reactively based on terminal size, mode, and FlexNode slots
2. **set_layout()** is called in render effect, making layout available for input processing
3. **stick_to_bottom effect** creates reactive dependency on layout changes
4. **Guard patterns** (try_get_layout) prevent panics during component initialization

## Commits

1. `feat(04-05): add global layout accessor to pipeline` - 7f5f67e
2. `refactor(04-05): update scroll handlers to use pipeline::get_layout()` - f0847d3
3. `feat(04-05): add stick_to_bottom reactive effect in Box component` - f9e54bd
