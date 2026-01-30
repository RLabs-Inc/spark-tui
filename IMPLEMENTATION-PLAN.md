# SparkTUI AoS Pipeline Implementation Plan

**Created:** Session 136 (January 28, 2026)
**Status:** Phase 1 agents launched, waiting for completion

## Executive Summary

The AoS layout engine is **proven fast** (3-5x faster than pure Rust). The OLD SoA path is complete. We need to replicate that completeness for AoS.

**Benchmark results:**
```
AoS Hybrid:          Pure Rust:          RESULT:
111 nodes:  0.54μs   vs 3.98μs   →  7.4x FASTER
551 nodes:  4.95μs   vs 15.41μs  →  3.1x FASTER
1101 nodes: 10.01μs  vs 32.15μs  →  3.2x FASTER
5501 nodes: 47.92μs  vs 196.78μs →  4.1x FASTER
20K nodes: 0.30ms = 3,337 FPS
```

---

## Phase 1: AoSBuffer Method Completion ⏳ IN PROGRESS

**Goal:** Add all missing accessors so framebuffer and input systems can use AoSBuffer.

**File:** `rust/src/shared_buffer_aos.rs`

### 1.1 Output Section READS (CRITICAL)
```rust
output_x(node) -> f32
output_y(node) -> f32
output_width(node) -> f32
output_height(node) -> f32
output_scrollable(node) -> bool
output_max_scroll_x(node) -> f32
output_max_scroll_y(node) -> f32
set_output_scroll(node, scrollable, max_x, max_y)
```

### 1.2 Color Accessors
```rust
// Packed u32
fg_color(node), bg_color(node), border_color(node)
border_color_top/right/bottom/left(node)
cursor_fg_color(node), cursor_bg_color(node)

// Rgba decoders
fg_rgba(node), bg_rgba(node), border_rgba(node)
border_color_top/right/bottom/left_rgba(node)
cursor_fg_rgba(node), cursor_bg_rgba(node)
unpack_to_rgba(packed) -> Rgba
```

### 1.3 Visual/Metadata
```rust
// Border styles
border_style(node), border_style_top/right/bottom/left(node)
border_top/right/bottom/left_width(node)

// Visual
opacity(node), opacity_f32(node), z_index(node)

// Flags
focusable(node), show_focus_ring(node), mouse_enabled(node)

// Text meta
text_align(node), text_wrap(node), text_attrs(node)
```

### 1.4 Interaction State
```rust
// Scroll
scroll_offset_x(node), scroll_offset_y(node)
set_scroll_offset(node, x, y)

// Tab
tab_index(node)

// Cursor
cursor_position(node), set_cursor_position(node, pos)
selection_start(node), selection_end(node), set_selection(node, start, end)
cursor_visible(node), set_cursor_visible(node, bool)
cursor_char(node), cursor_alt_char(node)

// Hover/pressed (use header globals)
hovered(node), pressed(node)
set_hovered(node), set_pressed(node)
```

### 1.5 Text Content
```rust
text_content(node) -> &str
write_text(node, &str) -> bool
text_offset(node), text_length(node)
text_pool_write_ptr(), text_pool_capacity(), text_pool_remaining()
```

### 1.6 Utility Methods
```rust
consume_wake() -> bool
set_wake_flag()
clear_dirty(node), clear_all_dirty(node)
increment_generation()
increment_render_count()
```

---

## Phase 2: TuiBuffer Trait

**Goal:** Create a trait that both SharedBuffer and AoSBuffer implement so framebuffer can work with either.

**Files:**
- Create: `rust/src/buffer_trait.rs`
- Update: `rust/src/shared_buffer.rs` (impl TuiBuffer)
- Update: `rust/src/shared_buffer_aos.rs` (impl TuiBuffer)
- Update: `rust/src/framebuffer/render_tree.rs` (use trait)

