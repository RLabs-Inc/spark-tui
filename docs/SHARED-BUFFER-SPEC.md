# SparkTUI Shared Buffer Specification

**Version**: 3.0
**Date**: January 31, 2026
**Authors**: Rusty & Claude

This document defines the memory layout contract between TypeScript and Rust.
Both implementations MUST match this spec exactly. **This is the source of truth.**

---

## Design Principles

1. **Cache-Aligned Access** — Fields grouped by access pattern, aligned to 64-byte cache lines
2. **Zero-Copy** — Both sides read/write the same memory, no serialization
3. **Atomic Wake** — Futex-based notification for instant cross-language signaling
4. **Future-Proof** — 1024-byte stride with reserved space for animation, effects, physics
5. **Full CSS Grid** — 32 column + 32 row tracks, first TUI framework with complete Grid support
6. **Configurable Scale** — MAX_NODES and TEXT_POOL_SIZE tunable per application
7. **Safe Boundaries** — Explicit limits with bounds checking in Rust

---

## Memory Layout Overview

```
┌─────────────────────────────────────────┐
│ HEADER (256 bytes)                      │  Fixed size, global state
├─────────────────────────────────────────┤
│ NODES (1024 bytes × MAX_NODES)          │  Per-component data
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
NODE_STRIDE      = 1024 bytes
MAX_NODES        = configurable (default: 10,000)
TEXT_POOL_SIZE   = configurable (default: 10,485,760 = 10 MB)
EVENT_RING_SIZE  = 5,132 bytes

Buffer size = HEADER_SIZE + (NODE_STRIDE × MAX_NODES) + TEXT_POOL_SIZE + EVENT_RING_SIZE
```

### Default Configuration (~20 MB)

```
256 + (1024 × 10,000) + 10,485,760 + 5,132 = 20,731,148 bytes (~19.8 MB)
```

### Sizing Examples

| Use Case | MAX_NODES | TEXT_POOL | Total |
|----------|-----------|-----------|-------|
| Tiny CLI | 100 | 100 KB | ~206 KB |
| Simple TUI | 1,000 | 1 MB | ~2 MB |
| Typical App | 10,000 | 10 MB | ~20 MB |
| Dashboard | 50,000 | 50 MB | ~101 MB |
| Game/Massive | 100,000 | 100 MB | ~202 MB |

---

## Header Layout (256 bytes)

The header contains global state shared between TypeScript and Rust.

### Bytes 0-63: Core

| Offset | Size | Type | Name | Writer | Description |
|--------|------|------|------|--------|-------------|
| 0 | 4 | u32 | `version` | TS | Buffer format version (currently 3) |
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

## Node Layout (1024 bytes per node)

Each node occupies exactly 1024 bytes, organized in 16 cache lines (64 bytes each).

### Overview by Cache Line

| Lines | Bytes | Purpose |
|-------|-------|---------|
| 1-4 | 0-255 | Layout props (Flexbox + Grid container) |
| 5-7 | 256-447 | Grid column tracks (32 tracks × 6 bytes) |
| 8-10 | 448-639 | Grid row tracks (32 tracks × 6 bytes) |
| 11-12 | 640-767 | Computed output + visual props |
| 13-14 | 768-895 | Colors + text properties |
| 15-16 | 896-1023 | Interaction + reserved (animation/effects) |

---

### Cache Lines 1-4 — Bytes 0-255: Layout Properties

**Access pattern**: Taffy reads all together during layout pass.

#### Line 1 (0-63): Core Layout Dimensions

