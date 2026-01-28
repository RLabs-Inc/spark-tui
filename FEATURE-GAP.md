# SparkTUI Feature Gap Analysis

**Generated:** Session 5 — Full reconnaissance of original TS implementation vs current Rust engine.

## Legend

- **HAVE** = Exists and works in Rust today
- **PARTIAL** = Code exists but incomplete or not wired up
- **MISSING** = Needs to be built or ported
- **REF** = Exists in `rs/` reference code (can be ported)

---

## 1. SHARED BUFFER CONTRACT

| Feature | Status | Notes |
|---------|--------|-------|
| Header (64B, 16 u32) | **HAVE** | v2, text pool fields included |
| Metadata (32B/node) | **HAVE** | Layout + visual + text + interaction + dirty |
| Floats (24 f32/node) | **HAVE** | Dimensions, flex, padding, margin, gap, insets |
| Colors (10 u32/node) | **HAVE** | ARGB packed, fg/bg/border/cursor |
| Interaction (12 i32/node) | **HAVE** | Scroll, cursor, selection, hover/pressed |
| Hierarchy (1 i32/node) | **HAVE** | Parent indices |
| Output (7 f32/node) | **HAVE** | x/y/w/h + scroll info |
| Text Index (2 u32/node) | **HAVE** | Offset + length into pool |
| Text Pool (1MB UTF-8) | **HAVE** | Bump-allocated, compact support |

**Status: COMPLETE** — Done this session.

---

## 2. LAYOUT ENGINE

| Feature | Status | Notes |
|---------|--------|-------|
| Taffy flexbox integration | **HAVE** | Low-level trait API — LayoutTree implements 6 Taffy traits directly on SharedBuffer |
| NodeId = component index | **HAVE** | Zero HashMap, zero SlotMap, zero translation |
| Flex direction/wrap | **HAVE** | All 4 directions, 3 wrap modes |
| Justify content | **HAVE** | All 6 modes |
| Align items/self | **HAVE** | All modes |
| Align content | **HAVE** | All modes |
| Flex grow/shrink/basis | **HAVE** | |
| Padding/margin | **HAVE** | Per-side |
| Border as layout spacing | **HAVE** | 0/1 per side |
| Gap (row/column) | **HAVE** | Separate row_gap/column_gap |
| Min/max dimensions | **HAVE** | |
| Percentage dimensions | **HAVE** | 0-1 = percent |
| Absolute positioning | **HAVE** | Insets wired (FLOAT_TOP/RIGHT/BOTTOM/LEFT) |
| Overflow (visible/hidden/scroll) | **HAVE** | Scroll bounds via content_size |
| Text measurement | **HAVE** | Unicode-aware: string_width, grapheme_width, CJK, emoji, ANSI stripping |
| Text wrapping | **HAVE** | Word-boundary-aware, measure_text_height |
| Text truncation | **HAVE** | Configurable suffix (ellipsis) |
| Content-based auto-sizing | **HAVE** | Taffy measure function reads text pool, measures correctly |
| Layout caching | **HAVE** | Per-node Taffy Cache (9 slots) |
| Pixel-snapping (rounding) | **HAVE** | round_layout via RoundTree trait |

**Status: COMPLETE** — 13 layout_tree tests, 113 total tests passing.

---

## 3. FRAMEBUFFER BUILDING

| Feature | Status | Notes |
|---------|--------|-------|
| 2D Cell grid (FrameBuffer) | **REF** | Full impl in `rust/src/renderer/buffer.rs` (809 lines) |
| Background fill | **REF** | `fill_rect()` with alpha blending |
| Border drawing (10 styles) | **REF** | `draw_border()`, `draw_border_sides()` |
| Per-side border styles + colors | **REF** | Independent style/color per side |
| Text rendering | **REF** | `draw_text()`, `draw_text_centered()`, `draw_text_right()` |
| Text wrapping | **REF** | `wrap_text()` in `layout/text_measure.rs` |
| Text truncation with ellipsis | **REF** | `truncate_text()` with configurable suffix |
| Text alignment (L/C/R) | **REF** | All 3 modes |
| Wide character support (CJK/emoji) | **REF** | `char_width()` with lookup tables |
| Clipping (ClipRect) | **REF** | Intersection, bounds checking |
| Scrollbar drawing | **REF** | `draw_scrollbar_v()`, `draw_scrollbar_h()` |
| Progress bar drawing | **REF** | `draw_progress()` |
| Alpha blending / opacity | **REF** | Porter-Duff in `types.rs` |
| Z-index sorting | **REF** | In `frame_buffer_derived.rs` |
| Color inheritance (parent chain) | **MISSING** | TS walks parent chain for fg/bg/border — need to port |
| Opacity cascade (multiply ancestors) | **MISSING** | TS multiplies opacity down tree |
| Recursive tree rendering | **REF** | `frame_buffer_derived.rs` (969 lines) |
| Scroll offset accumulation | **REF** | Nested scrollable containers |
| HitRegion collection | **REF** | Mouse hit area data collection |
| Input cursor rendering | **REF** | Block (inverse), bar, underline, blink |
| Scroll-to-cursor in input | **MISSING** | TS auto-scrolls input to keep cursor visible |

