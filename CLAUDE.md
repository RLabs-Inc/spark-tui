# SparkTUI - Architecture Bible

**Read this before every session. This is the ground truth.**

## Core Architecture: Pure Reactive Propagation

SparkTUI has **NO loops, NO polling, NO fixed FPS, NO event loops, NO animation frames, NO tick cycles**. None. Ever.

The entire rendering pipeline is **purely reactive**:

```
Developer changes a prop (signal, derived, manual, keyboard/mouse input)
  → Value is written to the shared array
    → Rust is notified in real-time (no delay, no polling)
      → Reactivity propagates through the pipeline:
        → Layout calculated (IF a layout property changed)
        → FrameBuffer calculated
          → ONE effect at the end fires the re-render
```

That's it. Nothing else. Nothing less.

### The Smart Skip

If a **visual property** changes (color, text content, border style) but NOT a **layout property** (width, height, flex, padding, margin), the pipeline **skips layout entirely** and jumps straight to framebuffer calculation. The reactive graph knows which deriveds depend on which arrays.

### The Single Effect

There is exactly **ONE effect** at the end of the pipeline. It triggers when the framebuffer derived changes. That effect renders to the terminal. No render loops. No scheduling. No requestAnimationFrame equivalent. The effect fires because the data changed. Period.

## Common Wrong Patterns - DO NOT USE

These patterns are **WRONG** for SparkTUI. Do not suggest, implement, or describe the architecture using any of these:

- "event loop" - NO. Reactive propagation, not a loop.
- "render loop" - NO. One effect that fires on change.
- "game loop" - NO. This is not a game engine pattern.
- "tick" / "frame tick" - NO. No fixed timing.
- "polling" - NO. Change notification, not polling.
- "requestAnimationFrame" / "rAF" - NO. No frame scheduling.
- "fixed FPS" / "target FPS" - NO. Renders exactly when data changes.
- "sleep" / "wait loop" - NO. Reactive notification.
- "autonomous loop" - NO. Reactive propagation.
- "futex_wait in a loop" - NO. The Rust side has a sleepy thread that wakes when notified. That's it. Not a loop - a notification mechanism.
- "run its own loop" - NO. Reacts to changes.

### The Right Mental Model

Think of it like a **spreadsheet**. You change cell A1, and every cell that depends on A1 recalculates automatically. There's no loop checking if A1 changed. The dependency graph propagates the change. SparkTUI works the same way - signals and deriveds form a dependency graph, and changes propagate through it instantly.

## What SparkTUI Is

A **hybrid TUI framework**:
- **TypeScript** = developer-facing API (primitives, signals, reactivity)
- **Rust** = engine (layout via Taffy, framebuffer, rendering, terminal output)
- **SharedArrayBuffer** = zero-copy, zero-serialization bridge between them

**Tagline**: *All Rust benefits without borrowing a single thing.*

## The Shared Arrays ARE the Architecture

The parallel arrays / ECS pattern ("father state pattern") is the foundation:

- Data IS flat typed arrays indexed by component ID
- SharedArrayBuffer bridges TS and Rust with identical memory layouts
- Both sides read/write the same memory at the same offsets
- Zero serialization, zero copying

### Array Categories

ALL of these must exist on BOTH sides (TypeScript AND Rust):

1. **Layout arrays** - width, height, min/max dimensions, flex properties, padding, margin, gap, hierarchy (parent indices)
2. **Visual arrays** - foreground color, background color, border style, border color, opacity, visibility, z-index
3. **Text arrays** - text content, text alignment, text wrap, text overflow
4. **Interaction arrays** - focusable, focused, hovered, pressed, disabled
5. **Dirty arrays** - per-node dirty flags for layout, visual, text (so the pipeline knows what changed)
6. **Output arrays** - computed x, y, width, height (written by Rust after layout)

### Current State of Arrays

**All sections exist on both sides (v2 buffer, ~2MB):**
- Header (64 bytes: version, node_count, max_nodes, terminal size, wake flag, generation, text pool management)
- Metadata (32 bytes/node: layout enums, visual props, text props, interaction flags, dirty flags)
- Floats (24 f32/node: dimensions, flex, padding, margin, gap, absolute positioning insets)
- Colors (10 u32/node: fg, bg, border per-side, focus ring, cursor colors — packed ARGB)
- Interaction (12 i32/node: scroll, tab index, cursor, selection, hover/pressed state)
- Hierarchy (1 i32/node: parent indices)
- Output (7 f32/node: computed x/y/w/h, scrollable, max scroll)
- Text Index (2 u32/node: offset + length into text pool)
- Text Pool (1MB: raw UTF-8 bump-allocated)

## The Rendering Pipeline (Reactive)

```
Shared Arrays (signals)
  │
  ├─→ layoutDerived (depends on: layout arrays, hierarchy)
  │     Runs Taffy flexbox → writes computed positions to output arrays
  │     ONLY recalculates when layout-affecting properties change
  │
  ├─→ frameBufferDerived (depends on: output arrays, visual arrays, text arrays)
  │     Fills the 2D cell grid from computed layout + visual properties
  │     Recalculates when ANY visual or layout output changes
  │     Skips layout if only visual props changed
  │
  └─→ renderEffect (depends on: frameBufferDerived)
        ONE effect. Diffs the framebuffer, outputs ANSI to terminal.
        Fires automatically when framebuffer changes. No loop. No poll.
```

## Project Structure

