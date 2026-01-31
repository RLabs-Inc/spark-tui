# SparkTUI Shared Buffer Specification

**Version**: 2.0
**Date**: January 30, 2026
**Authors**: Rusty & Claude

This document defines the memory layout contract between TypeScript and Rust.
Both implementations MUST match this spec exactly. **This is the source of truth.**

---

## Design Principles

1. **Cache-Aligned Access** — Fields grouped by access pattern, aligned to 64-byte cache lines
2. **Zero-Copy** — Both sides read/write the same memory, no serialization
3. **Atomic Wake** — Futex-based notification for instant cross-language signaling
4. **Future-Proof** — 512-byte stride with reserved space for animation, effects, physics
5. **Configurable Scale** — MAX_NODES and TEXT_POOL_SIZE tunable per application
6. **Safe Boundaries** — Explicit limits with bounds checking in Rust

---

## Memory Layout Overview

```
┌─────────────────────────────────────────┐
│ HEADER (256 bytes)                      │  Fixed size, global state
├─────────────────────────────────────────┤
│ NODES (512 bytes × MAX_NODES)           │  Per-component data
├─────────────────────────────────────────┤
│ TEXT POOL (TEXT_POOL_SIZE bytes)        │  UTF-8 text content
├─────────────────────────────────────────┤
│ EVENT RING (5,132 bytes)                │  Rust → TS event queue
└─────────────────────────────────────────┘
```

---

## Constants

```
HEADER_SIZE      = 256 bytes
NODE_STRIDE      = 512 bytes
MAX_NODES        = configurable (default: 10,000)
TEXT_POOL_SIZE   = configurable (default: 10,485,760 = 10 MB)
EVENT_RING_SIZE  = 5,132 bytes

Buffer size = HEADER_SIZE + (NODE_STRIDE × MAX_NODES) + TEXT_POOL_SIZE + EVENT_RING_SIZE
```

### Default Configuration (~15 MB)

```
256 + (512 × 10,000) + 10,485,760 + 5,132 = 15,611,148 bytes (~14.9 MB)
```

### Sizing Examples

| Use Case | MAX_NODES | TEXT_POOL | Total |
|----------|-----------|-----------|-------|
| Tiny CLI | 100 | 100 KB | ~156 KB |
| Simple TUI | 1,000 | 1 MB | ~1.5 MB |
| Typical App | 10,000 | 10 MB | ~15 MB |
| Dashboard | 50,000 | 50 MB | ~76 MB |
| Game/Massive | 100,000 | 100 MB | ~151 MB |

---

## Header Layout (256 bytes)

The header contains global state shared between TypeScript and Rust.

### Bytes 0-63: Core

| Offset | Size | Type | Name | Writer | Description |
|--------|------|------|------|--------|-------------|
| 0 | 4 | u32 | `version` | TS | Buffer format version (currently 2) |
| 4 | 4 | u32 | `node_count` | TS | Active nodes in buffer |
| 8 | 4 | u32 | `max_nodes` | TS | Configured MAX_NODES |
| 12 | 4 | u32 | `terminal_width` | TS | Terminal columns |
| 16 | 4 | u32 | `terminal_height` | TS | Terminal rows |
| 20 | 4 | u32 | `generation` | TS | Incremented on structural changes |
| 24 | 4 | u32 | `text_pool_size` | TS | Configured text pool size |
| 28 | 4 | u32 | `text_pool_write_ptr` | TS | Next write position in text pool |
| 32-63 | 32 | — | _reserved_ | — | Future core fields |

### Bytes 64-95: Wake & Sync

**Must be 4-byte aligned for Atomics.wait/notify.**

| Offset | Size | Type | Name | Writer | Description |
|--------|------|------|------|--------|-------------|
| 64 | 4 | u32 | `wake_rust` | TS | TS sets to 1 + futex_wake to wake Rust |
| 68 | 4 | u32 | `wake_ts` | Rust | Rust sets to 1 + futex_wake to wake TS |
| 72-95 | 24 | — | _reserved_ | — | Future sync primitives |

