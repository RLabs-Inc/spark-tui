/**
 * SharedArrayBuffer memory layout for SparkTUI — SoA (Structure of Arrays).
 *
 * This is the CONTRACT between TypeScript and Rust.
 * Both sides define the same offsets. If you change this, change rust/src/shared_buffer.rs too.
 *
 * Flow: TS writes → SharedArrayBuffer → Rust reads → Taffy → Renderer → Terminal
 *
 * SoA Layout (~2.0MB for 4096 nodes):
 *   Header          64 bytes       (16 × u32)
 *   Dirty Flags     4,096 bytes    (1 byte per node)
 *   Float32 Fields  507,904 bytes  (31 fields × 4096 × 4)
 *   Uint32 Fields   196,608 bytes  (12 fields × 4096 × 4)
 *   Int32 Fields    212,992 bytes  (13 fields × 4096 × 4)
 *   Uint8 Fields    114,688 bytes  (28 fields × 4096)
 *   Text Pool       1,048,576 bytes
 *
 * Each field is a contiguous TypedArray of MAX_NODES elements.
 * Access pattern: field_array[node_index] — no stride, no multiplication.
 */

// =============================================================================
// CONSTANTS (must match rust/src/shared_buffer.rs)
// =============================================================================

export const MAX_NODES = 4096

/** Header: 16 x u32 = 64 bytes */
export const HEADER_BYTES = 64
export const HEADER_U32_COUNT = 16

// Header field offsets (in u32 units)
export const HEADER_VERSION = 0
export const HEADER_NODE_COUNT = 1
export const HEADER_MAX_NODES = 2
export const HEADER_TERMINAL_WIDTH = 3
export const HEADER_TERMINAL_HEIGHT = 4
export const HEADER_WAKE_FLAG = 5
export const HEADER_GENERATION = 6
export const HEADER_TEXT_POOL_WRITE_PTR = 7
export const HEADER_TEXT_POOL_CAPACITY = 8
export const HEADER_RENDER_COUNT = 9
// 10-15 reserved

// =============================================================================
// SOA SECTION LAYOUT
// =============================================================================

/** Field counts per type */
export const F32_FIELD_COUNT = 31
export const U32_FIELD_COUNT = 12
export const I32_FIELD_COUNT = 13
export const U8_FIELD_COUNT = 28

/** Per-field array size in bytes */
const F32_FIELD_BYTES = MAX_NODES * 4
const U32_FIELD_BYTES = MAX_NODES * 4
const I32_FIELD_BYTES = MAX_NODES * 4
const U8_FIELD_BYTES = MAX_NODES

/** Text pool size in bytes (1MB) */
export const TEXT_POOL_SIZE = 1048576

// Section offsets (computed incrementally)
export const SECTION_DIRTY_OFFSET = HEADER_BYTES
export const SECTION_DIRTY_SIZE = MAX_NODES

export const SECTION_F32_OFFSET = SECTION_DIRTY_OFFSET + SECTION_DIRTY_SIZE
export const SECTION_F32_SIZE = F32_FIELD_COUNT * F32_FIELD_BYTES

export const SECTION_U32_OFFSET = SECTION_F32_OFFSET + SECTION_F32_SIZE
export const SECTION_U32_SIZE = U32_FIELD_COUNT * U32_FIELD_BYTES

export const SECTION_I32_OFFSET = SECTION_U32_OFFSET + SECTION_U32_SIZE
export const SECTION_I32_SIZE = I32_FIELD_COUNT * I32_FIELD_BYTES

export const SECTION_U8_OFFSET = SECTION_I32_OFFSET + SECTION_I32_SIZE
export const SECTION_U8_SIZE = U8_FIELD_COUNT * U8_FIELD_BYTES

export const SECTION_TEXT_POOL_OFFSET = SECTION_U8_OFFSET + SECTION_U8_SIZE
export const SECTION_TEXT_POOL_SIZE = TEXT_POOL_SIZE

export const TOTAL_BUFFER_SIZE = SECTION_TEXT_POOL_OFFSET + SECTION_TEXT_POOL_SIZE

// =============================================================================
// FLOAT32 FIELD INDICES (0-30)
// =============================================================================

