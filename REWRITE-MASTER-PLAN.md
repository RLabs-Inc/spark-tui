# SparkTUI Rust Engine: The Master Rewrite Plan

**Version:** 1.0
**Date:** January 30, 2026
**Objective:** Transform the "organically grown" prototype into a production-grade, memory-safe, zero-allocation TUI engine.

---

## 1. The Current State (The "Raw Truth")

Our deep audit of the codebase revealed a high-performance architecture (213k FPS potential) held back by fragile implementation details.

### Critical Findings
1.  **Root Cause of Scroll Bugs:** `ClipRect` uses `u16` coordinates. Negative scroll positions (e.g., scrolling up) are impossible to represent, leading to incorrect clamping and layout resets.
2.  **Memory Safety Minefield:** `shared_buffer_aos.rs` relies on raw pointer arithmetic (`ptr.add`) with **zero** bounds checking. A single bad index from TypeScript causes a Segfault.
3.  **Performance Leaks:**
    *   **Layout:** The Taffy layout cache is cleared *every frame*, defeating the purpose of incremental layout.
    *   **Rendering:** The `DiffRenderer` clones the entire `FrameBuffer` (100KB+) every frame instead of using double-buffering.
    *   **Text Editing:** Input handling re-allocates `String` buffers on every keystroke (O(N) allocation).
4.  **Shadow State:** Input managers (`FocusManager`, `ScrollManager`) maintain internal state that can drift from the `SharedBuffer` source of truth.
5.  **Code Duplication:** `InlineRenderer`, `DiffRenderer`, and `AppendRenderer` share ~80% of their logic but are separate, duplicated implementations.

---

## 2. The Target Architecture

We are not changing the core idea (SharedArrayBuffer + Reactivity). We are enforcing it.

### Core Principles
1.  **Single Source of Truth:** `SharedBuffer` is the *only* place state lives. No shadow state in Rust structs.
2.  **Strict Types:**
    *   **Layout/Clipping:** `i32` (supports negative scroll/off-screen content).
    *   **Screen/Render:** `u16` (terminal cells are always positive).
    *   **Indices:** `usize` (memory access).
3.  **Zero-Copy / Zero-Allocation:**
    *   Layout: Incremental updates (only dirty nodes).
    *   Render: Double-buffer swapping (no clones).
    *   Text: Edit in-place in the text pool.
4.  **Safety First:** All `SharedBuffer` access must be bounds-checked. `unsafe` is contained to one small, audited module.

---

## 3. Execution Plan (Atomic & Test-Driven)

We will execute this in **8 Phases**. Each phase is a strict sequence:
1.  **Spec:** Define types/traits.
2.  **Test:** Write tests that fail (proving the need).
3.  **Impl:** Write code to pass tests.
4.  **Verify:** Run suite.

### Phase 1: The Foundation (`types.rs`)
*Goal: Fix the coordinate system and consolidate types.*

- [ ] **1.1 Move Types:** Move `Rgba`, `Cell`, `Attr`, `ClipRect` from `utils/mod.rs` to `types.rs`.
- [ ] **1.2 Fix ClipRect:** Change `ClipRect` fields from `u16` to `i32`.
    -   *Test:* `test_clip_rect_negative_coords` (Verify intersection works with negative/off-screen rects).
- [ ] **1.3 Consolidate Enums:** Ensure all enums (`BorderStyle`, `ComponentType`) are in `types.rs` and have `From<u8>`.
    -   *Test:* `test_enums_from_u8` (Verify safe conversion for all valid/invalid bytes).
- [ ] **1.4 Fix Colors:** Ensure `Rgba` handles `TERMINAL_DEFAULT` correctly vs `WHITE`.
    -   *Test:* `test_rgba_terminal_default`.

### Phase 2: Safe Data Access (`shared_buffer_aos.rs`)
*Goal: Eliminate segfault risks and magic numbers.*

