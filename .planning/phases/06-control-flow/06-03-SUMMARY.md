---
phase: 06-control-flow
plan: 03
subsystem: ui
tags: [async, when, state-machine, control-flow, reactive]

# Dependency graph
requires:
  - phase: 06-01
    provides: "show() conditional rendering pattern and EffectScope cleanup"
  - phase: 06-02
    provides: "each() list rendering pattern"
provides:
  - when() async state rendering function
  - AsyncState<T, E> enum with Pending/Resolved/Rejected variants
  - WhenOptions struct for configuring when() render functions
affects: ["06-04"]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Polling-based async state (user manages async, when() just renders)"
    - "State machine rendering: match AsyncState -> render branch"
    - "Unhandled rejection logging to stderr"

key-files:
  modified:
    - "src/primitives/control_flow.rs"
    - "src/primitives/mod.rs"

key-decisions:
  - "Polling-based rather than Future integration - users manage their own async"
  - "WhenOptions uses struct literal syntax (no new() constructor)"
  - "then_fn receives T by value (ownership transferred), enabling move semantics"
  - "Unhandled rejections logged to stderr without crashing"

patterns-established:
  - "AsyncState enum: Polling-based async state tracking"
  - "WhenOptions struct literal: Direct construction without builder pattern"

# Metrics
duration: 3min
completed: 2026-01-23
---

# Phase 6 Plan 3: when() Async Rendering Summary

**Polling-based when() async state rendering with AsyncState<T, E> enum and 10 comprehensive tests**

## Performance

- **Duration:** 3 min
- **Started:** 2026-01-23T15:57:48Z
- **Completed:** 2026-01-23T16:00:38Z
- **Tasks:** 3
- **Files modified:** 2

## Accomplishments
- AsyncState<T, E> enum with Pending/Resolved/Rejected variants for async state tracking
- WhenOptions struct for configuring pending/then_fn/catch_fn render functions
- when() function with state machine rendering and proper cleanup between state transitions
- 10 comprehensive tests covering all states, transitions, and edge cases

## Task Commits

Each task was committed atomically:

1. **Task 1: AsyncState enum and WhenOptions struct** - `c0b0061` (feat)
2. **Task 2: when() function** - included in `c0b0061` (same file, combined)
3. **Task 3: Comprehensive tests** - `dfbb23b` (test)

## Files Created/Modified
- `src/primitives/control_flow.rs` - Added when(), AsyncState, WhenOptions (1533 lines total)
- `src/primitives/mod.rs` - Added when export

## Decisions Made
- **Polling-based approach:** Users manage their own async operations and update Signal<AsyncState<T, E>>. when() just renders based on current state. This avoids async runtime dependencies and works with any async executor.
- **then_fn receives T by value:** Enables move semantics for owned data. Users can clone if needed.
- **No WhenOptions::new():** Function types don't have sensible Default impls, so direct struct literal construction is cleaner.
- **Unhandled rejections logged:** When catch_fn is None and state is Rejected, error is logged to stderr but no crash occurs.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None - straightforward implementation following established show() and each() patterns.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- All control flow primitives now complete: show(), each(), when()
- Ready for 06-04 integration and edge case testing
- 447 total tests passing (428 unit + 19 doc)

---
*Phase: 06-control-flow*
*Completed: 2026-01-23*