// Layout input (0-23)
export const F32_WIDTH = 0
export const F32_HEIGHT = 1
export const F32_MIN_WIDTH = 2
export const F32_MAX_WIDTH = 3
export const F32_MIN_HEIGHT = 4
export const F32_MAX_HEIGHT = 5
export const F32_GROW = 6
export const F32_SHRINK = 7
export const F32_BASIS = 8
export const F32_GAP = 9
export const F32_PADDING_TOP = 10
export const F32_PADDING_RIGHT = 11
export const F32_PADDING_BOTTOM = 12
export const F32_PADDING_LEFT = 13
export const F32_MARGIN_TOP = 14
export const F32_MARGIN_RIGHT = 15
export const F32_MARGIN_BOTTOM = 16
export const F32_MARGIN_LEFT = 17
export const F32_INSET_TOP = 18
export const F32_INSET_RIGHT = 19
export const F32_INSET_BOTTOM = 20
export const F32_INSET_LEFT = 21
export const F32_ROW_GAP = 22
export const F32_COLUMN_GAP = 23

// Layout output (24-30) — Rust writes
export const F32_COMPUTED_X = 24
export const F32_COMPUTED_Y = 25
export const F32_COMPUTED_WIDTH = 26
export const F32_COMPUTED_HEIGHT = 27
export const F32_SCROLLABLE = 28
export const F32_MAX_SCROLL_X = 29
export const F32_MAX_SCROLL_Y = 30

// =============================================================================
// UINT32 FIELD INDICES (0-11)
// =============================================================================

// Colors (0-9) — packed ARGB
export const U32_FG_COLOR = 0
export const U32_BG_COLOR = 1
export const U32_BORDER_COLOR = 2
export const U32_BORDER_COLOR_TOP = 3
export const U32_BORDER_COLOR_RIGHT = 4
export const U32_BORDER_COLOR_BOTTOM = 5
export const U32_BORDER_COLOR_LEFT = 6
export const U32_FOCUS_RING_COLOR = 7
export const U32_CURSOR_FG_COLOR = 8
export const U32_CURSOR_BG_COLOR = 9

// Text index (10-11)
export const U32_TEXT_OFFSET = 10
export const U32_TEXT_LENGTH = 11

// =============================================================================
// INT32 FIELD INDICES (0-12)
// =============================================================================

// Hierarchy
export const I32_PARENT_INDEX = 0

// Interaction (1-12)
export const I32_SCROLL_X = 1
export const I32_SCROLL_Y = 2
export const I32_TAB_INDEX = 3
export const I32_CURSOR_POS = 4
export const I32_SELECTION_START = 5
export const I32_SELECTION_END = 6
export const I32_CURSOR_CHAR = 7
export const I32_CURSOR_ALT_CHAR = 8
export const I32_CURSOR_BLINK_FPS = 9
export const I32_HOVERED = 10
export const I32_PRESSED = 11
export const I32_CURSOR_VISIBLE = 12

// =============================================================================
// UINT8 FIELD INDICES (0-27)
// =============================================================================

// Layout enums (0-13)
export const U8_COMPONENT_TYPE = 0
export const U8_VISIBLE = 1
export const U8_FLEX_DIRECTION = 2
export const U8_FLEX_WRAP = 3
export const U8_JUSTIFY_CONTENT = 4
export const U8_ALIGN_ITEMS = 5
export const U8_ALIGN_SELF = 6
export const U8_ALIGN_CONTENT = 7
export const U8_OVERFLOW = 8
export const U8_POSITION = 9
export const U8_BORDER_TOP_WIDTH = 10
export const U8_BORDER_RIGHT_WIDTH = 11
export const U8_BORDER_BOTTOM_WIDTH = 12
export const U8_BORDER_LEFT_WIDTH = 13

// Visual (14-21)
export const U8_BORDER_STYLE = 14
export const U8_BORDER_STYLE_TOP = 15
export const U8_BORDER_STYLE_RIGHT = 16
export const U8_BORDER_STYLE_BOTTOM = 17
export const U8_BORDER_STYLE_LEFT = 18
export const U8_SHOW_FOCUS_RING = 19
export const U8_OPACITY = 20
export const U8_Z_INDEX = 21