### Trait Definition
```rust
pub trait TuiBuffer {
    // Header
    fn node_count(&self) -> usize;
    fn terminal_width(&self) -> u16;
    fn terminal_height(&self) -> u16;

    // Layout output
    fn output_x(&self, i: usize) -> f32;
    fn output_y(&self, i: usize) -> f32;
    fn output_width(&self, i: usize) -> f32;
    fn output_height(&self, i: usize) -> f32;
    fn output_scrollable(&self, i: usize) -> bool;
    fn output_max_scroll_x(&self, i: usize) -> f32;
    fn output_max_scroll_y(&self, i: usize) -> f32;

    // Visual
    fn component_type(&self, i: usize) -> u8;
    fn visible(&self, i: usize) -> bool;
    fn parent_index(&self, i: usize) -> Option<usize>;
    fn z_index(&self, i: usize) -> u8;
    fn opacity(&self, i: usize) -> u8;
    fn fg_rgba(&self, i: usize) -> Rgba;
    fn bg_rgba(&self, i: usize) -> Rgba;

    // Borders
    fn border_style(&self, i: usize) -> u8;
    fn border_style_top(&self, i: usize) -> u8;
    fn border_style_right(&self, i: usize) -> u8;
    fn border_style_bottom(&self, i: usize) -> u8;
    fn border_style_left(&self, i: usize) -> u8;
    fn border_top_width(&self, i: usize) -> u8;
    fn border_right_width(&self, i: usize) -> u8;
    fn border_bottom_width(&self, i: usize) -> u8;
    fn border_left_width(&self, i: usize) -> u8;
    fn border_color_top_rgba(&self, i: usize) -> Rgba;
    fn border_color_right_rgba(&self, i: usize) -> Rgba;
    fn border_color_bottom_rgba(&self, i: usize) -> Rgba;
    fn border_color_left_rgba(&self, i: usize) -> Rgba;

    // Padding (for content area calc)
    fn padding_top(&self, i: usize) -> f32;
    fn padding_right(&self, i: usize) -> f32;
    fn padding_bottom(&self, i: usize) -> f32;
    fn padding_left(&self, i: usize) -> f32;

    // Text
    fn text_content(&self, i: usize) -> &str;
    fn text_attrs(&self, i: usize) -> u8;
    fn text_align(&self, i: usize) -> u8;
    fn text_wrap(&self, i: usize) -> u8;

    // Interaction
    fn scroll_offset_x(&self, i: usize) -> i32;
    fn scroll_offset_y(&self, i: usize) -> i32;
    fn focusable(&self, i: usize) -> bool;
    fn show_focus_ring(&self, i: usize) -> bool;
    fn cursor_position(&self, i: usize) -> i32;
    fn cursor_visible(&self, i: usize) -> bool;
    fn cursor_char(&self, i: usize) -> u32;
    fn cursor_alt_char(&self, i: usize) -> u32;
    fn cursor_fg_rgba(&self, i: usize) -> Rgba;
    fn cursor_bg_rgba(&self, i: usize) -> Rgba;
    fn selection_start(&self, i: usize) -> i32;
    fn selection_end(&self, i: usize) -> i32;
}

pub trait TuiBufferMut: TuiBuffer {
    fn set_scroll_offset(&self, i: usize, x: i32, y: i32);
    fn set_cursor_position(&self, i: usize, pos: i32);
    fn set_cursor_visible(&self, i: usize, visible: bool);
    fn set_selection(&self, i: usize, start: i32, end: i32);
    fn set_hovered_index(&self, idx: i32);
    fn set_pressed_index(&self, idx: i32);
    fn write_text(&self, i: usize, text: &str) -> bool;
}
```

### Update render_tree.rs
```rust
pub fn compute_framebuffer<B: TuiBuffer>(
    buf: &B,
    width: u16,
    height: u16,
) -> (FrameBuffer, Vec<HitRegion>)
```

---

## Phase 3: Input System for AoS

**Goal:** Update all input modules to work with TuiBuffer trait.

**Files:**
- `rust/src/input/focus.rs`
- `rust/src/input/keyboard.rs`
- `rust/src/input/mouse.rs`
- `rust/src/input/text_edit.rs`
- `rust/src/input/scroll.rs`
- `rust/src/input/cursor.rs`

### Pattern
Change all functions from:
```rust
pub fn dispatch_key(buf: &SharedBuffer, ...)
```
To:
```rust
pub fn dispatch_key<B: TuiBuffer + TuiBufferMut>(buf: &B, ...)
```

---

## Phase 4: Full AoS Engine Pipeline

**Goal:** `spark_init_aos()` starts complete reactive pipeline.

**Files:**
- `rust/src/pipeline/setup.rs` (add `start_aos()`)
- `rust/src/pipeline/mod.rs` (export)
- `rust/src/lib.rs` (update `spark_init_aos()`)

### Key Components

1. **Terminal setup** - raw mode, alternate screen (fullscreen) or normal (inline/append)

2. **Renderer selection** based on `config_flags`:
   - Mode 0: DiffRenderer (fullscreen, terminal size)
   - Mode 1: InlineRenderer (content height, erase+rewrite)
   - Mode 2: AppendRenderer (history + active regions)

3. **Reactive signals:**
   ```rust
   let generation: Signal<u64> = signal(0);
   ```

4. **Layout derived:**
   ```rust
   let layout_derived = derived(move || {
       let _gen = generation.get();
       // Check dirty flags, smart skip
       if needs_layout(buf) {
           layout::compute_layout_aos(buf);
       }
       _gen
   });
   ```

