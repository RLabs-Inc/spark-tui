# spark-tui State

## Project Reference

See: .planning/PROJECT.md (updated 2026-01-22)

**Core value:** Reactive correctness AND TypeScript-like ergonomics
**Current focus:** Phase 1 - Mouse System + Event Wiring (COMPLETE)

---

## Current Position

**Phase:** 1 of 6 (Mouse System + Event Wiring) - COMPLETE
**Plan:** 3 of 3 complete
**Status:** Phase complete, ready for Phase 2

Last activity: 2026-01-22 - Completed 01-03-PLAN.md

Progress: [##########] 100% (Phase 1)

---

## Current Phase

**Phase 1: Mouse System + Event Wiring**

Status: COMPLETE (3/3 plans complete)

### Requirements Progress
- [x] R1.1: HitGrid for O(1) coordinate lookup (01-01)
- [x] R1.2: Mouse event dispatch (01-01)
- [x] R1.3: Hover tracking (enter/leave) (01-01)
- [x] R1.4: Click detection (01-01)
- [x] R1.5: Event conversion and polling (01-02)
- [x] R1.6: Event callback wiring (01-03)

### Plans
- [x] 01-01: Mouse types, HitGrid, handlers, dispatch
- [x] 01-02: Input module with event conversion and polling
- [x] 01-03: Event callback wiring

---

## Progress Summary

| Phase | Status | Progress |
|-------|--------|----------|
| 1. Mouse + Events | Complete | 100% (3/3) |
| 2. Theme System | Not Started | 0% |
| 3. Input Component | Not Started | 0% |
| 4. Scroll System | Not Started | 0% |
| 5. Cursor System | Not Started | 0% |
| 6. Control Flow | Not Started | 0% |

---

## Decisions Made

| ID | Decision | Rationale | Date |
|----|----------|-----------|------|
| hitgrid-location | HitGrid in mouse.rs with global thread_local! | Centralized state, dispatch can access without params | 2026-01-22 |
| handler-pattern | Mirror keyboard.rs registry pattern | Consistency, proven cleanup pattern | 2026-01-22 |
| click-detection | Track pressed component+button, compare on up | Matches TypeScript exactly | 2026-01-22 |
| scroll-info-struct | Use ScrollInfo struct matching mouse.rs | Consistency with existing mouse module types | 2026-01-22 |
| meta-key-false | Meta key always false in convert_modifiers | crossterm doesn't expose meta key state | 2026-01-22 |
| rc-callbacks | Use Rc<dyn Fn> instead of Box<dyn Fn> for MouseHandlers | Allows cloning callbacks into closures (e.g., click-to-focus) | 2026-01-22 |
| click-to-focus-wrap | Wrap user on_click with focus::focus() for focusable boxes | Automatic focus on click matches expected behavior | 2026-01-22 |

---

## Session Log

### 2026-01-22 — Plan 01-03 Execution
- Updated src/primitives/types.rs with callback type aliases
- Added mouse/keyboard callback props to BoxProps
- Added on_click to TextProps
- Updated src/primitives/box_primitive.rs with handler registration
- Implemented click-to-focus for focusable boxes
- Updated src/primitives/text.rs with on_click wiring
- Updated src/state/mouse.rs to use Rc<dyn Fn> for handlers
- All 153 tests pass

### 2026-01-22 — Plan 01-02 Execution
- Created src/state/input.rs (532 lines)
- convert_mouse_event, convert_key_event conversions
- InputEvent unified enum for all terminal events
- poll_event, read_event, route_event API
- enable_mouse/disable_mouse for mouse capture
- Made focus/keyboard modules public (blocking fix)
- Fixed focus function call path (bug fix)
- 17 new tests, all passing
- Total: 153 tests pass

### 2026-01-22 — Plan 01-01 Execution
- Created src/state/mouse.rs (1134 lines)
- MouseEvent, MouseAction, MouseButton, ScrollDirection types
- HitGrid with O(1) lookup, moved from mount.rs
- Handler registry with cleanup closures
- dispatch() with hover/click detection
- 14 new tests, all passing
- Updated mount.rs to use mouse module
- Total: 136 tests pass

### 2026-01-22 — GSD Initialization
- Created PROJECT.md with core values and requirements
- Created REQUIREMENTS.md with detailed specs for all 6 phases
- Created ROADMAP.md with phase dependencies and execution order
- Created STATE.md (this file)
- Ready to begin Phase 1

---

## Session Continuity

Last session: 2026-01-22 13:38 UTC
Stopped at: Completed Phase 1 (all 3 plans)
Resume file: None - Phase 1 complete, ready for Phase 2

---

## Blockers

None currently.

---

## Notes

- TypeScript reference at `/Users/rusty/Documents/Projects/TUI/tui/`
- Spec files at `crates/tui/docs/specs/` are comprehensive
- spark-signals (crates/signals/) is complete and production-ready
- TDD approach: write tests first
- Phase 1 complete! Mouse system is fully wired up.

---

*Last updated: 2026-01-22*