// Text (22-25)
export const U8_TEXT_ATTRS = 22
export const U8_TEXT_ALIGN = 23
export const U8_TEXT_WRAP = 24
export const U8_ELLIPSIS_MODE = 25

// Interaction (26-27)
export const U8_FOCUSABLE = 26
export const U8_MOUSE_ENABLED = 27

// =============================================================================
// DIRTY FLAG BITS (per-node dirty byte in the dirty flags section)
// =============================================================================

export const DIRTY_LAYOUT = 1 << 0
export const DIRTY_VISUAL = 1 << 1
export const DIRTY_TEXT = 1 << 2
export const DIRTY_INTERACTION = 1 << 3
export const DIRTY_HIERARCHY = 1 << 4

// =============================================================================
// COMPONENT TYPE CONSTANTS
// =============================================================================

export const COMPONENT_NONE = 0
export const COMPONENT_BOX = 1
export const COMPONENT_TEXT = 2
export const COMPONENT_INPUT = 3

// =============================================================================
// BACKWARD-COMPAT ALIASES — old constant names → new SoA names
// =============================================================================

// Float field aliases (old FLOAT_ prefix → new F32_ prefix)
export const FLOAT_WIDTH = F32_WIDTH
export const FLOAT_HEIGHT = F32_HEIGHT
export const FLOAT_MIN_WIDTH = F32_MIN_WIDTH
export const FLOAT_MAX_WIDTH = F32_MAX_WIDTH
export const FLOAT_MIN_HEIGHT = F32_MIN_HEIGHT
export const FLOAT_MAX_HEIGHT = F32_MAX_HEIGHT
export const FLOAT_GROW = F32_GROW
export const FLOAT_SHRINK = F32_SHRINK
export const FLOAT_BASIS = F32_BASIS
export const FLOAT_GAP = F32_GAP
export const FLOAT_PADDING_TOP = F32_PADDING_TOP
export const FLOAT_PADDING_RIGHT = F32_PADDING_RIGHT
export const FLOAT_PADDING_BOTTOM = F32_PADDING_BOTTOM
export const FLOAT_PADDING_LEFT = F32_PADDING_LEFT
export const FLOAT_MARGIN_TOP = F32_MARGIN_TOP
export const FLOAT_MARGIN_RIGHT = F32_MARGIN_RIGHT
export const FLOAT_MARGIN_BOTTOM = F32_MARGIN_BOTTOM
export const FLOAT_MARGIN_LEFT = F32_MARGIN_LEFT
export const FLOAT_TOP = F32_INSET_TOP
export const FLOAT_RIGHT = F32_INSET_RIGHT
export const FLOAT_BOTTOM = F32_INSET_BOTTOM
export const FLOAT_LEFT = F32_INSET_LEFT
export const FLOAT_ROW_GAP = F32_ROW_GAP
export const FLOAT_COLUMN_GAP = F32_COLUMN_GAP

// Output field aliases
export const OUTPUT_X = F32_COMPUTED_X
export const OUTPUT_Y = F32_COMPUTED_Y
export const OUTPUT_WIDTH = F32_COMPUTED_WIDTH
export const OUTPUT_HEIGHT = F32_COMPUTED_HEIGHT
export const OUTPUT_SCROLLABLE = F32_SCROLLABLE
export const OUTPUT_MAX_SCROLL_X = F32_MAX_SCROLL_X
export const OUTPUT_MAX_SCROLL_Y = F32_MAX_SCROLL_Y

// Color field aliases (old COLOR_ prefix → new U32_ prefix)
export const COLOR_FG = U32_FG_COLOR
export const COLOR_BG = U32_BG_COLOR
export const COLOR_BORDER = U32_BORDER_COLOR
export const COLOR_BORDER_TOP = U32_BORDER_COLOR_TOP
export const COLOR_BORDER_RIGHT = U32_BORDER_COLOR_RIGHT
export const COLOR_BORDER_BOTTOM = U32_BORDER_COLOR_BOTTOM
export const COLOR_BORDER_LEFT = U32_BORDER_COLOR_LEFT
export const COLOR_FOCUS_RING = U32_FOCUS_RING_COLOR
export const COLOR_CURSOR_FG = U32_CURSOR_FG_COLOR
export const COLOR_CURSOR_BG = U32_CURSOR_BG_COLOR