5. **Framebuffer derived:**
   ```rust
   let fb_derived = derived(move || {
       let _layout = layout_derived.get();
       compute_framebuffer(buf, tw, th)  // Works with AoS via trait
   });
   ```

6. **Render effect (ONE effect):**
   ```rust
   effect(move || {
       let result = fb_derived.get();
       // Update hit grid
       // Render to terminal
       renderer.render(&result.buffer);
   });
   ```

7. **Content height for inline/append:**
   ```rust
   fn compute_content_height<B: TuiBuffer>(buf: &B) -> u16 {
       let mut max_y = 0u16;
       for i in 0..buf.node_count() {
           if buf.parent_index(i).is_none() && buf.visible(i) {
               max_y = max_y.max(buf.output_y(i) as u16 + buf.output_height(i) as u16);
           }
       }
       max_y
   }
   ```

---

## Phase 5: Event Ring Buffer in Shared Memory

**Goal:** Move events from in-memory Vec to SharedBuffer section so TS can read them.

**Layout (already defined in AoS):**
```
EVENT_RING_OFFSET = HEADER_SIZE + MAX_NODES * STRIDE + TEXT_POOL_SIZE
EVENT_RING_HEADER_SIZE = 12  (write_idx, read_idx, reserved)
EVENT_SLOT_SIZE = 20
MAX_EVENTS = 256
EVENT_RING_SIZE = 5132 bytes
```

**Files:**
- `rust/src/shared_buffer_aos.rs` (already has `push_event`)
- `rust/src/input/keyboard.rs` (use `buf.push_event()`)
- `rust/src/input/mouse.rs`
- `rust/src/input/text_edit.rs`
- `rust/src/input/focus.rs`

**TS side:** Already implemented in `ts/engine/events.ts` using `Atomics.waitAsync` (NOT polling).

---

## Phase 6: Wire Render Modes

**Goal:** Proper fullscreen/inline/append mode selection.

**Config reading:**
```rust
let config = buf.config_flags();
let render_mode = buf.render_mode();  // 0=fullscreen, 1=inline, 2=append
```

**Mode behavior:**
- Fullscreen: alternate screen, terminal size, DiffRenderer
- Inline: normal buffer, content height, InlineRenderer (erase+rewrite)
- Append: normal buffer, history + active, AppendRenderer

---

## Implementation Order

```
Phase 1 ─────→ Phase 2 ─────→ Phase 3 ─────→ Phase 4 ─────→ Phase 5 ─────→ Phase 6
(methods)     (trait)        (input)        (pipeline)     (events)       (modes)
```

Each phase depends on the previous.

---

## CRITICAL ARCHITECTURE RULES

1. **NO LOOPS** - Channel-driven, not loop-driven
2. **NO POLLING** - Adaptive spin-wait detects changes, doesn't poll
3. **NO FIXED FPS** - Renders exactly when data changes
4. **ONE EFFECT** - Single render effect at the end
5. **SMART SKIP** - Layout skipped if only visual props changed
6. **ONLY TIMING: BLINK** - Cursor blink is the ONLY timer, acts as signal source

---

## Files Reference

### Rust Files to Modify
- `rust/src/shared_buffer_aos.rs` - Phase 1
- `rust/src/buffer_trait.rs` (new) - Phase 2
- `rust/src/shared_buffer.rs` - Phase 2
- `rust/src/framebuffer/render_tree.rs` - Phase 2
- `rust/src/input/*.rs` - Phase 3
- `rust/src/pipeline/setup.rs` - Phase 4
- `rust/src/lib.rs` - Phase 4

### TS Files (already done)
- `ts/bridge/shared-buffer-aos.ts` - AoS layout ✅
- `ts/engine/events.ts` - Event reader ✅
- `ts/state/keyboard.ts`, `mouse.ts`, `focus.ts` - State modules ✅
- `ts/primitives/animation.ts` - cycle(), pulse() ✅

---

## Current Session Status

**Phase 1 agents launched (5 parallel):**
1. Output section reads - running
2. Color accessors - running
3. Visual/metadata methods - running
4. Interaction state methods - running
5. Text content accessor - running

**Next steps after Phase 1 completes:**
1. Run `cargo test` to verify all methods work
2. Launch Phase 2 agent for TuiBuffer trait
3. Continue through phases sequentially

---

## TS Side Already Complete

The TS side is feature-complete:
- box.ts, text.ts migrated to AoS ✅
- input.ts migrated to AoS ✅
- Animation primitives (cycle, pulse) ✅
- Event system with Atomics.waitAsync ✅
- State modules (keyboard, mouse, focus) ✅
- drawnCursor.ts ✅

What's missing is the **Rust rendering pipeline** to actually show pixels on screen.
