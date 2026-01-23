---
phase: 05
plan: 02
status: complete
subsystem: cursor-system
tags: [cursor, terminal, ansi, arrays, interaction]

dependency-graph:
  requires: []
  provides:
    - terminal-native-cursor-api
    - cursor-character-arrays
    - cursor-style-accessors
  affects:
    - 05-03 (drawn cursor module uses cursor arrays)
    - 05-04 (integration uses cursor state)

tech-stack:
  added: []
  patterns:
    - thread-local-state-tracking
    - ansi-wrapper-api

file-tracking:
  created:
    - src/state/cursor.rs
    - src/state/animate.rs (fixed)
  modified:
    - src/state/mod.rs
    - src/engine/arrays/interaction.rs

decisions:
  - id: cursor-shape-vs-style
    choice: "Keep CursorShape (ansi.rs) and CursorStyle (types.rs) separate"
    rationale: "They serve different purposes: Shape for terminal control, Style for component config"

metrics:
  duration: ~7 minutes
  completed: 2026-01-23
---

# Phase 05 Plan 02: Terminal Cursor API and Arrays Summary

Terminal native cursor API wrapper with thread-local state tracking, plus cursor character arrays for drawn cursor configuration.

## What Was Built

### 1. Terminal Cursor Module (`src/state/cursor.rs`)

Created a clean API for terminal cursor control:

**Visibility Control:**
- `cursor_show()` - Show terminal cursor
- `cursor_hide()` - Hide terminal cursor

**Position Control:**
- `cursor_move_to(x, y)` - Position cursor (0-indexed)

**Shape Control:**
- `cursor_set_shape(shape, blinking)` - Set Block/Bar/Underline

**Save/Restore:**
- `cursor_save()` - Save position (DEC sequence)
- `cursor_restore()` - Restore position (DEC sequence)

**State Query:**
- `cursor_is_visible()` - Check visibility state
- `cursor_position()` - Get (x, y) position
- `cursor_shape()` - Get current shape
- `cursor_is_blinking()` - Check blink state

**Testing:**
- `reset_cursor_state()` - Reset to defaults

All functions:
1. Update internal thread-local state
2. Call appropriate ANSI helper from `ansi.rs`
3. Flush stdout immediately

### 2. Cursor Character Arrays (`src/engine/arrays/interaction.rs`)

Added three new arrays for drawn cursor configuration:

| Array | Type | Default | Purpose |
|-------|------|---------|---------|
| CURSOR_CHAR | u32 | 0 | Cursor character codepoint (0=block inverse, 0x2502=bar, 0x5F=underline) |
| CURSOR_ALT_CHAR | u32 | 0 | Alternate char for blink off phase (0=show original text) |
| CURSOR_STYLE | u8 | 0 | Style enum (0=Block, 1=Bar, 2=Underline) |

**All 6 accessor functions:**
- `get_cursor_char(index) -> u32`
- `set_cursor_char(index, char)`
- `get_cursor_alt_char(index) -> u32`
- `set_cursor_alt_char(index, char)`
- `get_cursor_style(index) -> u8`
- `set_cursor_style(index, style)`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed animate.rs thread-safety issue**

- **Found during:** Build verification
- **Issue:** `Signal<T>` uses `Rc<RefCell>` internally, cannot be sent between threads. The animate module from Plan 05-01 was spawning a background thread with a Signal.
- **Fix:** Changed to use `Arc<AtomicBool>` for cross-thread communication, syncing to Signal when `get_blink_phase()` is called.
- **Files modified:** src/state/animate.rs
- **Commit:** 947123c

## Test Results

- 5 new tests for state::cursor module
- 3 new tests for interaction arrays (cursor_char, cursor_alt_char, cursor_style)
- All 394 unit tests + 19 doc tests = 413 total tests pass

## Key Files

| File | Lines | Purpose |
|------|-------|---------|
| src/state/cursor.rs | 343 | Terminal native cursor API |
| src/engine/arrays/interaction.rs | 466 | Interaction arrays with cursor arrays |

## Links to ANSI Module

The cursor module wraps `src/renderer/ansi.rs`:

```rust
use crate::renderer::ansi;

// Uses:
// - ansi::cursor_show()
// - ansi::cursor_hide()
// - ansi::cursor_to()
// - ansi::cursor_shape()
// - ansi::cursor_save()
// - ansi::cursor_restore()
```

## Next Phase Readiness

**Plan 05-03: Drawn Cursor Module** is unblocked:
- Cursor character arrays exist with all accessors
- Cursor style can be read/written per-component
- Blink animation infrastructure fixed

**Commits for this plan:**
1. b1e3c17: feat(05-02): create terminal cursor module
2. 947123c: fix(05-02): use atomic for cross-thread blink phase sync
3. b473445: feat(05-02): add cursor character arrays with accessors
