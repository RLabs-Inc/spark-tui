# SparkTUI Session Handoff — January 27, 2026 (Session 131)

## READ FIRST

1. Read `CLAUDE.md` (architecture bible, source of truth)
2. Read THIS file (session state)
3. Read `.planning/reactive-shared-memory-plan.md` (full 6-phase plan)

## What SparkTUI Is

A **hybrid TUI framework** where TypeScript handles the developer-facing API (primitives, signals, reactivity) and Rust handles the engine (layout, rendering, terminal output). Connected by **SharedArrayBuffer** — zero-copy, zero-serialization shared memory.

**Tagline**: *All Rust benefits without borrowing a single thing.*

## What Was Completed (Sessions 130-131)

### Phase 1 + 2: SharedSlotBuffer, Repeater, Notifier (DONE)

Three-layer reactive shared memory primitives implemented in BOTH packages:
- **SharedSlotBuffer** — Reactive typed arrays backed by shared memory
- **Repeater** — New graph node that forwards inline during markReactions (~40-50 bytes per binding)
- **Notifier** — Pluggable cross-side wake (AtomicsNotifier / NoopNotifier)

Published: `@rlabs-inc/signals` v1.13.0, `spark-signals` v0.3.0

### Phase 3: SoA Layout + SparkTUI Wiring (DONE)

Rewrote the SharedArrayBuffer bridge from AoS to SoA layout in both TS and Rust.

**Memory layout v3 (~2.0MB for 4096 nodes):**
```
Header (64B) | Dirty Flags (4096B) | Float32 (31×4096×4B = 508KB) |
Uint32 (12×4096×4B = 192KB) | Int32 (13×4096×4B = 208KB) |
Uint8 (28×4096 = 112KB) | Text Pool (1MB)
```

Each field is a **contiguous array** of MAX_NODES elements. Access: `field_array[node_index]` — no stride, no multiplication. This enables SharedSlotBuffer per field.

**Files changed:**
- **REWRITTEN** `ts/bridge/shared-buffer.ts` — SoA layout, per-field TypedArray views, backward-compat aliases for all old constant names (FLOAT_*, META_*, COLOR_*, INTERACT_* → F32_*, U8_*, U32_*, I32_*)
- **NEW** `ts/bridge/reactive-arrays.ts` — 84 SharedSlotBuffers (31 f32 + 12 u32 + 13 i32 + 28 u8) with named access (e.g. `arrays.width`, `arrays.fgColor`)
- **NEW** `ts/bridge/notify.ts` — AtomicsNotifier bridge for wake flag
- **REWRITTEN** `rust/src/shared_buffer.rs` — Matching SoA layout, facade API preserved for all 17 consumer files
- **FIXED** `rust/src/layout/layout_tree.rs` — Test helpers updated from AoS to SoA addressing

**Verification:** `cargo build --release` compiles clean. `cargo test` — 174 tests, 0 failures.

### Also done this session:
- Initialized SparkTUI as a **monorepo** (one git repo for both ts/ and rust/)
- Updated dependencies: `@rlabs-inc/signals` ^1.13.0, `spark-signals` 0.3

---

## What Comes Next

### Phase 4: Migrate Primitives to repeat()

**Goal:** Replace the old `setSource()` pattern with `repeat()` so primitives write directly to shared memory.

**Pattern change:**
```ts
// BEFORE (two layers: SlotArray → sync → SharedBuffer):
taffy.width.setSource(index, taffy.dimensionSource(props.width))

// AFTER (one layer: signal → repeat → SharedSlotBuffer → notifies Rust):
const dispose = repeat(props.width, reactiveArrays.width, index)
```

**Files to modify:**
- `ts/primitives/box.ts` — Replace setSource calls with repeat()
- `ts/primitives/text.ts` — Same
- `ts/primitives/input.ts` — Same
- `ts/engine/registry.ts` — Initialize reactive arrays, pass to primitives
- `ts/engine/lifecycle.ts` — Cleanup repeat() disposers on unmount

**Key question:** The old `ts/engine/arrays/taffy.ts` uses `typedSlotArrayGroup` with `syncAndGetDirty()`. The new pattern doesn't need sync — repeaters write to shared memory inline. But we still need dirty tracking so Rust knows WHICH nodes changed. The SharedSlotBuffer dirty flags handle this.

### Phase 5: Wire Rust Reactive Pipeline

**Goal:** Rust side uses spark-signals reactive graph. SharedSlotBuffers participate as sources.

