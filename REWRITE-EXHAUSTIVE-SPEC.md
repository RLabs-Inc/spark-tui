# SparkTUI Rust Engine: Exhaustive Rewrite Specification

**Goal:** Transform the "organically grown" prototype into a production-grade, memory-safe, zero-allocation TUI engine.
**Constraint:** Absolute adherence to the Reactive AoS architecture. Zero shadow state. Test-Driven everything.

---

## 1. Foundation: Types & Geometry (`types.rs`, `utils/mod.rs`)

### Current State & Issues
- **Type Scattering:** Logic is split between `types.rs`, `utils/mod.rs`, and `layout/types.rs`.
- **The Scroll Bug:** `ClipRect` uses `u16`. In `render_tree.rs`, `abs_x` is calculated as `parent_abs + rel - scroll`. If `scroll > parent_abs + rel`, the result is negative. `u16` cannot represent this, forcing a clamp to 0, which destroys nested scrolling.
- **Color Collision:** `Rgba` white (`0xFFFFFFFF`) is bit-identical to `TERMINAL_DEFAULT` (`-1` packed).
- **Enum Fragility:** Many enums lack `From<u8>`, leading to unsafe `match` statements or `unwrap()` calls in the hot path.
- **Border Inelegance:** `BorderStyle::chars()` returns a 6-tuple, forcing callers to remember the order (TL, TR, BR, BL, H, V).

### Actionable Atomic Tasks
- [ ] **Task 1.1: Unified Primitives.** Move `Rgba`, `Cell`, `Attr`, `ClipRect` to `types.rs`. Delete `utils/mod.rs`.
- [ ] **Task 1.2: i32 Geometry.** Change `ClipRect` coordinates (`x`, `y`, `width`, `height`) to `i32`.
    - *Test:* `test_clip_rect_intersection_negative` (Verify `(-5, -5, 10, 10)` intersected with `(0, 0, 10, 10)` yields `(0, 0, 5, 5)`).
- [ ] **Task 1.3: BorderChars Struct.** Replace 6-tuple with `struct BorderChars { horizontal, vertical, top_left, ... }`.
- [ ] **Task 1.4: Safe Enum Registry.** Implement `From<u8>` for `ComponentType`, `BorderStyle`, `FlexDirection`, `FlexWrap`, `JustifyContent`, `AlignItems`, `AlignSelf`, `AlignContent`, `Overflow`, `Position`, `TextAlign`, `TextWrap`, `CursorStyle`, `RenderMode`.
    - *Test:* `test_enum_fallbacks` (Ensure `255` byte returns `Default` for all enums).
- [ ] **Task 1.5: Color Logic.** Update `Rgba::is_terminal_default` to check for both `-1` marker and `0xFFFFFFFF` specifically if it originated from the "default" slot.

---

## 2. The Data Layer: Safe AoS (`shared_buffer_aos.rs`)

### Current State & Issues
- **Memory Safety:** Every accessor (`width()`, `height()`, etc.) uses `ptr.add()` with zero bounds checking.
- **Magic Numbers:** Field offsets are hardcoded in tests and some logic (`HEADER_SIZE + index * STRIDE`).
- **Overlap:** `U_CURSOR_ALT_CHAR` (233-236) overlaps with `C_BORDER_TOP_COLOR` (236-239) by one byte.
- **Event Ring:** `EVENT_RING_TOTAL` calculation is inconsistent with `input/events.rs`.

### Actionable Atomic Tasks
- [ ] **Task 2.1: Bounds Guard.** Implement `fn node_ptr(&self, index: usize) -> *const u8` with a `debug_assert!(index < MAX_NODES)`.
- [ ] **Task 2.2: Offset Constants.** Ensure every byte from 0 to 255 in the stride has a named constant.
- [ ] **Task 2.3: Fix Overlap.** Move `U_CURSOR_ALT_CHAR` or `C_BORDER_TOP_COLOR` to ensure zero overlap.
- [ ] **Task 2.4: Type-Safe Accessors.** Update `scroll_x/y` and `computed_x/y` to return `i32` (matching Phase 1).
- [ ] **Task 2.5: Header Accessors.** Add `mouse_x()`, `mouse_y()`, `exit_requested()` safely.

---

## 3. The Logic Layer: Render Tree (`framebuffer/render_tree.rs`, `inheritance.rs`)

### Current State & Issues
- **Parameter Explosion:** `render_component` takes 10 arguments. It's unreadable.
- **The Propagation Bug:** `render_children` passes `0, 0` for scroll if the node isn't "scrollable", but it should pass the accumulated `parent_scroll`.
- **Inheritance Trap:** `get_inherited_fg/bg` walks up the tree for every single node (O(N²)).
- **Duplication:** `render_borders` logic is a copy-paste of `FrameBuffer` logic.

