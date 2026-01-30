# SparkTUI: AoS Pipeline Migration Plan

**Created**: Session 137 (January 28, 2026)
**Status**: In Progress
**Goal**: Migrate Rust pipeline from SoA SharedBuffer to AoS AoSBuffer

---

## The Problem

TypeScript writes to **AoS buffer** (256-byte stride per node).
Rust pipeline reads from **SoA buffer** (scattered arrays).
**They're not the same memory!**

The AoS buffer methods are 90% complete. We just need to:
1. Add ~8 missing methods to AoSBuffer
2. Change all Rust pipeline code to use AoSBuffer instead of SharedBuffer
3. Delete the old SoA code

---

## Architecture Principles (NON-NEGOTIABLE)

### Pure Reactivity - NO LOOPS
```
Props → Buffer → Notify → Layout Derived → Framebuffer Derived → Render Effect → Screen
```

- **NO render loops**
- **NO polling**
- **NO fixed FPS**
- **NO animation frames**
- **NO tick cycles**
- **NO futex_wait in loops**

The ONLY timing is TS-side animation signals (`pulse()`, `cycle()`) which are signal SOURCES that write to buffer. Rust just reads.

### Focus Indicator (not "ring")
- Single `*` character at top-right of focused focusables
- For text components: `*` at end of text
- Color: theme accent
- Configurable: replace char or disable entirely
- Default: enabled
- Developers can implement their own focus states

### Auto Scroll
- During layout: if child > parent dimensions:
  - Parent becomes focusable automatically
  - Parent becomes scrollable automatically
  - Scrolls via default controls (mouse + keyboard)
- Opt-out per component (enabled by default)

### Render Modes

| Mode | Height | Mouse | Rendering | Auto Scroll |
|------|--------|-------|-----------|-------------|
| Fullscreen (0) | Terminal | Enabled | Diff | Root gets it if content > terminal |
| Inline (1) | Content | Disabled | Full redraw | N/A (terminal native) |
| Append (2) | Content | Disabled | Full redraw | N/A (terminal native) |

---

## Current State

### TypeScript (AoS) - COMPLETE
- [x] `ts/bridge/shared-buffer-aos.ts` - Memory layout (256-byte stride)
- [x] `ts/bridge/reactive-arrays-aos.ts` - Reactive wrappers via aosSlotBuffer
- [x] `ts/bridge/aos-slot-buffer.ts` - Direct DataView writes + Atomics notify
- [x] `ts/primitives/box.ts` - Uses repeat() + AoS arrays
- [x] `ts/primitives/text.ts` - Uses repeat() + AoS arrays
- [x] `ts/primitives/input.ts` - Uses repeat() + AoS arrays

### Rust AoS Buffer - COMPLETE
- [x] `rust/src/shared_buffer_aos.rs` - 1600 lines, 195 tests, all core accessors
- [x] `rust/src/layout/layout_tree_aos.rs` - Taffy traits for AoS

### Rust Pipeline (SoA) - NEEDS MIGRATION
- [ ] `rust/src/framebuffer/render_tree.rs` - Uses SharedBuffer
- [ ] `rust/src/framebuffer/inheritance.rs` - Uses SharedBuffer
- [ ] `rust/src/pipeline/setup.rs` - Uses SharedBuffer
- [ ] `rust/src/pipeline/wake.rs` - Uses SharedBuffer
- [ ] `rust/src/input/keyboard.rs` - Uses SharedBuffer
- [ ] `rust/src/input/mouse.rs` - Uses SharedBuffer
- [ ] `rust/src/input/focus.rs` - Uses SharedBuffer
- [ ] `rust/src/input/scroll.rs` - Uses SharedBuffer
- [ ] `rust/src/input/text_edit.rs` - Uses SharedBuffer
- [ ] `rust/src/input/cursor.rs` - Uses SharedBuffer

---

## Phase 1: Add Missing Methods to AoSBuffer

**File**: `rust/src/shared_buffer_aos.rs`

