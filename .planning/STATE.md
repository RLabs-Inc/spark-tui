# spark-tui State

## Project Reference

See: .planning/PROJECT.md (updated 2026-01-22)

**Core value:** Reactive correctness AND TypeScript-like ergonomics
**Current focus:** Phase 5 - Cursor System (COMPLETE)

---

## Current Position

**Phase:** 5 of 6 (Cursor System)
**Plan:** 4 of 4 complete
**Status:** Phase Complete

Last activity: 2026-01-23 - Completed 05-04-PLAN.md

Progress: [####################] 100% (21/21 total plans)

---

## Current Phase

**Phase 5: Cursor System**

Status: COMPLETE (4/4 plans complete)

### Requirements Progress
- [x] R5.1: Blink animation module (05-01)
- [x] R5.2: Terminal cursor API + arrays (05-02)
- [x] R5.3: Drawn cursor module (05-03)
- [x] R5.4: Pipeline integration (05-04)

### Plans
- [x] 05-01: Blink animation module
- [x] 05-02: Terminal cursor API and arrays
- [x] 05-03: Drawn cursor module
- [x] 05-04: Pipeline integration

---

## Progress Summary

| Phase | Status | Progress |
|-------|--------|----------|
| 1. Mouse + Events | Complete | 100% (4/4) |
| 2. Theme System | Complete | 100% (4/4) |
| 3. Input Component | Complete | 100% (4/4) |
| 4. Scroll System | Complete | 100% (5/5) |
| 5. Cursor System | Complete | 100% (4/4) |
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
| crates-io-signals | Use spark-signals = "0.1.0" from crates.io | Published crate for production setup | 2026-01-22 |
| get-reactive-theme | Renamed reactive_theme() to get_reactive_theme() | Avoid conflict with derive macro generated function | 2026-01-22 |
| accessor-stores-signals | ThemeAccessor stores Signal<ThemeColor> not Derived | Simpler with published spark-signals API | 2026-01-22 |
| two-accessor-methods | .primary() returns Rgba, .primary_signal() returns Signal | Ergonomic access + reactive tracking support | 2026-01-22 |
| oklch-contrast-direction | Use OKLCH lightness for contrast direction instead of relative luminance | Consistency with OKLCH-based adjustments | 2026-01-22 |
| contrast-fallback | Try opposite direction if initial fails to achieve contrast | Handles white-on-medium-bg edge cases | 2026-01-22 |
| input-props-new | InputProps::new(value) instead of Default | Signal<String> has no Default impl | 2026-01-22 |
| input-always-focusable | Inputs always focusable (no focusable prop) | Matches expected input behavior | 2026-01-22 |
| display-text-getter | Display text computed via getter for placeholder+masking | Ensures reactive correctness | 2026-01-22 |
| word-boundary-alphanumeric | Word boundary uses is_alphanumeric() | Simple, handles Unicode, matches common editor behavior | 2026-01-22 |
| punctuation-word-separator | Punctuation treated as word separator like whitespace | Consistent with most text editors | 2026-01-22 |
| internal-clipboard-buffer | Clipboard uses internal thread_local buffer | No external dependencies, works in all environments | 2026-01-22 |
| navigation-clears-selection | Arrow without Shift clears selection and moves to boundary | Matches standard editor behavior | 2026-01-22 |
| history-position-negative-one | InputHistory uses position=-1 for "not browsing" state | Clear sentinel value, allows >= 0 check for browsing | 2026-01-22 |
| history-skip-empty-duplicates | History push skips empty entries and consecutive duplicates | Cleaner history, matches shell behavior | 2026-01-22 |
| scroll-offset-interaction-array | Scroll offset stored in interaction arrays | Renderer can access without prop drilling | 2026-01-22 |
| keyboard-no-chaining | Keyboard scroll does NOT chain to parent | Would conflict with focus management; mouse wheel chains, keyboard doesn't | 2026-01-22 |
| arrow-key-conditions | Arrow keys only scroll without Ctrl/Alt modifiers | Ctrl+Arrow used for word navigation in inputs | 2026-01-22 |
| scrollbar-inside-borders | Scrollbar on right edge inside borders | Standard UI convention, visible with content | 2026-01-22 |
| overflow-scroll-full-bar | overflow:scroll shows full scrollbar (track + thumb) | Full track + thumb for explicit scroll mode | 2026-01-22 |
| overflow-auto-indicator | overflow:auto shows minimal scroll indicator | Less intrusive for auto mode | 2026-01-22 |
| prev-max-for-growth | prev_max_scroll_y for content growth detection | Compare before/after to detect content addition | 2026-01-22 |
| global-layout-accessor | Thread-local cache with set/get/clear for layout | Scroll handlers can't receive layout as parameter from keyboard/mouse dispatch | 2026-01-22 |
| cursor-shape-vs-style | Keep CursorShape (ansi.rs) and CursorStyle (types.rs) separate | They serve different purposes: Shape for terminal control, Style for component config | 2026-01-23 |
| atomic-blink-phase | Use Arc<AtomicBool> for cross-thread blink phase | Signal<T> uses Rc<RefCell> which isn't Send | 2026-01-23 |
| box-tests-for-cursor | Use Box primitives in drawn_cursor tests | Input creates its own cursor, tests need focusable-only components | 2026-01-23 |

---

## Session Log

### 2026-01-23 — Plan 05-04 Execution
- Added render_input_selection() with INVERSE highlighting
- Added render_input_cursor() supporting Block/Bar/Underline styles
- Updated render_input() to integrate selection and cursor rendering
- Selection respects scroll offset and clips to content area
- Cursor only renders when focused AND visible (respects blink)
- All 421 tests pass (402 unit + 19 doc)
- Phase 5 Complete!

### 2026-01-23 — Plan 05-03 Execution
- Created src/state/drawn_cursor.rs (587 lines) with DrawnCursor control object
- DrawnCursorConfig for style, char, blink, fps, alt_char configuration
- create_cursor() sets arrays and creates cursor_visible getter closure
- Focus callbacks trigger blink subscribe (on_focus) / unsubscribe (on_blur)
- cursor_visible getter checks: manual override -> focus state -> blink phase
- dispose_cursor() for cleanup
- Integrated into Input component: creates cursor on mount, disposes on cleanup
- Fixed tests to use Box primitives (Input now creates its own cursor)
- 8 new tests for drawn cursor module
- All 421 tests pass (402 unit + 19 doc)

### 2026-01-23 — Plan 05-02 Execution
- Created src/state/cursor.rs (343 lines) with terminal cursor API
- cursor_show/hide, cursor_move_to, cursor_set_shape, cursor_save/restore
- State query: cursor_is_visible, cursor_position, cursor_shape, cursor_is_blinking
- Thread-local state tracking for persistence
- Fixed animate.rs: Changed to Arc<AtomicBool> for cross-thread blink phase
- Added CURSOR_CHAR, CURSOR_ALT_CHAR, CURSOR_STYLE arrays to interaction.rs
- Added 6 accessor functions for cursor arrays
- 8 new tests (5 cursor + 3 interaction arrays)
- All 413 tests pass (394 unit + 19 doc)

### 2026-01-22 — Plan 04-05 Execution (Gap Closure)
- Added global layout accessor to layout_derived.rs (set_layout, get_layout, try_get_layout, clear_layout)
- Render effect in mount.rs now calls set_layout() after layout computation
- Refactored scroll handlers to use get_layout() internally (no layout parameter)
- Fixed mouse wheel focused fallback to use chaining (was missing)
- Added stick_to_bottom reactive effect in Box component
- Effect watches layout via try_get_layout() and calls handle_stick_to_bottom()
- Effect cleans up when Box is destroyed
- Updated global_keys.rs, mouse.rs, focus.rs to use new scroll API
- All 379 tests pass

### 2026-01-22 — Plan 04-04 Execution
- Added STICK_TO_BOTTOM and PREV_MAX_SCROLL_Y arrays to interaction.rs
- Implemented scrollbar rendering: render_scrollbar, render_full_scrollbar, render_scroll_indicator
- overflow:scroll shows track (o) and thumb (X)
- overflow:auto shows position indicator (|)
- Added stick_to_bottom prop to BoxProps
- handle_stick_to_bottom for auto-scroll on content growth
- update_stick_to_bottom_on_scroll for user scroll handling
- is_at_bottom helper
- 5 new tests for stick_to_bottom
- All 357 tests pass

### 2026-01-22 — Plan 04-03 Execution
- Wired mouse wheel scroll into dispatch_scroll as default behavior
- Added find_scrollable_ancestor to walk parent chain
- Added scroll_focused_into_view high-level helper
- Focus changes now trigger scrollIntoView automatically
- Added get_component_at alias for hit_test
- 13 new tests for mouse scroll and scrollIntoView
- All 370 tests pass

### 2026-01-22 — Plan 04-02 Execution
- Added layout accessor pattern: set_current_layout, with_current_layout, clear_current_layout
- Added keyboard scroll handlers: handle_arrow_scroll, handle_page_scroll, handle_home_end
- Added get_focused_scrollable helper
- Added handle_wheel_scroll for mouse wheel (with chaining)
- Added scroll_into_view for focus visibility
- Extended GlobalKeysHandle with scroll_cleanup
- Wired keyboard scroll handlers into setup_global_keys()
- Arrow keys, PageUp/Down, Ctrl+Home/End
- 20 new tests (14 scroll + 6 global_keys)
- All 352 tests pass

### 2026-01-22 — Plan 04-01 Execution
- Created src/state/scroll.rs (409 lines)
- Scroll constants: LINE_SCROLL, WHEEL_SCROLL, PAGE_SCROLL_FACTOR
- State access: is_scrollable, get_scroll_offset, get_max_scroll
- Operations: set_scroll_offset, scroll_by, scroll_to_top/bottom/start/end
- Chaining: scroll_by_with_chaining for parent fallback
- scroll_by returns bool for boundary detection
- 14 new tests for scroll module
- All 333 tests pass

### 2026-01-22 — Plan 03-04 Execution
- Added InputHistory struct with navigation methods to types.rs
- Added history prop to InputProps
- Up/Down arrow history navigation in keyboard handler
- Auto-add to history on Enter (submit)
- ensure_cursor_visible helper for scroll offset
- Scroll offset tracking in interaction arrays
- 13 new tests for history and scroll offset
- All 319 tests pass

### 2026-01-22 — Plan 03-03 Execution
- Created src/state/clipboard.rs with copy/paste/cut functions
- Added selection helpers to input.rs (has_selection, get_selected_text, delete_selection)
- Shift+Arrow character selection
- Shift+Ctrl+Arrow word selection
- Shift+Home/End boundary selection
- Ctrl+C copies, Ctrl+V pastes, Ctrl+X cuts
- Typing with selection replaces selection
- 21 new tests for clipboard and selection
- All 306 tests pass

### 2026-01-22 — Plan 03-02 Execution
- Added find_word_start/find_word_end helpers to input.rs
- Enhanced keyboard handler with Ctrl+ combinations
- Ctrl+Left/Right word navigation
- Ctrl+Backspace/Delete word deletion
- Ctrl+A select all (sets selection range)
- Added selection getters/setters to interaction.rs
- 10 new tests for word boundary helpers
- All 285 tests pass

### 2026-01-22 — Plan 03-01 Execution
- Added CursorStyle enum (Block, Bar, Underline) to types.rs
- Added BlinkConfig, CursorConfig, InputProps structs to primitives/types.rs
- Added callback type aliases (InputChangeCallback, etc.)
- Created src/primitives/input.rs (640 lines)
- Input component with two-way value binding, keyboard handling
- Placeholder, password mode, cursor position tracking
- 7 new input tests
- All 275 tests pass

### 2026-01-22 — Plan 02-04 Execution
- Verified modifiers.rs complete (lighten/darken/alpha/mix/contrast)
- Added contrast() and contrast_with() to ThemeAccessor
- Created ModifiableColor for chainable color manipulation
- Added ResolvedTheme and resolved_theme() to reactive.rs
- Fixed ensure_contrast() to try opposite direction on failure
- 9 new accessor tests
- All 268 tests pass

### 2026-01-22 — Plan 02-03 Execution
- Created src/theme/variant.rs with Variant enum (14 variants)
- VariantStyle struct with fg/bg/border/border_focus
- get_variant_style() with automatic WCAG contrast calculation
- variant_style() reactive derived
- All tests pass

### 2026-01-22 — Plan 02-02 Execution
- Updated Cargo.toml to spark-signals = "0.1.0" from crates.io
- Added #[derive(Reactive)] to Theme struct
- Created src/theme/reactive.rs with ReactiveTheme state
- Created src/theme/accessor.rs with ThemeAccessor and t() function
- Fine-grained reactivity proven: changing primary doesn't trigger secondary effects
- 9 new tests (4 reactive + 5 accessor)
- All 224 tests pass

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

Last session: 2026-01-23 14:35 UTC
Stopped at: Completed 05-04-PLAN.md - Phase 5 Complete
Resume file: None - ready for Phase 6

---

## Blockers

None currently.

---

## Notes

- TypeScript reference at `/Users/rusty/Documents/Projects/TUI/tui/`
- Spec files at `crates/tui/docs/specs/` are comprehensive
- spark-signals now from crates.io (0.1.0) instead of path dependency
- TDD approach: write tests first
- Phase 1 complete!
- Phase 2 complete! 268 tests total.
- Phase 3 complete! 319 tests total.
- Phase 4 complete! 398 tests total. (gap closure: 379 -> 398)
- Phase 5 complete! 421 tests total.

---

*Last updated: 2026-01-23*