### Bytes 96-127: State

**Rust writes these, TypeScript reads.**

| Offset | Size | Type | Name | Writer | Description |
|--------|------|------|------|--------|-------------|
| 96 | 4 | i32 | `focused_index` | Rust | Currently focused component (-1 = none) |
| 100 | 4 | i32 | `hovered_index` | Rust | Currently hovered component (-1 = none) |
| 104 | 4 | i32 | `pressed_index` | Rust | Currently pressed component (-1 = none) |
| 108 | 2 | u16 | `mouse_x` | Rust | Last mouse X position |
| 110 | 2 | u16 | `mouse_y` | Rust | Last mouse Y position |
| 112-127 | 16 | — | _reserved_ | — | Future state fields |

### Bytes 128-159: Config

**TypeScript writes these, Rust reads.**

| Offset | Size | Type | Name | Writer | Description |
|--------|------|------|------|--------|-------------|
| 128 | 4 | u32 | `config_flags` | TS | Bitfield (see Config Flags section) |
| 132 | 4 | u32 | `render_mode` | TS | 0=diff, 1=inline, 2=append |
| 136 | 4 | u32 | `cursor_config` | TS | Packed: visibility, shape, blink |
| 140 | 4 | u32 | `scroll_speed` | TS | Lines per scroll wheel tick (default: 3) |
| 144-159 | 16 | — | _reserved_ | — | Future config fields |

### Bytes 160-191: Events

| Offset | Size | Type | Name | Writer | Description |
|--------|------|------|------|--------|-------------|
| 160 | 4 | u32 | `event_write_idx` | Rust | Ring buffer write position |
| 164 | 4 | u32 | `event_read_idx` | TS | Ring buffer read position |
| 168 | 1 | u8 | `exit_requested` | Rust | Set to 1 on exit event |
| 169-191 | 23 | — | _reserved_ | — | Future event fields |

### Bytes 192-255: Stats & Debug

| Offset | Size | Type | Name | Writer | Description |
|--------|------|------|------|--------|-------------|
| 192 | 4 | u32 | `render_count` | Rust | Frames rendered (for FPS tracking) |
| 196 | 4 | u32 | `layout_count` | Rust | Layout passes (for profiling) |
| 200-255 | 56 | — | _reserved_ | — | Future metrics, profiling |

---

## Node Layout (512 bytes per node)

Each node occupies exactly 512 bytes, organized by access pattern.
Each 64-byte section aligns to a CPU cache line.

### Cache Line 1 — Bytes 0-63: Layout Dimensions

**Access pattern**: Layout pass reads all together.

| Offset | Size | Type | Name | Default | Description |
|--------|------|------|------|---------|-------------|
| 0 | 4 | f32 | `width` | NaN | Width (NaN = auto) |
| 4 | 4 | f32 | `height` | NaN | Height (NaN = auto) |
| 8 | 4 | f32 | `min_width` | NaN | Minimum width (NaN = none) |
| 12 | 4 | f32 | `min_height` | NaN | Minimum height (NaN = none) |
| 16 | 4 | f32 | `max_width` | NaN | Maximum width (NaN = none) |
| 20 | 4 | f32 | `max_height` | NaN | Maximum height (NaN = none) |
| 24 | 4 | f32 | `flex_basis` | NaN | Flex basis (NaN = auto) |
| 28 | 4 | f32 | `flex_grow` | 0.0 | Flex grow factor |
| 32 | 4 | f32 | `flex_shrink` | 1.0 | Flex shrink factor |
| 36 | 4 | f32 | `padding_top` | 0.0 | Top padding |
| 40 | 4 | f32 | `padding_right` | 0.0 | Right padding |
| 44 | 4 | f32 | `padding_bottom` | 0.0 | Bottom padding |
| 48 | 4 | f32 | `padding_left` | 0.0 | Left padding |
| 52 | 4 | f32 | `margin_top` | 0.0 | Top margin |
| 56 | 4 | f32 | `margin_right` | 0.0 | Right margin |
| 60 | 4 | f32 | `margin_bottom` | 0.0 | Bottom margin |