### Methods to Add

**NO LEGACY SUPPORT NEEDED** - we're implementing from scratch, do it clean.

```rust
// 1. Clear all dirty flags for a node
pub fn clear_all_dirty(&self, i: usize) {
    self.write_node_u8(i, U_DIRTY_FLAGS, 0);
}

// 2. Increment render count (for FPS tracking)
// Add H_RENDER_COUNT constant to header (offset 88)
pub fn increment_render_count(&self) {
    let count = self.read_header_u32(H_RENDER_COUNT);
    self.write_header_u32(H_RENDER_COUNT, count.wrapping_add(1));
}

// 3. Per-side border styles (add constants in reserved space 143-146)
pub const U_BORDER_STYLE_TOP: usize = 143;
pub const U_BORDER_STYLE_RIGHT: usize = 144;
pub const U_BORDER_STYLE_BOTTOM: usize = 145;
pub const U_BORDER_STYLE_LEFT: usize = 146;

#[inline]
pub fn border_style_top(&self, i: usize) -> u8 { self.read_node_u8(i, U_BORDER_STYLE_TOP) }
#[inline]
pub fn border_style_right(&self, i: usize) -> u8 { self.read_node_u8(i, U_BORDER_STYLE_RIGHT) }
#[inline]
pub fn border_style_bottom(&self, i: usize) -> u8 { self.read_node_u8(i, U_BORDER_STYLE_BOTTOM) }
#[inline]
pub fn border_style_left(&self, i: usize) -> u8 { self.read_node_u8(i, U_BORDER_STYLE_LEFT) }

// 4. Focus indicator config (single '*' char, theme accent, configurable)
pub const U_FOCUS_INDICATOR_CHAR: usize = 147; // u8, default '*' (0x2A)
pub const U_FOCUS_INDICATOR_ENABLED: usize = 148; // u8, 1=enabled (default), 0=disabled

#[inline]
pub fn focus_indicator_char(&self, i: usize) -> char {
    let ch = self.read_node_u8(i, U_FOCUS_INDICATOR_CHAR);
    if ch == 0 { '*' } else { ch as char }
}

#[inline]
pub fn focus_indicator_enabled(&self, i: usize) -> bool {
    self.read_node_u8(i, U_FOCUS_INDICATOR_ENABLED) != 0
}

// 5. Cursor alt char (character shown during blink-off phase)
pub const U_CURSOR_ALT_CHAR: usize = 233; // u32 in reserved output space

#[inline]
pub fn cursor_alt_char(&self, i: usize) -> u32 {
    self.read_node_u32(i, U_CURSOR_ALT_CHAR)
}

// 6. Interaction flag setters (Rust writes these during input processing)
pub fn set_hovered(&self, i: usize, val: bool) {
    let flags = self.interaction_flags(i);
    let new_flags = if val { flags | FLAG_HOVERED } else { flags & !FLAG_HOVERED };
    self.write_node_u8(i, U_INTERACTION_FLAGS, new_flags);
}

pub fn set_pressed(&self, i: usize, val: bool) {
    let flags = self.interaction_flags(i);
    let new_flags = if val { flags | FLAG_PRESSED } else { flags & !FLAG_PRESSED };
    self.write_node_u8(i, U_INTERACTION_FLAGS, new_flags);
}

pub fn set_focused(&self, i: usize, val: bool) {
    let flags = self.interaction_flags(i);
    let new_flags = if val { flags | FLAG_FOCUSED } else { flags & !FLAG_FOCUSED };
    self.write_node_u8(i, U_INTERACTION_FLAGS, new_flags);
}
```