// Text index aliases
export const TEXT_OFFSET = U32_TEXT_OFFSET
export const TEXT_LENGTH = U32_TEXT_LENGTH

// Interaction field aliases (old INTERACT_ prefix → new I32_ prefix)
export const INTERACT_SCROLL_X = I32_SCROLL_X
export const INTERACT_SCROLL_Y = I32_SCROLL_Y
export const INTERACT_TAB_INDEX = I32_TAB_INDEX
export const INTERACT_CURSOR_POS = I32_CURSOR_POS
export const INTERACT_SELECTION_START = I32_SELECTION_START
export const INTERACT_SELECTION_END = I32_SELECTION_END
export const INTERACT_CURSOR_CHAR = I32_CURSOR_CHAR
export const INTERACT_CURSOR_ALT_CHAR = I32_CURSOR_ALT_CHAR
export const INTERACT_CURSOR_BLINK_FPS = I32_CURSOR_BLINK_FPS
export const INTERACT_HOVERED = I32_HOVERED
export const INTERACT_PRESSED = I32_PRESSED
export const INTERACT_CURSOR_VISIBLE = I32_CURSOR_VISIBLE

// Metadata field aliases (old META_ prefix → new U8_ prefix)
export const META_COMPONENT_TYPE = U8_COMPONENT_TYPE
export const META_VISIBLE = U8_VISIBLE
export const META_FLEX_DIRECTION = U8_FLEX_DIRECTION
export const META_FLEX_WRAP = U8_FLEX_WRAP
export const META_JUSTIFY_CONTENT = U8_JUSTIFY_CONTENT
export const META_ALIGN_ITEMS = U8_ALIGN_ITEMS
export const META_ALIGN_SELF = U8_ALIGN_SELF
export const META_ALIGN_CONTENT = U8_ALIGN_CONTENT
export const META_OVERFLOW = U8_OVERFLOW
export const META_POSITION = U8_POSITION
export const META_BORDER_TOP_WIDTH = U8_BORDER_TOP_WIDTH
export const META_BORDER_RIGHT_WIDTH = U8_BORDER_RIGHT_WIDTH
export const META_BORDER_BOTTOM_WIDTH = U8_BORDER_BOTTOM_WIDTH
export const META_BORDER_LEFT_WIDTH = U8_BORDER_LEFT_WIDTH
export const META_BORDER_STYLE = U8_BORDER_STYLE
export const META_BORDER_STYLE_TOP = U8_BORDER_STYLE_TOP
export const META_BORDER_STYLE_RIGHT = U8_BORDER_STYLE_RIGHT
export const META_BORDER_STYLE_BOTTOM = U8_BORDER_STYLE_BOTTOM
export const META_BORDER_STYLE_LEFT = U8_BORDER_STYLE_LEFT
export const META_SHOW_FOCUS_RING = U8_SHOW_FOCUS_RING
export const META_OPACITY = U8_OPACITY
export const META_Z_INDEX = U8_Z_INDEX
export const META_TEXT_ATTRS = U8_TEXT_ATTRS
export const META_TEXT_ALIGN = U8_TEXT_ALIGN
export const META_TEXT_WRAP = U8_TEXT_WRAP
export const META_ELLIPSIS_MODE = U8_ELLIPSIS_MODE
export const META_FOCUSABLE = U8_FOCUSABLE
export const META_MOUSE_ENABLED = U8_MOUSE_ENABLED

/** @deprecated Use META_BORDER_TOP_WIDTH */
export const META_BORDER_TOP = U8_BORDER_TOP_WIDTH
/** @deprecated Use META_BORDER_RIGHT_WIDTH */
export const META_BORDER_RIGHT = U8_BORDER_RIGHT_WIDTH
/** @deprecated Use META_BORDER_BOTTOM_WIDTH */
export const META_BORDER_BOTTOM = U8_BORDER_BOTTOM_WIDTH
/** @deprecated Use META_BORDER_LEFT_WIDTH */
export const META_BORDER_LEFT = U8_BORDER_LEFT_WIDTH

