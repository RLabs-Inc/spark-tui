# Phase 1: Mouse System + Event Wiring - Research

**Researched:** 2026-01-22
**Domain:** Terminal mouse input, event dispatch, hit testing
**Confidence:** HIGH

## Summary

This phase implements the mouse event system that enables components to respond to mouse interactions. The TypeScript reference implementation provides a clear, battle-tested pattern: a `HitGrid` for O(1) coordinate-to-component lookup, an `InputBuffer` for parsing SGR mouse protocol, and a dispatch system that handles hover tracking, click detection, and event callbacks.

The Rust codebase already has significant infrastructure in place:
- `HitGrid` exists in `mount.rs` but is local to the render effect (needs extraction)
- `HitRegion` collection exists in `frame_buffer_derived.rs`
- `keyboard` module has dispatch patterns we can mirror for mouse
- Interaction arrays (`hovered`, `pressed`, `mouse_enabled`) are ready

**Primary recommendation:** Extract the existing HitGrid, create a dedicated mouse module paralleling the keyboard module, and wire callbacks through BoxProps using the TypeScript pattern.

## 1. TypeScript Implementation Analysis

### 1.1 Module Structure (TypeScript)

```
src/state/
├── input.ts      # Owns stdin, InputBuffer class, parses both keyboard + mouse
├── keyboard.ts   # KeyboardEvent types, handler registry, dispatch
├── mouse.ts      # MouseEvent types, HitGrid, handler registry, dispatch
├── global-keys.ts # Wires everything together, global shortcuts
├── scroll.ts     # Scroll state and handlers
└── focus.ts      # Focus state and navigation
```

**Key insight:** `input.ts` is the ONLY module that touches stdin. It parses raw bytes and routes typed events to `keyboard.ts` and `mouse.ts`.

### 1.2 TypeScript MouseEvent Structure

```typescript
export type MouseAction = 'down' | 'up' | 'move' | 'drag' | 'scroll'

export enum MouseButton {
  LEFT = 0,
  MIDDLE = 1,
  RIGHT = 2,
  NONE = 3,
}

export interface MouseEvent {
  action: MouseAction
  button: MouseButton | number
  x: number
  y: number
  shiftKey: boolean
  altKey: boolean
  ctrlKey: boolean
  scroll?: { direction: 'up' | 'down' | 'left' | 'right', delta: number }
  componentIndex: number  // Filled by dispatch from HitGrid
}
```

### 1.3 TypeScript HitGrid Implementation

```typescript
export class HitGrid {
  private grid: Int16Array  // -1 = no component
  private _width: number
  private _height: number

  get(x: number, y: number): number {
    if (x < 0 || x >= this._width || y < 0 || y >= this._height) return -1
    return this.grid[y * this._width + x]!
  }

  fillRect(x: number, y: number, width: number, height: number, componentIndex: number): void {
    // Clips to bounds, fills rectangle
  }
}
```

**Key insight:** Uses `Int16Array` (not usize) to allow -1 for "no component". The Rust version uses `usize::MAX` which is equivalent but requires `Option` wrapping for ergonomics.

### 1.4 TypeScript Dispatch Flow

```typescript
export function dispatch(event: MouseEvent): boolean {
  // 1. Fill componentIndex from HitGrid
  event.componentIndex = hitGrid.get(event.x, event.y)

  // 2. Update reactive state
  lastMouseEvent.value = event
  mouseX.value = event.x
  mouseY.value = event.y
  isMouseDown.value = event.action === 'down' || (event.action !== 'up' && isMouseDown.value)

  // 3. Handle hover (enter/leave detection)
  if (componentIndex !== hoveredComponent) {
    // Fire onMouseLeave on previous
    // Fire onMouseEnter on new
    // Update interaction.hovered arrays
    hoveredComponent = componentIndex
  }

  // 4. Handle scroll
  if (event.action === 'scroll') { /* dispatch to handlers */ }

  // 5. Handle down - track pressed component
  if (event.action === 'down') {
    pressedComponent = componentIndex
    pressedButton = event.button
    // Update interaction.pressed array
    // Fire onMouseDown handlers
  }

  // 6. Handle up - detect clicks
  if (event.action === 'up') {
    // Clear interaction.pressed
    // Fire onMouseUp handlers
    // Click detection: down + up on same component
    if (pressedComponent === componentIndex && pressedButton === event.button) {
      // Fire onClick handlers
    }
    pressedComponent = -1
  }

  return consumed
}
```

### 1.5 TypeScript Handler Registration