```rust
// Wake thread:
loop {
    wait_for_wake(&wake_flag);           // blocks until TS notifies
    width_buf.notify_changed();          // → mark_reactions → layout_derived dirty
    // reactive graph propagates: layout → framebuffer → render effect → terminal
}
```

### Phase 6: Clean Up Legacy

Delete `ts/engine/arrays/` (old SlotArray-based system), `ts/arrays/` (standalone copies), potentially `rs/` (old reference pipeline).

---

## What HASN'T Been Tested Yet

Phase 3 changed the memory layout from AoS to SoA. The Rust side is fully tested (174 tests pass). The TS side **has NOT been runtime-tested** because:

1. The TS primitives still use the old engine/arrays (SlotArray-based), not the new bridge/reactive-arrays
2. The existing examples (`proof.ts`) use the old SharedBuffer API via utility functions (`setNodeFloat` etc.) — these functions were updated to SoA addressing but haven't been run
3. No integration test exists that creates a SharedArrayBuffer, writes from TS, reads from Rust

**Recommended test before Phase 4:**
Write a small test that:
1. Creates SharedBuffer views via `createSharedBuffer()`
2. Creates reactive arrays via `createReactiveArrays(views, notifier)`
3. Writes values via `arrays.width.set(0, 100)` (TS side)
4. Verifies the raw SharedArrayBuffer has the correct value at the correct SoA offset
5. Creates a Rust SharedBuffer from the same buffer, calls `buffer.width(0)` and verifies = 100

This would validate the TS↔Rust SoA contract end-to-end.

---

## Project Structure

```
SparkTUI/                     # Git monorepo
├── ts/
│   ├── bridge/
│   │   ├── shared-buffer.ts  # SoA layout contract (v3) — THE source of truth
│   │   ├── reactive-arrays.ts # 84 SharedSlotBuffers backed by SAB
│   │   ├── notify.ts         # AtomicsNotifier for wake flag
│   │   ├── buffer.ts         # Singleton + color packing utilities
│   │   └── ffi.ts            # Bun FFI definitions
│   ├── primitives/           # box, text, input, each, show, when
│   ├── engine/               # registry, lifecycle
│   │   └── arrays/           # OLD SlotArray-based (Phase 6: delete)
│   └── arrays/               # OLD standalone copies (Phase 6: delete)
├── rust/
│   └── src/
│       ├── shared_buffer.rs  # SoA layout (MUST match ts/bridge/shared-buffer.ts)
│       ├── lib.rs            # FFI exports
│       ├── types.rs          # Rgba, Dimension, etc.
│       ├── layout/           # Taffy trait API, text measurement
│       ├── framebuffer/      # Render tree, inheritance
│       ├── renderer/         # ANSI, diff, inline, append
│       ├── input/            # Keyboard, mouse, focus, scroll, cursor
│       ├── pipeline/         # Setup, terminal, wake
│       └── arrays/           # OLD reference (Phase 6: delete)
├── rs/                       # OLD reference pipeline (Phase 6: delete)
├── examples/                 # proof.ts, bench.ts
├── .planning/                # Phase plans
├── CLAUDE.md                 # Architecture bible
└── SESSION-HANDOFF.md        # This file
```

## Dependencies

- `@rlabs-inc/signals` v1.13.0 — TS reactive signals (SharedSlotBuffer, Repeater, Notifier)
- `spark-signals` v0.3.0 — Rust reactive signals (SharedSlotBuffer, Repeater, Notifier)
- `taffy` 0.7 (+content_size) — W3C flexbox layout
- `bitflags` 2.9, `unicode-width` 0.2, `unicode-segmentation` 1

## SoA Field Inventory (Quick Reference)

| Section | Fields | Type | Total |
|---------|--------|------|-------|
| Float32 | width, height, min/max_w/h, grow, shrink, basis, gap, pad×4, margin×4, inset×4, row/col_gap, computed_x/y/w/h, scrollable, max_scroll_x/y | f32 | 31 |
| Uint32 | fg/bg/border colors (×7), focus_ring, cursor_fg/bg, text_offset, text_length | u32 | 12 |
| Int32 | parent_index, scroll_x/y, tab_index, cursor_pos, selection_start/end, cursor_char/alt/blink, hovered, pressed, cursor_visible | i32 | 13 |
| Uint8 | component_type, visible, flex_dir/wrap, justify/align×3, overflow, position, border_width×4, border_style×5, focus_ring, opacity, z_index, text_attrs/align/wrap/ellipsis, focusable, mouse_enabled | u8 | 28 |

---

*Session 131 • January 27, 2026 • São Paulo*
