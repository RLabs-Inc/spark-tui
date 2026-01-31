# SparkTUI Rust Engine: Comprehensive Codebase Audit
**Date:** January 30, 2026
**Scope:** Full line-by-line review of `rust/src` (36 files)
**Goal:** Identify all technical debt, bugs, and architectural violations to guide the "Clean Rewrite".

---

## 1. Executive Summary

The architecture (SharedArrayBuffer + Atomic Wake + Reactive Graph) is sound and high-performance (200k+ FPS potential). However, the implementation is "organically grown" and fragile.

**Critical Issues:**
1.  **Unsafe Memory Access:** `shared_buffer_aos.rs` uses raw pointer arithmetic without bounds checking.
2.  **Type Inconsistency:** Coordinates mix `u16` (screen) and `i32` (scroll), leading to clipping bugs (Root Cause of Scroll Bug).
3.  **Optimization Misses:** Layout cache is cleared every frame; TextEditor re-allocates strings on every keystroke; FrameBuffer is cloned on every frame.
4.  **Code Duplication:** Renderers share 80% logic; Constants duplicated across 3 modules.
5.  **Shadow State:** Input managers (`FocusManager`, `ScrollManager`) maintain state that should live purely in SharedBuffer.

---

## 2. File-by-File Assessment

### Group 1: Foundation & Data (`src/types.rs`, `src/shared_buffer_aos.rs`)

**`src/types.rs`**
*   **Status:** Incomplete.
*   **Issues:**
    *   Missing core types: `ClipRect`, `Rgba`, `Cell` (currently in `utils/mod.rs`).
    *   `ClipRect` uses `u16`, preventing negative scroll coordinates (CRITICAL BUG).
    *   `BorderStyle::chars()` returns a 6-tuple instead of a struct.
    *   Missing `From<u8>` for many enums.

**`src/shared_buffer_aos.rs`**
*   **Status:** The "Minefield".
*   **Issues:**
    *   **Safety:** Hundreds of `ptr.add(...)` calls with NO bounds checking against `MAX_NODES`.
    *   **Magic Numbers:** Field offsets hardcoded (`HEADER_SIZE + 0 * STRIDE`).
    *   **API:** Mixes `read_node_u32` and direct `ptr::read`.
    *   **Concurrency:** `EventRingBuffer` logic duplicated here vs `input/events.rs`.

### Group 2: The Core Pipeline (`src/pipeline/*.rs`)

**`src/pipeline/setup.rs`**
*   **Status:** Monolithic "God Function".
*   **Issues:**
    *   `run_engine` handles terminal setup, threads, channel creation, reactive graph, and event loop.
    *   **Timeout Logic:** Mixes `blink.next_deadline()` with `recv_timeout`. Hard to reason about.
    *   **Shutdown:** Relies on channel disconnect; no clean teardown signal propagation.

**`src/pipeline/terminal.rs`**
*   **Status:** "Batteries Included" but messy globals.
    *   Uses `static mut ORIGINAL_TERMIOS` (unsafe).
    *   Enables *all* features (Mouse, Kitty, Paste, Focus) regardless of config.

**`src/pipeline/wake.rs`**
*   **Status:** Excellent.
    *   Adaptive spin-wait (spin -> yield -> sleep) is perfect for this architecture.
    *   Coalescing wakes is a good optimization.

### Group 3: FrameBuffer & Layout (`src/framebuffer/*.rs`, `src/layout/*.rs`)

**`src/framebuffer/render_tree.rs`**
*   **Status:** The Logic Knot.
    *   **Argument Explosion:** `render_component` takes 10 arguments.
    *   **Clipping Bug:** `abs_x.max(0) as u16` destroys negative scroll positions before clipping can happen.
    *   **Duplication:** `render_borders` logic duplicates `FrameBuffer::draw_border`.

**`src/layout/layout_tree_aos.rs`**
*   **Status:** Unoptimized Taffy Integration.
    *   **Cache Clear:** `ctx.cache[i].clear()` runs every frame for every node. Defeats Taffy's incremental layout.
    *   **RefCell:** Uses `thread_local` correctly to avoid allocation.

**`src/framebuffer/inheritance.rs`**
*   **Status:** O(N²) Performance Trap.
    *   `get_inherited_fg` walks up the parent tree for *every node*. Should be computed top-down.

### Group 4: Input System (`src/input/*.rs`)

**`src/input/reader.rs`**
*   **Status:** Potential Deadlock.
    *   Uses `stdin.lock().read()` (blocking) in a thread. Thread drop relies on process exit/stdin close.

**`src/input/cursor.rs`**
*   **Status:** Dead Code.
    *   `BlinkManager` is a stub.

**`src/input/text_edit.rs`**
*   **Status:** Inefficient.
    *   Allocates `Vec<char>` -> modifies -> `String` on every keystroke. O(N) allocation for text editing.

### Group 5: Renderer (`src/renderer/*.rs`)

**`src/renderer/diff.rs`**
*   **Status:** Allocation Heavy.
    *   `self.previous = Some(buffer.clone())` allocates 100KB+ per frame. Needs `mem::swap`.

**`src/renderer/buffer.rs`**
*   **Status:** Solid.
    *   Flat `Vec<Cell>` is correct.
    *   Clipping logic is robust (except for `u16` limitation).

---

## 3. The Refactor Mandate

We will proceed with the **Test-Driven Rewrite** as planned, but with these specific enforcements:

1.  **Strict Types:** `types.rs` is the Law. `ClipRect` uses `i32`.
2.  **Safety Wrappers:** `shared_buffer_aos.rs` gets a `NodeRef` wrapper with bounds checking.
3.  **Context Structs:** No functions with > 3 arguments. Use `RenderContext`, `LayoutContext`.
4.  **Zero Allocation:**
    *   Double-buffer `FrameBuffer` (swap, don't clone).
    *   Incremental Taffy layout (don't clear cache).
    *   Text editing operates on bytes/chars without full reallocation.

**Next Step:** Phase 1 Implementation (`types.rs`).
