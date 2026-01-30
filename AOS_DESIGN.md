# AoS SharedBuffer Redesign

## Current Problem

SoA (Struct of Arrays) layout causes ~100x slowdown:
- Each property in separate TypedArray
- Reading one node's style = 40 scattered memory accesses
- Cache misses kill performance

## New Design: AoS (Array of Structs)

Each node's data is contiguous in memory.

### Node Layout (256 bytes per node, aligned)

```
Offset  Size  Field
──────────────────────────────────────────────────────────
LAYOUT FLOATS (96 bytes = 24 × f32)
0       4     width
4       4     height
8       4     min_width
12      4     min_height
16      4     max_width
20      4     max_height
24      4     flex_basis
28      4     flex_grow
32      4     flex_shrink
36      4     padding_top
40      4     padding_right
44      4     padding_bottom
48      4     padding_left
52      4     margin_top
56      4     margin_right
60      4     margin_bottom
64      4     margin_left
68      4     gap
72      4     row_gap
76      4     column_gap
80      4     inset_top
84      4     inset_right
88      4     inset_bottom
92      4     inset_left

LAYOUT ENUMS (16 bytes = 16 × u8)
96      1     flex_direction
97      1     flex_wrap
98      1     justify_content
99      1     align_items
100     1     align_content
101     1     align_self
102     1     position
103     1     overflow
104     1     display
105     1     border_top
106     1     border_right
107     1     border_bottom
108     1     border_left
109     1     component_type
110     1     visible
111     1     _reserved

VISUAL (40 bytes)
112     4     fg_color (ARGB u32)
116     4     bg_color (ARGB u32)
120     4     border_color (ARGB u32)
124     4     focus_ring_color (ARGB u32)
128     4     cursor_color (ARGB u32)
132     4     selection_color (ARGB u32)
136     1     opacity (u8, 0-255)
137     1     z_index (i8)
138     2     _reserved

INTERACTION (24 bytes)
140     4     scroll_x (i32)
144     4     scroll_y (i32)
148     4     tab_index (i32)
152     4     cursor_position (i32)
156     4     selection_start (i32)
160     4     selection_end (i32)

FLAGS (8 bytes)
164     1     dirty_flags (layout|visual|text|hierarchy)
165     1     interaction_flags (focusable|focused|hovered|pressed|disabled)
166     2     _reserved
168     4     parent_index (i32, -1 = root)

TEXT (16 bytes)
172     4     text_offset (u32 into text pool)
176     4     text_length (u32)
180     1     text_align
181     1     text_wrap
182     1     text_overflow
183     1     _reserved

OUTPUT (28 bytes) - Written by Rust after layout
184     4     computed_x (f32)
188     4     computed_y (f32)
192     4     computed_width (f32)
196     4     computed_height (f32)
200     4     scroll_width (f32)
204     4     scroll_height (f32)
208     4     content_width (f32)

PADDING (48 bytes to reach 256)
212-255       _padding

──────────────────────────────────────────────────────────
TOTAL: 256 bytes per node (power of 2, cache-aligned)
```

### Buffer Layout

```
┌─────────────────────────────────────────────────────────┐
│ Header (256 bytes)                                      │
│   0-3:   version (u32)                                  │
│   4-7:   node_count (u32)                               │
│   8-11:  max_nodes (u32)                                │
│   12-15: terminal_width (u32)                           │
│   16-19: terminal_height (u32)                          │
│   20-23: wake_flag (u32)                                │
│   24-27: generation (u32)                               │
│   28-31: text_pool_size (u32)                           │
│   32-35: text_pool_write_ptr (u32)                      │
│   36-255: reserved                                      │
├─────────────────────────────────────────────────────────┤
│ Node 0 (256 bytes)                                      │
├─────────────────────────────────────────────────────────┤
│ Node 1 (256 bytes)                                      │
├─────────────────────────────────────────────────────────┤
│ ...                                                     │
├─────────────────────────────────────────────────────────┤
│ Node N-1 (256 bytes)                                    │
├─────────────────────────────────────────────────────────┤
│ Text Pool (1MB)                                         │
└─────────────────────────────────────────────────────────┘

Total for 4096 nodes: 256 + (4096 × 256) + 1MB = ~2MB
```

### TypeScript API

```typescript
const STRIDE = 256
const HEADER_SIZE = 256

// Field offsets within a node
const F_WIDTH = 0
const F_HEIGHT = 4
const F_MIN_WIDTH = 8
// ... etc

// Create views
const buffer = new SharedArrayBuffer(HEADER_SIZE + MAX_NODES * STRIDE + TEXT_POOL_SIZE)
const view = new DataView(buffer)

// Write helpers
function setNodeF32(nodeIndex: number, field: number, value: number) {
  view.setFloat32(HEADER_SIZE + nodeIndex * STRIDE + field, value, true)
}

function setNodeU8(nodeIndex: number, field: number, value: number) {
  view.setUint8(HEADER_SIZE + nodeIndex * STRIDE + field, value)
}

function setNodeU32(nodeIndex: number, field: number, value: number) {
  view.setUint32(HEADER_SIZE + nodeIndex * STRIDE + field, value, true)
}

// Node writer object (can use Proxy for nice syntax)
function nodeWriter(index: number) {
  const base = HEADER_SIZE + index * STRIDE
  return {
    set width(v: number) { view.setFloat32(base + F_WIDTH, v, true) },
    set height(v: number) { view.setFloat32(base + F_HEIGHT, v, true) },
    set flexDirection(v: number) { view.setUint8(base + F_FLEX_DIR, v) },
    // ... etc
  }
}
```

### Rust API

```rust
const STRIDE: usize = 256;
const HEADER_SIZE: usize = 256;

// Field offsets
const F_WIDTH: usize = 0;
const F_HEIGHT: usize = 4;
// ... etc

impl SharedBuffer {
    fn node_slice(&self, node_index: usize) -> &[u8] {
        let start = HEADER_SIZE + node_index * STRIDE;
        &self.data[start..start + STRIDE]
    }

    fn read_style(&self, node_index: usize) -> Style {
        let node = self.node_slice(node_index);

        // ALL reads from contiguous memory - cache friendly!
        Style {
            size: Size {
                width: to_dim(f32::from_le_bytes(node[0..4].try_into().unwrap())),
                height: to_dim(f32::from_le_bytes(node[4..8].try_into().unwrap())),
            },
            // ... etc
        }
    }
}
```

### Expected Performance

With AoS:
- One cache fetch per node (~256 bytes = 4 cache lines)
- All subsequent field reads hit L1 cache
- Should match pure Rust performance (~30μs for 1000 nodes)

### Implementation Plan

1. Create new `shared-buffer-aos.ts` with new layout
2. Create new `shared_buffer_aos.rs` with new layout
3. Update primitives to use new write helpers
4. Update LayoutTree to read from new layout
5. Benchmark and compare
