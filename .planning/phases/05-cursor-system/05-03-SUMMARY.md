---
phase: 05-cursor-system
plan: 03
type: summary
subsystem: cursor
tags: [cursor, blink, focus, input, animation]

dependency_graph:
  requires:
    - 05-01 (animate.rs - blink animation system)
    - 05-02 (cursor arrays in interaction.rs)
  provides:
    - Drawn cursor module with blink integration
    - DrawnCursor control object
    - DrawnCursorConfig for configuration
  affects:
    - 05-04 (pipeline integration)

tech_stack:
  added: []
  patterns:
    - focus callbacks for imperative blink subscribe/unsubscribe
    - cursor_visible getter closure for reactive visibility
    - registry pattern for active cursor tracking

key_files:
  created:
    - src/state/drawn_cursor.rs (587 lines)
  modified:
    - src/state/mod.rs
    - src/primitives/input.rs

decisions:
  - id: box-tests
    choice: Use Box primitives instead of Input in drawn_cursor tests
    reason: Input now creates its own cursor, so tests need components without cursors

metrics:
  duration: ~15 minutes
  completed: 2026-01-23
  tests_added: 8
  total_tests: 421
---

# Phase 5 Plan 3: Drawn Cursor Module Summary

Drawn cursor management for input components with blink animation integration.

## One-Liner

DrawnCursor module with focus-triggered blink subscription and cursor_visible getter closure

## What Was Built

### DrawnCursorConfig
Configuration struct for creating a drawn cursor:
- `style: CursorStyle` - Block, Bar, or Underline
- `char: Option<char>` - Custom character (overrides style)
- `blink: bool` - Enable blink animation (default: true)
- `fps: u8` - Blink FPS (default: 2 = 500ms cycle)
- `alt_char: Option<char>` - Character for blink "off" phase

### DrawnCursor Control Object
Returned by `create_cursor()`:
- `set_position(pos)` - Set cursor position in text
- `get_position()` - Get current position
- `show()` / `hide()` - Manual visibility override
- `clear_override()` - Return to blink-controlled visibility
- `is_visible()` - Check current visibility
- `dispose()` - Clean up resources

### Core Functions
- `create_cursor(index, config)` - Create and register a cursor
- `dispose_cursor(index)` - Clean up cursor resources
- `has_cursor(index)` - Check if component has active cursor
- `reset_cursors()` - Reset all active cursors (for testing)

### Cursor Visibility Logic
The cursor_visible getter is a closure that evaluates:
1. **Manual override** - If `show()`/`hide()` was called, use that
2. **Focus state** - If not focused, always visible (no blink)
3. **Blink phase** - If focused and blink enabled, return blink phase

### Focus Integration
Focus callbacks manage blink subscription imperatively:
- `on_focus`: Subscribe to blink clock at configured FPS
- `on_blur`: Unsubscribe from blink clock

### Input Component Integration
Input now creates a drawn cursor on mount:
- Reads `props.cursor` for configuration (or uses defaults)
- Creates cursor with `drawn_cursor::create_cursor()`
- Disposes cursor in cleanup with `drawn_cursor::dispose_cursor()`

## Implementation Notes

### Active Cursors Registry
Thread-local HashMap tracks active cursors by component index:
- `unsubscribe_blink` - RefCell for blink unsubscribe function
- `unsubscribe_focus` - RefCell for focus callback cleanup
- `manual_visible` - Signal<Option<bool>> for manual override
- `blink_enabled` - Whether blink is enabled for this cursor
- `fps` - Blink FPS for this cursor

### Cursor Character Constants
- `CURSOR_CHAR_BLOCK = 0` - Block cursor (inverse rendering)
- `CURSOR_CHAR_BAR = 0x2502` - Bar cursor (|)
- `CURSOR_CHAR_UNDERLINE = 0x5F` - Underline cursor (_)

## Commits

| Hash | Type | Description |
|------|------|-------------|
| 2a34c38 | feat | Add drawn cursor module with blink integration |
| 57c2af5 | feat | Integrate drawn cursor into Input component |
| 65794e8 | test | Fix drawn cursor tests to use Box primitives |

## Test Coverage

8 new tests added:
- `test_create_cursor_sets_arrays` - Verify cursor arrays are set correctly
- `test_cursor_visible_when_not_focused` - Cursor visible when not focused
- `test_cursor_blinks_when_focused` - Focus triggers blink subscription
- `test_cursor_stops_blink_on_blur` - Blur unsubscribes from blink
- `test_dispose_cursor_cleans_up` - Cleanup removes cursor and clears arrays
- `test_manual_show_hide_override` - show/hide override blink
- `test_cursor_styles` - Block, Bar, Underline set correct chars
- `test_custom_cursor_char` - Custom char overrides style preset

Total tests: 421 (402 unit + 19 doc)

## Deviations from Plan

### Test Updates
**[Rule 1 - Bug] Fixed tests using Input instead of Box**
- Tests were using Input primitives which now create their own cursor
- Changed to Box primitives for focusable-only components
- No functional change, just test isolation

## Dependencies Used

From Wave 1:
- `animate::subscribe_to_blink()` - Subscribe to blink clock
- `animate::get_blink_phase()` - Get current blink visibility
- `focus::register_callbacks()` - Register on_focus/on_blur callbacks
- `interaction::set_cursor_visible_getter()` - Set visibility getter closure
- Cursor arrays from 05-02: `set_cursor_char`, `set_cursor_alt_char`, `set_cursor_style`

## Next Phase Readiness

Plan 05-04 (Pipeline Integration) can now proceed:
- Drawn cursor module provides visibility getter
- Cursor arrays are set by create_cursor
- Renderer can read `get_cursor_visible()` for rendering decisions
- Blink animation is working and tested