### Checklist
- [ ] Add `H_RENDER_COUNT` constant (offset 88 in header)
- [ ] Add `clear_all_dirty()`
- [ ] Add `increment_render_count()`
- [ ] Add per-side border style constants (U_BORDER_STYLE_TOP/RIGHT/BOTTOM/LEFT)
- [ ] Add `border_style_top/right/bottom/left()` methods
- [ ] Add focus indicator constants (U_FOCUS_INDICATOR_CHAR, U_FOCUS_INDICATOR_ENABLED)
- [ ] Add `focus_indicator_char()` and `focus_indicator_enabled()` methods
- [ ] Add `U_CURSOR_ALT_CHAR` constant and `cursor_alt_char()` method
- [ ] Add `set_hovered()`, `set_pressed()`, `set_focused()`
- [ ] Run tests: `cargo test` (should still pass 195+)

---

## Phase 2: Migrate Framebuffer

**Files**:
- `rust/src/framebuffer/render_tree.rs`
- `rust/src/framebuffer/inheritance.rs`

### Changes Required

1. Change import:
```rust
// FROM:
use crate::shared_buffer::SharedBuffer;
// TO:
use crate::shared_buffer_aos::AoSBuffer;
```

2. Change function signatures:
```rust
// FROM:
pub fn compute_framebuffer(buf: &SharedBuffer, ...) -> ...
// TO:
pub fn compute_framebuffer(buf: &AoSBuffer, ...) -> ...
```

3. Update method calls to use AoS names (no aliases, clean implementation):
- `scroll_offset_x()` → `scroll_x()`
- `scroll_offset_y()` → `scroll_y()`
- `set_scroll_offset()` → `set_scroll()`
- `border_top_width()` → `border_top()`
- `show_focus_ring()` → `focus_indicator_enabled()` + `focusable()`

### Checklist
- [ ] Update `render_tree.rs` imports
- [ ] Update `render_tree.rs` function signatures
- [ ] Update `inheritance.rs` imports
- [ ] Update `inheritance.rs` function signatures
- [ ] Run tests: `cargo test`

---

## Phase 3: Migrate Pipeline

**Files**:
- `rust/src/pipeline/setup.rs`
- `rust/src/pipeline/wake.rs`

### Changes Required

1. Change imports from `shared_buffer` to `shared_buffer_aos`
2. Change `SharedBuffer` to `AoSBuffer` in all signatures
3. Change layout call:
```rust
// FROM:
layout::compute_layout_direct(buf);
// TO:
layout::compute_layout_aos(buf);
```

4. Update framebuffer call (after Phase 2):
```rust
framebuffer::compute_framebuffer(buf, tw, th)
```

### Checklist
- [ ] Update `setup.rs` imports
- [ ] Update `setup.rs` type signatures
- [ ] Change `compute_layout_direct` → `compute_layout_aos`
- [ ] Update `wake.rs` imports
- [ ] Update `wake.rs` type signatures
- [ ] Run tests: `cargo test`

---

## Phase 4: Migrate Input Modules

**Files** (6 total):
- `rust/src/input/keyboard.rs`
- `rust/src/input/mouse.rs`
- `rust/src/input/focus.rs`
- `rust/src/input/scroll.rs`
- `rust/src/input/text_edit.rs`
- `rust/src/input/cursor.rs`

### Changes Required

Same pattern for each:
1. Change import from `shared_buffer::SharedBuffer` to `shared_buffer_aos::AoSBuffer`
2. Change all `&SharedBuffer` to `&AoSBuffer` in function signatures
3. Update any method calls that have different names

### Checklist
- [ ] Migrate `keyboard.rs`
- [ ] Migrate `mouse.rs`
- [ ] Migrate `focus.rs`
- [ ] Migrate `scroll.rs`
- [ ] Migrate `text_edit.rs`
- [ ] Migrate `cursor.rs`
- [ ] Run tests: `cargo test`

---

## Phase 5: Update lib.rs and FFI

**File**: `rust/src/lib.rs`

### Changes Required

1. Remove SoA module declarations:
```rust
// REMOVE:
mod shared_buffer;
```

2. Update FFI functions to use AoS:
```rust
// The spark_init, spark_compute_layout, etc. should use AoSBuffer
```