| Offset | Size | Type | Name | Default | Description |
|--------|------|------|------|---------|-------------|
| 0 | 4 | f32 | `width` | NaN | Width (NaN = auto) |
| 4 | 4 | f32 | `height` | NaN | Height (NaN = auto) |
| 8 | 4 | f32 | `min_width` | NaN | Minimum width (NaN = none) |
| 12 | 4 | f32 | `min_height` | NaN | Minimum height (NaN = none) |
| 16 | 4 | f32 | `max_width` | NaN | Maximum width (NaN = none) |
| 20 | 4 | f32 | `max_height` | NaN | Maximum height (NaN = none) |
| 24 | 4 | f32 | `aspect_ratio` | NaN | Aspect ratio (NaN = none) |
| 28 | 1 | u8 | `component_type` | 0 | 0=none, 1=box, 2=text, 3=input |
| 29 | 1 | u8 | `display` | 1 | 0=none, 1=flex, 2=grid |
| 30 | 1 | u8 | `position` | 0 | 0=relative, 1=absolute |
| 31 | 1 | u8 | `overflow` | 0 | 0=visible, 1=hidden, 2=scroll |
| 32 | 1 | u8 | `visible` | 1 | 0=hidden, 1=visible |
| 33 | 1 | u8 | `box_sizing` | 0 | 0=border-box, 1=content-box |
| 34 | 1 | u8 | `dirty_flags` | 0 | Bitfield (see Dirty Flags) |
| 35 | 1 | u8 | _reserved_ | 0 | — |
| 36-63 | 28 | — | _reserved_ | 0 | Future core props |

#### Line 2 (64-127): Flexbox Properties

| Offset | Size | Type | Name | Default | Description |
|--------|------|------|------|---------|-------------|
| 64 | 1 | u8 | `flex_direction` | 0 | 0=row, 1=column, 2=row-reverse, 3=col-reverse |
| 65 | 1 | u8 | `flex_wrap` | 0 | 0=nowrap, 1=wrap, 2=wrap-reverse |
| 66 | 1 | u8 | `justify_content` | 0 | 0=start, 1=end, 2=center, 3=between, 4=around, 5=evenly |
| 67 | 1 | u8 | `align_items` | 0 | 0=start, 1=end, 2=center, 3=baseline, 4=stretch |
| 68 | 1 | u8 | `align_content` | 0 | Same values as justify_content |
| 69 | 1 | u8 | `align_self` | 0 | 0=auto, then same as align_items |
| 70 | 2 | — | _reserved_ | 0 | Alignment padding |
| 72 | 4 | f32 | `flex_grow` | 0.0 | Flex grow factor |
| 76 | 4 | f32 | `flex_shrink` | 1.0 | Flex shrink factor |
| 80 | 4 | f32 | `flex_basis` | NaN | Flex basis (NaN = auto) |
| 84 | 4 | f32 | `gap` | 0.0 | Gap between children (both axes) |
| 88 | 4 | f32 | `row_gap` | 0.0 | Row gap (overrides gap if non-zero) |
| 92 | 4 | f32 | `column_gap` | 0.0 | Column gap (overrides gap if non-zero) |
| 96-127 | 32 | — | _reserved_ | 0 | Future flex props |

#### Line 3 (128-191): Spacing Properties

| Offset | Size | Type | Name | Default | Description |
|--------|------|------|------|---------|-------------|
| 128 | 4 | f32 | `padding_top` | 0.0 | Top padding |
| 132 | 4 | f32 | `padding_right` | 0.0 | Right padding |
| 136 | 4 | f32 | `padding_bottom` | 0.0 | Bottom padding |
| 140 | 4 | f32 | `padding_left` | 0.0 | Left padding |
| 144 | 4 | f32 | `margin_top` | 0.0 | Top margin |
| 148 | 4 | f32 | `margin_right` | 0.0 | Right margin |
| 152 | 4 | f32 | `margin_bottom` | 0.0 | Bottom margin |
| 156 | 4 | f32 | `margin_left` | 0.0 | Left margin |
| 160 | 4 | f32 | `inset_top` | NaN | Top inset for positioned elements |
| 164 | 4 | f32 | `inset_right` | NaN | Right inset |
| 168 | 4 | f32 | `inset_bottom` | NaN | Bottom inset |
| 172 | 4 | f32 | `inset_left` | NaN | Left inset |
| 176 | 1 | u8 | `border_width_top` | 0 | Top border width in cells |
| 177 | 1 | u8 | `border_width_right` | 0 | Right border width |
| 178 | 1 | u8 | `border_width_bottom` | 0 | Bottom border width |
| 179 | 1 | u8 | `border_width_left` | 0 | Left border width |
| 180 | 4 | i32 | `parent_index` | -1 | Parent node index (-1 = root) |
| 184 | 4 | i32 | `tab_index` | 0 | Tab order (0 = not focusable via tab) |
| 188 | 4 | — | _reserved_ | 0 | — |

