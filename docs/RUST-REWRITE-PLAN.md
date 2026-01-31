# SparkTUI Rust Engine Rewrite Plan

**Goal:** Replace organically-grown implementation with clean, tested code.
**Approach:** Extract specs → Write tests → Implement cleanly → One file at a time.
**Why:** 112 issues across 8 modules. Patching makes it worse. We have all the knowledge now.

---

## Phase Order (Dependencies)

```
1. types.rs (foundation - all other files depend on this)
   ↓
2. shared_buffer_aos.rs (data layer - rendering depends on this)
   ↓
3. buffer.rs (FrameBuffer - render_tree depends on this)
   ↓
4. render_tree.rs (main rendering - uses all above)
   ↓
5. input/* (focus, mouse, scroll, keyboard - uses SharedBuffer)
   ↓
6. setup.rs (pipeline orchestration - uses all above)
   ↓
7. renderer/* (diff, inline, append - uses FrameBuffer)
   ↓
8. layout/* (already decent, polish last)
```

---

## Phase 1: types.rs

### Current Issues (10 total)
- ClipRect uses u16, can't handle negative positions
- Missing From<u8> for Position, TextAlign, TextWrap, CursorStyle
- ComponentType constants duplicated elsewhere
- White color could be confused with terminal default
- BorderStyle::chars() returns 6-tuple instead of struct

### Features to Preserve
- Rgba with OKLCH support, WCAG contrast, alpha blending
- Cell with char (u32), fg, bg, attrs
- All flex enums with From<u8>
- BorderStyle with all 10 styles
- Attr bitflags

### Key Changes
- [ ] ClipRect: x/y to i32, add to_screen() method for framebuffer output
- [ ] Add From<u8> for ALL enums consistently
- [ ] Add ComponentType::from(u8) with unknown fallback
- [ ] BorderChars struct instead of 6-tuple
- [ ] Document Rgba packing/unpacking contract clearly

### Tests to Write
- [ ] ClipRect::contains() with negative coords
- [ ] ClipRect::intersect() with negative coords
- [ ] ClipRect::to_screen() clamps correctly
- [ ] All enum From<u8> conversions
- [ ] Rgba terminal default detection after unpacking
- [ ] Rgba white vs terminal default distinction

---

## Phase 2: shared_buffer_aos.rs

### Current Issues (16 total)
- Cursor/border byte overlap (233-236)
- Event ring size mismatch (8204 vs 5132)
- Missing bounds checking on node_ptr()
- Missing mouse position getters
- Per-node focused flag never used

### Features to Preserve
- 256-byte stride AoS layout
- All accessor methods (layout, visual, text, interaction)
- Event ring buffer
- Atomic wake mechanism
- Text pool with bump allocation

### Key Changes
- [ ] Fix byte overlap - move cursor_alt_char OR border colors
- [ ] Fix EVENT_RING_TOTAL constant
- [ ] Add debug_assert bounds checking to node_ptr/node_ptr_mut
- [ ] Add read_header_u16, mouse_x(), mouse_y()
- [ ] Remove COMPONENT_* constants, use types::ComponentType

### Tests to Write
- [ ] Memory layout matches TS side exactly
- [ ] All offsets non-overlapping
- [ ] Bounds checking catches invalid indices
- [ ] Event ring wraparound works correctly
- [ ] Text pool overflow handled gracefully

---

## Phase 3: buffer.rs (FrameBuffer)

### Current Issues (15 total)
- Uses u16 coords only, can't handle negatives
- ClipRect::contains() overflow risk
- fill_rect clears char/attrs even when blending
- char_width() incomplete for Unicode
- resize() clears twice

### Features to Preserve
- Flat Vec<Cell> storage
- fill_rect, draw_text, draw_border, draw_progress, draw_scrollbar
- Clipping on all operations
- Alpha blending