- [ ] **2.1 Bounds Checking:** Implement a private `check_bounds(index)` method used by *all* accessors.
    -   *Test:* `test_out_of_bounds_access` (Verify panics/safe returns on bad indices).
- [ ] **2.2 Magic Number Cleanup:** Replace `0 * STRIDE` patterns with named constants.
- [ ] **2.3 Type Safety:** Update accessors to return `i32` for scroll/position fields (matching Phase 1).
- [ ] **2.4 Ring Buffer:** Move `EventRingBuffer` logic here (single writer implementation).

### Phase 3: The FrameBuffer (`renderer/buffer.rs` & `framebuffer/render_tree.rs`)
*Goal: Fix clipping logic and reduce argument explosion.*

- [ ] **3.1 Update FrameBuffer:** Update drawing methods to accept `ClipRect` with `i32` (convert to `u16` only at draw time).
    -   *Test:* `test_draw_with_negative_clip`.
- [ ] **3.2 Context Struct:** Introduce `RenderContext` to replace the 10-argument `render_component` signature.
- [ ] **3.3 Fix Render Tree:** Rewrite `render_component` to use `i32` for all position math. **This fixes the scroll bug.**
    -   *Test:* `test_render_nested_scroll` (Verify child coordinates are correct when parent is scrolled).

### Phase 4: Zero-Alloc Renderer (`renderer/diff.rs`)
*Goal: Eliminate the 100KB/frame allocation.*

- [ ] **4.1 Double Buffering:** Replace `previous: Option<FrameBuffer>` with `current` and `previous` buffers.
- [ ] **4.2 Swap:** Implement `mem::swap` for buffer flipping.
    -   *Test:* `test_double_buffer_swap` (Verify content persists without cloning).
- [ ] **4.3 Unified Core:** Extract `RendererCore` (cursor tracking) to be shared by Diff/Inline/Append renderers.

### Phase 5: Layout Engine (`layout/layout_tree_aos.rs`)
*Goal: Enable incremental layout (performance).*

- [ ] **5.1 Dirty Checking:** Update `compute_layout_aos` to check `DIRTY_LAYOUT` flags.
- [ ] **5.2 Incremental Cache:** **Stop clearing the entire cache.** Only clear cache for dirty nodes and their ancestors.
    -   *Test:* `test_incremental_layout` (Modify one node, ensure others aren't recomputed).
- [ ] **5.3 Fix Casts:** Replace `as usize` with `.ceil() as usize` for dimensions to avoid rounding errors.

### Phase 6: Input System (`input/*.rs`)
*Goal: Remove shadow state and unsafe threads.*

- [ ] **6.1 Safe Stdin:** Replace blocking `read()` thread with non-blocking polling or `mio` integration (or safe timeout logic).
- [ ] **6.2 Stateless Managers:** Refactor `FocusManager` and `ScrollManager` to read/write *directly* to `AoSBuffer`. Remove internal `Vec` state where possible.
- [ ] **6.3 Efficient Text:** Optimize `TextEditor` to avoid `String` allocation loop.

### Phase 7: Pipeline Integration (`pipeline/setup.rs`)
*Goal: Clean orchestration.*

- [ ] **7.1 Clean Setup:** Break `run_engine` into `setup_terminal`, `create_graph`, `event_loop`.
- [ ] **7.2 Blink Logic:** Integrate `BlinkManager` properly or remove it if TS handles it completely.
- [ ] **7.3 Shutdown:** Implement proper `AtomicBool` signal handling for clean exit.

### Phase 8: Verification
*Goal: Proof of life.*

- [ ] **8.1 Full Test Suite:** Run all unit tests.
- [ ] **8.2 Benchmarks:** Re-run `bench-e2e.ts`. Expect same/better FPS but with safer memory profile.
- [ ] **8.3 Visual Check:** Verify the "Counter" example (scrolling fix).

---

## 4. Immediate Next Step

**Execute Phase 1: The Foundation.**
We will rewrite `rust/src/types.rs` immediately.
