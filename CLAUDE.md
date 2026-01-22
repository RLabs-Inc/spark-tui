# spark-tui - Reactive Terminal UI Framework for Rust

## Project Vision

spark-tui is a fully reactive terminal UI framework built on [spark-signals](./crates/signals/), our production-ready reactive signals library for Rust. The goal is to achieve TypeScript-like ergonomics with Rust's performance and safety guarantees.

**Reference Implementation:** `/Users/rusty/Documents/Projects/TUI/tui` (TypeScript version, v0.8.2)

The TypeScript version is battle-tested and production-ready. This Rust port aims for feature parity with identical patterns and ergonomics.

## Key Architecture Patterns

### 1. Parallel Arrays (ECS Pattern)

Components are NOT objects. They're indices into parallel arrays:

```rust
// Component index `i` maps to:
component_type[i]    // Box, Text, Input, etc.
parent_index[i]      // Parent in hierarchy (-1 for root)
visible[i]           // Visibility state
fg_color[i]          // Foreground color
text_content[i]      // Text content (for Text/Input)
// ... 30+ arrays total
```

Arrays are organized into categories:
- **core** - componentType, parentIndex, visible, componentId
- **dimensions** - width, height, minWidth, maxWidth, minHeight, maxHeight
- **spacing** - margin (4), padding (4), gap
- **layout** - flexDirection, flexWrap, justifyContent, alignItems, zIndex, overflow
- **visual** - fgColor, bgColor, borderStyle, borderColor, opacity
- **text** - textContent, textAlign, textWrap, textAttrs
- **interaction** - focusable, tabIndex, cursorPosition, scrollOffset, hovered, pressed

### 2. FlexNode with Reactive Slots

Each component gets a persistent FlexNode with 33 reactive Slot properties:

```rust
pub struct FlexNode {
    pub index: usize,

    // Container (5): flexDirection, flexWrap, justifyContent, alignItems, alignContent
    // Item (5): flexGrow, flexShrink, flexBasis, alignSelf, order
    // Dimensions (6): width, height, minWidth, maxWidth, minHeight, maxHeight
    // Spacing (11): margin (4), padding (4), gap, rowGap, columnGap
    // Border (4): borderTop, borderRight, borderBottom, borderLeft
    // Other (2): overflow, position
}
```

Props bind directly to Slots - DON'T extract values, preserve reactivity:
```rust
// CORRECT - preserves reactivity
flex_node.width.source = props.width;

// WRONG - breaks reactivity
flex_node.width.source = props.width.get();
```

### 3. Component Registry

Index allocation with O(1) reuse:

```rust
// Allocation
let index = allocate_index(id);  // From pool or next_index++
ensure_capacity(index);          // Grow arrays if needed

// Parent context stack for nested creation
push_parent_context(parent_index);
children();
pop_parent_context();

// Release (recursive - releases children first)
release_index(index);
```

### 4. Render Pipeline

Pure derived-based pipeline:

```
Component Tree
    ↓
FlexNode Slots (reactive layout properties)
    ↓
layout_derived (Taffy flexbox → computed positions/sizes)
    ↓
frame_buffer_derived (renders tree → 2D cell grid)
    ↓
ONE render effect (diff renderer → terminal)
```

### 5. Three Rendering Modes

| Mode | Buffer | Height | Strategy |
|------|--------|--------|----------|
| **Fullscreen** | Alternate | Fixed (terminal) | Diff rendering |
| **Inline** | Normal | Content-determined | Full rebuild |
| **Append** | Normal | Content-determined | Active + frozen history |

## Component Primitives

### Box (Container)
- Flexbox layout (direction, wrap, justify, align)
- All dimensions (width, height, min/max)
- All spacing (margin, padding, gap)
- 10 border styles with per-side control
- Overflow modes (visible, hidden, scroll, auto)
- Mouse events (down, up, click, enter, leave, scroll)
- Keyboard events (when focused)
- Variant theming

### Text (Display)
- Reactive content (string or number)
- Alignment (left, center, right)
- Wrap modes (wrap, nowrap, truncate with ellipsis)
- 8 text attributes (bold, dim, italic, underline, blink, inverse, hidden, strikethrough)
- Mouse events (clickable text)

### Input (Text Entry)
- Two-way value binding
- Placeholder with separate color
- Password mode with custom mask char
- Full cursor system (block/bar/underline, blinking, custom colors)
- Keyboard nav (arrows, home, end, backspace, delete)
- Events: onChange, onSubmit, onCancel
- Auto-focus, max length

### Control Flow
- `show(condition, render, else)` - Conditional rendering
- `each(items, render, {key})` - List rendering with fine-grained updates
- `when(promise, {pending, then, catch})` - Async handling

## Systems

### Theme System
- 13 built-in presets (terminal, dracula, nord, catppuccin, etc.)
- ANSI colors respect terminal theme
- RGB and OKLCH color support
- 15 variants (primary, secondary, success, error, etc.)
- Reactive - changing theme updates everything
- Automatic contrast calculation (WCAG AA 4.5:1)

### Focus System
- Tab/Shift+Tab cycling sorted by tabIndex
- Focus trap for modals (push/pop stack)
- Focus history for restoration (max 10)
- Focus callbacks fire at state change source (onFocus/onBlur)
- Auto-focus on click for focusable components

### Event System
- Keyboard: handlers by key, by component, global
- Mouse: HitGrid for O(1) coordinate lookup
- Hover tracking with enter/leave detection
- Click detection (down + up on same component)
- Kitty keyboard protocol support