**Key gap:** The REF code exists in `rust/src/renderer/` and `rust/src/pipeline/` but reads from `TrackedSlotArray`, NOT from SharedBuffer. Needs to be rewired to read from SharedBuffer sections.

---

## 4. RENDERING (TERMINAL OUTPUT)

| Feature | Status | Notes |
|---------|--------|-------|
| ANSI escape codes (all) | **REF** | `renderer/ansi.rs` (563 lines) — cursor, colors, attrs, screen |
| TrueColor (38;2;r;g;b) | **REF** | |
| ANSI 256 palette | **REF** | |
| ANSI 16 basic | **REF** | |
| Terminal default color | **REF** | Reset codes |
| Text attributes (8 types) | **REF** | Bold, dim, italic, underline, blink, inverse, hidden, strikethrough |
| Diff renderer | **REF** | `renderer/diff.rs` (235 lines) — only changed cells |
| Inline renderer | **REF** | `renderer/inline.rs` (136 lines) |
| Append renderer | **REF** | `renderer/append.rs` (189 lines) |
| StatefulCellRenderer | **REF** | `renderer/output.rs` — tracks state, skips redundant codes |
| OutputBuffer (batched writes) | **REF** | |
| Synchronized output (?2026) | **REF** | Flicker-free |
| Alternative screen buffer | **REF** | |
| Cursor control (shapes, visibility) | **REF** | DECSCUSR sequences |
| Hyperlinks (OSC 8) | **REF** | |
| Mouse tracking (SGR) | **REF** | |
| Kitty keyboard protocol | **REF** | |
| Bracketed paste | **REF** | |
| Focus reporting | **REF** | |

**Key gap:** All this code is PRODUCTION-GRADE but not connected to the SharedBuffer reactive pipeline. It reads from the old TrackedSlotArray system.

---

## 5. STATE MANAGEMENT

| Feature | Status | Notes |
|---------|--------|-------|
| Focus management | **REF** | `arrays/interaction.rs` has focusable, tabIndex fields |
| Tab navigation (next/prev) | **MISSING** | TS sorts focusable indices by tabIndex, cycles through |
| Focus trapping (modals) | **MISSING** | TS supports push/pop focus traps |
| Focus history (save/restore) | **MISSING** | TS stores max 10 history entries |
| Keyboard event dispatch | **MISSING** | TS dispatches to focused component first, then user handlers |
| Key-specific handlers | **MISSING** | TS `onKey('Enter', handler)` pattern |
| Event consumption (return true) | **MISSING** | Handler can consume event to prevent propagation |
| Mouse HitGrid (O(1) lookup) | **MISSING** | TS uses Int16Array mapping every pixel to component index |
| Hover tracking | **MISSING** | TS auto-updates hovered[] on mouse move via HitGrid |
| Click detection | **MISSING** | TS tracks press+release on same component |
| Scroll management | **MISSING** | TS has keyboard scroll, wheel scroll, scroll chaining |
| Scroll-into-view | **MISSING** | TS auto-scrolls on focus change |
| Scroll chaining to parent | **MISSING** | TS bubbles scroll to parent when at boundary |
| Drawn cursor with blink | **MISSING** | TS has shared blink clocks per FPS |
| Theme system (12 presets) | **MISSING** | TS has dracula, nord, catppuccin, etc. |
| Theme variants (16 semantic) | **MISSING** | primary, secondary, success, error, ghost, outline... |
| Context API (provide/use) | **MISSING** | TS React-like context with ReactiveMap |
| Global keys orchestration | **MISSING** | TS wires Ctrl+C, Tab, arrows, page, home/end in one hub |
| stdin parsing (keyboard+mouse) | **MISSING** | TS parses escape sequences, Kitty protocol, SGR mouse |

---

## 6. REACTIVE PIPELINE

