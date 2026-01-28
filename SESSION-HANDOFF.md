# SparkTUI Session Handoff — January 28, 2026 (Session 132)

## READ FIRST

1. Read `CLAUDE.md` (architecture bible, source of truth)
2. Read THIS file (session state + what to do next)

## What SparkTUI Is

A **hybrid TUI framework** where TypeScript handles the developer-facing API (primitives, signals, reactivity) and Rust handles the engine (layout, rendering, terminal output). Connected by **SharedArrayBuffer** — zero-copy, zero-serialization shared memory.

**Tagline**: *All Rust benefits without borrowing a single thing.*

---

## What Just Happened (Session 132)

### Full Recon: 5-Agent Comparison of TS vs Rust

Deployed 5 parallel reconnaissance agents comparing the TS reference TUI (`../tui`) against the Rust SparkTUI. Covered every system: layout, rendering, input, primitives, bridge, pipeline.

**Key finding: The Rust engine is 90%+ complete.** All major systems are production-grade.

### Bugs Fixed (Wave 1)

**1. FlexDirection Enum SWAPPED (FIXED)**
- TS convention: row=0, column=1, row-reverse=2, column-reverse=3
- Rust HAD: Column=0, Row=1, ColumnReverse=2, RowReverse=3
- Files fixed: `rust/src/types.rs`, `rust/src/layout/layout_tree.rs` (enum + `to_flex_direction()` + 3 test cases)

**2. Color Packing Signed/Unsigned (FIXED)**
- JS bitwise `<<` returns signed 32-bit, `Uint32Array` stores unsigned
- `packColor(255,0,0,255)` returned -65536 (signed), but Uint32Array reads 4294901760 (unsigned)
- Fix: Added `>>> 0` to force unsigned in `ts/bridge/shared-buffer.ts:packColor()`

**Result: 31/31 proof tests pass. 174/174 Rust tests pass.**

---

## Current State: What's COMPLETE

### Rust Engine (ALL COMPLETE)
- **Layout**: Taffy low-level trait API, 174 tests, zero-copy (`rust/src/layout/layout_tree.rs`)
- **SharedBuffer**: SoA layout, all 84 fields, identical on both sides (`rust/src/shared_buffer.rs`)
- **Input Parser**: CSI, SS3, SGR mouse, Kitty keyboard, UTF-8 (`rust/src/input/parser.rs`)
- **Focus Manager**: Tab/Shift+Tab cycling, traps, history, focus-by-click (`rust/src/input/focus.rs`)
- **Scroll Manager**: Absolute, relative, chaining, scroll-into-view (`rust/src/input/scroll.rs`)
- **Mouse HitGrid**: O(1) lookup, hover tracking, click detection (`rust/src/input/mouse.rs`)
- **Text Editing**: Insert, delete, cursor, home/end for inputs (`rust/src/input/text_edit.rs`)
- **Cursor Blink**: Shared clocks per FPS, signal source pattern (`rust/src/input/cursor.rs`)
- **Terminal Setup**: Raw mode, alt screen, mouse, Kitty, paste, sync output (`rust/src/pipeline/terminal.rs`)
- **Framebuffer**: 2D cell grid, render tree, inheritance, z-index, opacity (`rust/src/framebuffer/`)
- **Diff Renderer**: Cell diff, synchronized ANSI output (`rust/src/renderer/diff.rs`)
- **Reactive Pipeline**: generation signal → layout_derived → fb_derived → render_effect (`rust/src/pipeline/setup.rs`)
- **Event Ring Buffer**: 256 events, 14 types, FIFO — IN-MEMORY ONLY (`rust/src/input/events.rs`)
- **Keyboard Dispatch**: Ctrl+C exit, Tab focus, arrow scroll, page scroll (`rust/src/input/keyboard.rs`)
- **stdin Reader**: Dedicated thread, blocking reads (`rust/src/input/reader.rs`)

