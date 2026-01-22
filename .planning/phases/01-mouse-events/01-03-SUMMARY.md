---
phase: 01-mouse-events
plan: 03
subsystem: primitives/events
tags: [mouse, callbacks, events, box, text, click-to-focus]

dependency_graph:
  requires:
    - 01-01 (mouse module with MouseHandlers and on_component)
  provides:
    - MouseCallback, MouseCallbackConsuming, KeyCallback type aliases
    - BoxProps with on_click, on_mouse_down, on_mouse_up, on_mouse_enter, on_mouse_leave, on_scroll, on_key
    - TextProps with on_click
    - Click-to-focus behavior for focusable boxes
    - Handler cleanup on component unmount
  affects:
    - Components using mouse events
    - Full event wiring integration (01-02)

tech_stack:
  added: []
  patterns:
    - Rc<dyn Fn> for callbacks (allows cloning into closures)
    - Handler cleanup via FnOnce closures
    - Click-to-focus wrapping (combines focus + user callback)

key_files:
  created: []
  modified:
    - src/primitives/types.rs
    - src/primitives/box_primitive.rs
    - src/primitives/text.rs
    - src/state/mouse.rs

decisions:
  - id: rc-callbacks
    choice: "Use Rc<dyn Fn> instead of Box<dyn Fn> for MouseHandlers"
    rationale: "Allows cloning callbacks into closures (e.g., for click-to-focus wrapper)"
  - id: non-consuming-callbacks
    choice: "Component handlers (on_click, on_mouse_down, etc.) don't return bool"
    rationale: "Only scroll callbacks need to consume events; simplifies API"
  - id: click-to-focus-wrap
    choice: "Wrap user on_click with focus::focus() for focusable boxes"
    rationale: "Automatic focus on click matches expected behavior; user handler still fires"

metrics:
  duration: "5 minutes"
  completed: "2026-01-22"
---

# Phase 01 Plan 03: Event Callback Wiring Summary

Mouse callback props on BoxProps/TextProps with handler registration in primitives, including click-to-focus for focusable boxes.

## What Was Done

### Task 1: Add Mouse Callback Props to BoxProps
- Added `MouseCallback`, `MouseCallbackConsuming`, `KeyCallback` type aliases using `Rc<dyn Fn>` for shared ownership
- Added event callback fields to BoxProps:
  - `on_click`: Click callback (down + up on same component)
  - `on_mouse_down`: Mouse button pressed
  - `on_mouse_up`: Mouse button released
  - `on_mouse_enter`: Hover starts
  - `on_mouse_leave`: Hover ends
  - `on_scroll`: Mouse wheel (returns bool to consume)
  - `on_key`: Keyboard events when focused (returns bool to consume)
- Updated `MouseHandlers` in mouse.rs to use `Rc<dyn Fn>` types

### Task 2: Wire Mouse Handlers in box_primitive
- Added mouse handler registration via `mouse::on_component()`
- Implemented click-to-focus: when a focusable box is clicked, it automatically gains focus
  - User's `on_click` handler is called after focus is set
- Added keyboard handler registration via `keyboard::on_focused()` when `on_key` is provided
- Cleanup function properly releases:
  - Mouse handlers via cleanup closure
  - Keyboard handlers via cleanup closure
  - Component state via `mouse::cleanup_index()` and `keyboard::cleanup_index()`
  - Component index via `release_index()`

### Task 3: Add onClick to TextProps
- Added `on_click: Option<MouseCallback>` field to TextProps
- Updated Default impl to include `on_click: None`
- Wired click handler in text primitive via `mouse::on_component()`
- Cleanup releases mouse handler and component state

## Technical Notes

### Rc<dyn Fn> Pattern
The callback types use `Rc<dyn Fn>` instead of `Box<dyn Fn>` to allow cloning callbacks into closures without ownership transfer. This is essential for the click-to-focus wrapper, which needs to capture the user's callback.

```rust
// Click-to-focus wrapper captures user callback via Rc::clone
let click_handler = if should_be_focusable {
    Some(Rc::new(move |event| {
        focus::focus(index);  // Focus first
        if let Some(ref handler) = user_on_click {
            handler(event);   // Then call user's handler
        }
    }))
} else {
    props.on_click.clone()  // No wrapping needed
};
```

### Handler Registration Flow
1. Check if any handlers are provided or component is focusable
2. Build click handler with optional click-to-focus wrapper
3. Create `MouseHandlers` struct with all callbacks
4. Register via `mouse::on_component(index, handlers)`
5. Store cleanup closure for later release

### HitRegion Integration Note
HitRegions for click detection are created by `frame_buffer_derived` in the pipeline, not by primitives. When a component registers handlers via `mouse::on_component()`, the mouse dispatch system routes events based on `HitGrid` lookup.

## Commits

| Hash | Description |
|------|-------------|
| 3ad8007 | Add mouse callback props to BoxProps |
| 4a64771 | Wire mouse handlers in box_primitive |
| f3751ef | Add onClick to TextProps and wire in text primitive |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] MouseHandlers needed Rc<dyn Fn> types**
- **Found during:** Task 1
- **Issue:** Plan specified Rc types for callbacks, but existing MouseHandlers used Box<dyn Fn>
- **Fix:** Updated MouseHandlers struct in mouse.rs to use Rc types
- **Files modified:** src/state/mouse.rs
- **Commit:** 3ad8007

**2. [Rule 3 - Blocking] Test compilation failures after Rc change**
- **Found during:** Task 1 verification
- **Issue:** Existing tests in mouse.rs used Box::new() for handlers
- **Fix:** Updated tests to use Rc::new()
- **Files modified:** src/state/mouse.rs
- **Commit:** 3ad8007

## Verification

- `cargo check -p spark-tui` - No compile errors
- `cargo test -p spark-tui -- --test-threads=1` - 153 tests pass
- BoxProps has all mouse callback props
- TextProps has on_click callback
- Handler registration verified via existing mouse tests

## Next Phase Readiness

Phase 1 is now complete with all 3 plans executed:
- 01-01: Mouse types, HitGrid, handlers, dispatch
- 01-02: Mount.rs mouse input integration (parallel with 01-03)
- 01-03: Event callback wiring (this plan)

Ready for Phase 2 (Theme System).