#### Line 4 (192-255): Grid Container Properties

| Offset | Size | Type | Name | Default | Description |
|--------|------|------|------|---------|-------------|
| 192 | 1 | u8 | `grid_auto_flow` | 0 | 0=row, 1=column, 2=row-dense, 3=col-dense |
| 193 | 1 | u8 | `justify_items` | 0 | 0=start, 1=end, 2=center, 3=stretch |
| 194 | 1 | u8 | `grid_column_count` | 0 | Number of explicit column tracks (0-32) |
| 195 | 1 | u8 | `grid_row_count` | 0 | Number of explicit row tracks (0-32) |
| 196 | 1 | u8 | `grid_auto_columns_type` | 0 | Track type (see Track Types) |
| 197 | 1 | u8 | `grid_auto_rows_type` | 0 | Track type |
| 198 | 2 | — | _reserved_ | 0 | Alignment padding |
| 200 | 4 | f32 | `grid_auto_columns_value` | 0.0 | Track value (for Length, Percent, Fr) |
| 204 | 4 | f32 | `grid_auto_rows_value` | 0.0 | Track value |
| 208 | 2 | i16 | `grid_column_start` | 0 | Column start line (0 = auto) |
| 210 | 2 | i16 | `grid_column_end` | 0 | Column end line (0 = auto, negative = span) |
| 212 | 2 | i16 | `grid_row_start` | 0 | Row start line (0 = auto) |
| 214 | 2 | i16 | `grid_row_end` | 0 | Row end line (0 = auto, negative = span) |
| 216 | 1 | u8 | `justify_self` | 0 | 0=auto, 1=start, 2=end, 3=center, 4=stretch |
| 217-219 | 3 | — | _reserved_ | 0 | Alignment padding |
| 220 | 4 | i32 | `first_child` | -1 | First child index (-1 = no children) |
| 224 | 4 | i32 | `prev_sibling` | -1 | Previous sibling index (-1 = first child) |
| 228 | 4 | i32 | `next_sibling` | -1 | Next sibling index (-1 = last child) |
| 232-255 | 24 | — | _reserved_ | 0 | Future grid/hierarchy props |

---

### Cache Lines 5-7 — Bytes 256-447: Grid Column Tracks

**32 column track definitions, 6 bytes each = 192 bytes**

Each track is stored as:

| Offset | Size | Type | Name | Description |
|--------|------|------|------|-------------|
| 0 | 1 | u8 | `type` | Track sizing type (see Track Types) |
| 1 | 1 | u8 | _padding_ | Alignment |
| 2 | 4 | f32 | `value` | Track value (for Length, Percent, Fr) |

**Track Types**

| Value | Name | Value Usage |
|-------|------|-------------|
| 0 | None | Track not used (sentinel for unused slots) |
| 1 | Auto | Value ignored |
| 2 | MinContent | Value ignored |
| 3 | MaxContent | Value ignored |
| 4 | Length | Fixed size in terminal cells |
| 5 | Percent | Percentage of container (0.0-1.0) |
| 6 | Fr | Fractional unit |
| 7 | FitContent | Maximum size (clamped to content) |

**Example**: `"1fr 2fr auto 100px"` stored as:
```
Track 0: type=6 (Fr), value=1.0
Track 1: type=6 (Fr), value=2.0
Track 2: type=1 (Auto), value=0.0
Track 3: type=4 (Length), value=100.0
Track 4-31: type=0 (None)
```

---

### Cache Lines 8-10 — Bytes 448-639: Grid Row Tracks

