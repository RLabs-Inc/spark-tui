# Session Handoff — SparkTUI Rust Rewrite

**Date:** January 31, 2026
**Phase:** File-by-file rewrite following reactive data flow
**Status:** lib.rs, setup.rs, wake.rs DONE — render_tree.rs NEXT

---

## What We Did This Session

### 1. Fixed EmptyLineNames Lifetime Issue
- Made `EmptyLineNames<'a>` generic over lifetime
- Layout engine now 100% complete with Grid support
- All Taffy 0.9 traits properly implemented

### 2. Cleaned Layout Module
- Deleted `types.rs` (dead code, duplicate Overflow enum)
- Deleted all backup files (.bak, .old)
- Layout module is now clean: `layout_tree.rs` + `text_measure/`

### 3. Added Buffer Size Helpers
- `calculate_buffer_size(max_nodes, text_pool_size)` in shared_buffer.rs
- `DEFAULT_BUFFER_SIZE` constant (~20.7 MB)
- Enables TS to allocate correct buffer size

### 4. Rewrote lib.rs (Clean, from scratch)
- Uses `SharedBuffer` only (not AoSBuffer)
- 4 FFI exports: `spark_init`, `spark_buffer_size`, `spark_buffer_size_custom`, `spark_wake`, `spark_cleanup`
- Wake test functions moved to pipeline/wake.rs, re-exported
- Removed `spark_compute_layout()` and `spark_render()` (imperative paths — we're fully reactive)

### 5. Rewrote pipeline/setup.rs (Clean, from scratch)
- Uses `SharedBuffer` and `RenderMode` enum
- Same reactive structure: generation → layout_derived → fb_derived → render_effect
- Updated method names: `output_width` → `computed_width`, `clear_all_dirty` → `clear_dirty`

### 6. Updated pipeline/wake.rs
- Uses `SharedBuffer` instead of `AoSBuffer`
- Contains wake test FFI functions (moved from lib.rs)

---

## Files Changed

```
DONE (SharedBuffer):
  rust/src/lib.rs              — Clean rewrite, SharedBuffer only
  rust/src/pipeline/setup.rs   — Clean rewrite, reactive graph
  rust/src/pipeline/wake.rs    — Updated + wake test FFI functions
  rust/src/shared_buffer.rs    — Added calculate_buffer_size(), DEFAULT_BUFFER_SIZE
  rust/src/layout/layout_tree.rs — Fixed EmptyLineNames<'a> lifetime
  rust/src/layout/mod.rs       — Clean (removed types reference)

DELETED:
  rust/src/layout/types.rs     — Dead code
  rust/src/layout/*.bak        — Old backups

BACKUP (will delete when done):
  rust/src/lib.rs.bkp
  rust/src/pipeline/setup.rs.bkp

STILL ON AoSBuffer (next to rewrite):
  rust/src/framebuffer/render_tree.rs  ← NEXT (key file!)
  rust/src/framebuffer/inheritance.rs
  rust/src/input/keyboard.rs
  rust/src/input/mouse.rs
  rust/src/input/scroll.rs
  rust/src/input/focus.rs
  rust/src/input/text_edit.rs
  rust/src/input/cursor.rs
```

---

## Current Cascade Errors

```
framebuffer/render_tree.rs:57  — compute_framebuffer() expects AoSBuffer
input/keyboard.rs:23           — dispatch_key() expects AoSBuffer
input/mouse.rs:104             — MouseManager::dispatch() expects AoSBuffer
input/cursor.rs                — BlinkManager::tick() expects AoSBuffer
```

---

## Next Session: render_tree.rs

This is the KEY file that drove the entire rewrite. It:
1. Reads layout output (computed_x/y/w/h) from SharedBuffer
2. Reads visual properties (colors, borders, opacity)
3. Builds 2D Cell grid (framebuffer)
4. Collects hit regions for mouse dispatch

**Before rewriting:**
1. Read current render_tree.rs together
2. Map all features/behaviors
3. Agree on spec
4. Write tests
5. Implement clean

---

## Architecture Reminders

### Reactive Flow
```
SharedBuffer (props)
  → layout_derived (Taffy → computed positions)
    → fb_derived (render_tree → Cell grid)
      → render_effect (diff → ANSI → terminal)
```

### Buffer Sizes
- Node stride: 1024 bytes (16 cache lines)
- Default: 10,000 nodes, 10MB text pool (~20.7 MB total)
- Configurable via `spark_buffer_size_custom()`

### Key Method Mappings (AoS → SharedBuffer)
- `output_width(i)` → `computed_width(i)`
- `output_height(i)` → `computed_height(i)`
- `clear_all_dirty(i)` → `clear_dirty(i)`
- `render_mode()` returns `RenderMode` enum (not u8)

---

## Rewrite Philosophy (Never Forget)

- **No patches, no workarounds** — delete and rewrite clean
- **Production-grade code** — something we'll be proud of years later
- **No feature regression** — we only improve or add
- **Follow reactive flow** — file by file, prop by prop
- **SSOT** — SharedBuffer is the single source of truth

---

*Sherlock & Watson — lib.rs and setup.rs done, render_tree.rs next*