// =============================================================================
// COLOR PACKING
// =============================================================================

/**
 * Pack RGBA into a single u32 in ARGB format.
 * 0x00000000 = null/inherit (no color set).
 */
export function packColor(r: number, g: number, b: number, a: number = 255): number {
  return (((a & 0xFF) << 24) | ((r & 0xFF) << 16) | ((g & 0xFF) << 8) | (b & 0xFF)) >>> 0
}

/**
 * Unpack a u32 ARGB color into components.
 * Returns { r, g, b, a }. Returns null components if value is 0 (inherit).
 */
export function unpackColor(packed: number): { r: number, g: number, b: number, a: number } {
  return {
    a: (packed >>> 24) & 0xFF,
    r: (packed >>> 16) & 0xFF,
    g: (packed >>> 8) & 0xFF,
    b: packed & 0xFF,
  }
}

// =============================================================================
// SHARED BUFFER VIEWS — SoA per-field TypedArray views
// =============================================================================

export interface SharedBufferViews {
  /** The raw SharedArrayBuffer */
  buffer: SharedArrayBuffer
  /** Header as u32 array (16 elements) */
  header: Uint32Array
  /** Int32 view of header (for Atomics.wait/notify on wake_flag) */
  headerI32: Int32Array
  /** Per-node dirty flags (MAX_NODES bytes) */
  dirty: Uint8Array
  /** Float32 per-field views (31 views, indexed by F32_* constants) */
  f32: Float32Array[]
  /** Uint32 per-field views (12 views, indexed by U32_* constants) */
  u32: Uint32Array[]
  /** Int32 per-field views (13 views, indexed by I32_* constants) */
  i32: Int32Array[]
  /** Uint8 per-field views (28 views, indexed by U8_* constants) */
  u8: Uint8Array[]
  /** Text pool: raw UTF-8 bytes, bump-allocated */
  textPool: Uint8Array
}

/**
 * Create the SharedArrayBuffer and SoA typed views.
 *
 * Call once at startup. Pass buffer to Rust via FFI.
 */