### Cache Line 2 — Bytes 64-127: Layout Spacing & Enums

**Access pattern**: Layout pass reads all together.

| Offset | Size | Type | Name | Default | Description |
|--------|------|------|------|---------|-------------|
| 64 | 4 | f32 | `margin_left` | 0.0 | Left margin |
| 68 | 4 | f32 | `gap` | 0.0 | Gap between children (both axes) |
| 72 | 4 | f32 | `row_gap` | 0.0 | Row gap (overrides gap if non-zero) |
| 76 | 4 | f32 | `column_gap` | 0.0 | Column gap (overrides gap if non-zero) |
| 80 | 4 | f32 | `inset_top` | NaN | Top inset for positioned elements |
| 84 | 4 | f32 | `inset_right` | NaN | Right inset |
| 88 | 4 | f32 | `inset_bottom` | NaN | Bottom inset |
| 92 | 4 | f32 | `inset_left` | NaN | Left inset |
| 96 | 1 | u8 | `flex_direction` | 0 | 0=row, 1=column, 2=row-reverse, 3=col-reverse |
| 97 | 1 | u8 | `flex_wrap` | 0 | 0=nowrap, 1=wrap, 2=wrap-reverse |
| 98 | 1 | u8 | `justify_content` | 0 | 0=start, 1=end, 2=center, 3=between, 4=around, 5=evenly |
| 99 | 1 | u8 | `align_items` | 0 | 0=start, 1=end, 2=center, 3=baseline, 4=stretch |
| 100 | 1 | u8 | `align_content` | 0 | Same values as justify_content |
| 101 | 1 | u8 | `align_self` | 0 | 0=auto, then same as align_items |
| 102 | 1 | u8 | `position` | 0 | 0=relative, 1=absolute |
| 103 | 1 | u8 | `overflow` | 0 | 0=visible, 1=hidden, 2=scroll |
| 104 | 1 | u8 | `display` | 1 | 0=none, 1=flex |
| 105 | 1 | u8 | `border_width_top` | 0 | Top border width in cells |
| 106 | 1 | u8 | `border_width_right` | 0 | Right border width |
| 107 | 1 | u8 | `border_width_bottom` | 0 | Bottom border width |
| 108 | 1 | u8 | `border_width_left` | 0 | Left border width |
| 109 | 1 | u8 | `component_type` | 0 | 0=none, 1=box, 2=text, 3=input |
| 110 | 1 | u8 | `visible` | 1 | 0=hidden, 1=visible |
| 111 | 1 | u8 | _reserved_ | 0 | — |
| 112 | 4 | i32 | `parent_index` | -1 | Parent node index (-1 = root) |
| 116 | 4 | i32 | `tab_index` | 0 | Tab order (0 = not focusable via tab) |
| 120 | 4 | i32 | `child_count` | 0 | Number of children (Rust may compute) |
| 124 | 4 | — | _reserved_ | 0 | — |

### Cache Line 3 — Bytes 128-191: Output & Colors

**Access pattern**: Render pass reads. Rust writes output fields.

| Offset | Size | Type | Name | Default | Writer | Description |
|--------|------|------|------|---------|--------|-------------|
| 128 | 4 | f32 | `computed_x` | 0.0 | Rust | Computed X position |
| 132 | 4 | f32 | `computed_y` | 0.0 | Rust | Computed Y position |
| 136 | 4 | f32 | `computed_width` | 0.0 | Rust | Computed width |
| 140 | 4 | f32 | `computed_height` | 0.0 | Rust | Computed height |
| 144 | 4 | f32 | `scroll_width` | 0.0 | Rust | Total scrollable content width |
| 148 | 4 | f32 | `scroll_height` | 0.0 | Rust | Total scrollable content height |
| 152 | 4 | f32 | `max_scroll_x` | 0.0 | Rust | Maximum horizontal scroll |
| 156 | 4 | f32 | `max_scroll_y` | 0.0 | Rust | Maximum vertical scroll |
| 160 | 4 | u32 | `fg_color` | 0 | TS | Foreground color (ARGB) |
| 164 | 4 | u32 | `bg_color` | 0 | TS | Background color (ARGB) |
| 168 | 4 | u32 | `border_color` | 0 | TS | Default border color (ARGB) |
| 172 | 4 | u32 | `border_top_color` | 0 | TS | Top border (0 = use default) |
| 176 | 4 | u32 | `border_right_color` | 0 | TS | Right border |
| 180 | 4 | u32 | `border_bottom_color` | 0 | TS | Bottom border |
| 184 | 4 | u32 | `border_left_color` | 0 | TS | Left border |
| 188 | 4 | u32 | `focus_ring_color` | 0 | TS | Focus indicator color |

