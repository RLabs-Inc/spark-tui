# SparkTUI Session Handoff — January 27, 2026

## What SparkTUI Is

A **hybrid TUI framework** where TypeScript handles the developer-facing API (primitives, signals, reactivity) and Rust handles the engine (layout, rendering, terminal output). Connected by **SharedArrayBuffer** — zero-copy, zero-serialization shared memory.

**Tagline**: *All Rust benefits without borrowing a single thing.*

## What Was Completed This Session

### Phase 1 + 2: Cross-Language Reactive Shared Memory (DONE)

Implemented the three-layer architecture in BOTH `@rlabs-inc/signals` (TS) and `spark-signals` (Rust):

**Layer 1: SharedSlotBuffer** — Reactive typed arrays backed by shared memory. `get()` tracks deps, `set()` writes + notifies reactive graph + notifies cross-side.

**Layer 2: Repeater** — A NEW reactive graph node (not effect, not derived). Runs INLINE during `markReactions`. Connects any reactive source to a buffer position. ~40-50 bytes per binding vs ~200+ for Effect.

**Layer 3: Notifier** — Pluggable cross-side notification. AtomicsNotifier batches via microtask (TS) or direct store+wake (Rust). NoopNotifier for testing.

### TS Changes (`@rlabs-inc/signals`)
**Source**: `/Users/rusty/Documents/Projects/AI/Tools/ClaudeTools/memory-ts/packages/signals/`

New files:
- `src/shared/notifier.ts` — Notifier interface, AtomicsNotifier, NoopNotifier
- `src/shared/shared-slot-buffer.ts` — SharedSlotBuffer interface + sharedSlotBuffer() factory + sharedSlotBufferGroup()
- `src/primitives/repeater.ts` — RepeaterNode, repeat() factory, forwardRepeater()

Modified files:
- `src/core/constants.ts` — Added `REPEATER = 1 << 19`
- `src/reactivity/tracking.ts` — REPEATER branch in markReactions: calls forwardRepeater() inline, marks CLEAN
- `src/index.ts` — All new exports

**Verification**: `tsc --noEmit` passes with zero errors.

### Rust Changes (`spark-signals`)
**Source**: `/Users/rusty/Documents/Projects/TUI/tui-rust/crates/spark-signals/`

New files:
- `src/shared/notify.rs` — Notifier trait, AtomicsNotifier, NoopNotifier, platform_wake()
- `src/shared/shared_slot_buffer.rs` — SharedSlotBuffer<T> with reactive get/set
- `src/primitives/repeater.rs` — RepeaterInner (AnyReaction with REPEATER flag), repeat() factory

Modified files:
- `src/core/constants.rs` — Added `REPEATER = 1 << 19`
- `src/reactivity/tracking.rs` — REPEATER branch in mark_reactions
- `src/shared/mod.rs` — Declared notify + shared_slot_buffer submodules
- `src/primitives/mod.rs` — Declared repeater submodule
- `src/lib.rs` — All new exports

**Verification**: `cargo build` zero warnings. `cargo test` — 343 tests, 0 failures.

---

## What Comes Next

**Full plan with field inventory, SoA layout, and verification checklists:**
`.planning/reactive-shared-memory-plan.md`

### Phase 3: Wire into SparkTUI

**SparkTUI path**: `/Users/rusty/Documents/Projects/TUI/SparkTUI/`

**Files to create/rewrite:**
- **REWRITE** `ts/bridge/shared-buffer.ts` — SoA layout + per-field TypedArray views
- **NEW** `ts/bridge/reactive-arrays.ts` — SharedSlotBuffers backed by SharedArrayBuffer
- **NEW** `ts/bridge/notify.ts` — AtomicsNotifier wired to wake flag
- **REWRITE** `rust/src/shared_buffer.rs` — SoA layout matching TS

**SoA Layout** (~2.1MB for 4096 nodes):
```
Header (64B) | Dirty Flags (4096B) | Float32 (31 fields × 4096 × 4B) |
Uint32 (12 × 4096 × 4B) | Int32 (13 × 4096 × 4B) | Uint8 (28 × 4096) | Text Pool (1MB)
```

### Phase 4: Migrate Primitives + Wire Rust Pipeline

- `ts/primitives/box.ts`, `text.ts`, `input.ts` — use `repeat()` instead of `setSource()`
- `ts/engine/registry.ts` — cleanup uses new arrays
- `rust/src/lib.rs` — reactive pipeline with SharedSlotBuffers

**Pattern:**
```ts
// BEFORE: taffy.width.setSource(index, taffy.dimensionSource(props.width))
// AFTER:  const dispose = repeat(props.width, layout.arrays.width, index)
```

**Rust pipeline:**
```rust
loop {
    wait_for_wake(&wake_flag);           // blocks until TS notifies
    width_buf.notify_changed();          // → mark_reactions → layout_derived dirty
    // reactive graph propagates: layout → framebuffer → render effect → terminal
}
```

### Phase 5: Interaction / Event System (later)
### Phase 6: Clean Up Legacy Arrays — delete `ts/engine/arrays/`

---

## The Full Reactive Chain (Now Implemented)

```
Developer: myWidth.value = 150
  → markReactions(myWidth, DIRTY)
    → walks reactions, finds REPEATER node
      → REPEATER.forward() runs INLINE during markReactions:
        → reads myWidth (already updated to 150)
        → writes buffer[nodeIndex] = 150 (shared memory)
        → sets dirty flag
        → notifier.notify() (batched via microtask)
    → markReactions continues...
  → microtask: Atomics.notify → Rust wakes
    → Rust notify_changed() → Rust reactive graph propagates
      → layout derived re-evaluates
        → framebuffer derived re-evaluates
          → ONE render effect → terminal updates
```

## Project Structure

```
SparkTUI/
├── ts/                       # TypeScript source
│   ├── bridge/               # SharedArrayBuffer + FFI
│   ├── primitives/           # box, text, input
│   ├── engine/               # registry, lifecycle, arrays
│   └── arrays/               # Standalone array copies
├── rust/                     # Rust cdylib engine
│   └── src/
│       ├── lib.rs            # FFI exports (82 lines)
│       ├── shared_buffer.rs  # Memory layout contract
│       ├── types.rs          # Core types
│       └── layout/           # Taffy trait API + text measure
├── examples/                 # proof.ts, bench.ts
├── CLAUDE.md                 # Architecture bible (source of truth)
└── SESSION-HANDOFF.md        # This file
```

## Dependencies

- `@rlabs-inc/signals` v1.12.0 — TS reactive signals (source: memory-ts/packages/signals/)
- `spark-signals` v0.2.0 — Rust reactive signals (source: tui-rust/crates/spark-signals/)
- `taffy` 0.7 (+content_size) — W3C flexbox layout
- `bitflags` 2.9, `unicode-width` 0.2, `unicode-segmentation` 1

---

*Session • January 27, 2026 • São Paulo*