3. Add missing FFI exports:
```rust
#[no_mangle]
pub extern "C" fn spark_wake(ptr: *mut u8, len: usize) -> i32 { ... }

#[no_mangle]
pub extern "C" fn spark_cleanup(ptr: *mut u8, len: usize) -> i32 { ... }
```

### Checklist
- [ ] Update module declarations in `lib.rs`
- [ ] Update FFI to use AoSBuffer
- [ ] Add `spark_wake` FFI export
- [ ] Add `spark_cleanup` FFI export
- [ ] Run tests: `cargo test`
- [ ] Build: `cargo build --release`

---

## Phase 6: Delete SoA Code

**Files to DELETE**:

### Rust
- [ ] `rust/src/shared_buffer.rs` (SoA buffer)
- [ ] `rust/src/layout/layout_tree.rs` (SoA layout)
- [ ] `rust/src/layout/taffy_bridge.rs` (legacy, cfg-gated)
- [ ] `rust/src/layout/titan.rs` (legacy, cfg-gated)
- [ ] `rust/src/shared_buffer_aos.rs.backup` (backup file)

### TypeScript
- [ ] `ts/bridge/shared-buffer.ts` (SoA)
- [ ] `ts/bridge/reactive-arrays.ts` (SoA)
- [ ] `ts/bridge/buffer.ts` (SoA helpers)

### Update Exports
- [ ] Update `ts/bridge/index.ts` to remove SoA exports
- [ ] Update `rust/src/layout/mod.rs` to remove SoA exports

---

## Phase 7: Hello World Test

Create a working hello world that proves the full pipeline:

```typescript
// examples/hello-world-aos.ts
import { initBridgeAoS, getAoSArrays, getAoSNotifier } from '../ts/bridge'
import { box, text } from '../ts/primitives'
import { spark_init } from '../ts/bridge/ffi'

// 1. Initialize AoS bridge
const bridge = initBridgeAoS()

// 2. Create UI
box({
  width: 40,
  height: 10,
  border: 1,
  children: () => {
    text({ content: 'Hello, SparkTUI!' })
  }
})

// 3. Set terminal size in header
setTerminalSize(bridge.buf, 80, 24)

// 4. Start Rust engine (enters fullscreen, starts reactive pipeline)
spark_init(bridge.buf.buffer, bridge.buf.buffer.byteLength)

// The engine will:
// - Read dirty flags
// - Run layout if dirty
// - Build framebuffer
// - Render to terminal
// - Wait for next wake or input
```

### Checklist
- [ ] Add FFI exports to `ts/bridge/ffi.ts`
- [ ] Create `examples/hello-world-aos.ts`
- [ ] Test: `bun examples/hello-world-aos.ts`
- [ ] Verify text appears on screen
- [ ] Verify Ctrl+C exits cleanly

---

## Verification

After all phases complete:

```bash
# All Rust tests pass
cargo test

# Build succeeds
cargo build --release

# Hello world runs
bun examples/hello-world-aos.ts

# No SoA references remain
grep -r "SharedBuffer" rust/src/ --include="*.rs" | grep -v shared_buffer_aos
# Should return nothing
```

---

## Important Notes

### NO LEGACY SUPPORT
We are still implementing this framework. There are no users, no backwards compatibility requirements. Do it the RIGHT way:
- Delete old code aggressively
- Use clean naming (no aliases for old names)
- One architecture, one implementation
- If it's SoA, it's gone

### BlinkManager Timing Violation
`cursor.rs` has timing logic (Instant, Duration) that violates "NO timing in Rust" principle. After migration, cursor blink should come from TS `pulse()` signal writing to SharedBuffer. Rust just reads `cursor_visible`. **Delete the BlinkManager timing code.**

### Duplicated TS Helpers
`toDim()`, `unwrap()`, `isReactive()` etc. are copy-pasted in box.ts, text.ts, input.ts. Consolidate to `ts/primitives/utils.ts`. Low priority - do after migration.

---

*Last updated: Session 137*