| Feature | Status | Notes |
|---------|--------|-------|
| spark-signals crate | **HAVE** | v0.2.0, signals + deriveds + effects |
| Layout derived | **PARTIAL** | `pipeline/layout_derived.rs` exists but uses explicit deriveds |
| Framebuffer derived | **REF** | `pipeline/frame_buffer_derived.rs` (969 lines) |
| Single render effect | **MISSING** | TS has ONE effect in mount.ts that fires on framebuffer change |
| Dirty flag optimization | **MISSING** | Flags exist in shared buffer but Rust ignores them |
| Smart skip (visual-only = skip layout) | **MISSING** | Architecture supports it, not implemented |
| Terminal mount/unmount | **REF** | `pipeline/mount.rs` (564 lines) |
| Event thread (stdin) | **REF** | `pipeline/events.rs` (372 lines) |
| Terminal size tracking | **REF** | `pipeline/terminal.rs` (112 lines) |

---

## 7. PRIMITIVES (TS-SIDE)

| Feature | Status | Notes |
|---------|--------|-------|
| Box primitive | **EXISTS** | In `ts/primitives/box.ts` — has pre-existing type errors from missing state modules |
| Text primitive | **EXISTS** | In `ts/primitives/text.ts` — same |
| Input primitive | **EXISTS** | In `ts/primitives/input.ts` — same |
| `each()` reactive list | **MISSING** | Keyed list with fine-grained signal per item |
| `show()` conditional | **MISSING** | Boolean toggle with cleanup |
| `when()` async | **MISSING** | Promise state (pending/then/catch) |
| `useAnimation()` | **MISSING** | Shared clocks per FPS, frame cycling |
| Scope management | **MISSING** | Auto-cleanup collection for components |

---

## RECOMMENDED PHASE ORDER

### ~~Phase 1: Fix Text Measurement~~ DONE (Session 130)
~~Wire Taffy measure function to read text content from SharedBuffer text pool.~~
- `text_measure::string_width()` + `measure_text_height()` wired into LayoutTree
- Unicode-aware: CJK, emoji, combining marks, ANSI escape stripping
- Content-based auto-sizing works for Text (wrapping) and Input (single-line)

### Phase 2: Wire Framebuffer to SharedBuffer
Port `frame_buffer_derived.rs` to read from SharedBuffer instead of TrackedSlotArray.
- Color reads: `buf.fg_color()`, `buf.bg_color()`, `buf.border_color()`
- Text reads: `buf.text_content()`, `buf.text_attrs()`, `buf.text_align()`
- Interaction reads: `buf.scroll_offset_y()`, `buf.cursor_position()`
- Add color inheritance (walk parent chain via `buf.parent_index()`)
- Add opacity cascade

### Phase 3: Connect Render Pipeline
Wire framebuffer → diff renderer → terminal output.
- The renderers (diff, inline, append) already work
- Need: FFI export that triggers full pipeline (layout → framebuffer → render)
- Or: reactive pipeline using spark-signals (layout derived → framebuffer derived → render effect)

### Phase 4: Input System
- stdin parsing (keyboard + mouse escape sequences)
- HitGrid for O(1) mouse-to-component lookup
- Focus management (tab navigation)
- Keyboard dispatch to focused component
- Mouse hover/press state tracking

### Phase 5: State Management
- Scroll management (keyboard + wheel + chaining)
- Drawn cursor with blink animation
- Theme system

### Phase 6: Control Flow Primitives (TS-side)
- Port missing state modules to TS
- `each()`, `show()`, `when()`, `useAnimation()`
- Scope management with auto-cleanup

---

## ARCHITECTURE NOTE

The Rust engine's active code lives in `rust/src/`:
- `layout/layout_tree.rs` — Taffy low-level trait API on SharedBuffer (complete, tested)
- `layout/text_measure/` — Unicode text measurement (complete, tested)
- `shared_buffer.rs` — Complete v2 memory contract
- `types.rs` — Full color system with OKLCH, all enums
- `lib.rs` — Clean FFI exports (82 lines)

Reference code in `rs/` has ~15,000 lines of production-grade implementations:
- `renderer/` — ANSI, diff, inline, append, buffer, output
- `pipeline/` — layout derived, framebuffer derived, mount, events
- `layout/` — Old Taffy bridge (superseded by layout_tree.rs)

The primary work for framebuffer and renderer is **rewiring reference code** to read from SharedBuffer instead of TrackedSlotArray. The rendering and ANSI code is all there — it just needs to be connected to the shared memory bridge.
