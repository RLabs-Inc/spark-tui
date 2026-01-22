---
phase: 01-mouse-events
plan: 01
subsystem: state/mouse
tags: [mouse, hitgrid, events, handlers, dispatch]

dependency_graph:
  requires: []
  provides:
    - MouseEvent type
    - MouseAction, MouseButton, ScrollDirection enums
    - HitGrid O(1) coordinate lookup
    - Handler registry (per-component and global)
    - dispatch() function with hover/click detection
  affects:
    - 01-02 (mount.rs mouse input integration)
    - 01-03 (event callback wiring)

tech_stack:
  added: []
  patterns:
    - thread_local! for global state (matching keyboard.rs)
    - Signal-based reactive state
    - Handler ID generation for cleanup

key_files:
  created:
    - src/state/mouse.rs
  modified:
    - src/state/mod.rs
    - src/pipeline/mount.rs

decisions:
  - id: hitgrid-location
    choice: "HitGrid moved to mouse.rs, global thread_local! storage"
    rationale: "Centralized mouse state, allows dispatch to access without parameters"
  - id: handler-pattern
    choice: "Mirror keyboard.rs handler registry pattern"
    rationale: "Consistency across event systems, proven cleanup pattern"
  - id: click-detection
    choice: "Track pressed component and button, compare on up event"
    rationale: "Matches TypeScript behavior exactly"

metrics:
  duration: "4 minutes"
  completed: "2026-01-22"
---

# Phase 01 Plan 01: Mouse Types and HitGrid Summary

Mouse module with O(1) coordinate lookup, event types, handler registry, and dispatch with hover/click detection.

## What Was Built

### MouseEvent Types (matching TypeScript)
- `MouseAction`: Down, Up, Move, Drag, Scroll
- `MouseButton`: Left, Middle, Right, None
- `ScrollDirection`: Up, Down, Left, Right
- `ScrollInfo`: direction + delta
- `MouseEvent`: action, button, x, y, modifiers, scroll, component_index

### HitGrid (O(1) Coordinate Lookup)
- Moved from `mount.rs` to centralized `mouse.rs`
- Global `thread_local!` storage with helper functions:
  - `resize_hit_grid(w, h)`
  - `clear_hit_grid()`
  - `fill_hit_rect(x, y, w, h, index)`
  - `hit_test(x, y) -> Option<usize>`
  - `hit_grid_size() -> (u16, u16)`

### Reactive State Signals
- `LAST_EVENT: Signal<Option<MouseEvent>>`
- `MOUSE_X: Signal<u16>`
- `MOUSE_Y: Signal<u16>`
- `IS_MOUSE_DOWN: Signal<bool>`
- `HOVERED_COMPONENT: Signal<Option<usize>>`
- `PRESSED_COMPONENT: Signal<Option<usize>>`

### Handler Registry
- Per-component handlers via `on_component(index, MouseHandlers)`
- Global handlers: `on_mouse_down`, `on_mouse_up`, `on_click`, `on_scroll`
- All return cleanup closures (FnOnce)
- Handler ID generation for precise removal

### Dispatch Function
- Fills `component_index` from HitGrid
- Updates reactive state
- Handles hover enter/leave with callbacks + interaction arrays
- Handles mouse down/up with pressed tracking
- Detects clicks (same component + same button)
- Updates `interaction::set_hovered()` and `interaction::set_pressed()`

## Commits

| Hash | Type | Description |
|------|------|-------------|
| f3b08b1 | feat | Create mouse module with types, HitGrid, handlers, dispatch |
| dc28f1d | refactor | Move HitGrid to mouse module, update mount.rs |

## Tests Added

14 new tests in `src/state/mouse.rs`:
- HitGrid: get/set, fill_rect, resize, out_of_bounds, clear
- Global HitGrid: fill and test
- Handlers: registration, cleanup
- Dispatch: state updates, hover detection, click detection, scroll

## Verification

- `cargo check -p spark-tui` - Compiles with 0 errors
- `cargo test -p spark-tui` - 136 tests pass
- Module exports verified: `MouseEvent`, `dispatch`, `on_component`, `HitGrid`
- Key links verified:
  - `mouse.rs` uses `interaction::set_hovered` and `interaction::set_pressed`
  - `mount.rs` imports and uses `crate::state::mouse`

## Deviations from Plan

None - plan executed exactly as written.

## Next Steps

Plan 01-02 will integrate mouse input parsing from terminal events and wire it to `mouse::dispatch()`.