Two levels of handlers:
1. **Component handlers** - Registered via `onMouseComponent(index, handlers)`, stored in a Map
2. **Global handlers** - Registered via `onMouseDown(handler)`, stored in Sets

```typescript
const componentHandlers = new Map<number, MouseHandlers>()
const globalHandlers = {
  onMouseDown: new Set<MouseHandler>(),
  onMouseUp: new Set<MouseHandler>(),
  onClick: new Set<MouseHandler>(),
  onScroll: new Set<MouseHandler>(),
}

export function onComponent(index: number, handlers: MouseHandlers): () => void {
  componentHandlers.set(index, handlers)
  return () => componentHandlers.delete(index)
}
```

### 1.6 TypeScript Box.ts Mouse Wiring

```typescript
// In box.ts
const hasMouseHandlers = props.onMouseDown || props.onMouseUp || props.onClick || ...

if (shouldBeFocusable || hasMouseHandlers) {
  unsubMouse = onMouseComponent(index, {
    onMouseDown: props.onMouseDown,
    onMouseUp: props.onMouseUp,
    onClick: (event) => {
      if (shouldBeFocusable) {
        focusComponent(index)  // Click-to-focus
      }
      return props.onClick?.(event)
    },
    onMouseEnter: props.onMouseEnter,
    onMouseLeave: props.onMouseLeave,
    onScroll: props.onScroll,
  })
}
```

## 2. Existing Rust Infrastructure

### 2.1 HitGrid in mount.rs (Already Exists)

```rust
pub struct HitGrid {
    width: u16,
    height: u16,
    cells: Vec<usize>,  // usize::MAX = empty
}

impl HitGrid {
    pub fn new(width: u16, height: u16) -> Self { ... }
    pub fn resize(&mut self, width: u16, height: u16) { ... }
    pub fn clear(&mut self) { ... }
    pub fn fill_rect(&mut self, x: u16, y: u16, width: u16, height: u16, index: usize) { ... }
    pub fn get(&self, x: u16, y: u16) -> Option<usize> { ... }
}
```

**Status:** Implemented, tested, but local to mount.rs. Needs to be extracted to a shared location.

### 2.2 HitRegion in frame_buffer_derived.rs

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HitRegion {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub component_index: usize,
}

pub struct FrameBufferResult {
    pub buffer: FrameBuffer,
    pub hit_regions: Vec<HitRegion>,  // Collected during render
    pub terminal_size: (u16, u16),
}
```

**Status:** HitRegions are collected during render. Currently applied to HitGrid inside the render effect. This separation is correct - regions are computed reactively, grid is a mutable lookup structure.

### 2.3 Keyboard Module (Pattern to Follow)

```rust
// src/state/keyboard.rs - Good pattern to follow

pub type KeyHandler = Box<dyn Fn(&KeyboardEvent) -> bool>;

struct HandlerRegistry {
    global_handlers: Vec<(usize, KeyHandler)>,
    key_handlers: HashMap<String, Vec<(usize, KeySpecificHandler)>>,
    focused_handlers: HashMap<usize, Vec<(usize, KeyHandler)>>,
    next_id: usize,
}

pub fn dispatch(event: KeyboardEvent) -> bool { ... }
pub fn dispatch_focused(focused_index: i32, event: &KeyboardEvent) -> bool { ... }
pub fn on<F>(handler: F) -> impl FnOnce() { ... }  // Returns cleanup
pub fn on_focused<F>(index: usize, handler: F) -> impl FnOnce() { ... }
```

### 2.4 Interaction Arrays (Ready)

```rust
// src/engine/arrays/interaction.rs
HOVERED: TrackedSlotArray<bool>      // Is component hovered
PRESSED: TrackedSlotArray<bool>      // Is component pressed (mouse down)
MOUSE_ENABLED: TrackedSlotArray<bool> // Is mouse enabled for this component

pub fn get_hovered(index: usize) -> bool { ... }
pub fn set_hovered(index: usize, hovered: bool) { ... }
pub fn get_pressed(index: usize) -> bool { ... }
pub fn set_pressed(index: usize, pressed: bool) { ... }
```

### 2.5 Focus Integration (Ready)

```rust
// src/state/focus.rs
pub fn focus(index: usize) -> bool { ... }
pub fn is_focused(index: usize) -> bool { ... }
```

## 3. Technical Details: SGR Mouse Protocol

### 3.1 Enabling Mouse Tracking

```rust
/// Enable all mouse modes
pub const MOUSE_ENABLE: &str = concat!(
    "\x1b[?1000h",  // Basic mouse tracking (clicks only)
    "\x1b[?1002h",  // Button-event tracking (drag)
    "\x1b[?1003h",  // All motion tracking (move)
    "\x1b[?1006h",  // SGR extended mode (coordinates > 223)
);