### Cache Line 4 — Bytes 192-255: Visual Properties

**Access pattern**: Render pass reads.

| Offset | Size | Type | Name | Default | Description |
|--------|------|------|------|---------|-------------|
| 192 | 4 | u32 | `cursor_fg_color` | 0 | Cursor foreground (input components) |
| 196 | 4 | u32 | `cursor_bg_color` | 0 | Cursor background |
| 200 | 4 | u32 | `selection_color` | 0 | Selection highlight color |
| 204 | 1 | u8 | `opacity` | 255 | Opacity 0-255 (255 = fully opaque) |
| 205 | 1 | i8 | `z_index` | 0 | Stacking order (-128 to 127) |
| 206 | 1 | u8 | `border_style` | 0 | Default: 0=none, 1=single, 2=double, 3=rounded, 4=thick, 5=dashed, 6=dotted |
| 207 | 1 | u8 | `border_style_top` | 0 | Top (0 = use default) |
| 208 | 1 | u8 | `border_style_right` | 0 | Right |
| 209 | 1 | u8 | `border_style_bottom` | 0 | Bottom |
| 210 | 1 | u8 | `border_style_left` | 0 | Left |
| 211 | 1 | u8 | `scrollable_flags` | 0 | Bit 0: scrollable, Bit 1: show scrollbar |
| 212 | 2 | u16 | `border_char_h` | 0 | Custom horizontal border char (0 = use style) |
| 214 | 2 | u16 | `border_char_v` | 0 | Custom vertical border char |
| 216 | 2 | u16 | `border_char_tl` | 0 | Custom top-left corner |
| 218 | 2 | u16 | `border_char_tr` | 0 | Custom top-right corner |
| 220 | 2 | u16 | `border_char_bl` | 0 | Custom bottom-left corner |
| 222 | 2 | u16 | `border_char_br` | 0 | Custom bottom-right corner |
| 224 | 1 | u8 | `focus_indicator_char` | 0x2A | Focus marker character (default '*') |
| 225 | 1 | u8 | `focus_indicator_enabled` | 1 | 0=disabled, 1=enabled |
| 226-255 | 30 | — | _reserved_ | 0 | Future visual properties |

### Cache Line 5 — Bytes 256-319: Text Properties

**Access pattern**: Text measurement and render.

| Offset | Size | Type | Name | Default | Description |
|--------|------|------|------|---------|-------------|
| 256 | 4 | u32 | `text_offset` | 0 | Offset into text pool |
| 260 | 4 | u32 | `text_length` | 0 | Text length in bytes |
| 264 | 1 | u8 | `text_align` | 0 | 0=left, 1=center, 2=right |
| 265 | 1 | u8 | `text_wrap` | 0 | 0=nowrap, 1=wrap, 2=truncate |
| 266 | 1 | u8 | `text_overflow` | 0 | 0=clip, 1=ellipsis, 2=fade |
| 267 | 1 | u8 | `text_attrs` | 0 | Bitfield (see Text Attributes) |
| 268 | 1 | u8 | `text_decoration` | 0 | 0=none, 1=underline, 2=overline, 3=line-through |
| 269 | 1 | u8 | `text_decoration_style` | 0 | 0=solid, 1=double, 2=dotted, 3=dashed, 4=wavy |
| 270 | 4 | u32 | `text_decoration_color` | 0 | Decoration color (0 = use fg) |
| 274 | 1 | u8 | `line_height` | 0 | Line height (0 = auto/1.0) |
| 275 | 1 | u8 | `letter_spacing` | 0 | Extra spacing between chars |
| 276 | 1 | u8 | `max_lines` | 0 | Max lines to show (0 = unlimited) |
| 277-319 | 43 | — | _reserved_ | 0 | Future text properties |