**32 row track definitions, 6 bytes each = 192 bytes**

Same format as column tracks.

---

### Cache Lines 11-12 — Bytes 640-767: Output & Visual

**Access pattern**: Rust writes output during layout. Render pass reads.

#### Line 11 (640-703): Computed Output

| Offset | Size | Type | Name | Default | Writer | Description |
|--------|------|------|------|---------|--------|-------------|
| 640 | 4 | f32 | `computed_x` | 0.0 | Rust | Computed X position |
| 644 | 4 | f32 | `computed_y` | 0.0 | Rust | Computed Y position |
| 648 | 4 | f32 | `computed_width` | 0.0 | Rust | Computed width |
| 652 | 4 | f32 | `computed_height` | 0.0 | Rust | Computed height |
| 656 | 4 | f32 | `content_width` | 0.0 | Rust | Content width (for scroll) |
| 660 | 4 | f32 | `content_height` | 0.0 | Rust | Content height |
| 664 | 4 | f32 | `max_scroll_x` | 0.0 | Rust | Maximum horizontal scroll |
| 668 | 4 | f32 | `max_scroll_y` | 0.0 | Rust | Maximum vertical scroll |
| 672 | 1 | u8 | `is_scrollable` | 0 | Rust | 0=no, 1=yes |
| 673-703 | 31 | — | _reserved_ | 0 | — | Future output fields |

#### Line 12 (704-767): Visual Properties

| Offset | Size | Type | Name | Default | Description |
|--------|------|------|------|---------|-------------|
| 704 | 4 | f32 | `opacity` | 1.0 | Opacity 0.0-1.0 |
| 708 | 4 | i32 | `z_index` | 0 | Stacking order |
| 712 | 1 | u8 | `border_style` | 0 | Default: 0=none, 1=single, 2=double, etc. |
| 713 | 1 | u8 | `border_style_top` | 0 | Top (0 = use default) |
| 714 | 1 | u8 | `border_style_right` | 0 | Right |
| 715 | 1 | u8 | `border_style_bottom` | 0 | Bottom |
| 716 | 1 | u8 | `border_style_left` | 0 | Left |
| 717 | 1 | u8 | `scrollbar_visibility` | 0 | 0=auto, 1=always, 2=never |
| 718 | 2 | u16 | `border_char_h` | 0 | Custom horizontal border char (0 = use style) |
| 720 | 2 | u16 | `border_char_v` | 0 | Custom vertical border char |
| 722 | 2 | u16 | `border_char_tl` | 0 | Custom top-left corner |
| 724 | 2 | u16 | `border_char_tr` | 0 | Custom top-right corner |
| 726 | 2 | u16 | `border_char_bl` | 0 | Custom bottom-left corner |
| 728 | 2 | u16 | `border_char_br` | 0 | Custom bottom-right corner |
| 730 | 1 | u8 | `focus_indicator_char` | 0x2A | Focus marker character (default '*') |
| 731 | 1 | u8 | `focus_indicator_enabled` | 1 | 0=disabled, 1=enabled |
| 732-767 | 36 | — | _reserved_ | 0 | Future visual properties |

---

### Cache Lines 13-14 — Bytes 768-895: Colors & Text

**Access pattern**: Render pass reads colors, text measurement reads text refs.

#### Line 13 (768-831): Colors

| Offset | Size | Type | Name | Default | Description |
|--------|------|------|------|---------|-------------|
| 768 | 4 | u32 | `fg_color` | 0 | Foreground color (ARGB) |
| 772 | 4 | u32 | `bg_color` | 0 | Background color (ARGB) |
| 776 | 4 | u32 | `border_color` | 0 | Default border color (ARGB) |
| 780 | 4 | u32 | `border_top_color` | 0 | Top border (0 = use default) |
| 784 | 4 | u32 | `border_right_color` | 0 | Right border |
| 788 | 4 | u32 | `border_bottom_color` | 0 | Bottom border |
| 792 | 4 | u32 | `border_left_color` | 0 | Left border |
| 796 | 4 | u32 | `focus_ring_color` | 0 | Focus indicator color |
| 800 | 4 | u32 | `cursor_fg_color` | 0 | Cursor foreground (input) |
| 804 | 4 | u32 | `cursor_bg_color` | 0 | Cursor background |
| 808 | 4 | u32 | `selection_color` | 0 | Selection highlight color |
| 812-831 | 20 | — | _reserved_ | 0 | Future colors |

