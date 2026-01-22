---
phase: 04-scroll-system
plan: 01
subsystem: state
tags: [scroll, state-management, chaining]
duration: "3 minutes"
completed: 2026-01-22

dependency-graph:
  requires: [phase-01-mouse-events, phase-03-input]
  provides: [scroll-operations, scroll-constants, scroll-chaining]
  affects: [04-02, 04-03, 04-04]

tech-stack:
  added: []
  patterns: [parent-chaining, clamped-operations]

key-files:
  created:
    - src/state/scroll.rs
  modified:
    - src/state/mod.rs

decisions:
  - id: layout-param
    choice: "Pass &ComputedLayout as parameter"
    rationale: "Rust functions can't access global derived like TypeScript"

metrics:
  tests-added: 14
  tests-total: 333
  lines-added: 417
  files-changed: 2
---

# Phase 04 Plan 01: Scroll Core Module Summary

**One-liner:** Scroll operations with clamping, boundary detection, and parent chaining for mouse wheel fallback.

## What Was Built

Created `src/state/scroll.rs` (409 lines) implementing the core scroll module with:

### Constants (matching TypeScript)
- `LINE_SCROLL: u16 = 1` - Arrow key scroll amount
- `WHEEL_SCROLL: u16 = 3` - Mouse wheel scroll amount
- `PAGE_SCROLL_FACTOR: f32 = 0.9` - PageUp/Down multiplier

### State Access
- `is_scrollable(layout, index)` - Check if component has overflow:scroll
- `get_scroll_offset(index)` - Read current (x, y) from interaction arrays
- `get_max_scroll(layout, index)` - Read limits from computed layout

### Operations
- `set_scroll_offset(layout, index, x, y)` - Set with clamping
- `scroll_by(layout, index, delta_x, delta_y) -> bool` - Delta scroll with boundary detection
- `scroll_to_top/bottom(layout, index)` - Jump to Y boundaries
- `scroll_to_start/end(layout, index)` - Jump to X boundaries

### Chaining
- `scroll_by_with_chaining(layout, index, delta_x, delta_y)` - Parent fallback at boundary

## Key Implementation Details

1. **Clamping**: All operations clamp to `0..max_scroll` using i32 arithmetic for negative deltas
2. **Boolean return**: `scroll_by` returns `true` if scroll occurred, `false` at boundary - enables chaining logic
3. **Parent chaining**: `scroll_by_with_chaining` recursively tries parent when child is at boundary
4. **Layout parameter**: Unlike TypeScript which accesses global `layoutDerived`, Rust passes `&ComputedLayout` explicitly

## Tests Added

14 comprehensive tests covering:
- `test_is_scrollable` / `test_is_scrollable_empty_layout`
- `test_get_scroll_offset` / `test_get_max_scroll`
- `test_set_scroll_offset_clamps` / `test_set_scroll_offset_not_scrollable`
- `test_scroll_by_returns_bool` / `test_scroll_by_negative` / `test_scroll_by_not_scrollable`
- `test_scroll_to_top_bottom` / `test_scroll_to_start_end`
- `test_scroll_by_with_chaining` / `test_scroll_by_with_chaining_no_parent`
- `test_constants`

## Deviations from Plan

None - plan executed exactly as written.

## Verification

- [x] `cargo build -p spark-tui` compiles
- [x] `cargo test -p spark-tui scroll` passes all 14 tests
- [x] `cargo test -p spark-tui` passes (333 tests total, no regressions)
- [x] scroll.rs exported from state/mod.rs with all public functions

## Commit

- `02b377b`: feat(04-01): add scroll module with core operations

## Next Phase Readiness

Ready for 04-02 (Keyboard Handlers):
- scroll_by, scroll_to_top/bottom available for arrow/page/home/end handlers
- LINE_SCROLL and PAGE_SCROLL_FACTOR constants ready
- Boolean return enables "did scroll" feedback for event handling