### Key Changes
- [ ] Accept i32 coords, clamp to bounds internally
- [ ] Use saturating_add in all bounds checks
- [ ] Separate fill_bg() from fill_rect()
- [ ] Import string_width from layout/text_measure (don't duplicate)
- [ ] Remove redundant clear() from resize()

### Tests to Write
- [ ] Negative coords handled (clipped correctly)
- [ ] Overflow coords handled (don't panic)
- [ ] ClipRect intersection with negative rects
- [ ] Wide character continuation markers
- [ ] Alpha blending preserves text when blending bg

---

## Phase 4: render_tree.rs

### Current Issues (14 total)
- Scroll propagation resets to 0,0 (THE BUG)
- Negative position clamping doesn't adjust width/height
- Constant duplication (COMP_*, ALIGN_*, WRAP_*)
- 10-parameter function signatures
- Hardcoded scrollbar characters

### Features to Preserve
- DFS traversal with z-index sorting
- Color inheritance via inheritance.rs
- Component type dispatch (box, text, input, select, progress)
- Hit region collection
- Border rendering
- Scrollbar rendering

### Key Changes
- [ ] Pass parent_scroll through to children (fix lines 213, 221, 248)
- [ ] Adjust width/height when clamping negative positions
- [ ] Import constants from types.rs
- [ ] Create RenderContext struct to reduce parameters
- [ ] Make scrollbar chars configurable (or move to SharedBuffer)

### Tests to Write
- [ ] Nested scroll containers render correctly
- [ ] Component scrolled to negative coords clips correctly
- [ ] Z-index sorting works
- [ ] Hit regions match visible bounds
- [ ] Focus indicator renders at correct position

---

## Phase 5: input/* (focus.rs, mouse.rs, scroll.rs, keyboard.rs)

### Current Issues (14 total)
- Shadow state (focused_index, hovered, pressed) duplicates SharedBuffer
- Per-node focused flag never set
- Focus traps/history only in Rust (TS can't see)
- Scroll chaining claims success even when nothing moved

### Features to Preserve
- Tab navigation with tab_index
- Focus traps
- Mouse hover/press tracking
- Scroll with chaining
- Keyboard dispatch

### Key Changes
- [ ] Remove FocusManager.focused_index, read from buf.focused_index()
- [ ] Remove MouseManager.hovered/pressed_*, read from SharedBuffer
- [ ] Set per-node FLAG_FOCUSED in focus()
- [ ] Add early return if already focused
- [ ] Return false from scroll when nothing moved

### Tests to Write
- [ ] Focus state lives only in SharedBuffer
- [ ] Tab cycles through focusable components
- [ ] Focus traps contain navigation
- [ ] Scroll chaining stops at boundaries correctly
- [ ] Hover/press state accessible from TS

---

## Phase 6: setup.rs

### Current Issues (16 total)
- Dirty flags cleared before layout uses them
- BlinkManager is dead code
- run_engine is 223 lines (does 7 things)
- Generation counter overflow
- Rc<RefCell<MouseManager>> when not concurrent

### Features to Preserve
- Reactive graph (signal → layout derived → fb derived → render effect)
- Unified channel (stdin + wake)
- Terminal setup/cleanup
- Event dispatch

### Key Changes
- [ ] Clear dirty flags AFTER layout completes
- [ ] Remove BlinkManager entirely
- [ ] Extract: setup_terminal, create_reactive_graph, run_event_loop, cleanup
- [ ] Use saturating_add for generation
- [ ] Remove Rc<RefCell<>> if not needed

### Tests to Write
- [ ] Dirty flags visible to layout derived
- [ ] Generation increments correctly
- [ ] Exit requested terminates loop
- [ ] Render effect fires when framebuffer changes

---

## Phase 7: renderer/* (diff.rs, inline.rs, append.rs)

### Current Issues (12 total)
- 80% code duplication
- FrameBuffer clone every frame (diff.rs)
- Hardcoded ANSI escapes instead of ansi module
- Bypasses OutputBuffer in append.rs

### Features to Preserve
- Diff rendering (only changed cells)
- Inline rendering (cursor-addressable)
- Append rendering (log-style)
- Synchronized output blocks

### Key Changes
- [ ] Extract RendererCore with shared logic
- [ ] Implement double-buffering with swap instead of clone
- [ ] Use ansi module consistently
- [ ] Use OutputBuffer in append.rs write_history

### Tests to Write
- [ ] Diff only outputs changed cells
- [ ] No clone per frame (measure allocations)
- [ ] Synchronized output wraps render
- [ ] Each mode produces correct terminal output

---

## Phase 8: layout/* (polish)

### Current Issues (15 total)
- Lossy f32→usize casts
- MinContent=1 infinite loop risk
- Cache cleared every frame (defeats optimization)
- No layout tests

### Features to Preserve
- Taffy low-level trait implementation
- Text measurement with Unicode support
- Auto-scroll detection
- Scroll limits written to SharedBuffer

### Key Changes
- [ ] Use .ceil() for f32→usize conversions
- [ ] MinContent=2 minimum (for wide chars)
- [ ] Only clear cache for dirty nodes
- [ ] Add comprehensive layout tests

### Tests to Write
- [ ] Simple flex row/column
- [ ] Nested containers
- [ ] Text wrapping affects layout
- [ ] Scroll detection for overflow content

---

## Execution Strategy

For each file:
1. **DISCUSS TOGETHER** - Read current implementation, map ALL features/behaviors
2. **AGREE ON SPEC** - What stays, what changes, what's missing
3. **Write tests** that verify correct behavior
4. **Implement** clean version that passes tests
5. **Delete** old code entirely (no patching)
6. **Run full test suite** to catch regressions

**Rule:** No solo runs. Every file gets a discussion BEFORE any code.
**Rule:** No half-measures. Either the file is completely rewritten or untouched.

---

## Session Handoff Notes

Context at 86%. Continue in next session with:
1. Read this plan
2. Pick next phase
3. Execute: spec → test → implement

**Remember:** Two friends on a Friday afternoon. Methodical. No rush.