### Actionable Atomic Tasks
- [ ] **Task 3.1: RenderContext Struct.** Create `struct RenderContext<'a>` containing `buffer`, `buf` (AoS), `child_map`, `hit_regions`.
- [ ] **Task 3.2: Fix Scroll Propagation.** Rewrite `render_component` to correctly pass `abs_x/y` and `clip` through the recursion.
    - *Test:* `test_nested_scroll_clipping` (Verify child at `(10, 10)` in parent scrolled by `5, 5` renders at `(5, 5)`).
- [ ] **Task 3.3: Top-Down Inheritance.** Pass `inherited_fg`, `inherited_bg`, and `inherited_opacity` as arguments in the DFS pass instead of walking up.
- [ ] **Task 3.4: HitRegion Precision.** `HitRegion` uses `u16`. Change to `i32` during calculation, then clamp to `u16` only for the `HitGrid`.

---

## 4. The Geometry Layer: FrameBuffer (`renderer/buffer.rs`)

### Current State & Issues
- **u16 Coordinates:** All drawing methods take `u16`, forcing premature clamping.
- **ClipRect Overflow:** `ClipRect::contains` does `px >= self.x && px < self.x + self.width`. If `x + width > 65535`, it wraps and breaks.
- **Wide Char markers:** Continuation markers (`char = 0`) are hardcoded.

### Actionable Atomic Tasks
- [ ] **Task 4.1: i32 Drawing.** Update `set_cell`, `fill_rect`, `draw_text`, `draw_border` to take `i32` coordinates.
- [ ] **Task 4.2: Safe Bounds.** Implement `to_index(x: i32, y: i32) -> Option<usize>` which returns `None` for any coordinate outside `(0..width, 0..height)`.
- [ ] **Task 4.3: Fast-Path Fill.** Implement `fill_bg` which only touches the `bg` field of cells, skipping `char/fg/attr` logic for background containers.

---

## 5. Performance Layer: Zero-Allocation Renderer (`renderer/diff.rs`, `inline.rs`, `append.rs`)

### Current State & Issues
- **The 100KB Clone:** `self.previous = Some(buffer.clone())` at the end of every frame.
- **Logic Duplication:** `Diff` and `Inline` renderers both implement stateful cursor tracking independently.
- **ANSI Inconsistency:** `inline.rs` uses raw strings (`\x1b[2J`) while `diff.rs` uses the `ansi` module.

### Actionable Atomic Tasks
- [ ] **Task 5.1: Double Buffer Swap.** Update `DiffRenderer` to hold `prev` and `curr` `FrameBuffer`s. Use `std::mem::swap`.
- [ ] **Task 5.2: RendererCore.** Extract `OutputBuffer` and `StatefulCellRenderer` into a shared `RendererCore` struct.
- [ ] **Task 5.3: Sync Output.** Ensure all renderers use `ansi::begin_sync` and `ansi::end_sync` consistently.

---

## 6. Optimization Layer: Layout Engine (`layout/layout_tree_aos.rs`)

### Current State & Issues
- **Cache Invalidation:** `ctx.cache[i].clear()` runs every frame. Taffy is running from scratch every time.
- **Rounding Gaps:** `as usize` casts cause 1px gaps when Taffy returns fractional coordinates (e.g., `49.99 -> 49`).

### Actionable Atomic Tasks
- [ ] **Task 6.1: Incremental Layout.** Read `DIRTY_LAYOUT` flag. Only clear Taffy cache if dirty.
- [ ] **Task 6.2: Bubble-Up Dirty.** When a node is dirty, mark all ancestors dirty in the Taffy tree to ensure the layout path is recomputed.
- [ ] **Task 6.3: Safe Rounding.** Use `.ceil() as i32` for all Taffy output values.

---

## 7. Input & Pipeline (`pipeline/setup.rs`, `input/*`)

### Current State & Issues
- **Monolith:** `run_engine` is 223 lines of mixed concerns.
- **Shadow State:** `FocusManager` keeps a `focused_index` field. If TS writes to `H_FOCUSED_INDEX`, Rust won't know.
- **Blocking IO:** Stdin thread can hang on shutdown.

### Actionable Atomic Tasks
- [ ] **Task 7.1: Stateless Input.** Remove `focused_index`, `hovered_index` from Rust structs. Read them from `AoSBuffer` every time.
- [ ] **Task 7.2: Unified Setup.** Split `run_engine` into `setup_io`, `init_reactive_graph`, and `main_loop`.
- [ ] **Task 7.3: Dirty Flag Lifecycle.** Ensure `clear_all_dirty` happens *after* layout and framebuffer computation, not before.

---

## 8. FFI & Testing (`lib.rs`)

### Current State & Issues
- **Test Code Leak:** `spark_test_atomic_wait` is in the production dylib.
- **Global Lock:** `static BUFFER` prevents parallel unit testing.

### Actionable Atomic Tasks
- [ ] **Task 8.1: Feature Gating.** Move test FFI to `#[cfg(feature = "test-utils")]`.
- [ ] **Task 8.2: Integration Tests.** Create `tests/e2e_layout.rs` that allocates a real `Vec<u8>`, wraps it in `AoSBuffer`, and runs a full layout/render pass in pure Rust.
