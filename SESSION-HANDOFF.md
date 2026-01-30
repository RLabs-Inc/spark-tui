# SparkTUI Session Handoff — January 28, 2026 (Session 138)

## 🚨 CRITICAL: Events Not Wired

**The counter.ts renders but onClick/onKey handlers never fire.**

Why Tab navigation works but buttons don't respond:
- Tab → handled entirely in Rust → writes `focused_index` to SharedBuffer → TS sees it ✅
- onClick/onKey → Rust generates Event → pushes to **in-memory Vec** → TS never sees it ❌

**The fix is simple but touches many files:**

1. Rust has `buf.push_event(&event)` method (shared_buffer_aos.rs:450) - **EXISTS, NOT USED**
2. TS has `readEvents(buf)` function (ts/engine/events.ts:245) - **EXISTS, ALREADY CALLED**
3. Missing link: Rust input handlers push to in-memory `EventRingBuffer`, not SharedBuffer

**Files to change (replace `events.push(event)` → `buf.push_event(&event)`):**
- `rust/src/input/keyboard.rs` - 3 places
- `rust/src/input/mouse.rs` - 8 places
- `rust/src/input/focus.rs` - 3 places
- `rust/src/input/text_edit.rs` - 5 places

Will need to thread `&AoSBuffer` through the function signatures.

---

## Session 138: SoA Cleanup + Event Discovery

### What We Did

1. **Codebase audit** with 6 parallel agents - found SoA/AoS mismatch, dead code, type errors
2. **Deleted SoA code** - removed legacy files that were superseded by AoS
3. **Fixed type errors** - AoSSlotBuffer now implements SharedSlotBuffer interface
4. **Library now clean** - 0 type errors in ts/primitives, ts/engine, ts/bridge, ts/state
5. **Discovered event gap** - events stuck in Rust memory, never reach TS handlers

### Files Deleted (SoA cleanup)
```
rust/src/arrays/          (entire directory - old SoA)
rust/src/layout/titan.rs  (legacy layout engine)
rust/src/layout/taffy_bridge.rs (legacy)
ts/bridge/shared-buffer.ts (old SoA)
ts/bridge/reactive-arrays.ts (old SoA)
ts/bridge/buffer.ts (old SoA)
ts/engine/arrays/ (entire directory - old SoA)
examples/proof.ts, reactive-proof.ts, hello-world.ts (old SoA examples)
examples/bench.ts, bench-complete.ts, bench-headless.ts, bench-stress.ts
```

### Files Fixed
- `ts/bridge/index.ts` - removed SoA imports, AoS only now
- `ts/bridge/notify.ts` - removed SoA notifier
- `ts/bridge/aos-slot-buffer.ts` - implements full SharedSlotBuffer interface
- `ts/bridge/reactive-arrays-aos.ts` - uses SharedSlotBuffer type
- `ts/engine/inheritance.ts` - migrated from SoA arrays to AoS
- `ts/primitives/types.ts` - fixed KeyHandler, ScrollEvent types

### Remaining Technical Debt

| Issue | File | Severity |
|-------|------|----------|
| Text pool overflow silently corrupts | text.ts:144, input.ts:155 | CRITICAL |
| focusNext/Prev/First/Last are stubs | focus.ts:237-273 | HIGH |
| maxLength not enforced | text_edit.rs:86 | MEDIUM |

### Next Session: Wire Events

Priority order:
1. **Wire Rust events to SharedBuffer** (the blocker)
2. Test counter.ts onClick/onKey work
3. Fix text pool overflow (throw error instead of silent corruption)
4. Implement focusNext/Prev/First/Last properly

---

## READ FIRST

1. Read `CLAUDE.md` (architecture bible, source of truth)
2. Read `MIGRATION-AOS-PIPELINE.md` (detailed migration plan with checkboxes) **← START HERE**
3. Read `DESIGN-EVENT-BRIDGE.md` (event bridge design)

## What SparkTUI Is

