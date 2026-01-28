# Plan: Cross-Language Reactive Shared Memory

## Status

- **Phase 1 (TS SharedSlotBuffer + Repeater + markReactions)** — DONE
- **Phase 2 (Rust SharedSlotBuffer + Repeater + mark_reactions)** — DONE
- **Phase 3 (Wire into SparkTUI)** — NEXT
- **Phase 4 (Migrate Primitives + Wire Rust Pipeline)** — TODO
- **Phase 5 (Interaction / Event System)** — TODO (later)
- **Phase 6 (Clean Up Legacy Arrays)** — TODO

---

## Three-Layer Architecture

### Layer 1: SharedSlotBuffer (generic, signals packages)
Reactive typed arrays backed by shared memory. `get()` tracks dependencies, `set()` writes to shared memory + notifies reactive graph + notifies cross-side. **No source binding here** — the buffer is just a reactive read/write array.

### Layer 2: Repeater (generic, signals packages)
A **new reactive graph node** — NOT an effect, NOT a derived. A purpose-built forwarding node that runs **inline during markReactions**. Connects any reactive source (signal/derived/getter) to any reactive target (like a buffer position). When source changes → repeater forwards value to target. Zero scheduling overhead.

### Layer 3: Primitive Components (SparkTUI-specific)
Developer-facing API (`box()`, `text()`, `input()`). Uses repeaters to connect developer props to buffer positions. Users never see arrays — they work with props (signals, deriveds, getters, static values).

### The Full Reactive Chain

```
Developer: myWidth.value = 150
  → markReactions(myWidth, DIRTY)
    → walks reactions, finds REPEATER node
      → REPEATER.forward() runs INLINE during markReactions:
        → reads myWidth.v (already updated to 150)
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

No effects. No scheduling for the binding. No sync() calls. No FFI calls after init. The repeater runs inline during the reactive graph traversal.

---

## Phase 1: TS SharedSlotBuffer + Repeater + markReactions Modification (DONE)

**Package**: `@rlabs-inc/signals`
**Source**: `/Users/rusty/Documents/Projects/AI/Tools/ClaudeTools/memory-ts/packages/signals/`

**New files**:
- `packages/signals/src/shared/shared-slot-buffer.ts` — SharedSlotBuffer class
- `packages/signals/src/shared/notifier.ts` — Notifier interface + AtomicsNotifier + NoopNotifier
- `packages/signals/src/primitives/repeater.ts` — Repeater node + repeat() factory

**Modified files**:
- `packages/signals/src/reactivity/tracking.ts` — Add REPEATER handling in markReactions
- `packages/signals/src/core/constants.ts` — Add REPEATER flag (1 << 19)
- `packages/signals/src/index.ts` — Add exports

**Verification**: `tsc --noEmit` — zero errors.

---

## Phase 2: Rust SharedSlotBuffer + Repeater + mark_reactions Modification (DONE)

**Package**: `spark-signals`
**Source**: `/Users/rusty/Documents/Projects/TUI/tui-rust/crates/spark-signals/`

**New files**:
- `crates/spark-signals/src/shared/shared_slot_buffer.rs`
- `crates/spark-signals/src/shared/notify.rs`
- `crates/spark-signals/src/primitives/repeater.rs`

**Modified files**:
- `crates/spark-signals/src/reactivity/tracking.rs` — Add REPEATER handling in mark_reactions
- `crates/spark-signals/src/core/constants.rs` — Add REPEATER flag (1 << 19)
- `crates/spark-signals/src/shared/mod.rs` — Add modules + platform_wake()
- `crates/spark-signals/src/primitives/mod.rs` — Add repeater module
- `crates/spark-signals/src/lib.rs` — Add exports

**Verification**: `cargo build` zero warnings. `cargo test` — 343 tests, 0 failures.

---

## Phase 3: Wire into SparkTUI (NEXT)

**SparkTUI files**:
- **REWRITE**: `ts/bridge/shared-buffer.ts` — SoA layout + per-field TypedArray views
- **NEW**: `ts/bridge/reactive-arrays.ts` — SharedSlotBuffers backed by SharedArrayBuffer
- **NEW**: `ts/bridge/notify.ts` — AtomicsNotifier wired to wake flag
- **REWRITE**: `rust/src/shared_buffer.rs` — SoA layout

**SoA Layout** (~2.1MB for 4096 nodes):
```
Header (64B) | Dirty Flags (4096B) | Float32 (31 fields × 4096 × 4B) |
Uint32 (12 × 4096 × 4B) | Int32 (13 × 4096 × 4B) | Uint8 (28 × 4096) | Text Pool (1MB)
```

### What to build:

1. **SoA SharedBuffer layout (TS)** — Rewrite `ts/bridge/shared-buffer.ts`:
   - Calculate byte offsets for each field as contiguous typed arrays
   - Each field (width, height, flex_grow, fg_color, etc.) gets its own contiguous array
   - All arrays are views over a single SharedArrayBuffer
   - Export offset constants and helper functions

2. **Reactive arrays (TS)** — New `ts/bridge/reactive-arrays.ts`:
   - Create SharedSlotBuffers backed by the SharedArrayBuffer views
   - Group them: `layoutArrays`, `visualArrays`, `textArrays`, `interactionArrays`
   - Wire to a single AtomicsNotifier pointing at the wake flag
   - Share dirty flags across all buffers

3. **Notify bridge (TS)** — New `ts/bridge/notify.ts`:
   - Create AtomicsNotifier wired to the header's wake flag offset
   - Export for use by reactive-arrays

4. **SoA SharedBuffer layout (Rust)** — Rewrite `rust/src/shared_buffer.rs`:
   - Same byte offsets as TS (MUST match exactly)
   - Create SharedSlotBuffers for each field
   - Use `notify_changed()` after wake to propagate to Rust reactive graph

### SoA Field Inventory

**Float32 fields (31 per node):**
width, height, min_width, min_height, max_width, max_height,
flex_grow, flex_shrink, flex_basis,
padding_top, padding_right, padding_bottom, padding_left,
margin_top, margin_right, margin_bottom, margin_left,
gap_row, gap_column,
inset_top, inset_right, inset_bottom, inset_left,
opacity,
computed_x, computed_y, computed_width, computed_height,
scrollable_width, scrollable_height, max_scroll

**Uint32 fields (12 per node):**
fg_color, bg_color,
border_top_color, border_right_color, border_bottom_color, border_left_color,
focus_ring_color, cursor_color,
text_offset, text_length,
z_index, tab_index

**Int32 fields (13 per node):**
parent_index,
scroll_x, scroll_y,
cursor_position, selection_start, selection_end,
hover_state, pressed_state, focused_state,
content_width_cache, content_height_cache,
first_child_index, child_count

**Uint8 fields (28 per node):**
component_type, visible, display, position_type,
flex_direction, flex_wrap, justify_content, align_items, align_self, align_content,
overflow_x, overflow_y,
border_top_style, border_right_style, border_bottom_style, border_left_style,
border_top_width, border_right_width, border_bottom_width, border_left_width,
text_align, text_wrap, text_overflow,
focusable, disabled,
dirty_layout, dirty_visual, dirty_text

---

## Phase 4: Migrate Primitives + Wire Rust Pipeline (TODO)

**Modified SparkTUI files**:
- `ts/primitives/box.ts`, `text.ts`, `input.ts` — use `repeat()` instead of `setSource()`
- `ts/engine/registry.ts` — cleanup uses new arrays
- `rust/src/lib.rs` — reactive pipeline with SharedSlotBuffers

**Primitive migration**:
```ts
// BEFORE:
taffy.width.setSource(index, taffy.dimensionSource(props.width))