### Cache Line 6 — Bytes 320-383: Interaction State

**Access pattern**: Input handling, focus management.

| Offset | Size | Type | Name | Default | Description |
|--------|------|------|------|---------|-------------|
| 320 | 4 | i32 | `scroll_x` | 0 | Current horizontal scroll position |
| 324 | 4 | i32 | `scroll_y` | 0 | Current vertical scroll position |
| 328 | 4 | i32 | `cursor_position` | 0 | Cursor position in text (input) |
| 332 | 4 | i32 | `selection_start` | -1 | Selection start (-1 = none) |
| 336 | 4 | i32 | `selection_end` | -1 | Selection end |
| 340 | 4 | u32 | `cursor_char` | 0 | Custom cursor character (0 = default) |
| 344 | 4 | u32 | `cursor_alt_char` | 0 | Cursor blink alternate char |
| 348 | 1 | u8 | `dirty_flags` | 0 | Bitfield (see Dirty Flags) |
| 349 | 1 | u8 | `interaction_flags` | 0 | Bitfield (see Interaction Flags) |
| 350 | 1 | u8 | `cursor_flags` | 0 | Bit 0: visible, Bit 1: blink enabled |
| 351 | 1 | u8 | `cursor_style` | 0 | 0=block, 1=bar, 2=underline |
| 352 | 1 | u8 | `cursor_blink_rate` | 0 | Blink interval (0 = default 530ms) |
| 353 | 1 | u8 | `max_length` | 0 | Max input length (0 = unlimited) |
| 354 | 2 | u16 | `input_type` | 0 | 0=text, 1=password, 2=number, 3=email |
| 356-383 | 28 | — | _reserved_ | 0 | Future interaction properties |

### Cache Line 7 — Bytes 384-447: Animation (Reserved)

**Reserved for future animation system.**

| Offset | Size | Type | Name | Default | Description |
|--------|------|------|------|---------|-------------|
| 384-447 | 64 | — | _reserved_ | 0 | Animation: id, state, timing, easing, phase, keyframes |

### Cache Line 8 — Bytes 448-511: Effects & Transforms (Reserved)

**Reserved for future visual effects and transforms.**

| Offset | Size | Type | Name | Default | Description |
|--------|------|------|------|---------|-------------|
| 448-511 | 64 | — | _reserved_ | 0 | Effects: gradients, shadows, glow, transforms, physics |

---

## Text Pool

The text pool stores UTF-8 encoded text content for all nodes.

**Location**: `HEADER_SIZE + (NODE_STRIDE × MAX_NODES)`

### Allocation Strategy

- **Bump allocator**: New text appends at `text_pool_write_ptr`
- **Slot reuse**: When updating text, reuse existing slot if new text fits
- **Compaction**: Call `compactTextPool()` when fragmented

### Node Reference

Each node references its text via:
- `text_offset` (bytes 256-259): Start position in pool
- `text_length` (bytes 260-263): Length in bytes

### Example

```
Node 0: "Hello" → offset=0, length=5
Node 1: "World" → offset=5, length=5
Node 2: "!" → offset=10, length=1

Text pool: [H][e][l][l][o][W][o][r][l][d][!]...
            0  1  2  3  4  5  6  7  8  9  10
```

---

## Event Ring Buffer

Events flow from Rust to TypeScript via a lock-free single-producer single-consumer ring buffer.

**Location**: `HEADER_SIZE + (NODE_STRIDE × MAX_NODES) + TEXT_POOL_SIZE`

