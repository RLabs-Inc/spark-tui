---
phase: 06-control-flow
plan: 02
subsystem: ui
tags: [control-flow, list-rendering, reactivity, fine-grained, each]

# Dependency graph
requires:
  - phase: 06-01
    provides: show() conditional rendering, EffectScope cleanup pattern
provides:
  - each() list rendering with key-based reconciliation
  - Per-item signals for fine-grained updates
  - Duplicate key warning without crash
affects: [06-03, 06-04]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Key-based reconciliation: track items by unique key for minimal updates"
    - "Per-item signals: each item gets its own Signal for fine-grained reactivity"
    - "Getter pattern: render_fn receives Rc<dyn Fn() -> T> for reactive item access"

key-files:
  created: []
  modified:
    - src/primitives/control_flow.rs
    - src/primitives/mod.rs

key-decisions:
  - "Per-item signals: each item gets its own Signal so updates don't recreate components"
  - "Getter closure pattern: render_fn receives Rc<dyn Fn() -> T> to enable reactive access"
  - "Duplicate key warning: eprintln warning, skip duplicate, no crash (matches TypeScript)"

patterns-established:
  - "each() pattern: items_getter + render_fn + key_fn for dynamic lists"
  - "Fine-grained updates: signal.set() for existing keys, no component recreation"

# Metrics
duration: 2min
completed: 2026-01-23
---

# Phase 06 Plan 02: each() List Rendering Summary

**Key-based list rendering with per-item signals for fine-grained reactivity - new items create components, existing items update signals only**

## Performance

- **Duration:** 2 min 20 sec
- **Started:** 2026-01-23T15:52:58Z
- **Completed:** 2026-01-23T15:55:18Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Implemented each() function with key-based reconciliation
- Per-item signals enable fine-grained updates without component recreation
- Comprehensive test coverage with 9 new tests
- Total: 418 unit tests + 19 doc tests pass

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement each() with key-based reconciliation** - `68f203b` (feat)
2. **Task 2: Add comprehensive tests for each()** - `3c480f7` (test)

## Files Created/Modified

- `src/primitives/control_flow.rs` - Added each() function (1024 lines total)
- `src/primitives/mod.rs` - Exported each from primitives

## Decisions Made

| Decision | Rationale |
|----------|-----------|
| Per-item signals | Each item gets its own Signal so updates don't recreate components |
| Getter closure pattern | render_fn receives `Rc<dyn Fn() -> T>` to enable reactive item access |
| Duplicate key warning | eprintln warning + skip (no crash) matches TypeScript console.warn behavior |

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None - implementation followed TypeScript reference directly.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- each() complete, ready for integration with when() (06-03)
- Control flow primitives now cover conditional (show) and list (each) rendering
- when() will add async handling for complete control flow suite

---
*Phase: 06-control-flow*
*Completed: 2026-01-23*