### TypeScript (ALL COMPLETE)
- **Primitives**: box, text, input — all use `repeat()` to wire props to SharedBuffer
- **Control flow**: each(), show(), when() — pure TS component lifecycle
- **Theme**: 13 presets, WCAG AA contrast, variant styles (`ts/state/theme.ts`)
- **Animation**: useAnimation with shared clocks (`ts/primitives/animation.ts`)
- **Registry**: Node alloc/dealloc, parent context (`ts/engine/registry.ts`)
- **Lifecycle**: onMount, onDestroy hooks (`ts/engine/lifecycle.ts`)
- **Scope**: Auto-cleanup collection (`ts/primitives/scope.ts`)
- **Bridge**: SoA SharedBuffer, 84 reactive arrays, wake notifier (`ts/bridge/`)

---

## WHAT'S NEXT: Wave 2 — Event Bridge (THE critical path)

This is the #1 gap. Everything in Rust WORKS — keyboard, mouse, focus, scroll, text editing — but TS cannot receive events from Rust. Once this bridge exists, all callbacks light up.

### The Problem

Rust writes events to an **in-memory** ring buffer (`rust/src/input/events.rs:127-184`). TS has no way to read them. The ring buffer is `Vec<Event>` inside the Rust process — not in SharedArrayBuffer.

### The Solution: Event Ring Buffer in SharedBuffer

Add a new section to SharedArrayBuffer for events. Rust writes, TS reads. Lock-free SPSC (single producer, single consumer).

**Event structure** (already defined in `events.rs:31-46`):
```
EventType: u8 (14 types: Key=1, MouseDown=2, MouseUp=3, Click=4, etc.)
ComponentIndex: u16
Data: [u8; 16] (packed payload — keycode+modifiers, mouse x/y+button, scroll dx/dy, etc.)
Total: 19 bytes → pad to 20 bytes
```

**Ring buffer** (already defined in `events.rs:127-184`):
```
MAX_EVENTS = 256
Header: 12 bytes (write_idx: u32, read_idx: u32, count: u32)
Events: 256 × 20 = 5,120 bytes
Total: 5,132 bytes
```

**Steps:**
1. Add event ring buffer section to SharedBuffer memory layout (both TS + Rust)
2. Rust: write events to SharedBuffer instead of in-memory Vec
3. TS: read events from SharedBuffer, dispatch to component handlers
4. Create `ts/state/keyboard.ts`, `ts/state/mouse.ts`, `ts/state/focus.ts` — these modules bridge events to callbacks

### Missing TS State Modules

Primitives already import from these paths but they DON'T EXIST:
- `ts/state/keyboard.ts` — key event dispatch to focused component
- `ts/state/mouse.ts` — mouse event dispatch to hit component
- `ts/state/focus.ts` — focus state tracking, blur/focus callbacks

### After Wave 2: Wave 3 (App Lifecycle)

- `mount()` / `createApp()` API that orchestrates startup
- Terminal size → reactive graph → resize handling
- Graceful shutdown (cleanup effects, restore terminal)

### After Wave 3: Wave 4 (Integration Testing)

- E2E test: TS writes → Rust layout → Rust render → verify output
- Event round-trip: TS click → Rust dispatch → TS callback fires

---

## Enum Convention Reference (TS is source of truth)

**FlexDirection**: 0=row, 1=column, 2=row-reverse, 3=column-reverse
**FlexWrap**: 0=nowrap, 1=wrap, 2=wrap-reverse
**JustifyContent**: 0=flex-start, 1=center, 2=flex-end, 3=space-between, 4=space-around, 5=space-evenly
**AlignItems**: 0=stretch, 1=flex-start, 2=center, 3=flex-end, 4=baseline
**AlignSelf**: 0=auto, 1=stretch, 2=flex-start, 3=center, 4=flex-end, 5=baseline
**AlignContent**: 0=stretch, 1=flex-start, 2=center, 3=flex-end, 4=space-between, 5=space-around
**Overflow**: 0=visible, 1=hidden, 2=scroll, 3=auto
**Position**: 0=relative, 1=absolute
**ComponentType**: 0=none, 1=box, 2=text, 3=input, 4=select, 5=progress, 6=canvas
**BorderStyle**: 0=none, 1=single, 2=double, 3=rounded, 4=bold, 5=dashed, 6=dotted, 7=ascii, 8=block, 9=double-horz, 10=double-vert
**TextAlign**: 0=left, 1=center, 2=right
**TextWrap**: 0=nowrap, 1=wrap, 2=truncate
**Color packing**: ARGB format, `>>> 0` for unsigned. 0x00000000 = inherit. 0x01XX0000 = ANSI index.