### Constants

```
EVENT_RING_HEADER_SIZE = 12 bytes
EVENT_SLOT_SIZE = 20 bytes
MAX_EVENTS = 256
EVENT_RING_SIZE = 12 + (20 × 256) = 5,132 bytes
```

### Ring Header (12 bytes)

| Offset | Size | Type | Name | Description |
|--------|------|------|------|-------------|
| 0 | 4 | u32 | `write_idx` | Next write position (Rust increments) |
| 4 | 4 | u32 | `read_idx` | Next read position (TS increments) |
| 8 | 4 | u32 | _reserved_ | Future use |

### Event Slot (20 bytes)

| Offset | Size | Type | Name | Description |
|--------|------|------|------|-------------|
| 0 | 1 | u8 | `event_type` | Event type (see Event Types) |
| 1 | 1 | u8 | _padding_ | Alignment |
| 2 | 2 | u16 | `component_index` | Target component (0xFFFF = global) |
| 4 | 16 | [u8;16] | `data` | Event-specific payload |

### Event Types

| Value | Name | Data Layout |
|-------|------|-------------|
| 0 | None | — |
| 1 | Key | `key_code:u32, modifiers:u8, char:u32, repeat:u8` |
| 2 | MouseDown | `button:u8, x:u16, y:u16, modifiers:u8` |
| 3 | MouseUp | `button:u8, x:u16, y:u16, modifiers:u8` |
| 4 | Click | `button:u8, x:u16, y:u16, click_count:u8` |
| 5 | MouseEnter | `x:u16, y:u16` |
| 6 | MouseLeave | `x:u16, y:u16` |
| 7 | MouseMove | `x:u16, y:u16, modifiers:u8` |
| 8 | Scroll | `delta_x:i16, delta_y:i16, modifiers:u8` |
| 9 | Focus | — |
| 10 | Blur | — |
| 11 | ValueChange | `cursor_pos:i32, selection_start:i32, selection_end:i32` |
| 12 | Submit | — |
| 13 | Cancel | — |
| 14 | Exit | `exit_code:u8` |
| 15 | Resize | `width:u16, height:u16` |

---

## Bitfield Definitions

### Config Flags (Header offset 128)

| Bit | Name | Description |
|-----|------|-------------|
| 0 | `EXIT_ON_CTRL_C` | Exit application on Ctrl+C |
| 1 | `TAB_NAVIGATION` | Tab key moves focus between focusables |
| 2 | `ARROW_SCROLL` | Arrow keys scroll focused scrollable |
| 3 | `PAGE_SCROLL` | Page Up/Down scroll focused |
| 4 | `HOME_END_SCROLL` | Home/End scroll to start/end |
| 5 | `WHEEL_SCROLL` | Mouse wheel scrolls hovered |
| 6 | `FOCUS_ON_CLICK` | Click focuses focusable components |
| 7 | `MOUSE_ENABLED` | Enable mouse tracking |
| 8 | `KITTY_KEYBOARD` | Use Kitty keyboard protocol |

**Default**: `0x00FF` (bits 0-7 enabled)

### Dirty Flags (Node offset 348)

| Bit | Name | Description |
|-----|------|-------------|
| 0 | `DIRTY_LAYOUT` | Layout properties changed |
| 1 | `DIRTY_VISUAL` | Visual properties changed |
| 2 | `DIRTY_TEXT` | Text content changed |
| 3 | `DIRTY_HIERARCHY` | Parent/children changed |

### Interaction Flags (Node offset 349)

| Bit | Name | Description |
|-----|------|-------------|
| 0 | `FOCUSABLE` | Can receive focus |
| 1 | `FOCUSED` | Currently has focus |
| 2 | `HOVERED` | Mouse is over this component |
| 3 | `PRESSED` | Mouse button down on this |
| 4 | `DISABLED` | Interaction disabled |

### Text Attributes (Node offset 267)