A **hybrid TUI framework** where TypeScript handles the developer-facing API (primitives, signals, reactivity) and Rust handles the engine (layout, rendering, terminal output). Connected by **SharedArrayBuffer** — zero-copy, zero-serialization shared memory.

**Tagline**: *All Rust benefits without borrowing a single thing.*

---

## 🎯 SESSION 135: Event Bridge Design Complete

### What We Did

Comprehensive design session for the **Rust → TS event bridge**. Created `DESIGN-EVENT-BRIDGE.md` covering:

1. **Feature inventory** — Every feature from original TS implementation cataloged
2. **SharedBuffer layout** — AoS memory map with event ring buffer (~26.7MB total)
3. **Event ring buffer** — 256 slots × 20 bytes, Rust writes, TS reads
4. **Config flags** — All framework behaviors configurable (Ctrl+C, Tab nav, scroll, etc.)
5. **Public API** — No indices exposed! Users work with refs and IDs
6. **Event flow** — Full trace from user input to handler callback (<50μs latency)

### Key Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Index exposure | Hidden | Users use refs/IDs, never see indices |
| Event notification | Atomics.waitAsync | Instant, non-blocking, no polling |
| Config location | SharedBuffer header | Rust reads config before processing |
| Event ring buffer | In SharedBuffer | Zero-copy, same memory |
| Fixed FPS | **None** | Fully reactive, event-driven |
| Cursor blink | `pulse()` as prop | Animation value flows through repeat() |
| Animation naming | `cycle()`, `pulse()` | No React-style `use*` prefix |
| Rust timing | **None** | No BlinkManager - Rust just reads values |

### User-Facing API (No Indices!)

```typescript
// Components register handlers via props
box({
  id: 'my-box',
  onClick: (e) => console.log('clicked'),
  onFocus: () => console.log('focused'),
})

// Programmatic control via refs or IDs
const inputRef = createRef<InputHandle>()
input({ ref: inputRef, value: text })
inputRef.current?.focus()

// Or by ID
focus('my-box')
```

---

## 🎉 AoS MIGRATION STATUS (Session 137)

### THE PROBLEM
TS writes to AoS buffer (256-byte stride). Rust pipeline reads from SoA buffer (scattered arrays).
**THEY'RE NOT THE SAME MEMORY!**

### TypeScript (AoS) - COMPLETE ✅
| Component | Status |
|-----------|--------|
| `shared-buffer-aos.ts` | ✅ Complete |
| `reactive-arrays-aos.ts` | ✅ Complete |
| `aos-slot-buffer.ts` | ✅ Complete |
| `box.ts` | ✅ Migrated |
| `text.ts` | ✅ Migrated |
| `input.ts` | ✅ Migrated |

### Rust AoS Buffer - COMPLETE ✅
| Component | Status |
|-----------|--------|
| `shared_buffer_aos.rs` | ✅ 1600 lines, 195 tests |
| `layout_tree_aos.rs` | ✅ Taffy traits |

### Rust Pipeline - NEEDS MIGRATION ❌
| Component | Status |
|-----------|--------|
| `framebuffer/render_tree.rs` | ❌ Uses SoA SharedBuffer |
| `framebuffer/inheritance.rs` | ❌ Uses SoA SharedBuffer |
| `pipeline/setup.rs` | ❌ Uses SoA SharedBuffer |
| `pipeline/wake.rs` | ❌ Uses SoA SharedBuffer |
| `input/*.rs` (6 files) | ❌ Uses SoA SharedBuffer |

**See `MIGRATION-AOS-PIPELINE.md` for detailed migration plan with checkboxes.**

### Performance (AoS validated)

| Nodes | Time | vs Pure Rust |
|-------|------|--------------|
| 111 | 0.54μs | 7.4x FASTER |
| 5501 | 1.5ms | 4.1x FASTER |
| 20K | 0.3ms | 3,337 FPS |

---

## Next Steps: AoS Pipeline Migration