## SoA Field Inventory (Quick Reference)

| Section | Fields | Type | Count |
|---------|--------|------|-------|
| Float32 | width, height, min/max_w/h, grow, shrink, basis, gap, pad×4, margin×4, inset×4, row/col_gap, computed_x/y/w/h, scrollable, max_scroll_x/y | f32 | 31 |
| Uint32 | fg/bg/border colors (×7), focus_ring, cursor_fg/bg, text_offset, text_length | u32 | 12 |
| Int32 | parent_index, scroll_x/y, tab_index, cursor_pos, selection_start/end, cursor_char/alt/blink, hovered, pressed, cursor_visible | i32 | 13 |
| Uint8 | component_type, visible, flex_dir/wrap, justify/align×3, overflow, position, border_width×4, border_style×5, focus_ring, opacity, z_index, text_attrs/align/wrap/ellipsis, focusable, mouse_enabled | u8 | 28 |

## FFI Exports (Rust → TS)

| Function | Args | Returns | Purpose |
|----------|------|---------|---------|
| spark_init | ptr, len | u32 (0=ok) | Init engine with SharedArrayBuffer |
| spark_compute_layout | - | u32 (nodes) | Run Taffy flexbox |
| spark_render | - | u32 (cells) | Layout + framebuffer + diff render |
| spark_buffer_size | - | u32 (bytes) | Total buffer size needed |
| spark_wake | - | - | Wake engine after TS writes |
| spark_cleanup | - | - | Stop engine, cleanup |

## Project Structure

```
SparkTUI/                     # Git monorepo
├── ts/
│   ├── bridge/
│   │   ├── shared-buffer.ts  # SoA layout contract — THE source of truth
│   │   ├── reactive-arrays.ts # 84 SharedSlotBuffers backed by SAB
│   │   ├── notify.ts         # AtomicsNotifier for wake flag
│   │   ├── buffer.ts         # Singleton + color packing utilities
│   │   └── ffi.ts            # Bun FFI definitions (6 functions)
│   ├── primitives/           # box, text, input, each, show, when, animation, scope
│   ├── state/                # theme.ts, drawnCursor.ts (keyboard/mouse/focus MISSING)
│   ├── engine/               # registry, lifecycle
│   │   └── arrays/           # OLD SlotArray-based (delete after migration)
│   └── arrays/               # OLD standalone copies (delete after migration)
├── rust/
│   └── src/
│       ├── shared_buffer.rs  # SoA layout (MUST match ts/bridge/shared-buffer.ts)
│       ├── lib.rs            # FFI exports
│       ├── types.rs          # Rgba, Dimension, FlexDirection, etc.
│       ├── layout/           # Taffy trait API + text measurement
│       ├── framebuffer/      # Render tree, inheritance, hit regions
│       ├── renderer/         # ANSI, diff, inline, append, output buffer
│       ├── input/            # parser, keyboard, mouse, focus, scroll, cursor, text_edit, events, reader
│       ├── pipeline/         # setup (reactive graph), terminal, wake
│       └── arrays/           # OLD reference (delete after migration)
├── rs/                       # OLD reference pipeline (delete after migration)
├── examples/                 # reactive-proof.ts (31/31), proof.ts, bench.ts
├── CLAUDE.md                 # Architecture bible
└── SESSION-HANDOFF.md        # THIS FILE
```

## Dependencies

- `@rlabs-inc/signals` v1.13.0 — TS reactive signals (SharedSlotBuffer, Repeater, Notifier)
- `spark-signals` v0.3.0 — Rust reactive signals (SharedSlotBuffer, Repeater, Notifier)
- `taffy` 0.7 (+content_size) — W3C flexbox layout
- `bitflags` 2.9, `unicode-width` 0.2, `unicode-segmentation` 1

---

*Session 132 • January 28, 2026 • Sao Paulo*