#### Line 14 (832-895): Text Properties

| Offset | Size | Type | Name | Default | Description |
|--------|------|------|------|---------|-------------|
| 832 | 4 | u32 | `text_offset` | 0 | Offset into text pool |
| 836 | 4 | u32 | `text_length` | 0 | Text length in bytes |
| 840 | 1 | u8 | `text_align` | 0 | 0=left, 1=center, 2=right |
| 841 | 1 | u8 | `text_wrap` | 0 | 0=nowrap, 1=wrap, 2=truncate |
| 842 | 1 | u8 | `text_overflow` | 0 | 0=clip, 1=ellipsis, 2=fade |
| 843 | 1 | u8 | `text_attrs` | 0 | Bitfield (see Text Attributes) |
| 844 | 1 | u8 | `text_decoration` | 0 | 0=none, 1=underline, 2=overline, 3=line-through |
| 845 | 1 | u8 | `text_decoration_style` | 0 | 0=solid, 1=double, 2=dotted, 3=dashed, 4=wavy |
| 846 | 2 | — | _reserved_ | 0 | Alignment padding |
| 848 | 4 | u32 | `text_decoration_color` | 0 | Decoration color (0 = use fg) |
| 852 | 1 | u8 | `line_height` | 0 | Line height (0 = auto/1.0) |
| 853 | 1 | u8 | `letter_spacing` | 0 | Extra spacing between chars |
| 854 | 1 | u8 | `max_lines` | 0 | Max lines to show (0 = unlimited) |
| 855-895 | 41 | — | _reserved_ | 0 | Future text properties |

---

### Cache Lines 15-16 — Bytes 896-1023: Interaction & Reserved

**Access pattern**: Input handling, focus management, future animation.

#### Line 15 (896-959): Interaction State

| Offset | Size | Type | Name | Default | Description |
|--------|------|------|------|---------|-------------|
| 896 | 4 | i32 | `scroll_x` | 0 | Current horizontal scroll position |
| 900 | 4 | i32 | `scroll_y` | 0 | Current vertical scroll position |
| 904 | 4 | i32 | `cursor_position` | 0 | Cursor position in text (input) |
| 908 | 4 | i32 | `selection_start` | -1 | Selection start (-1 = none) |
| 912 | 4 | i32 | `selection_end` | -1 | Selection end |
| 916 | 4 | u32 | `cursor_char` | 0 | Custom cursor character (0 = default) |
| 920 | 4 | u32 | `cursor_alt_char` | 0 | Cursor blink alternate char |
| 924 | 1 | u8 | `interaction_flags` | 0 | Bitfield (see Interaction Flags) |
| 925 | 1 | u8 | `cursor_flags` | 0 | Bit 0: visible, Bit 1: blink enabled |
| 926 | 1 | u8 | `cursor_style` | 0 | 0=block, 1=bar, 2=underline |
| 927 | 1 | u8 | `cursor_blink_rate` | 0 | Blink interval (0 = default 530ms) |
| 928 | 1 | u8 | `max_length` | 0 | Max input length (0 = unlimited) |
| 929 | 1 | u8 | `input_type` | 0 | 0=text, 1=password, 2=number, 3=email |
| 930-959 | 30 | — | _reserved_ | 0 | Future interaction properties |

#### Line 16 (960-1023): Reserved (Animation, Effects, Transforms)

**Reserved for future systems.**

| Offset | Size | Type | Name | Default | Description |
|--------|------|------|------|---------|-------------|
| 960-1023 | 64 | — | _reserved_ | 0 | Animation, effects, transforms, physics |

---

## Hierarchy Management

