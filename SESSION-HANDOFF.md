# SparkTUI Session Handoff — January 30, 2026 (Session 139)

## ✅ VALIDATED: 512-byte Buffer Has No Performance Regression

**Benchmarked layout computation on both buffer layouts:**
- OLD (256 bytes/node): 321 µs @ 1K nodes, 1731 µs @ 5K nodes
- NEW (512 bytes/node): 323 µs @ 1K nodes, 1744 µs @ 5K nodes
- **Result: 1.01x slower = essentially identical**

Cache-line alignment compensates for larger memory footprint. Safe to proceed with rewrite.

---

## Session 139: Buffer Layout Validation

### What We Did

1. **Simplified layout enum accessors** - Now return raw `u8` instead of typed enums (direct to Taffy, no intermediate conversion)
2. **Created `layout_tree.rs`** - New implementation using 512-byte SharedBuffer
3. **Added compatibility methods** - `terminal_width()`, `terminal_height()`, `set_output_scroll()`, `border_top()` aliases
4. **Fixed pre-existing broken tests** - Added `Rgba::ansi()`, color constants (RED, BLUE, etc.)
5. **Wrote benchmark** - `bench_layout.rs` comparing old vs new buffer layouts
6. **All 179 tests passing**

### Files Changed
```
rust/src/shared_buffer.rs      - layout enums return u8, added compat methods
rust/src/layout/layout_tree.rs - NEW: uses SharedBuffer (512 bytes/node)
rust/src/layout/mod.rs         - exports both compute_layout and compute_layout_aos
rust/src/utils/mod.rs          - added Rgba::ansi(), color constants
rust/src/bench_layout.rs       - NEW: benchmark comparing layouts
rust/src/lib.rs                - added bench_layout module
```

### Next: Continue Rewrite

Now that we've validated no regression, continue the calm, file-by-file rewrite:
1. Trace layout props through pipeline (SharedBuffer → Taffy → output)
2. Rewrite each file as encountered
3. Delete old shared_buffer_aos.rs once all files migrated

### Counter Example Status
The `examples/counter.ts` works perfectly:
- Terminal theme with 13 theme cycling (press 't')
- All features functional
- Great reference for testing during rewrite

---

## What SparkTUI Is

A **hybrid TUI framework** where TypeScript handles the developer-facing API (primitives, signals, reactivity) and Rust handles the engine (layout, rendering, terminal output). Connected by **SharedArrayBuffer** — zero-copy, zero-serialization shared memory.

**Tagline**: *All Rust benefits without borrowing a single thing.*

---

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

## Commands

```bash
# Build Rust
cd rust && cargo build --release
```

---

## Remember

- **No fixed FPS** — fully reactive, event-driven
- **No indices exposed** — users work with refs and IDs
- **<50μs event latency** — instant feel