```
SparkTUI/
├── ts/                  # TypeScript source
│   ├── bridge/          # SharedArrayBuffer + FFI definitions
│   ├── primitives/      # box, text, input, each, show, when
│   ├── engine/          # registry, lifecycle, arrays
│   └── arrays/          # typed array definitions
├── rust/                # Active Rust cdylib engine
│   └── src/
│       ├── lib.rs           # FFI exports (spark_init, spark_compute_layout, spark_buffer_size)
│       ├── shared_buffer.rs # Memory layout contract + ALL enums (SSOT)
│       ├── shared_buffer_aos.rs # Legacy AoS buffer (being migrated)
│       ├── utils/           # Rendering infrastructure (Rgba, Cell, Attr, ClipRect)
│       └── layout/
│           ├── mod.rs           # Module root
│           ├── layout_tree.rs   # Low-level Taffy trait API (LayoutTree struct)
│           ├── text_measure/    # Unicode-aware text measurement
│           │   ├── mod.rs       # Re-exports
│           │   ├── width.rs     # string_width, char_width, grapheme_width
│           │   ├── wrap.rs      # wrap_text, measure_text_height
│           │   ├── truncate.rs  # truncate_text with configurable suffix
│           │   └── ansi.rs      # ANSI escape sequence stripping
│           ├── types.rs         # Layout type definitions
│           ├── taffy_bridge.rs  # Legacy TaffyTree bridge (cfg-gated, reference)
│           └── titan.rs         # Legacy TITAN engine (cfg-gated, reference)
├── rs/                  # Reference pipeline (old TrackedSlotArray-based)
├── examples/            # proof.ts (7/7), bench.ts
└── SESSION-HANDOFF.md   # Session handoff notes
```

## Dependencies

- `@rlabs-inc/signals` v1.12.0 (npm) - Reactive signals for TypeScript
- `spark-signals` v0.2.0 (crates.io) - Reactive signals for Rust
- `taffy` 0.7 (features: content_size) - W3C flexbox layout engine
- `bitflags` 2.9 - Cell attributes

## Current Status

**Layout engine: COMPLETE.** Taffy low-level trait API (`LayoutTree`) implements all 6 Taffy traits directly on SharedBuffer. NodeId = component index. Zero-copy, zero-translation. Unicode-aware text measurement. 113 tests passing.

**Next: Framebuffer computation** — read layout output + visual arrays, build 2D cell grid.

## Current Mission

1. **Build the framebuffer** - Read layout output + visual/text arrays → 2D Cell grid
2. **Wire the diff renderer** - Compare framebuffers → ANSI terminal output
3. **Connect the reactive pipeline** - layout derived → framebuffer derived → render effect
4. The pipeline is REACTIVE - deriveds and one effect, not loops

## Design Principles

- **Pure reactivity** - No loops. Change propagates through the dependency graph.
- **Smart skipping** - Layout-only changes skip visual recalc. Visual-only changes skip layout.
- **One effect** - The entire terminal output is driven by a single effect on the framebuffer.
- **Zero-copy bridge** - SharedArrayBuffer. Same memory, both sides.
- **All behaviors overridable** - Sane defaults, full configuration surface for power users.
- **Full spec, no shortcuts** - Never propose "acceptable limitations." If CSS flexbox supports it, we support it.
- **Rewrite over patch** - If the implementation drifts, delete and rewrite. No workarounds.

---

## Rust Engine Rewrite Standards (January 2026)

We are doing a ground-up rewrite of the Rust engine. These standards apply to every file we touch.

### Core Principles

1. **No Regressions** - Features only increase, never decrease. Add BorderStyle variants, never remove them. This applies to every enum, every capability, every file.

2. **Single Source of Truth** - One definition, one place, period. No duplicated constants. No parallel enum definitions. If it exists in shared_buffer.rs, that's where it lives.

3. **Now Is The Time** - We're rewriting from ground up. This is when we establish patterns. Not "we'll clean it up later." Now.

### Coding Standards

| Standard | Rule | Example |
|----------|------|---------|
| **Enums location** | All enums live in `shared_buffer.rs` only | No types.rs, no local constants |
| **Enum naming** | Start/End/Center, not FlexStart/FlexEnd | `JustifyContent::Start` (context is clear) |
| **Enum repr** | `#[repr(u8)]` for all enums | Zero-cost, matches buffer bytes |
| **Enum conversion** | Every enum has `From<u8>` | Safe conversion from buffer values |
| **Helper methods** | On enums when useful | `FlexDirection::is_row()`, `BorderStyle::chars()` |
| **Offset constants** | H_*, F_*, U_*, C_*, I_* in shared_buffer.rs | These define the buffer layout contract |
| **No magic numbers** | Use enums and named constants everywhere | Never `if value == 3`, always `if value == ComponentType::Input as u8` |
| **i32 for coordinates** | ClipRect, scroll positions use i32 | Handles negative scroll, clamped at render |

### File-by-File Approach

For each file in the rewrite:
1. **Read together** - Understand current implementation fully
2. **Map features** - List every behavior, every edge case
3. **Agree on spec** - What stays, what changes, what's added
4. **Write tests** - Test the spec, not the implementation
5. **Implement** - Clean, following these standards

### Standards Log

*Add new standards here as we discover them during the rewrite:*

- **2026-01-30**: Deleted `types.rs` - all enums now live in `shared_buffer.rs` (SSOT achieved)
- **2026-01-30**: `BorderStyle::chars()` returns `(char, char, char, char, char, char)` directly, not `&str`
- **2026-01-30**: Added `SharedBuffer::border_chars(node)` for unified predefined/custom border handling

---

*This document is the source of truth. If SESSION-HANDOFF.md or any other doc contradicts this, THIS wins.*