| Bit | Name | Description |
|-----|------|-------------|
| 0 | `BOLD` | Bold text |
| 1 | `ITALIC` | Italic text |
| 2 | `UNDERLINE` | Underlined text |
| 3 | `STRIKETHROUGH` | Strikethrough text |
| 4 | `DIM` | Dimmed/faint text |
| 5 | `BLINK` | Blinking text |
| 6 | `REVERSE` | Reverse video (swap fg/bg) |
| 7 | `HIDDEN` | Hidden text |

---

## Color Format

All colors are 32-bit packed ARGB:

```
Bits 31-24: Alpha (0 = transparent, 255 = opaque)
Bits 23-16: Red
Bits 15-8:  Green
Bits 7-0:   Blue
```

### Special Values

| Value | Meaning |
|-------|---------|
| `0x00000000` | Transparent / inherit / use terminal default |
| `0xFFrrggbb` | Fully opaque color |

### Example

```
Red:    0xFFFF0000
Green:  0xFF00FF00
Blue:   0xFF0000FF
White:  0xFFFFFFFF
50% transparent black: 0x80000000
```

---

## Border Styles

| Value | Name | Characters |
|-------|------|------------|
| 0 | None | (no border) |
| 1 | Single | `─ │ ┌ ┐ └ ┘` |
| 2 | Double | `═ ║ ╔ ╗ ╚ ╝` |
| 3 | Rounded | `─ │ ╭ ╮ ╰ ╯` |
| 4 | Thick | `━ ┃ ┏ ┓ ┗ ┛` |
| 5 | Dashed | `╌ ╎ ┌ ┐ └ ┘` |
| 6 | Dotted | `┄ ┆ ┌ ┐ └ ┘` |
| 7 | ASCII | `- | + + + +` |
| 8-255 | _reserved_ | Future styles |

---

## Implementation Notes

### TypeScript

```typescript
// Use DataView for unaligned access
const view = new DataView(buffer);
view.getFloat32(offset, true);  // true = little-endian

// Use Atomics for wake flags
Atomics.store(u32View, WAKE_RUST_IDX, 1);
Atomics.notify(u32View, WAKE_RUST_IDX, 1);

// Use Atomics.waitAsync for non-blocking wait
Atomics.waitAsync(u32View, WAKE_TS_IDX, 0).value.then(() => {
  // Rust woke us up
});
```

### Rust

```rust
// Use unaligned reads/writes
let value = ptr::read_unaligned(ptr.add(offset) as *const f32);
ptr::write_unaligned(ptr.add(offset) as *mut f32, value);

// Bounds check all node access
debug_assert!(node_index < self.max_nodes());

// Use atomic_wait crate for futex
atomic_wait::wake_one(wake_ptr);
```

### Cache Optimization

| Pass | Cache Lines Read | Bytes |
|------|------------------|-------|
| Layout | Lines 1-2 (0-127) | 128 |
| Render | Lines 3-6 (128-383) | 256 |
| Input | Line 6 (320-383) | 64 |
| Animation | Line 7 (384-447) | 64 |
| Effects | Line 8 (448-511) | 64 |

---

## Configuration API

```typescript
interface SharedBufferConfig {
  /** Maximum number of components. Default: 10,000 */
  maxNodes?: number;

  /** Text pool size in bytes. Default: 10 MB */
  textPoolSize?: number;

  /** Event ring capacity. Default: 256 */
  maxEvents?: number;
}

// Create buffer with custom config
const buffer = createSharedBuffer({
  maxNodes: 1_000,
  textPoolSize: 1_000_000,  // 1 MB
});
```

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 2.0 | 2026-01-30 | Complete redesign: 512-byte stride, cache-aligned, future-proof |
| 1.0 | 2025-xx-xx | Original organic growth version (256-byte stride) |

---

## Checksums

To verify both implementations match:

```
Header size: 256 bytes
Node stride: 512 bytes
Text offset in node: 256
Scroll offset in node: 320
Event slot size: 20 bytes
```

**If these don't match between TS and Rust, the implementations are out of sync.**

---

*This specification is the source of truth. TypeScript and Rust implementations MUST match exactly.*
