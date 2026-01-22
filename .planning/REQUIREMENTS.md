# spark-tui Requirements

## Overview

Complete port of TypeScript TUI framework to Rust with identical ergonomics and reactive correctness.

## R1: Mouse System + Event Wiring

**Goal:** Enable components to respond to mouse interactions.

### R1.1: HitGrid
- O(1) coordinate-to-component lookup
- Rebuild on layout changes (derived from ComputedLayout)
- Handle overlapping components via z-index
- Transparent components pass through to components below

### R1.2: Mouse Event Dispatch
- Parse SGR mouse protocol from terminal input
- MouseEvent struct: button, position, modifiers, event_type
- Event types: down, up, move, scroll (wheel up/down)
- Dispatch to component at coordinates via HitGrid

### R1.3: Hover Tracking
- Track currently hovered component index
- Fire onMouseEnter when hover changes to component
- Fire onMouseLeave when hover leaves component
- Handle nested components (deepest match wins)

### R1.4: Click Detection
- Track mousedown component
- Click = mousedown + mouseup on same component
- Fire onClick after mouseup if matched
- Support onMouseDown, onMouseUp separately

### R1.5: Event Callback Wiring
- Connect Box onClick, onMouseDown, onMouseUp, onMouseEnter, onMouseLeave, onScroll
- Connect Text onClick (clickable text)
- Connect onKey callbacks to focused component
- Global key handlers (Ctrl+C at mount level)

**Status:** COMPLETE (2026-01-22)

**Success Criteria:**
- [x] Click on Box fires onClick callback
- [x] Hover over Box fires onMouseEnter/Leave
- [x] Mouse wheel on scrollable Box fires onScroll
- [x] Keyboard input reaches focused component's onKey

---

## R2: Theme System

**Goal:** Enable visual customization with reactive theme colors.