/// Disable all mouse modes
pub const MOUSE_DISABLE: &str = concat!(
    "\x1b[?1000l",
    "\x1b[?1002l",
    "\x1b[?1003l",
    "\x1b[?1006l",
);
```

### 3.2 SGR Mouse Protocol Format

```
ESC [ < Cb ; Cx ; Cy M    // Press
ESC [ < Cb ; Cx ; Cy m    // Release

Where:
- Cb = button code (see below)
- Cx = x position (1-indexed)
- Cy = y position (1-indexed)
- M = press, m = release
```

**Button code (Cb) bits:**
```
bits 0-1: button (0=left, 1=middle, 2=right, 3=release)
bit 2:    shift pressed
bit 3:    meta/alt pressed
bit 4:    ctrl pressed
bit 5:    motion flag (drag/move)
bits 6-7: 00=normal, 01=wheel up, 10=wheel down
```

### 3.3 Parsing Example (from TypeScript)

```rust
fn parse_mouse_sgr(data: &str) -> Option<MouseEvent> {
    // Match: ESC [ < Cb ; Cx ; Cy M/m
    let re = Regex::new(r"^\x1b\[<(\d+);(\d+);(\d+)([Mm])").ok()?;
    let caps = re.captures(data)?;

    let button_code: u16 = caps[1].parse().ok()?;
    let x: u16 = caps[2].parse().ok()? - 1;  // Convert to 0-indexed
    let y: u16 = caps[3].parse().ok()? - 1;
    let is_release = &caps[4] == "m";

    let base_button = button_code & 0b11;
    let shift = button_code & 0b100 != 0;
    let alt = button_code & 0b1000 != 0;
    let ctrl = button_code & 0b10000 != 0;
    let is_motion = button_code & 0b100000 != 0;
    let wheel = (button_code >> 6) & 0b11;

    // ... decode to MouseEvent
}
```

### 3.4 crossterm 0.28 Mouse API

crossterm provides mouse event parsing already. We should use it instead of hand-rolling:

```rust
use crossterm::event::{
    read, poll,
    Event, MouseEvent, MouseEventKind, MouseButton, KeyModifiers,
};

// Read event (blocking)
match read()? {
    Event::Mouse(MouseEvent { kind, column, row, modifiers }) => {
        // kind: MouseEventKind::{Down, Up, Drag, Moved, ScrollUp, ScrollDown, ...}
        // column, row: u16 coordinates
        // modifiers: KeyModifiers (SHIFT, CONTROL, ALT)
    }
    Event::Key(key_event) => { ... }
    Event::Resize(width, height) => { ... }
    _ => {}
}

// Enable mouse capture
crossterm::execute!(stdout, EnableMouseCapture)?;

// Disable mouse capture
crossterm::execute!(stdout, DisableMouseCapture)?;
```

**Key insight:** crossterm handles SGR parsing internally. We don't need to parse raw bytes - just convert `crossterm::event::MouseEvent` to our `MouseEvent` type.

## 4. HitGrid Design

### 4.1 Current Design (Good)

The HitGrid is populated from `HitRegion` data during the render effect. This maintains reactivity - when layout changes, regions are recomputed, grid is updated.

```
FrameBufferDerived
    ↓
FrameBufferResult { hit_regions }
    ↓
Render Effect (applies to HitGrid)
    ↓
HitGrid (O(1) lookup)
```

### 4.2 Z-Index / Overlap Handling

Components are rendered in z-order (sorted by `visual::get_z_index`). The HitGrid uses simple overwrite - later components (higher z-index) overwrite earlier ones:

```rust
// In render_component (frame_buffer_derived.rs)
// Components with higher z-index render after lower ones
// Their fillRect overwrites, achieving correct hit order
hit_regions.push(HitRegion { ... });

// In render effect (mount.rs)
for region in &result.hit_regions {
    hit_grid.fill_rect(region.x, region.y, region.width, region.height, region.component_index);
}
```

This is correct - the TypeScript version works the same way.

### 4.3 Transparent Components

Currently, ALL visible components get hit regions. For transparent pass-through, we need a flag:

```rust
// Option 1: Only interactive components get hit regions
let is_interactive = interaction::get_focusable(index)
    || component_has_mouse_handlers(index)  // Need to track this
    || interaction::get_mouse_enabled(index);

// Option 2: Explicit pointer-events property (like CSS)
// pointer_events: 'auto' | 'none'
```

**Recommendation:** Use `mouse_enabled` array (already exists). Default to true, set to false for transparent pass-through.

## 5. Event Flow Architecture

### 5.1 Proposed Rust Event Flow

```
Terminal (stdin via crossterm)
        ↓
crossterm::event::read()
        ↓
Event::Mouse → convert to our MouseEvent
        ↓
mouse::dispatch(event, &hit_grid)
        ↓
├── Update reactive state (lastEvent, mouseX, mouseY, isDown)
├── Hover detection (enter/leave)
├── Click detection (down + up matching)
└── Handler dispatch (component → global)
        ↓
Interaction arrays updated (hovered, pressed)
        ↓
Reactive effects fire (UI updates)
```

### 5.2 Module Structure for Rust

```
src/state/
├── mod.rs           # Re-exports
├── focus.rs         # [EXISTS] Focus management
├── keyboard.rs      # [EXISTS] Keyboard events
├── mouse.rs         # [NEW] Mouse events, HitGrid (extract from mount.rs)
├── input.rs         # [NEW] stdin ownership, event loop integration
├── global_keys.rs   # [NEW] Global shortcuts (Ctrl+C, Tab, etc.)
└── scroll.rs        # [FUTURE] Scroll state management
```

## 6. Integration Points

### 6.1 BoxProps Callbacks (To Add)

```rust
pub struct BoxProps {
    // ... existing props ...

    // Mouse callbacks
    pub on_click: Option<Box<dyn Fn(&MouseEvent)>>,
    pub on_mouse_down: Option<Box<dyn Fn(&MouseEvent)>>,
    pub on_mouse_up: Option<Box<dyn Fn(&MouseEvent)>>,
    pub on_mouse_enter: Option<Box<dyn Fn(&MouseEvent)>>,
    pub on_mouse_leave: Option<Box<dyn Fn(&MouseEvent)>>,
    pub on_scroll: Option<Box<dyn Fn(&MouseEvent)>>,

    // Keyboard callback
    pub on_key: Option<Box<dyn Fn(&KeyboardEvent) -> bool>>,
}
```

### 6.2 box_primitive Wiring (To Add)

```rust
// In box_primitive()
if should_be_focusable || has_mouse_handlers {
    let unsub_mouse = mouse::on_component(index, MouseHandlers {
        on_mouse_down: props.on_mouse_down,
        on_mouse_up: props.on_mouse_up,
        on_click: if should_be_focusable {
            Some(Box::new(move |event| {
                focus::focus(index);
                if let Some(handler) = &props.on_click {
                    handler(event);
                }
            }))
        } else {
            props.on_click
        },
        on_mouse_enter: props.on_mouse_enter,
        on_mouse_leave: props.on_mouse_leave,
        on_scroll: props.on_scroll,
    });
    // Store unsub for cleanup
}
```

### 6.3 Mount Integration

The render effect in `mount.rs` already updates the HitGrid. Mouse dispatch needs access to it:

```rust
// Option 1: Make HitGrid globally accessible (thread_local!)
thread_local! {
    static HIT_GRID: RefCell<HitGrid> = RefCell::new(HitGrid::new(80, 24));
}

// Option 2: Pass reference to dispatch
pub fn dispatch(event: MouseEvent, hit_grid: &HitGrid) -> bool { ... }
```

**Recommendation:** Use thread_local! for simplicity, matching the keyboard module pattern.

## 7. Key Decisions Needed

### 7.1 Event Loop Ownership

**Options:**
1. **crossterm poll/read in render effect** - Keeps everything reactive but may block
2. **Separate event loop thread** - Non-blocking but adds complexity
3. **Async crossterm** - Clean but adds async complexity

**Recommendation:** Start with option 1 (poll with timeout in render loop). TypeScript uses this pattern and it works.

### 7.2 HitGrid Location

**Options:**
1. **Keep in mount.rs** - Requires passing reference or making public
2. **Move to mouse.rs as thread_local!** - Matches keyboard pattern
3. **New dedicated module (hit_grid.rs)** - Clean separation

**Recommendation:** Option 2 - Move to `mouse.rs` as thread_local!

### 7.3 Callback Storage

**Options:**
1. **Store in interaction arrays** - Requires new TrackedSlotArray<Option<Callback>>
2. **Store in HashMap in mouse.rs** - Matches TypeScript pattern
3. **Store in component registry** - Centralizes all component data

**Recommendation:** Option 2 - HashMap in mouse.rs, matches TypeScript pattern exactly.

## 8. Recommended Approach

### Phase 1A: Mouse Module Foundation

1. Create `src/state/mouse.rs`:
   - Move HitGrid from mount.rs
   - Define MouseEvent, MouseButton, MouseAction types
   - Create handler registry (HashMap<usize, MouseHandlers>)
   - Implement dispatch function with hover/click detection

2. Update `src/state/mod.rs`:
   - Export mouse module

3. Update `src/pipeline/mount.rs`:
   - Import HitGrid from mouse module
   - Keep hit_regions collection and grid population

### Phase 1B: Input Integration

4. Create `src/state/input.rs`:
   - Wrapper around crossterm event reading
   - Convert crossterm::MouseEvent to our MouseEvent
   - Route to mouse::dispatch

5. Create `src/state/global_keys.rs`:
   - Initialize input system
   - Wire Ctrl+C, Tab navigation
   - Mouse wheel to scroll (future)

### Phase 1C: Component Wiring

6. Update `src/primitives/types.rs`:
   - Add callback props to BoxProps

7. Update `src/primitives/box_primitive.rs`:
   - Register mouse handlers via mouse::on_component
   - Implement click-to-focus
   - Store cleanup functions

### Phase 1D: Event Loop

8. Update `src/pipeline/mount.rs`:
   - Integrate crossterm event polling
   - Call appropriate dispatch functions

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| crossterm | 0.28 | Terminal I/O, mouse/key events | Already in use, handles SGR parsing |
| spark-signals | local | Reactive state | Project foundation |

### Supporting
No additional libraries needed - crossterm provides all required functionality.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| SGR mouse parsing | Custom parser | `crossterm::event::MouseEvent` | Handles all edge cases, tested |
| Stdin handling | Raw reads | `crossterm::event::read()/poll()` | Handles raw mode, buffering |
| Key parsing | Custom escape parser | `crossterm::event::KeyEvent` | Already working in spec |

## Common Pitfalls

### Pitfall 1: Forgetting Click-to-Focus
**What goes wrong:** Focusable components don't focus when clicked
**Why it happens:** Mouse handlers registered without focus integration
**How to avoid:** Always check `should_be_focusable` in onClick, call `focus::focus(index)`
**Warning signs:** Tab navigation works but clicking doesn't update focus

### Pitfall 2: Hover State Leaks
**What goes wrong:** Component stays "hovered" after mouse leaves
**Why it happens:** onMouseLeave not fired when mouse moves to empty area
**How to avoid:** Track `hoveredComponent`, fire leave when it changes to -1
**Warning signs:** Hover styles persist after cursor moves away

### Pitfall 3: HitGrid Not Updated
**What goes wrong:** Mouse events dispatch to wrong components
**Why it happens:** HitGrid not cleared/repopulated after layout changes
**How to avoid:** Ensure render effect clears and refills grid every frame
**Warning signs:** Clicks register on old component positions

### Pitfall 4: Cleanup Memory Leaks
**What goes wrong:** Handlers accumulate after component unmount
**Why it happens:** Cleanup function not stored/called
**How to avoid:** Return cleanup from `on_component`, store in component, call on release
**Warning signs:** Performance degradation, handlers fire for unmounted components

## Code Examples

### MouseEvent Type (Verified from TypeScript)

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum MouseAction {
    Down,
    Up,
    Move,
    Drag,
    Scroll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MouseEvent {
    pub action: MouseAction,
    pub button: MouseButton,
    pub x: u16,
    pub y: u16,
    pub modifiers: Modifiers,  // Reuse from keyboard.rs
    pub scroll: Option<(ScrollDirection, u16)>,  // (direction, delta)
    pub component_index: Option<usize>,  // Filled by dispatch
}
```

### Handler Registration Pattern (from keyboard.rs)

```rust
pub type MouseHandler = Box<dyn Fn(&MouseEvent) -> bool>;

pub struct MouseHandlers {
    pub on_mouse_down: Option<MouseHandler>,
    pub on_mouse_up: Option<MouseHandler>,
    pub on_click: Option<MouseHandler>,
    pub on_mouse_enter: Option<Box<dyn Fn(&MouseEvent)>>,  // No return
    pub on_mouse_leave: Option<Box<dyn Fn(&MouseEvent)>>,  // No return
    pub on_scroll: Option<MouseHandler>,
}

pub fn on_component(index: usize, handlers: MouseHandlers) -> impl FnOnce() {
    REGISTRY.with(|reg| {
        reg.borrow_mut().component_handlers.insert(index, handlers);
    });

    move || {
        REGISTRY.with(|reg| {
            reg.borrow_mut().component_handlers.remove(&index);
        });
    }
}
```

### crossterm to MouseEvent Conversion

```rust
fn convert_crossterm_mouse(event: crossterm::event::MouseEvent) -> MouseEvent {
    let (action, button) = match event.kind {
        MouseEventKind::Down(btn) => (MouseAction::Down, convert_button(btn)),
        MouseEventKind::Up(btn) => (MouseAction::Up, convert_button(btn)),
        MouseEventKind::Drag(btn) => (MouseAction::Drag, convert_button(btn)),
        MouseEventKind::Moved => (MouseAction::Move, MouseButton::None),
        MouseEventKind::ScrollUp => (MouseAction::Scroll, MouseButton::None),
        MouseEventKind::ScrollDown => (MouseAction::Scroll, MouseButton::None),
        _ => (MouseAction::Move, MouseButton::None),
    };

    let scroll = match event.kind {
        MouseEventKind::ScrollUp => Some((ScrollDirection::Up, 1)),
        MouseEventKind::ScrollDown => Some((ScrollDirection::Down, 1)),
        _ => None,
    };

    MouseEvent {
        action,
        button,
        x: event.column,
        y: event.row,
        modifiers: Modifiers {
            ctrl: event.modifiers.contains(KeyModifiers::CONTROL),
            alt: event.modifiers.contains(KeyModifiers::ALT),
            shift: event.modifiers.contains(KeyModifiers::SHIFT),
            meta: false,  // Not exposed by crossterm
        },
        scroll,
        component_index: None,  // Filled by dispatch
    }
}

fn convert_button(btn: crossterm::event::MouseButton) -> MouseButton {
    match btn {
        crossterm::event::MouseButton::Left => MouseButton::Left,
        crossterm::event::MouseButton::Right => MouseButton::Right,
        crossterm::event::MouseButton::Middle => MouseButton::Middle,
    }
}
```

## Open Questions

### 1. Event Loop Threading Model

**What we know:** TypeScript uses single-threaded event loop with stdin read
**What's unclear:** Best approach for Rust - async, threading, or blocking poll
**Recommendation:** Start with blocking poll in render loop, optimize later if needed

### 2. Scroll Event Handling

**What we know:** TypeScript has separate scroll.ts module, scroll events check HitGrid then focused
**What's unclear:** Should scroll handling be in Phase 1 or separate phase
**Recommendation:** Include basic scroll dispatch in Phase 1, defer scroll state management to Phase 2

## Sources

### Primary (HIGH confidence)
- `/Users/rusty/Documents/Projects/TUI/tui/src/state/mouse.ts` - TypeScript HitGrid, dispatch, handlers
- `/Users/rusty/Documents/Projects/TUI/tui/src/state/input.ts` - TypeScript SGR parsing
- `/Users/rusty/Documents/Projects/TUI/tui/src/primitives/box.ts` - TypeScript mouse wiring
- `/Users/rusty/Documents/Projects/TUI/tui-rust/crates/tui/docs/specs/05-event-system.md` - Detailed spec

### Existing Rust Code (HIGH confidence)
- `/Users/rusty/Documents/Projects/TUI/tui-rust/crates/tui/src/state/keyboard.rs` - Pattern to follow
- `/Users/rusty/Documents/Projects/TUI/tui-rust/crates/tui/src/pipeline/mount.rs` - HitGrid impl
- `/Users/rusty/Documents/Projects/TUI/tui-rust/crates/tui/src/pipeline/frame_buffer_derived.rs` - HitRegion collection
- `/Users/rusty/Documents/Projects/TUI/tui-rust/crates/tui/src/engine/arrays/interaction.rs` - hovered/pressed arrays

### crossterm (HIGH confidence)
- crossterm 0.28.1 docs - Mouse event types and API

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - crossterm already in use
- Architecture: HIGH - TypeScript provides proven pattern
- HitGrid design: HIGH - Already implemented and tested
- Event wiring: HIGH - keyboard.rs provides working pattern
- Pitfalls: HIGH - Based on TypeScript implementation experience

**Research date:** 2026-01-22
**Valid until:** Stable - core event architecture unlikely to change