// AFTER:
const dispose = repeat(props.width, layout.arrays.width, index)
```

**Rust reactive pipeline**:
```rust
loop {
    wait_for_wake(&wake_flag);           // blocks until TS notifies
    width_buf.notify_changed();          // → mark_reactions → layout_derived dirty
    // reactive graph propagates: layout → framebuffer → render effect → terminal
}
```

---

## Phase 5: Interaction / Event System (planned later)

## Phase 6: Clean Up Legacy Arrays

- **DELETE**: `ts/engine/arrays/` (core, dimensions, spacing, layout, visual, text, interaction, dirty, taffy)

---

## Key Design Decisions

1. **Three Layers**: SharedSlotBuffer (reactive array) + Repeater (forwarding node) + Primitives (wiring). Clean separation of concerns. Layers 1 and 2 are generic.

2. **Repeater runs inline in markReactions** — not scheduled, not deferred. One flag check added to the inner loop. No effects, no scheduling overhead.

3. **Permanent dependencies** — Repeater has ONE dep (set at creation, removed at disposal). No dep rewiring on every update. No reaction context setup.

4. **Pluggable Notifier** — Atomics is default, users can plug anything. Packages don't know about transport.

5. **Factory + External Buffer** — SharedSlotBuffer supports both internal allocation and external SharedArrayBuffer views (fixed capacity).

6. **One FFI call, then shared memory only** — All communication is shared memory + 1-bit Atomics notification after initial setup.

7. **Users never see arrays** — Users work with props. Primitives use repeaters. Arrays are internal plumbing.

---

## Verification Checklist

### Phase 1 (TS) — PASSED
1. ✅ Repeater test: `repeat(signal, buffer, 0)` → change signal → buffer[0] updated inline
2. ✅ SharedSlotBuffer: get() inside derived → set() → derived re-evaluates
3. ✅ Equality: set(same value) → no notification
4. ✅ Notification batching via microtask
5. ✅ External SharedArrayBuffer support
6. ✅ tsc --noEmit zero errors

### Phase 2 (Rust) — PASSED
1. ✅ SharedSlotBuffer: get/set with equality check and dirty flags
2. ✅ mark_reactions: REPEATER inline forwarding works
3. ✅ platform_wake: sets wake flag
4. ✅ `cargo test` — 343 tests, 0 failures

### Phase 3 (SparkTUI) — TODO
1. TS + Rust agree on SoA byte offsets
2. End-to-end: change TS signal → repeater writes to shared memory → Rust wakes → layout → framebuffer → render

### Phase 4 — TODO
1. All primitives use repeat() instead of setSource()
2. Rust pipeline driven by reactive graph (no polling)