export function createSharedBuffer(): SharedBufferViews {
  const buffer = new SharedArrayBuffer(TOTAL_BUFFER_SIZE)

  const header = new Uint32Array(buffer, 0, HEADER_U32_COUNT)
  const headerI32 = new Int32Array(buffer, 0, HEADER_U32_COUNT)
  const dirty = new Uint8Array(buffer, SECTION_DIRTY_OFFSET, MAX_NODES)

  // Create per-field Float32Array views
  const f32: Float32Array[] = new Array(F32_FIELD_COUNT)
  for (let i = 0; i < F32_FIELD_COUNT; i++) {
    f32[i] = new Float32Array(buffer, SECTION_F32_OFFSET + i * F32_FIELD_BYTES, MAX_NODES)
  }

  // Create per-field Uint32Array views
  const u32: Uint32Array[] = new Array(U32_FIELD_COUNT)
  for (let i = 0; i < U32_FIELD_COUNT; i++) {
    u32[i] = new Uint32Array(buffer, SECTION_U32_OFFSET + i * U32_FIELD_BYTES, MAX_NODES)
  }

  // Create per-field Int32Array views
  const i32: Int32Array[] = new Array(I32_FIELD_COUNT)
  for (let i = 0; i < I32_FIELD_COUNT; i++) {
    i32[i] = new Int32Array(buffer, SECTION_I32_OFFSET + i * I32_FIELD_BYTES, MAX_NODES)
  }

  // Create per-field Uint8Array views
  const u8: Uint8Array[] = new Array(U8_FIELD_COUNT)
  for (let i = 0; i < U8_FIELD_COUNT; i++) {
    u8[i] = new Uint8Array(buffer, SECTION_U8_OFFSET + i * U8_FIELD_BYTES, MAX_NODES)
  }

  const textPool = new Uint8Array(buffer, SECTION_TEXT_POOL_OFFSET, TEXT_POOL_SIZE)

  // Initialize header
  header[HEADER_VERSION] = 3  // v3 = SoA layout
  header[HEADER_NODE_COUNT] = 0
  header[HEADER_MAX_NODES] = MAX_NODES
  header[HEADER_TEXT_POOL_WRITE_PTR] = 0
  header[HEADER_TEXT_POOL_CAPACITY] = TEXT_POOL_SIZE

  // Initialize float defaults
  // NaN = auto for dimensions, 0 for spacing, 1 for shrink
  f32[F32_WIDTH].fill(NaN)
  f32[F32_HEIGHT].fill(NaN)
  f32[F32_MIN_WIDTH].fill(NaN)
  f32[F32_MAX_WIDTH].fill(NaN)
  f32[F32_MIN_HEIGHT].fill(NaN)
  f32[F32_MAX_HEIGHT].fill(NaN)
  f32[F32_GROW].fill(0)
  f32[F32_SHRINK].fill(1)
  f32[F32_BASIS].fill(NaN)
  // gap, padding, margin default to 0 (Float32Array zero-initialized)
  f32[F32_INSET_TOP].fill(NaN)
  f32[F32_INSET_RIGHT].fill(NaN)
  f32[F32_INSET_BOTTOM].fill(NaN)
  f32[F32_INSET_LEFT].fill(NaN)

  // Initialize u8 defaults
  u8[U8_VISIBLE].fill(1)           // visible by default
  u8[U8_FLEX_DIRECTION].fill(1)    // column (TUI default)
  u8[U8_OPACITY].fill(255)         // fully opaque
  u8[U8_Z_INDEX].fill(128)         // neutral z-index
  u8[U8_TEXT_WRAP].fill(1)         // wrap by default

  // Initialize i32 defaults
  i32[I32_PARENT_INDEX].fill(-1)         // no parent
  i32[I32_TAB_INDEX].fill(-1)            // not in tab order
  i32[I32_SELECTION_START].fill(-1)      // no selection
  i32[I32_SELECTION_END].fill(-1)        // no selection
  i32[I32_CURSOR_VISIBLE].fill(1)        // cursor visible by default

  return { buffer, header, headerI32, dirty, f32, u32, i32, u8, textPool }
}

// =============================================================================
// NODE WRITERS — SoA-based (direct per-field array access)
// =============================================================================

export function setNodeFloat(views: SharedBufferViews, index: number, field: number, value: number): void {
  views.f32[field][index] = value
}

export function getNodeFloat(views: SharedBufferViews, index: number, field: number): number {
  return views.f32[field][index]
}

export function setNodeColor(views: SharedBufferViews, index: number, field: number, value: number): void {
  views.u32[field][index] = value
}

export function getNodeColor(views: SharedBufferViews, index: number, field: number): number {
  return views.u32[field][index]
}

export function setNodeInteraction(views: SharedBufferViews, index: number, field: number, value: number): void {
  views.i32[field][index] = value
}

export function getNodeInteraction(views: SharedBufferViews, index: number, field: number): number {
  return views.i32[field][index]
}

export function setNodeMeta(views: SharedBufferViews, index: number, field: number, value: number): void {
  views.u8[field][index] = value
}

export function getNodeMeta(views: SharedBufferViews, index: number, field: number): number {
  return views.u8[field][index]
}

export function setNodeParent(views: SharedBufferViews, index: number, parent: number): void {
  views.i32[I32_PARENT_INDEX][index] = parent
}

export function setTerminalSize(views: SharedBufferViews, width: number, height: number): void {
  views.header[HEADER_TERMINAL_WIDTH] = width
  views.header[HEADER_TERMINAL_HEIGHT] = height
}

export function setNodeCount(views: SharedBufferViews, count: number): void {
  views.header[HEADER_NODE_COUNT] = count
}

/**
 * Set dirty flags for a node. OR's the new flags with existing.
 */
export function markDirty(views: SharedBufferViews, index: number, flags: number): void {
  views.dirty[index] |= flags
}

/**
 * Notify Rust that data has changed.
 * Sets wake flag and uses Atomics.notify.
 */
export function notifyRust(views: SharedBufferViews): void {
  Atomics.store(views.headerI32, HEADER_WAKE_FLAG, 1)
  Atomics.notify(views.headerI32, HEADER_WAKE_FLAG)
}