Parent-child relationships use a doubly-linked sibling list for O(1) child operations.

### Fields (per node, in Line 4)

| Field | Offset | Type | Description |
|-------|--------|------|-------------|
| `parent_index` | 180 | i32 | Parent node index (-1 = root) |
| `first_child` | 220 | i32 | First child index (-1 = no children) |
| `prev_sibling` | 224 | i32 | Previous sibling (-1 = first child) |
| `next_sibling` | 228 | i32 | Next sibling (-1 = last child) |

### Operations

**Add child (prepend, O(1)):**
```
child.parent_index = parent
child.prev_sibling = -1
child.next_sibling = parent.first_child
if parent.first_child >= 0:
    old_first.prev_sibling = child
parent.first_child = child
```

**Remove child (O(1)):**
```
if child.prev_sibling >= 0:
    prev.next_sibling = child.next_sibling
else:
    parent.first_child = child.next_sibling

if child.next_sibling >= 0:
    next.prev_sibling = child.prev_sibling

child.prev_sibling = -1
child.next_sibling = -1
```

**Iterate children (O(children)):**
```
child = parent.first_child
while child >= 0:
    yield child
    child = child.next_sibling
```

### Why Order Doesn't Matter

Z-index determines render order, not insertion order. The renderer sorts children by z-index before drawing.

---

## Grid Item Placement

Grid items use `grid_column_start/end` and `grid_row_start/end` (in Line 4):

| Value | Meaning |
|-------|---------|
| 0 | Auto placement |
| 1-32 | Explicit line number |
| -1 to -32 | Span count (e.g., -2 = span 2) |

**Examples**:
- `grid_column_start=1, grid_column_end=3` → columns 1-2 (between lines 1-3)
- `grid_column_start=1, grid_column_end=-2` → start at line 1, span 2 columns
- `grid_row_start=0, grid_row_end=0` → auto placement

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
- `text_offset` (bytes 832-835): Start position in pool
- `text_length` (bytes 836-839): Length in bytes

---

## Event Ring Buffer

Events flow from Rust to TypeScript via a lock-free SPSC ring buffer.

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

### Dirty Flags (Node offset 34)

| Bit | Name | Description |
|-----|------|-------------|
| 0 | `DIRTY_LAYOUT` | Layout properties changed |
| 1 | `DIRTY_VISUAL` | Visual properties changed |
| 2 | `DIRTY_TEXT` | Text content changed |
| 3 | `DIRTY_HIERARCHY` | Parent/children changed |

### Interaction Flags (Node offset 924)

| Bit | Name | Description |
|-----|------|-------------|
| 0 | `FOCUSABLE` | Can receive focus |
| 1 | `FOCUSED` | Currently has focus |
| 2 | `HOVERED` | Mouse is over this component |
| 3 | `PRESSED` | Mouse button down on this |
| 4 | `DISABLED` | Interaction disabled |

### Text Attributes (Node offset 843)

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
| Layout (Flex) | Lines 1-4 (0-255) | 256 |
| Layout (Grid) | Lines 1-10 (0-639) | 640 |
| Render | Lines 11-14 (640-895) | 256 |
| Input | Line 15 (896-959) | 64 |
| Animation | Line 16 (960-1023) | 64 |

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
| 3.0 | 2026-01-31 | Full CSS Grid support: 1024-byte stride, 32 column + 32 row tracks |
| 2.0 | 2026-01-30 | Cache-aligned 512-byte stride, future-proof reserved space |
| 1.0 | 2025-xx-xx | Original organic growth version (256-byte stride) |

---

## Checksums

To verify both implementations match:

```
Header size: 256 bytes
Node stride: 1024 bytes
Grid column tracks offset: 256
Grid row tracks offset: 448
Output offset: 640
Colors offset: 768
Text offset in node: 832
Scroll offset in node: 896
Event slot size: 20 bytes
```

**If these don't match between TS and Rust, the implementations are out of sync.**

---

*This specification is the source of truth. TypeScript and Rust implementations MUST match exactly.*