### R2.1: Color Types
- Rgba struct with r, g, b, a (0-255)
- DEFAULT sentinel (-1) for "use inherited/theme default"
- ANSI sentinel (-2) for "use terminal ANSI palette"
- Support RGB hex parsing (#RRGGBB, #RGB)

### R2.2: Theme Structure
- Theme struct with semantic color signals
- Colors: primary, secondary, success, warning, error, info
- Backgrounds: bg, bgSubtle, bgMuted
- Foregrounds: fg, fgSubtle, fgMuted
- Borders: border, borderSubtle
- Focus: focusRing

### R2.3: Theme Presets
- terminal preset (DEFAULT colors, respects terminal theme)
- dracula preset (dark purple/pink theme)
- nord preset (arctic blue theme)
- Easy to add more presets

### R2.4: Theme Accessor
- t.* pattern: t.primary, t.bg, t.fg, etc.
- Returns Signal<Rgba> for reactive binding
- Changing active theme updates all bound colors

### R2.5: Color Inheritance
- Components inherit fg/bg from parent if not set
- Inheritance follows component tree (parentIndex)
- DEFAULT sentinel triggers inheritance lookup

**Success Criteria:**
- [ ] `t.primary` returns reactive Signal<Rgba>
- [ ] Changing theme updates all components using t.* colors
- [ ] DEFAULT color inherits from parent
- [ ] At least 3 presets working (terminal, dracula, nord)

---

## R3: Input Component

**Goal:** Text entry primitive with full editing capabilities.

### R3.1: Value Binding
- value prop accepts Signal<String> for two-way binding
- onChange fires on every character change
- Support controlled and uncontrolled modes

### R3.2: Placeholder
- placeholder prop for empty state text
- placeholderColor for styling (default: fgMuted)
- Hide placeholder when value is non-empty

### R3.3: Password Mode
- password: bool prop
- maskChar: char prop (default: '●')
- Display mask chars instead of actual content

### R3.4: Cursor Management
- Track cursor position (0 to len)
- CursorStyle enum: Block, Bar, Underline
- cursorColor prop for custom color
- Cursor visible only when focused

### R3.5: Keyboard Navigation
- Left/Right: move cursor
- Home/End: jump to start/end
- Backspace: delete before cursor
- Delete: delete after cursor
- Ctrl+A: select all (future)

### R3.6: Events
- onChange(value: String)
- onSubmit(value: String) — Enter key
- onCancel() — Escape key

### R3.7: Auto-focus
- autoFocus prop for initial focus
- maxLength prop for character limit

**Success Criteria:**
- [ ] Type characters and see them appear
- [ ] Cursor moves with arrow keys
- [ ] Backspace/Delete work correctly
- [ ] Enter fires onSubmit, Escape fires onCancel
- [ ] Password mode masks characters
- [ ] Two-way binding updates external Signal

---

## R4: Scroll System

**Goal:** Handle content overflow with scrolling.

### R4.1: Overflow Modes
- visible: content extends beyond bounds (default)
- hidden: content clipped, no scroll
- scroll: always show scrollbar area
- auto: scrollbar only when content overflows

### R4.2: ScrollManager
- Per-component scroll state
- scrollX, scrollY: current scroll offset
- maxScrollX, maxScrollY: computed from content size
- Clamp scroll to valid range

### R4.3: Keyboard Scrolling
- Arrow keys: scroll by 1 line/char
- PageUp/PageDown: scroll by viewport height
- Home/End: scroll to start/end
- Only when component is focused

### R4.4: Mouse Wheel Scrolling
- Wheel up: scroll up
- Wheel down: scroll down
- Scroll amount configurable (default: 3 lines)

### R4.5: Scroll Chaining
- If at boundary (scrollY=0), propagate to parent
- Parent handles scroll if it has overflow
- Chain continues until handled or root

### R4.6: scrollIntoView
- On focus change, scroll to make focused component visible
- Smooth scrolling optional
- Respect component padding

**Success Criteria:**
- [ ] Content larger than container scrolls
- [ ] Arrow keys scroll focused scrollable
- [ ] Mouse wheel scrolls hovered scrollable
- [ ] Scroll chaining works at boundaries
- [ ] Focus change auto-scrolls to focused component

---

## R5: Cursor System

**Goal:** Visual cursor feedback for text entry.

### R5.1: Terminal Cursor
- Position terminal cursor at input cursor location
- Hide when no input focused
- Use crossterm cursor positioning

### R5.2: Drawn Cursor
- Render cursor as character in FrameBuffer
- Block: full cell inverse
- Bar: thin vertical line (left side of cell)
- Underline: bottom of cell

### R5.3: Blink Animation
- Shared blink clock (one timer for all)
- Default: 530ms on, 530ms off
- cursorBlink: bool prop to disable
- Only blink when focused

### R5.4: Focus Integration
- Show cursor when input focused
- Hide cursor when input blurred
- Reset blink phase on focus

**Success Criteria:**
- [ ] Cursor visible at correct position in Input
- [ ] Cursor blinks at correct rate
- [ ] Cursor hidden when Input not focused
- [ ] Different cursor styles render correctly

---

## R6: Control Flow

**Goal:** Ergonomic helpers for dynamic UI.

### R6.1: show()
- show(condition, render_fn, else_fn)
- Conditional rendering based on Signal<bool>
- else_fn optional (default: render nothing)
- Components destroyed when condition false

### R6.2: each()
- each(items, render_fn, options)
- items: Signal<Vec<T>> or ReactiveVec<T>
- render_fn(item, index) returns component
- key option for stable identity
- Fine-grained updates (add/remove/reorder)

### R6.3: when() (async)
- when(future, {pending, then, catch})
- pending: render while loading
- then(result): render on success
- catch(error): render on failure
- Handle cleanup on unmount

**Success Criteria:**
- [ ] show() toggles component visibility reactively
- [ ] each() renders list with fine-grained updates
- [ ] each() reorders without recreating all items
- [ ] when() shows pending/then/catch states correctly

---

## Cross-Cutting Requirements

### Testing (All Phases)
- TDD: Write tests before implementation
- Unit tests for core logic
- Integration tests for component behavior
- Edge cases: empty, max values, boundary conditions

### Ergonomics (All Phases)
- TypeScript-like API via macros where needed
- Consistent naming with TypeScript version
- Clear error messages

### Documentation (All Phases)
- Update CLAUDE.md as features complete
- Docstrings on public API
- Examples in docs/