// =============================================================================
// TEXT POOL — Bump-allocated UTF-8 string storage
// =============================================================================

const textEncoder = new TextEncoder()

/**
 * Write a text string for a node into the text pool.
 */
export function setNodeText(views: SharedBufferViews, index: number, text: string): void {
  const encoded = textEncoder.encode(text)
  const writePtr = views.header[HEADER_TEXT_POOL_WRITE_PTR]
  const capacity = views.header[HEADER_TEXT_POOL_CAPACITY]

  if (writePtr + encoded.length > capacity) {
    compactTextPool(views)
    const newPtr = views.header[HEADER_TEXT_POOL_WRITE_PTR]
    if (newPtr + encoded.length > capacity) {
      const available = capacity - newPtr
      if (available <= 0) return
      views.textPool.set(encoded.subarray(0, available), newPtr)
      views.u32[U32_TEXT_OFFSET][index] = newPtr
      views.u32[U32_TEXT_LENGTH][index] = available
      views.header[HEADER_TEXT_POOL_WRITE_PTR] = newPtr + available
      return
    }
    views.textPool.set(encoded, newPtr)
    views.u32[U32_TEXT_OFFSET][index] = newPtr
    views.u32[U32_TEXT_LENGTH][index] = encoded.length
    views.header[HEADER_TEXT_POOL_WRITE_PTR] = newPtr + encoded.length
    return
  }

  views.textPool.set(encoded, writePtr)
  views.u32[U32_TEXT_OFFSET][index] = writePtr
  views.u32[U32_TEXT_LENGTH][index] = encoded.length
  views.header[HEADER_TEXT_POOL_WRITE_PTR] = writePtr + encoded.length
}

const textDecoder = new TextDecoder()

/**
 * Read text content for a node from the text pool.
 */
export function getNodeText(views: SharedBufferViews, index: number): string {
  const offset = views.u32[U32_TEXT_OFFSET][index]
  const length = views.u32[U32_TEXT_LENGTH][index]
  if (length === 0) return ''
  return textDecoder.decode(views.textPool.subarray(offset, offset + length))
}

/**
 * Compact the text pool by repacking live node text to the front.
 */
export function compactTextPool(views: SharedBufferViews): void {
  const nodeCount = views.header[HEADER_NODE_COUNT]
  let writePtr = 0

  for (let i = 0; i < nodeCount; i++) {
    const offset = views.u32[U32_TEXT_OFFSET][i]
    const length = views.u32[U32_TEXT_LENGTH][i]

    if (length === 0) continue

    if (offset !== writePtr) {
      views.textPool.copyWithin(writePtr, offset, offset + length)
    }
    views.u32[U32_TEXT_OFFSET][i] = writePtr
    writePtr += length
  }

  views.header[HEADER_TEXT_POOL_WRITE_PTR] = writePtr
}

// =============================================================================
// OUTPUT READERS — Read computed output from Rust
// =============================================================================

/**
 * Read computed output for a node (from Rust).
 */
export function getNodeOutput(views: SharedBufferViews, index: number): { x: number, y: number, w: number, h: number } {
  return {
    x: views.f32[F32_COMPUTED_X][index],
    y: views.f32[F32_COMPUTED_Y][index],
    w: views.f32[F32_COMPUTED_WIDTH][index],
    h: views.f32[F32_COMPUTED_HEIGHT][index],
  }
}

/**
 * Read full computed output including scroll info.
 */
export function getNodeOutputFull(views: SharedBufferViews, index: number): {
  x: number, y: number, w: number, h: number,
  scrollable: boolean, maxScrollX: number, maxScrollY: number
} {
  return {
    x: views.f32[F32_COMPUTED_X][index],
    y: views.f32[F32_COMPUTED_Y][index],
    w: views.f32[F32_COMPUTED_WIDTH][index],
    h: views.f32[F32_COMPUTED_HEIGHT][index],
    scrollable: views.f32[F32_SCROLLABLE][index] !== 0,
    maxScrollX: views.f32[F32_MAX_SCROLL_X][index],
    maxScrollY: views.f32[F32_MAX_SCROLL_Y][index],
  }
}
