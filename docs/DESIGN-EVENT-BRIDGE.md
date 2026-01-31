# SparkTUI Event Bridge & TS API — Design Document

**Created:** January 28, 2026
**Status:** Draft — Ready for Implementation
**Authors:** Rusty & Claude

---

## Overview

This document defines the complete design for bridging Rust-side input processing back to TypeScript component callbacks. The goal is **zero-delay, fully reactive** event handling with a clean user-facing API that hides internal indices.

### Key Principles

1. **No fixed FPS** — Everything is reactive, event-driven
2. **No polling** — Atomics.waitAsync for instant wake
3. **No indices exposed** — Users work with refs and IDs
4. **AoS memory layout** — Cache-friendly for both TS writes and Rust reads
5. **Config via SharedBuffer** — TS configures, Rust respects

---

## Table of Contents

1. [Feature Inventory](#1-feature-inventory)
2. [SharedBuffer Layout](#2-sharedbuffer-layout)
3. [Event Ring Buffer](#3-event-ring-buffer)
4. [Config Flags](#4-config-flags)
5. [TS Public API](#5-ts-public-api)
6. [TS Internal API](#6-ts-internal-api)
7. [Component Props](#7-component-props)
8. [Refs & Handles](#8-refs--handles)
9. [Event Flow](#9-event-flow)
10. [Performance Guarantees](#10-performance-guarantees)
11. [Implementation Phases](#11-implementation-phases)

---

## 1. Feature Inventory

### From Original TS Implementation

#### Keyboard (`state/keyboard.ts`)
| Feature | Status | Location |
|---------|--------|----------|
| `lastEvent` signal | Need in TS | Reactive last key event |
| `lastKey` derived | Need in TS | Just the key string |
| `on(handler)` | Need in TS | Global key handler |
| `onKey(key, handler)` | Need in TS | Specific key handler |
| `onFocused(index, handler)` | Internal | Per-component when focused |
| `dispatch()` | Need in TS | Route events to handlers |
| `cleanupIndex()` | Internal | Cleanup on unmount |

#### Mouse (`state/mouse.ts`)
| Feature | Rust Has | TS Needs |
|---------|----------|----------|
| HitGrid O(1) lookup | ✓ | — |
| `lastMouseEvent` signal | — | ✓ |
| `mouseX`, `mouseY` signals | — | ✓ |
| `isMouseDown` signal | — | ✓ |
| Global handlers | — | ✓ |
| Per-component handlers | — | ✓ (internal) |
| Hover tracking | ✓ | — |
| Click detection | ✓ | — |
| ANSI mouse enable/disable | **MISSING** | Add to Rust |

#### Focus (`state/focus.ts`)
| Feature | Rust Has | TS Needs |
|---------|----------|----------|
| `focusedIndex` | ✓ (internal) | ✓ (reactive) |
| `focusNext()` / `focusPrevious()` | ✓ | Expose via API |
| `focus(index)` | ✓ | ✓ (via ref/ID) |
| `blur()` | ✓ | ✓ |
| `focusFirst()` / `focusLast()` | **MISSING** | Add to Rust |
| Focus traps | ✓ | ✓ |
| Focus history | ✓ | ✓ |
| Focus callbacks | — | ✓ |

#### Scroll (`state/scroll.ts`)
| Feature | Rust Has |
|---------|----------|
| `scrollTo`, `scrollBy` | ✓ |
| Scroll chaining | ✓ |
| `scrollIntoView` | ✓ |
| Arrow/Page/Home/End scroll | ✓ |
| Mouse wheel scroll | ✓ |

#### Terminal Cursor (`state/cursor.ts`)
| Feature | Location |
|---------|----------|
| `visible`, `shape`, `blinking` signals | TS |
| `show()`, `hide()`, `setShape()` | TS |
| `moveTo()`, `save()`, `restore()` | TS |
| Config written to SharedBuffer | TS → Rust |

#### Animation (`primitives/animation.ts`)
| Function | Purpose |
|----------|---------|
| `cycle(frames, options)` | Cycles through array of values at FPS |
| `pulse(options)` | Shorthand for `cycle([true, false])` — blink |
| `tween(from, to, options)` | Future: smooth interpolation |
| `spring(target, options)` | Future: physics-based animation |
| `wave(options)` | Future: continuous sine oscillation |

| Feature | Location |
|---------|----------|
| Shared clocks per FPS | TS (animationRegistry) |
| Reactive activation | TS (`active` option) |
| Auto-cleanup with scope | TS |
| Built-in frame sets | TS (Frames.spinner, Frames.dots, etc.) |

**Pattern:** `setInterval` → updates signal → propagates reactively → `repeat()` writes to SharedBuffer → Rust renders. All timing is TS signal sources.

#### Drawn Cursor (via animation)
| Feature | Location |
|---------|----------|
| Style presets | TS config → SharedBuffer |
| Custom char | TS config → SharedBuffer |
| Blink | **TS** `useAnimation([true, false], { fps })` |
| Position | Rust writes (text editing), TS can override |
| Colors (fg/bg) | TS config → SharedBuffer (can be animated!) |

**Key Insight:** Cursor blink is just another animation. User creates `pulse()` or `cycle()`, passes it to cursor config, value flows through `repeat()` to SharedBuffer. Rust has NO BlinkManager - just reads `cursor_visible` during render.

```typescript
// Simple blink with pulse()
input({
  value: text,
  cursor: {
    style: 'bar',
    visible: pulse({ fps: 2 }),  // ← Clean!
    fg: t.primary,
  }
})

// Custom blink pattern with cycle()
input({
  cursor: {
    visible: cycle([true, true, false], { fps: 3 }),  // On-on-off pattern
    fg: cycle([red, orange, yellow], { fps: 4 }),     // Color cycle
  }
})

// Spinner in text
text({
  content: cycle(Frames.spinner, { fps: 12 }),
})
```

#### Input Handling
| Feature | Rust Has |
|---------|----------|
| stdin raw mode | ✓ |
| Escape sequence parsing | ✓ |
| CSI, SS3, SGR mouse | ✓ |
| Kitty keyboard protocol | ✓ |

#### Global Keys (`state/global-keys.ts`)
| Feature | Configurable |
|---------|--------------|
| Ctrl+C exit | ✓ via config flag |
| Tab focus nav | ✓ via config flag |
| Arrow scroll | ✓ via config flag |
| Page scroll | ✓ via config flag |
| Home/End scroll | ✓ via config flag |
| Wheel scroll | ✓ via config flag |

#### Text Editing (`input/text_edit.rs`)
| Feature | Rust Has |
|---------|----------|
| Character insertion | ✓ |
| Backspace/Delete | ✓ |
| Cursor movement | ✓ |
| Enter → Submit | ✓ |
| Escape → Cancel | ✓ |
| maxLength | **TODO** (per-node) |

#### Theme & Context
| Feature | Location |
|---------|----------|
| 13 presets, 20 color slots | TS only |
| 14 variants, WCAG AA | TS only |
| createContext/provide/useContext | TS only |

---

## 2. SharedBuffer Layout

### Design Principles

1. **AoS for per-node data** — Rust iterates nodes, needs contiguous access
2. **Separate sections for different access patterns**
3. **Aligned for atomic operations** — Wake flags at 4-byte boundaries
4. **Clear read/write ownership**

### Memory Map

```
TOTAL SIZE: ~26.7 MB

┌─────────────────────────────────────────────────────────────────┐
│ HEADER (256 bytes)                                              │
├─────────────────────────────────────────────────────────────────┤
│ Core (0-63)                                                     │
│   0-3:   version (u32)                                          │
│   4-7:   node_count (u32)                                       │
│   8-11:  max_nodes (u32)                                        │
│  12-15:  terminal_width (u32)                                   │
│  16-19:  terminal_height (u32)                                  │
│  20-23:  generation (u32)                                       │
│  24-27:  text_pool_size (u32)                                   │
│  28-31:  text_pool_write_ptr (u32)                              │
│  32-35:  event_write_idx (u32)          ← Rust writes           │
│  36-39:  event_read_idx (u32)           ← TS writes             │
│  40-43:  focused_index (i32)            ← Rust writes           │
│  44-47:  hovered_index (i32)            ← Rust writes           │
│  48-51:  pressed_index (i32)            ← Rust writes           │
│  52-55:  mouse_x (u16) + mouse_y (u16)  ← Rust writes           │
│  56-63:  reserved                                               │
├─────────────────────────────────────────────────────────────────┤
│ Wake Flags (64-71) — Atomics                                    │
│  64-67:  wake_rust (u32)                ← TS notifies Rust      │
│  68-71:  wake_ts (u32)                  ← Rust notifies TS      │
├─────────────────────────────────────────────────────────────────┤
│ Config Flags (72-95) — TS writes, Rust reads                    │
│  72-75:  config_flags (u32)                                     │
│          bit 0: exit_on_ctrl_c                                  │
│          bit 1: tab_navigation                                  │
│          bit 2: arrow_scroll                                    │
│          bit 3: page_scroll                                     │
│          bit 4: home_end_scroll                                 │
│          bit 5: wheel_scroll                                    │
│          bit 6: focus_on_click                                  │
│          bit 7: mouse_enabled                                   │
│          bit 8: kitty_keyboard                                  │
│  76-79:  render_mode (u32)              0=full, 1=inline, 2=append
│  80-83:  cursor_config (u32)            packed: vis|shape|blink │
│  84-87:  scroll_speed (u32)             lines per wheel tick    │
│  88-95:  reserved                                               │
├─────────────────────────────────────────────────────────────────┤
│ Reserved (96-255)                                               │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ NODES (256 bytes × 100,000 = 25.6 MB)                          │
├─────────────────────────────────────────────────────────────────┤
│ Per-Node Layout (256 bytes):                                    │
│                                                                 │
│ Layout Floats (0-95) — 24 × f32                                 │
│   0-3: width         4-7: height        8-11: min_width         │
│  12-15: min_height  16-19: max_width   20-23: max_height        │
│  24-27: flex_basis  28-31: flex_grow   32-35: flex_shrink       │
│  36-39: padding_t   40-43: padding_r   44-47: padding_b         │
│  48-51: padding_l   52-55: margin_t    56-59: margin_r          │
│  60-63: margin_b    64-67: margin_l    68-71: gap               │
│  72-75: row_gap     76-79: column_gap  80-83: inset_t           │
│  84-87: inset_r     88-91: inset_b     92-95: inset_l           │
│                                                                 │
│ Layout Enums (96-111) — 16 × u8                                 │
│  96: flex_direction   97: flex_wrap    98: justify_content      │
│  99: align_items     100: align_content 101: align_self         │
│ 102: position        103: overflow     104: display             │
│ 105: border_top      106: border_right 107: border_bottom       │
│ 108: border_left     109: component_type 110: visible           │
│ 111: reserved                                                   │
│                                                                 │
│ Visual (112-147) — Colors + style                               │
│ 112-115: fg_color (u32 ARGB)                                    │
│ 116-119: bg_color (u32 ARGB)                                    │
│ 120-123: border_color (u32 ARGB)                                │
│ 124-127: focus_ring_color (u32 ARGB)                            │
│ 128-131: cursor_fg (u32 ARGB)                                   │
│ 132-135: cursor_bg (u32 ARGB)                                   │
│ 136-139: selection_color (u32 ARGB)                             │
│ 140: opacity (u8)                                               │
│ 141: z_index (i8)                                               │
│ 142: border_style (u8)                                          │
│ 143-147: reserved                                               │
│                                                                 │
│ Interaction (148-171) — Focus, cursor, scroll                   │
│ 148-151: scroll_x (i32)                                         │
│ 152-155: scroll_y (i32)                                         │
│ 156-159: tab_index (i32)                                        │
│ 160-163: cursor_position (i32)                                  │
│ 164-167: selection_start (i32)                                  │
│ 168-171: selection_end (i32)                                    │
│                                                                 │
│ Flags (172-179)                                                 │
│ 172: dirty_flags (u8)                                           │
│ 173: interaction_flags (u8)                                     │
│ 174: cursor_flags (u8)                                          │
│ 175: cursor_style (u8)                                          │
│ 176: cursor_fps (u8)                                            │
│ 177: max_length (u8)                                            │
│ 178-179: reserved                                               │
│                                                                 │
│ Hierarchy (180-183)                                             │
│ 180-183: parent_index (i32)                                     │
│                                                                 │
│ Text (184-195)                                                  │
│ 184-187: text_offset (u32)                                      │
│ 188-191: text_length (u32)                                      │
│ 192: text_align (u8)                                            │
│ 193: text_wrap (u8)                                             │
│ 194: text_overflow (u8)                                         │
│ 195: text_attrs (u8)                                            │
│                                                                 │
│ Cursor Character (196-199)                                      │
│ 196-199: cursor_char (u32)                                      │
│                                                                 │
│ Output — Rust writes (200-232)                                  │
│ 200-203: computed_x (f32)                                       │
│ 204-207: computed_y (f32)                                       │
│ 208-211: computed_width (f32)                                   │
│ 212-215: computed_height (f32)                                  │
│ 216-219: scroll_width (f32)                                     │
│ 220-223: scroll_height (f32)                                    │
│ 224-227: max_scroll_x (f32)                                     │
│ 228-231: max_scroll_y (f32)                                     │
│ 232: scrollable (u8)                                            │
│ 233-255: reserved                                               │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ TEXT POOL (1 MB)                                                │
│ Bump-allocated UTF-8 strings                                    │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ EVENT RING BUFFER (5,132 bytes)                                 │
├─────────────────────────────────────────────────────────────────┤
│ Header (12 bytes)                                               │
│   0-3: write_idx (u32)  ← Rust writes                           │
│   4-7: read_idx (u32)   ← TS writes                             │
│   8-11: reserved                                                │
├─────────────────────────────────────────────────────────────────┤
│ Events (256 × 20 bytes = 5,120 bytes)                           │
│ Each event (20 bytes):                                          │
│   0: event_type (u8)                                            │
│   1: reserved                                                   │
│   2-3: component_index (u16)                                    │
│   4-19: data[16]                                                │
└─────────────────────────────────────────────────────────────────┘
```

---

## 3. Event Ring Buffer

### Event Types

```typescript
enum EventType {
  None = 0,

  // Keyboard
  Key = 1,

  // Mouse
  MouseDown = 2,
  MouseUp = 3,
  Click = 4,
  MouseEnter = 5,
  MouseLeave = 6,
  MouseMove = 7,

  // Scroll
  Scroll = 8,

  // Focus
  Focus = 9,
  Blur = 10,

  // Input
  ValueChange = 11,
  Submit = 12,
  Cancel = 13,

  // System
  Exit = 14,
  Resize = 15,
}
```

### Event Data Layouts

```
Key Event:
  [0]: EventType.Key
  [2-3]: component_index
  [4-7]: keycode (u32)
  [8]: modifiers (ctrl|alt|shift|meta)
  [9]: key_state (press|repeat|release)

Mouse Event:
  [0]: EventType (Down/Up/Click/Enter/Leave)
  [2-3]: component_index
  [4-5]: x (u16)
  [6-7]: y (u16)
  [8]: button (left|middle|right)

Scroll Event:
  [0]: EventType.Scroll
  [2-3]: component_index
  [4-7]: delta_x (i32)
  [8-11]: delta_y (i32)

Focus/Blur Event:
  [0]: EventType (Focus|Blur)
  [2-3]: component_index

Resize Event:
  [0]: EventType.Resize
  [2-3]: 0
  [4-5]: width (u16)
  [6-7]: height (u16)
```

### Ring Buffer Operations

**Rust writes:**
```rust
fn push_event(&self, event: &Event) {
    let write_idx = self.event_write_idx() as usize;
    let slot = write_idx % MAX_EVENTS;
    self.write_event_at(slot, event);
    self.set_event_write_idx((write_idx + 1) as u32);
    self.notify_ts();
}
```

**TS reads:**
```typescript
function readEvents(): Event[] {
  const events: Event[] = []
  const writeIdx = view.getUint32(H_EVENT_WRITE_IDX, true)
  let readIdx = view.getUint32(H_EVENT_READ_IDX, true)

  while (readIdx < writeIdx) {
    const slot = readIdx % MAX_EVENTS
    events.push(parseEvent(slot))
    readIdx++
  }

  view.setUint32(H_EVENT_READ_IDX, readIdx, true)
  return events
}
```

---

## 4. Config Flags

### TS Config API

```typescript
interface SparkConfig {
  // Render mode
  renderMode: 'fullscreen' | 'inline' | 'append'

  // Input
  mouse: boolean
  kittyKeyboard: boolean

  // Framework behavior overrides
  exitOnCtrlC: boolean        // default: true
  tabNavigation: boolean      // default: true
  arrowScroll: boolean        // default: true
  pageScroll: boolean         // default: true
  homeEndScroll: boolean      // default: true
  wheelScroll: boolean        // default: true
  focusOnClick: boolean       // default: true

  // Scroll
  scrollSpeed: number         // default: 3

  // Terminal cursor
  cursor: {
    visible: boolean
    shape: 'block' | 'bar' | 'underline'
    blink: boolean
  }

  // Lifecycle
  onCleanup?: () => void
}
```

### Packed Config Flags

```
Bit 0: exit_on_ctrl_c
Bit 1: tab_navigation
Bit 2: arrow_scroll
Bit 3: page_scroll
Bit 4: home_end_scroll
Bit 5: wheel_scroll
Bit 6: focus_on_click
Bit 7: mouse_enabled
Bit 8: kitty_keyboard
```

---

## 5. TS Public API

### Main Exports

```typescript
// === Primitives ===
export { box, text, input, each, show, when }

// === Lifecycle ===
export { mount, unmount, createRef, onMount, onCleanup }

// === Focus ===
export {
  focus,           // focus(ref) or focus('id')
  focusNext,
  focusPrevious,
  focusFirst,
  focusLast,
  blur,
  useFocusedId,    // () => Signal<string | null>
  pushFocusTrap,
  popFocusTrap,
}

// === Keyboard ===
export {
  useLastKey,      // () => Signal<KeyboardEvent | null>
  onGlobalKey,     // (handler) => unsub
  onKey,           // (key, handler) => unsub
}

// === Mouse ===
export {
  useMousePosition, // () => { x: Signal, y: Signal }
  useMouseDown,     // () => Signal<boolean>
  onGlobalClick,    // (handler) => unsub
}

// === Cursor ===
export { cursor }   // cursor.show(), cursor.hide(), etc.

// === Animation ===
export {
  cycle,              // (frames, options) => Signal — cycle through values
  pulse,              // (options) => Signal<boolean> — blink (true/false)
  Frames,             // Built-in: spinner, dots, line, bar, clock
  // Future:
  // tween,           // (from, to, options) => Signal — smooth interpolation
  // spring,          // (target, options) => Signal — physics-based
  // wave,            // (options) => Signal — continuous oscillation
}

// === Theme ===
export { t, setTheme, getVariantStyle }

// === Context ===
export { createContext, provide, useContext }
```

### Key Principle: No Indices Exposed

Users work with:
- **Props** — `onClick`, `onFocus`, etc.
- **Refs** — `createRef()` + `ref.current.focus()`
- **IDs** — `focus('my-input')`

Never with indices.

---

## 6. TS Internal API

```typescript
// Registry (internal)
allocateIndex(id?: string): number
releaseIndex(index: number): void
getIndexById(id: string): number | undefined

// Handler registration (internal)
registerKeyHandler(index, handler): () => void
registerMouseHandlers(index, handlers): () => void
registerFocusCallbacks(index, callbacks): () => void

// Event dispatch (internal)
startEventLoop(): void
stopEventLoop(): void
dispatchEvent(event: RawEvent): void
```

---

## 7. Component Props

### BoxProps

```typescript
interface BoxProps {
  id?: string
  ref?: Ref<BoxHandle>

  // Layout
  width?, height?, minWidth?, maxWidth?, minHeight?, maxHeight?
  flexDirection?, flexWrap?, justifyContent?, alignItems?, gap?
  grow?, shrink?, flexBasis?, alignSelf?
  padding?, paddingTop?, paddingRight?, paddingBottom?, paddingLeft?
  margin?, marginTop?, marginRight?, marginBottom?, marginLeft?

  // Border
  border?, borderTop?, borderRight?, borderBottom?, borderLeft?
  borderStyle?, borderColor?

  // Visual
  fg?, bg?, opacity?, visible?, zIndex?

  // Scroll
  overflow?: 'visible' | 'hidden' | 'scroll'

  // Focus
  focusable?, tabIndex?
  onFocus?: () => void
  onBlur?: () => void

  // Keyboard
  onKey?: (event: KeyboardEvent) => boolean | void

  // Mouse
  onClick?, onMouseDown?, onMouseUp?
  onMouseEnter?, onMouseLeave?, onScroll?

  // Theme
  variant?: Variant

  // Children
  children?: () => void
}
```

### InputProps

```typescript
interface InputProps {
  id?: string
  ref?: Ref<InputHandle>

  value: WritableSignal<string>
  placeholder?, password?, maskChar?, maxLength?

  // Cursor config - all props can be signals (including from useAnimation!)
  cursor?: {
    style?: 'block' | 'bar' | 'underline'
    char?: string | Signal<string>           // Can animate!
    visible?: boolean | Signal<boolean>       // useAnimation for blink!
    fg?: Color | Signal<Color>                // Can animate colors!
    bg?: Color | Signal<Color>
  }

  // Layout subset...
  // Visual...

  tabIndex?, autoFocus?
  onFocus?, onBlur?
  onChange?, onSubmit?, onCancel?
  onKey?

  variant?: Variant
}
```

---

## 8. Refs & Handles

### BoxHandle

```typescript
interface BoxHandle {
  focus(): void
  blur(): void
  isFocused(): boolean

  scrollTo(x, y): void
  scrollBy(dx, dy): void
  scrollToTop(): void
  scrollToBottom(): void
  getScrollOffset(): { x, y }

  getBounds(): { x, y, width, height }
  isVisible(): boolean
}
```

### InputHandle

```typescript
interface InputHandle extends BoxHandle {
  getValue(): string
  setValue(value: string): void

  getCursorPosition(): number
  setCursorPosition(pos: number): void

  clear(): void
  selectAll(): void
}
```

---

## 9. Event Flow

```
User clicks at (x, y)
        │
        ▼
┌─────────────────────────────────────┐
│ RUST                                │
│ 1. HitGrid.hit_test(x, y) → idx 5   │
│ 2. Write event to ring buffer       │
│ 3. Atomics.notify(wake_ts)          │
└─────────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────────┐
│ TS EVENT LOOP                       │
│ 1. Atomics.waitAsync wakes          │
│ 2. Read events from ring buffer     │
│ 3. Dispatch to handlers             │
└─────────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────────┐
│ DISPATCH                            │
│ 1. Look up handlers.get(5)          │
│ 2. Call user's onClick(event)       │
│ 3. Handler updates signals          │
│ 4. repeat() → SharedBuffer → Rust   │
└─────────────────────────────────────┘

TOTAL LATENCY: < 50 microseconds
```

---

## 10. Performance Guarantees

### No Fixed FPS

| Component | Reactive? | Notes |
|-----------|-----------|-------|
| TS event loop | ✓ | Atomics.waitAsync — instant wake |
| Rust main loop | ✓ | Waits on wake + stdin — no polling |
| repeat() | ✓ | Inline on signal change |
| Animation | ✓ | setInterval as signal source (TS) |
| Cursor blink | ✓ | Uses animation pattern (TS) |

### All Timing Lives in TS

```
┌─────────────────────────────────────────────────────────────┐
│ TS (Timing Sources)                                         │
│                                                             │
│  pulse({ fps: 2 })  // or cycle([true, false], { fps: 2 }) │
│       │                                                     │
│       │  setInterval toggles signal every 500ms             │
│       ▼                                                     │
│  signal.value = true → false → true → ...                   │
│       │                                                     │
│       │  Passed as cursor.visible prop                      │
│       ▼                                                     │
│  repeat(cursor.visible, arrays.cursorVisible, index)        │
│       │                                                     │
│       │  Writes to SharedBuffer                             │
│       ▼                                                     │
│  SharedBuffer.cursorVisible[index] = 1 or 0                 │
│       │                                                     │
│       ▼                                                     │
│  Atomics.notify(wake_rust)                                  │
│                                                             │
└─────────────────────────────────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────────────────────────────────┐
│ RUST (Purely Reactive - NO TIMING)                          │
│                                                             │
│  Atomics.wait(wake_rust) ← wakes instantly                  │
│       │                                                     │
│       ▼                                                     │
│  Read SharedBuffer:                                         │
│    - cursor_visible[index] → draw cursor or not             │
│    - cursor_char[index] → which char to draw                │
│    - spinner_char[index] → animation frame                  │
│       │                                                     │
│       ▼                                                     │
│  Render (no timing decisions, just reads values)            │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**Rust has ZERO timing logic.** No BlinkManager, no intervals. It just reads whatever values TS wrote and renders them.

### Latency Budget

| Step | Time |
|------|------|
| Rust event detection | ~0 |
| Write to ring buffer | ~10ns |
| Atomics.notify | ~100ns |
| TS async wake | ~1-10μs |
| Read from buffer | ~100ns |
| Dispatch handler | ~1μs |
| **Total** | **< 20μs** |

---

## 11. Implementation Phases

### Phase 1: SharedBuffer Updates
- [ ] Add wake_ts flag to header
- [ ] Add config_flags section
- [ ] Add event ring buffer section
- [ ] Add per-node cursor config fields
- [ ] Update Rust SharedBuffer to match

### Phase 2: Rust Event Bridge
- [ ] Move EventRingBuffer from Vec to SharedBuffer
- [ ] Implement notify_ts()
- [ ] Read config_flags before processing
- [ ] Remove BlinkManager (cursor blink is now TS animation)
- [ ] Rust just reads cursor_visible during render
- [ ] Add focusFirst/focusLast
- [ ] Add ANSI mouse enable/disable

### Phase 3: TS Event Loop
- [ ] Create ts/engine/events.ts
- [ ] Implement Atomics.waitAsync loop
- [ ] Implement ring buffer reader
- [ ] Create event dispatch system

### Phase 4: TS State Modules
- [ ] ts/state/keyboard.ts — signals + global handlers
- [ ] ts/state/mouse.ts — signals + global handlers
- [ ] ts/state/focus.ts — useFocusedId, focus(ref/id)
- [ ] ts/state/cursor.ts — terminal cursor control
- [ ] ts/state/handlers.ts — internal registries
- [ ] ts/primitives/animation.ts — useAnimation, shared clocks
- [ ] ts/state/drawnCursor.ts — uses useAnimation for blink

### Phase 5: Primitive Integration
- [ ] Update box.ts — wire handlers, refs
- [ ] Update text.ts — wire handlers, refs
- [ ] Update input.ts — wire handlers, refs, cursor config

### Phase 6: Public API
- [ ] ts/api/ref.ts — createRef, handle types
- [ ] ts/api/mount.ts — mount with config
- [ ] Export everything from index.ts

### Phase 7: Testing
- [ ] Event ring buffer read/write
- [ ] Config flags propagation
- [ ] Handler registration and dispatch
- [ ] Ref focus/blur
- [ ] All event types

---

## Open Questions

1. ~~**Cursor blink timing**~~ — **RESOLVED:** Cursor blink uses TS `useAnimation()` pattern, not Rust BlinkManager. All timing is TS signal sources.

2. **ID table location** — TS-only Map, or in SharedBuffer for Rust access?

3. **Event overflow** — What happens if ring buffer fills? Drop oldest? Block?

## Design Decisions Log

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Cursor blink | `useAnimation()` passed as prop | Blink is just another animation value flowing through repeat() |
| Rust BlinkManager | **Removed** | No timing in Rust - it just reads cursor_visible |
| Animation values | Props can be signals | `cursor.visible`, `cursor.fg` can be animated |
| Index exposure | Hidden from users | Users work with refs/IDs |
| Event notification | Atomics.waitAsync | Instant, non-blocking |
| All timing | TS signal sources | Rust stays purely reactive, zero timing logic |

### Animation as Props Pattern

```typescript
// Any prop that supports Signal<T> can be animated:
input({
  cursor: {
    visible: pulse({ fps: 2 }),                // Blink
    fg: cycle([red, yellow], { fps: 4 }),      // Color cycle
  }
})

text({
  content: cycle(Frames.spinner, { fps: 12 }), // Spinner
  fg: cycle([dim, bright], { fps: 1 }),        // Fade
})
```

**Animation naming:**
- `cycle(values, options)` — cycle through array of values
- `pulse(options)` — shorthand for `cycle([true, false])`, perfect for blink
- Future: `tween()`, `spring()`, `wave()` for smooth/physics animations

This unifies all animation under one pattern: `cycle()`/`pulse()` → Signal → `repeat()` → SharedBuffer → Rust reads.

---

*This document will be updated as implementation progresses.*