**See `MIGRATION-AOS-PIPELINE.md` for full detailed plan with checkboxes.**

### Phase 1: Add Missing Methods to AoSBuffer
- `clear_all_dirty()`, `increment_render_count()`
- Per-side border styles, focus indicator config
- Interaction flag setters

### Phase 2: Migrate Framebuffer
- Change `SharedBuffer` → `AoSBuffer` in render_tree.rs, inheritance.rs

### Phase 3: Migrate Pipeline
- Change `SharedBuffer` → `AoSBuffer` in setup.rs, wake.rs
- Change `compute_layout_direct` → `compute_layout_aos`

### Phase 4: Migrate Input Modules
- 6 files: keyboard.rs, mouse.rs, focus.rs, scroll.rs, text_edit.rs, cursor.rs

### Phase 5: Update lib.rs and FFI
- Add spark_wake, spark_cleanup exports

### Phase 6: Delete SoA Code
- Remove shared_buffer.rs, layout_tree.rs, taffy_bridge.rs, titan.rs
- Remove TS SoA files

### Phase 7: Hello World Test
- Atomics.waitAsync loop
- Ring buffer reader
- Event dispatch system

### Phase 4: TS State Modules
- keyboard.ts, mouse.ts, focus.ts
- cursor.ts, handlers.ts (internal)

### Phase 5: Primitive Integration
- Wire handlers and refs to box/text/input

### Phase 6: Testing
- Full integration tests

---

## File Structure

### TypeScript (AoS)

```
ts/bridge/
├── shared-buffer-aos.ts    # AoS constants + buffer creation
├── aos-slot-buffer.ts      # Fast direct-write slot buffers
├── reactive-arrays-aos.ts  # All fields as AoSSlotBuffers
├── notify.ts               # Wake notifier
└── index.ts                # initBridgeAoS(), getAoSArrays()

ts/primitives/
├── box.ts                  # MIGRATED to AoS
├── text.ts                 # MIGRATED to AoS
└── input.ts                # TODO: migrate to AoS

ts/state/                   # TODO: implement
├── keyboard.ts             # Event signals + global handlers
├── mouse.ts                # Position signals + handlers
├── focus.ts                # useFocusedId, focus(ref/id)
├── cursor.ts               # Terminal cursor control
└── handlers.ts             # Internal registries
```

### Rust (AoS)

```
rust/src/
├── lib.rs                  # FFI exports
├── shared_buffer_aos.rs    # AoS buffer (matches TS layout)
├── layout/
│   └── layout_tree_aos.rs  # Taffy integration
└── input/
    ├── events.rs           # Ring buffer (move to SharedBuffer!)
    ├── keyboard.rs         # Key dispatch
    ├── mouse.rs            # HitGrid + mouse dispatch
    └── focus.rs            # Focus manager
```

---

## Commands

```bash
# Build Rust
cd rust && cargo build --release

# Run AoS benchmark
bun run examples/bench-aos-vs-pure.ts

# Test primitives
bun run examples/test-box-aos.ts
```

---

## Key Constants (AoS Layout)

- `NODE_STRIDE = 256` bytes per node
- `HEADER_SIZE = 256` bytes
- `MAX_NODES = 100,000`
- `TEXT_POOL_SIZE = 1MB`
- `EVENT_RING_SIZE = 5,132` bytes (256 events × 20 bytes + 12 byte header)
- **Total buffer: ~26.7MB**

---

## Open Questions

1. **Cursor blink** — Ensure BlinkManager doesn't create fixed FPS
2. **Event overflow** — What if ring fills? (probably drop oldest)
3. **ID table** — TS-only Map or in SharedBuffer?

---

## Remember

- **AoS is the architecture** — all new code uses AoS
- **No fixed FPS** — fully reactive, event-driven
- **No indices exposed** — users work with refs and IDs
- **<50μs event latency** — instant feel
- **Read DESIGN-EVENT-BRIDGE.md** — comprehensive design doc
