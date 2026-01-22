# spark-tui State

## Project Reference

See: .planning/PROJECT.md (updated 2026-01-22)

**Core value:** Reactive correctness AND TypeScript-like ergonomics
**Current focus:** Phase 2 - Theme System (IN PROGRESS)

---

## Current Position

**Phase:** 2 of 6 (Theme System)
**Plan:** 1 of 4 complete
**Status:** In progress

Last activity: 2026-01-22 - Completed 02-01-PLAN.md

Progress: [####------] 42% (5/12 total plans)

---

## Current Phase

**Phase 2: Theme System**

Status: IN PROGRESS (1/4 plans complete)

### Requirements Progress
- [x] R2.1: ThemeColor type (02-01)
- [x] R2.2: Theme struct with 20 semantic colors (02-01)
- [x] R2.3: 13 preset themes (02-01)
- [ ] R2.4: Reactive theme state
- [ ] R2.5: Color resolution
- [ ] R2.6: t.* accessor deriveds
- [ ] R2.7: Variant system

### Plans
- [x] 02-01: Theme types, ThemeColor, Theme struct, 13 presets
- [ ] 02-02: Reactive theme state
- [ ] 02-03: Color resolution and t.* accessors
- [ ] 02-04: Variant system

---

## Progress Summary

| Phase | Status | Progress |
|-------|--------|----------|
| 1. Mouse + Events | Complete | 100% (4/4) |
| 2. Theme System | In Progress | 25% (1/4) |
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
| global-keys-on | Use keyboard::on() for global handlers to access modifiers | on_key() doesn't expose modifier state needed for Ctrl+C, Shift+Tab | 2026-01-22 |
| tick-60fps | 16ms poll timeout for ~60fps event loop | Balance responsiveness with CPU usage | 2026-01-22 |
| theme-color-enum | ThemeColor enum with Default/Ansi/Rgb/Str variants | Matches TypeScript semantics exactly | 2026-01-22 |
| preset-functions | Functions returning Theme instead of const values | Theme has String fields requiring allocation | 2026-01-22 |
| case-insensitive-lookup | get_preset() normalizes to lowercase and strips underscores | User-friendly API | 2026-01-22 |

---

## Session Log

### 2026-01-22 — Plan 02-01 Execution
- Added 24 comprehensive Rgba color parsing tests
- Created src/theme/mod.rs with ThemeColor enum and Theme struct
- Created src/theme/presets.rs with all 13 TypeScript presets
- ThemeColor supports Default, Ansi, Rgb, Str variants
- Theme has 20 semantic color slots
- get_preset() with case-insensitive lookup
- 56 new tests (24 Rgba + 16 ThemeColor + 16 preset)
- All 224 tests pass (215 unit + 9 doc)

### 2026-01-22 — Plan 01-04 Execution
- Created src/state/global_keys.rs with GlobalKeysHandle
- setup_global_keys() for Ctrl+C, Tab, Shift+Tab handlers
- Integrated event loop into mount.rs (tick/run functions)
- Mouse capture enabled on mount, disabled on unmount
- 5 new tests for global keys
- All 159 tests pass

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

Last session: 2026-01-22 16:23 UTC
Stopped at: Completed 02-01-PLAN.md
Resume file: None - continue with 02-02-PLAN.md

---

## Blockers

None currently.

---

## Notes

- TypeScript reference at `/Users/rusty/Documents/Projects/TUI/tui/`
- Spec files at `crates/tui/docs/specs/` are comprehensive
- spark-signals (crates/signals/) is complete and production-ready
- TDD approach: write tests first
- Phase 1 complete! Phase 2 in progress.

---

*Last updated: 2026-01-22*