### Scroll System
- Content overflow detection
- Arrow key scrolling
- PageUp/PageDown/Home/End
- Mouse wheel
- Scroll chaining (parent fallback at boundaries)
- scrollIntoView for focus changes

### Cursor System
- Terminal native cursor (positioning)
- Drawn cursor for inputs (character rendered)
- Configurable style (block, bar, underline)
- Blink animation with shared clock per FPS
- Focus integration (blink when focused)

## Development Workflow

### Before Implementing Any Feature

1. **Read the TypeScript implementation** at `/Users/rusty/Documents/Projects/TUI/tui/src/`
2. **Understand the ergonomics** - how would a user use this?
3. **Write tests first** (TDD where sensible)
4. **Implement to pass tests**
5. **Add edge case tests**
6. **Document**

### Key TypeScript Files

| Feature | File |
|---------|------|
| Box primitive | `src/primitives/box.ts` |
| Text primitive | `src/primitives/text.ts` |
| Input primitive | `src/primitives/input.ts` |
| Registry | `src/engine/registry.ts` |
| FlexNode | `src/engine/FlexNode.ts` |
| Parallel arrays | `src/engine/arrays/*.ts` |
| Mount API | `src/api/mount.ts` |
| Theme system | `src/state/theme.ts` |
| Focus system | `src/state/focus.ts` |
| Keyboard | `src/state/keyboard.ts` |
| Mouse | `src/state/mouse.ts` |
| Scroll | `src/state/scroll.ts` |
| Cursor | `src/state/drawnCursor.ts` |

## Current State (January 2026)

### ✅ Implemented & Working

**spark-signals** (`crates/signals/`) - Production ready, 161 tests
- Signal, Derived, Effect, Batch
- Slot, SlotArray, TrackedSlotArray
- ReactiveMap, ReactiveSet, ReactiveVec
- EffectScope for cleanup

**Engine** (`src/engine/`) - Complete
- Registry with index allocation, ID mapping, parent context stack
- FlexNode with all 33 reactive Slot properties
- Parallel arrays: core, visual, text, interaction

**Layout** (`src/layout/`) - Complete
- Taffy bridge for flexbox computation
- Text measurement with unicode-width
- wrap_text, truncate_text utilities

**Renderer** (`src/renderer/`) - Complete
- FrameBuffer with Cell structure
- DiffRenderer (fullscreen differential)
- InlineRenderer (clear + redraw)
- AppendRenderer (history + active region)
- ANSI escape codes, StatefulCellRenderer

**Pipeline** (`src/pipeline/`) - Complete
- terminal.rs - size signals, render mode
- layout_derived - Taffy → ComputedLayout
- frame_buffer_derived - Layout → FrameBuffer + HitRegion
- inheritance - color inheritance
- mount/unmount API

**Primitives** (`src/primitives/`)
- Box - all layout props, borders, colors, focusable ✅
- Text - content, attrs, align, wrap ✅

**State** (`src/state/`)
- Focus - Tab cycling, trap, history, callbacks ✅
- Keyboard - event types, dispatch, handlers ✅

### ❌ Not Yet Implemented

**Primitives**
- Input component (two-way binding, cursor, keyboard handling)
- Control flow: show(), each(), when()

**Systems**
- Theme system (presets, variants, ANSI/RGB, contrast)
- Mouse system (HitGrid dispatch, hover, click detection)
- Scroll system (offsets, chaining, keyboard scroll, scrollIntoView)
- Cursor system (blink animation, drawn cursor for inputs)

**Event Wiring**
- Box/Text onClick, onKey, onMouse* callbacks not connected
- Mouse event dispatch from terminal input
- Global keys integration (Ctrl+C, Tab at mount level)

**Other**
- Context system (dependency injection)
- Per-side border styles/colors
- Variant theming support

### Next Steps (Priority Order)

1. **Mouse system** - HitGrid dispatch, hover/click detection
2. **Event wiring** - Connect callbacks to components
3. **Theme system** - At least terminal preset + t.* accessors
4. **Input component** - Two-way binding, cursor, keyboard
5. **Scroll system** - overflow:scroll support
6. **Control flow** - show(), each(), when()

## Testing Requirements

Every feature must have:
- Unit tests for core logic
- Integration tests for component behavior
- Edge case tests (empty arrays, max values, etc.)

Run tests: `cargo test -p spark-tui`

## Signals Usage Reference

spark-signals is at `crates/signals/`. Key primitives:

```rust
use spark_signals::*;

// Reactive value
let count = signal(0);
count.set(5);
let value = count.get();

// Computed value (auto-tracks dependencies)
let doubled = derived(move || count.get() * 2);

// Side effect (runs when dependencies change)
let _stop = effect(move || {
    println!("Count is: {}", count.get());
});

// Batch multiple updates (single notification)
batch(|| {
    count.set(1);
    other.set(2);
});

// Slot for flexible binding
let slot = slot(0);
slot.set_source(PropInput::Signal(count));
slot.set_source(PropInput::Getter(Rc::new(|| 42)));
slot.set_source(PropInput::Static(10));
```

## Dependencies

- `spark-signals` - Our reactive signals library
- `taffy` - Flexbox/Grid layout engine
- `crossterm` - Terminal I/O and events

## Author

Rusty & Watson (Claude) - Co-creators of the TUI Framework
