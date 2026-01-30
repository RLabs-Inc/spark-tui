/**
 * AoS (Array of Structs) SharedBuffer
 *
 * Each node's data is contiguous in memory for cache-friendly Rust reads.
 * TS writes individual fields, Rust reads entire nodes.
 */

// =============================================================================
// CONSTANTS
// =============================================================================

export const STRIDE = 256 // bytes per node
export const HEADER_SIZE = 256 // bytes for header
export const MAX_NODES = 100_000 // 100K nodes - plenty for any TUI

// Text pool - configurable, default 10MB
// ~100 bytes per node at max capacity, plenty for most apps
export const DEFAULT_TEXT_POOL_SIZE = 10 * 1024 * 1024 // 10MB default
export const TEXT_POOL_SIZE = DEFAULT_TEXT_POOL_SIZE // Legacy alias

// Dimension value for "auto" (maps to Taffy's Dimension::Auto)
export const AUTO = NaN

// Event ring buffer constants
export const EVENT_RING_HEADER_SIZE = 12 // write_idx(4) + read_idx(4) + reserved(4)
export const EVENT_SLOT_SIZE = 20 // type(1) + pad(1) + component(2) + data(16)
export const MAX_EVENTS = 256
export const EVENT_RING_SIZE = EVENT_RING_HEADER_SIZE + MAX_EVENTS * EVENT_SLOT_SIZE // 5132 bytes

// Helper to calculate buffer metrics for a given text pool size
export function getBufferMetrics(textPoolSize: number = DEFAULT_TEXT_POOL_SIZE) {
  const eventRingOffset = HEADER_SIZE + MAX_NODES * STRIDE + textPoolSize
  const totalSize = eventRingOffset + EVENT_RING_SIZE
  return { textPoolSize, eventRingOffset, totalSize }
}

// Default buffer metrics (for legacy exports)
const defaultMetrics = getBufferMetrics()
export const EVENT_RING_OFFSET = defaultMetrics.eventRingOffset
export const BUFFER_SIZE = defaultMetrics.totalSize // ~35MB with 10MB text pool

// =============================================================================
// HEADER OFFSETS (within header section, 256 bytes total)
// =============================================================================

// Core (0-35) - MUST match Rust shared_buffer_aos.rs!
export const H_VERSION = 0
export const H_NODE_COUNT = 4
export const H_MAX_NODES = 8
export const H_TERMINAL_WIDTH = 12
export const H_TERMINAL_HEIGHT = 16
export const H_WAKE_FLAG_LEGACY = 20 // Legacy, kept for Rust compatibility
export const H_GENERATION = 24
export const H_TEXT_POOL_SIZE = 28
export const H_TEXT_POOL_WRITE_PTR = 32

// Event indices (36-43) - Rust writes event_write, TS writes event_read
export const H_EVENT_WRITE_IDX = 36
export const H_EVENT_READ_IDX = 40

// State (44-59) - Rust writes
export const H_FOCUSED_INDEX = 44
export const H_HOVERED_INDEX = 48
export const H_PRESSED_INDEX = 52
export const H_MOUSE_X = 56 // u16
export const H_MOUSE_Y = 58 // u16

// Reserved (60-63)

// Wake flags (64-71) - must be 4-byte aligned for Atomics
export const H_WAKE_RUST = 64 // TS sets to wake Rust
export const H_WAKE_TS = 68 // Rust sets to wake TS

// Config flags (72-95) - TS writes, Rust reads
export const H_CONFIG_FLAGS = 72
export const H_RENDER_MODE = 76 // 0=full, 1=inline, 2=append
export const H_CURSOR_CONFIG = 80 // packed: vis|shape|blink
export const H_SCROLL_SPEED = 84 // lines per wheel tick
export const H_RENDER_COUNT = 88 // u32, Rust writes for FPS tracking
export const H_EXIT_REQUESTED = 92 // u8, Rust sets on Exit event

// Reserved (96-255)

// Legacy alias for backward compatibility
export const H_WAKE_FLAG = H_WAKE_RUST

// =============================================================================
// CONFIG FLAGS (bitfield)
// =============================================================================

export const CONFIG_EXIT_ON_CTRL_C = 1 << 0
export const CONFIG_TAB_NAVIGATION = 1 << 1
export const CONFIG_ARROW_SCROLL = 1 << 2
export const CONFIG_PAGE_SCROLL = 1 << 3
export const CONFIG_HOME_END_SCROLL = 1 << 4
export const CONFIG_WHEEL_SCROLL = 1 << 5
export const CONFIG_FOCUS_ON_CLICK = 1 << 6
export const CONFIG_MOUSE_ENABLED = 1 << 7
export const CONFIG_KITTY_KEYBOARD = 1 << 8

// Default config: all enabled except Kitty keyboard (bits 0-7 = 0xFF)
export const CONFIG_DEFAULT = 0xff

// =============================================================================
// EVENT TYPES
// =============================================================================

export enum EventType {
  None = 0,
  Key = 1,
  MouseDown = 2,
  MouseUp = 3,
  Click = 4,
  MouseEnter = 5,
  MouseLeave = 6,
  MouseMove = 7,
  Scroll = 8,
  Focus = 9,
  Blur = 10,
  ValueChange = 11,
  Submit = 12,
  Cancel = 13,
  Exit = 14,
  Resize = 15,
}

// =============================================================================
// NODE FIELD OFFSETS (within each 256-byte node)
// =============================================================================

// Layout floats (0-95, 24 × f32)
export const F_WIDTH = 0
export const F_HEIGHT = 4
export const F_MIN_WIDTH = 8
export const F_MIN_HEIGHT = 12
export const F_MAX_WIDTH = 16
export const F_MAX_HEIGHT = 20
export const F_FLEX_BASIS = 24
export const F_FLEX_GROW = 28
export const F_FLEX_SHRINK = 32
export const F_PADDING_TOP = 36
export const F_PADDING_RIGHT = 40
export const F_PADDING_BOTTOM = 44
export const F_PADDING_LEFT = 48
export const F_MARGIN_TOP = 52
export const F_MARGIN_RIGHT = 56
export const F_MARGIN_BOTTOM = 60
export const F_MARGIN_LEFT = 64
export const F_GAP = 68
export const F_ROW_GAP = 72
export const F_COLUMN_GAP = 76
export const F_INSET_TOP = 80
export const F_INSET_RIGHT = 84
export const F_INSET_BOTTOM = 88
export const F_INSET_LEFT = 92

// Layout enums (96-111, 16 × u8)
export const U_FLEX_DIRECTION = 96
export const U_FLEX_WRAP = 97
export const U_JUSTIFY_CONTENT = 98
export const U_ALIGN_ITEMS = 99
export const U_ALIGN_CONTENT = 100
export const U_ALIGN_SELF = 101
export const U_POSITION = 102
export const U_OVERFLOW = 103
export const U_DISPLAY = 104
export const U_BORDER_TOP = 105
export const U_BORDER_RIGHT = 106
export const U_BORDER_BOTTOM = 107
export const U_BORDER_LEFT = 108
export const U_COMPONENT_TYPE = 109
export const U_VISIBLE = 110
// 111 reserved

// Visual (112-147) - Colors + style
export const C_FG_COLOR = 112
export const C_BG_COLOR = 116
export const C_BORDER_COLOR = 120
export const C_FOCUS_RING_COLOR = 124
export const C_CURSOR_FG = 128
export const C_CURSOR_BG = 132
export const C_SELECTION_COLOR = 136
export const U_OPACITY = 140
export const I_Z_INDEX = 141
export const U_BORDER_STYLE = 142
export const U_BORDER_STYLE_TOP = 143
export const U_BORDER_STYLE_RIGHT = 144
export const U_BORDER_STYLE_BOTTOM = 145
export const U_BORDER_STYLE_LEFT = 146
export const U_FOCUS_INDICATOR_CHAR = 147 // u8, default '*' (0x2A)

// Interaction (148-171) - Focus, cursor, scroll
export const I_SCROLL_X = 148
export const I_SCROLL_Y = 152
export const I_TAB_INDEX = 156
export const I_CURSOR_POSITION = 160
export const I_SELECTION_START = 164
export const I_SELECTION_END = 168

// Flags (172-179)
export const U_DIRTY_FLAGS = 172
export const U_INTERACTION_FLAGS = 173
export const U_CURSOR_FLAGS = 174
export const U_CURSOR_STYLE = 175 // 0=block, 1=bar, 2=underline
export const U_CURSOR_FPS = 176
export const U_MAX_LENGTH = 177
export const U_FOCUS_INDICATOR_ENABLED = 178 // u8, 1=enabled, 0xFF=disabled
// 179 reserved

// Hierarchy (180-183)
export const I_PARENT_INDEX = 180

// Text (184-195)
export const U_TEXT_OFFSET = 184
export const U_TEXT_LENGTH = 188
export const U_TEXT_ALIGN = 192
export const U_TEXT_WRAP = 193
export const U_TEXT_OVERFLOW = 194
export const U_TEXT_ATTRS = 195

// Cursor character (196-199)
export const U_CURSOR_CHAR = 196 // u32 - custom cursor character (UTF-32)

// Output - written by Rust (200-232)
export const F_COMPUTED_X = 200
export const F_COMPUTED_Y = 204
export const F_COMPUTED_WIDTH = 208
export const F_COMPUTED_HEIGHT = 212
export const F_SCROLL_WIDTH = 216
export const F_SCROLL_HEIGHT = 220
export const F_MAX_SCROLL_X = 224
export const F_MAX_SCROLL_Y = 228
export const U_SCROLLABLE = 232
export const U_CURSOR_ALT_CHAR = 233 // u32 - alternate cursor char for blink (UTF-32)

// Per-side border colors (236-251)
export const C_BORDER_TOP_COLOR = 236
export const C_BORDER_RIGHT_COLOR = 240
export const C_BORDER_BOTTOM_COLOR = 244
export const C_BORDER_LEFT_COLOR = 248
// 252-255 reserved

// Legacy aliases for backward compatibility
export const C_CURSOR_COLOR = C_CURSOR_FG
export const F_CONTENT_WIDTH = F_SCROLL_WIDTH

// =============================================================================
// DIRTY FLAGS
// =============================================================================

export const DIRTY_LAYOUT = 0x01
export const DIRTY_VISUAL = 0x02
export const DIRTY_TEXT = 0x04
export const DIRTY_HIERARCHY = 0x08

// =============================================================================
// INTERACTION FLAGS
// =============================================================================

export const FLAG_FOCUSABLE = 0x01
export const FLAG_FOCUSED = 0x02
export const FLAG_HOVERED = 0x04
export const FLAG_PRESSED = 0x08
export const FLAG_DISABLED = 0x10

// =============================================================================
// COMPONENT TYPES
// =============================================================================

export const COMPONENT_NONE = 0
export const COMPONENT_BOX = 1
export const COMPONENT_TEXT = 2
export const COMPONENT_INPUT = 3

// =============================================================================
// BUFFER VIEWS
// =============================================================================

export interface AoSBuffer {
  buffer: SharedArrayBuffer
  view: DataView
  header: Uint32Array
  textPool: Uint8Array
  eventRing: Uint8Array
  /** The text pool size for this buffer (for error messages) */
  textPoolSize: number
}

export interface CreateBufferOptions {
  /**
   * Size of the text pool in bytes.
   * Default: 10MB (10 * 1024 * 1024)
   *
   * The text pool stores all text content (labels, input values, etc.)
   * For reference: 10MB = ~100 bytes per node at 100K nodes capacity.
   *
   * Increase if your app has lots of text content.
   * Decrease for memory-constrained environments.
   */
  textPoolSize?: number
}

export function createAoSBuffer(options: CreateBufferOptions = {}): AoSBuffer {
  const textPoolSize = options.textPoolSize ?? DEFAULT_TEXT_POOL_SIZE
  const { eventRingOffset, totalSize } = getBufferMetrics(textPoolSize)

  const buffer = new SharedArrayBuffer(totalSize)
  const view = new DataView(buffer)
  const header = new Uint32Array(buffer, 0, HEADER_SIZE / 4)
  const textPool = new Uint8Array(
    buffer,
    HEADER_SIZE + MAX_NODES * STRIDE,
    textPoolSize
  )
  const eventRing = new Uint8Array(buffer, eventRingOffset, EVENT_RING_SIZE)

  // Initialize header - Core section
  header[H_VERSION / 4] = 1
  header[H_NODE_COUNT / 4] = 0
  header[H_MAX_NODES / 4] = MAX_NODES
  header[H_TEXT_POOL_SIZE / 4] = textPoolSize
  header[H_TEXT_POOL_WRITE_PTR / 4] = 0

  // Initialize event indices to 0
  view.setUint32(H_EVENT_WRITE_IDX, 0, true)
  view.setUint32(H_EVENT_READ_IDX, 0, true)

  // Initialize state indices to -1 (none)
  view.setInt32(H_FOCUSED_INDEX, -1, true)
  view.setInt32(H_HOVERED_INDEX, -1, true)
  view.setInt32(H_PRESSED_INDEX, -1, true)
  view.setUint16(H_MOUSE_X, 0, true)
  view.setUint16(H_MOUSE_Y, 0, true)

  // Initialize wake flags to 0
  view.setUint32(H_WAKE_RUST, 0, true)
  view.setUint32(H_WAKE_TS, 0, true)

  // Initialize config with defaults
  view.setUint32(H_CONFIG_FLAGS, CONFIG_DEFAULT, true)
  view.setUint32(H_RENDER_MODE, 0, true) // fullscreen
  view.setUint32(H_CURSOR_CONFIG, 0, true)
  view.setUint32(H_SCROLL_SPEED, 3, true) // default: 3 lines per wheel tick

  // Initialize dimension fields to NaN (= Auto) for all nodes
  // Fields: width, height, min_width, min_height, max_width, max_height, flex_basis, insets
  const dimensionOffsets = [
    F_WIDTH, F_HEIGHT,
    F_MIN_WIDTH, F_MIN_HEIGHT,
    F_MAX_WIDTH, F_MAX_HEIGHT,
    F_FLEX_BASIS,
    F_INSET_TOP, F_INSET_RIGHT, F_INSET_BOTTOM, F_INSET_LEFT,
  ]
  for (let node = 0; node < MAX_NODES; node++) {
    const base = HEADER_SIZE + node * STRIDE
    for (const offset of dimensionOffsets) {
      view.setFloat32(base + offset, NaN, true)
    }
    // Opacity defaults to 255 (fully opaque)
    view.setUint8(base + U_OPACITY, 255)
  }

  return { buffer, view, header, textPool, eventRing, textPoolSize }
}

// =============================================================================
// LOW-LEVEL ACCESSORS
// =============================================================================

function nodeBase(nodeIndex: number): number {
  return HEADER_SIZE + nodeIndex * STRIDE
}

// Float32 read/write
export function getF32(buf: AoSBuffer, nodeIndex: number, field: number): number {
  return buf.view.getFloat32(nodeBase(nodeIndex) + field, true)
}

export function setF32(buf: AoSBuffer, nodeIndex: number, field: number, value: number): void {
  buf.view.setFloat32(nodeBase(nodeIndex) + field, value, true)
}

// Uint8 read/write
export function getU8(buf: AoSBuffer, nodeIndex: number, field: number): number {
  return buf.view.getUint8(nodeBase(nodeIndex) + field)
}

export function setU8(buf: AoSBuffer, nodeIndex: number, field: number, value: number): void {
  buf.view.setUint8(nodeBase(nodeIndex) + field, value)
}

// Uint32 read/write
export function getU32(buf: AoSBuffer, nodeIndex: number, field: number): number {
  return buf.view.getUint32(nodeBase(nodeIndex) + field, true)
}

export function setU32(buf: AoSBuffer, nodeIndex: number, field: number, value: number): void {
  buf.view.setUint32(nodeBase(nodeIndex) + field, value, true)
}

// Int32 read/write
export function getI32(buf: AoSBuffer, nodeIndex: number, field: number): number {
  return buf.view.getInt32(nodeBase(nodeIndex) + field, true)
}

export function setI32(buf: AoSBuffer, nodeIndex: number, field: number, value: number): void {
  buf.view.setInt32(nodeBase(nodeIndex) + field, value, true)
}

// =============================================================================
// HIGH-LEVEL NODE API
// =============================================================================

export function markDirty(buf: AoSBuffer, nodeIndex: number, flags: number): void {
  const current = getU8(buf, nodeIndex, U_DIRTY_FLAGS)
  setU8(buf, nodeIndex, U_DIRTY_FLAGS, current | flags)
}

export function clearDirty(buf: AoSBuffer, nodeIndex: number, flags: number): void {
  const current = getU8(buf, nodeIndex, U_DIRTY_FLAGS)
  setU8(buf, nodeIndex, U_DIRTY_FLAGS, current & ~flags)
}

export function isDirty(buf: AoSBuffer, nodeIndex: number, flags: number): boolean {
  return (getU8(buf, nodeIndex, U_DIRTY_FLAGS) & flags) !== 0
}

export function packColor(r: number, g: number, b: number, a: number): number {
  return ((a & 0xff) << 24) | ((r & 0xff) << 16) | ((g & 0xff) << 8) | (b & 0xff)
}

export function unpackColor(packed: number): { r: number; g: number; b: number; a: number } {
  return {
    a: (packed >>> 24) & 0xff,
    r: (packed >>> 16) & 0xff,
    g: (packed >>> 8) & 0xff,
    b: packed & 0xff,
  }
}

export function setTerminalSize(buf: AoSBuffer, width: number, height: number): void {
  buf.header[H_TERMINAL_WIDTH / 4] = width
  buf.header[H_TERMINAL_HEIGHT / 4] = height
}

export function setNodeCount(buf: AoSBuffer, count: number): void {
  buf.header[H_NODE_COUNT / 4] = count
}

export function getNodeCount(buf: AoSBuffer): number {
  return buf.header[H_NODE_COUNT / 4]
}

export function getParentIndex(buf: AoSBuffer, nodeIndex: number): number {
  return getI32(buf, nodeIndex, I_PARENT_INDEX)
}

export function setParentIndex(buf: AoSBuffer, nodeIndex: number, parentIndex: number): void {
  setI32(buf, nodeIndex, I_PARENT_INDEX, parentIndex)
}

// =============================================================================
// TEXT POOL
// =============================================================================

const textEncoder = new TextEncoder()

export function setNodeText(buf: AoSBuffer, nodeIndex: number, text: string): void {
  const encoded = textEncoder.encode(text)
  const writePtr = buf.header[H_TEXT_POOL_WRITE_PTR / 4]

  if (writePtr + encoded.length > buf.textPoolSize) {
    const sizeMB = (buf.textPoolSize / 1024 / 1024).toFixed(0)
    throw new Error(
      `Text pool overflow: pool is ${buf.textPoolSize} bytes (${sizeMB}MB), writePtr is at ${writePtr}, ` +
      `trying to write ${encoded.length} bytes. This usually means text content is being ` +
      `appended without reusing slots. Consider: 1) Reusing text slots for dynamic content, ` +
      `2) Implementing text compaction, or 3) Increasing textPoolSize in createAoSBuffer({ textPoolSize: ... }).`
    )
  }

  buf.textPool.set(encoded, writePtr)
  setU32(buf, nodeIndex, U_TEXT_OFFSET, writePtr)
  setU32(buf, nodeIndex, U_TEXT_LENGTH, encoded.length)
  buf.header[H_TEXT_POOL_WRITE_PTR / 4] = writePtr + encoded.length

  // Mark node dirty so layout recalculates text measurement
  const base = HEADER_SIZE + nodeIndex * STRIDE
  const current = buf.view.getUint8(base + U_DIRTY_FLAGS)
  buf.view.setUint8(base + U_DIRTY_FLAGS, current | DIRTY_TEXT)
}

// =============================================================================
// HEADER ACCESSORS
// =============================================================================

// Config flags
export function getConfigFlags(buf: AoSBuffer): number {
  return buf.view.getUint32(H_CONFIG_FLAGS, true)
}

export function setConfigFlags(buf: AoSBuffer, flags: number): void {
  buf.view.setUint32(H_CONFIG_FLAGS, flags, true)
}

export function hasConfigFlag(buf: AoSBuffer, flag: number): boolean {
  return (getConfigFlags(buf) & flag) !== 0
}

export function setConfigFlag(buf: AoSBuffer, flag: number, enabled: boolean): void {
  const current = getConfigFlags(buf)
  setConfigFlags(buf, enabled ? current | flag : current & ~flag)
}

// Event indices
export function getEventWriteIdx(buf: AoSBuffer): number {
  return buf.view.getUint32(H_EVENT_WRITE_IDX, true)
}

export function getEventReadIdx(buf: AoSBuffer): number {
  return buf.view.getUint32(H_EVENT_READ_IDX, true)
}

export function setEventReadIdx(buf: AoSBuffer, idx: number): void {
  buf.view.setUint32(H_EVENT_READ_IDX, idx, true)
}

// State indices (Rust writes these)
export function getFocusedIndex(buf: AoSBuffer): number {
  return buf.view.getInt32(H_FOCUSED_INDEX, true)
}

export function getHoveredIndex(buf: AoSBuffer): number {
  return buf.view.getInt32(H_HOVERED_INDEX, true)
}

export function getPressedIndex(buf: AoSBuffer): number {
  return buf.view.getInt32(H_PRESSED_INDEX, true)
}

export function getMousePosition(buf: AoSBuffer): { x: number; y: number } {
  return {
    x: buf.view.getUint16(H_MOUSE_X, true),
    y: buf.view.getUint16(H_MOUSE_Y, true),
  }
}

// Render mode
export function getRenderMode(buf: AoSBuffer): number {
  return buf.view.getUint32(H_RENDER_MODE, true)
}

export function setRenderMode(buf: AoSBuffer, mode: number): void {
  buf.view.setUint32(H_RENDER_MODE, mode, true)
}

// Scroll speed
export function getScrollSpeed(buf: AoSBuffer): number {
  return buf.view.getUint32(H_SCROLL_SPEED, true)
}

export function setScrollSpeed(buf: AoSBuffer, speed: number): void {
  buf.view.setUint32(H_SCROLL_SPEED, speed, true)
}

// =============================================================================
// EVENT RING BUFFER HELPERS
// =============================================================================

export function getEventSlotOffset(slotIndex: number): number {
  return EVENT_RING_OFFSET + EVENT_RING_HEADER_SIZE + slotIndex * EVENT_SLOT_SIZE
}

export function hasEvents(buf: AoSBuffer): boolean {
  return getEventWriteIdx(buf) > getEventReadIdx(buf)
}

export function getEventCount(buf: AoSBuffer): number {
  return getEventWriteIdx(buf) - getEventReadIdx(buf)
}

// =============================================================================
// NODE WRITER (convenient API)
// =============================================================================

export interface NodeWriter {
  // Layout floats
  width: number
  height: number
  minWidth: number
  minHeight: number
  maxWidth: number
  maxHeight: number
  flexBasis: number
  flexGrow: number
  flexShrink: number
  paddingTop: number
  paddingRight: number
  paddingBottom: number
  paddingLeft: number
  marginTop: number
  marginRight: number
  marginBottom: number
  marginLeft: number
  gap: number
  rowGap: number
  columnGap: number

  // Layout enums
  flexDirection: number
  flexWrap: number
  justifyContent: number
  alignItems: number
  alignContent: number
  alignSelf: number
  position: number
  overflow: number
  display: number
  borderTop: number
  borderRight: number
  borderBottom: number
  borderLeft: number
  componentType: number
  visible: number

  // Visual
  fgColor: number
  bgColor: number
  borderColor: number
  opacity: number
  zIndex: number

  // Hierarchy
  parentIndex: number

  // Methods
  markDirty(flags: number): void
  setText(text: string): void
}

export function createNodeWriter(buf: AoSBuffer, nodeIndex: number): NodeWriter {
  const base = nodeBase(nodeIndex)
  const v = buf.view

  return {
    // Layout floats
    set width(val: number) { v.setFloat32(base + F_WIDTH, val, true) },
    get width() { return v.getFloat32(base + F_WIDTH, true) },
    set height(val: number) { v.setFloat32(base + F_HEIGHT, val, true) },
    get height() { return v.getFloat32(base + F_HEIGHT, true) },
    set minWidth(val: number) { v.setFloat32(base + F_MIN_WIDTH, val, true) },
    get minWidth() { return v.getFloat32(base + F_MIN_WIDTH, true) },
    set minHeight(val: number) { v.setFloat32(base + F_MIN_HEIGHT, val, true) },
    get minHeight() { return v.getFloat32(base + F_MIN_HEIGHT, true) },
    set maxWidth(val: number) { v.setFloat32(base + F_MAX_WIDTH, val, true) },
    get maxWidth() { return v.getFloat32(base + F_MAX_WIDTH, true) },
    set maxHeight(val: number) { v.setFloat32(base + F_MAX_HEIGHT, val, true) },
    get maxHeight() { return v.getFloat32(base + F_MAX_HEIGHT, true) },
    set flexBasis(val: number) { v.setFloat32(base + F_FLEX_BASIS, val, true) },
    get flexBasis() { return v.getFloat32(base + F_FLEX_BASIS, true) },
    set flexGrow(val: number) { v.setFloat32(base + F_FLEX_GROW, val, true) },
    get flexGrow() { return v.getFloat32(base + F_FLEX_GROW, true) },
    set flexShrink(val: number) { v.setFloat32(base + F_FLEX_SHRINK, val, true) },
    get flexShrink() { return v.getFloat32(base + F_FLEX_SHRINK, true) },
    set paddingTop(val: number) { v.setFloat32(base + F_PADDING_TOP, val, true) },
    get paddingTop() { return v.getFloat32(base + F_PADDING_TOP, true) },
    set paddingRight(val: number) { v.setFloat32(base + F_PADDING_RIGHT, val, true) },
    get paddingRight() { return v.getFloat32(base + F_PADDING_RIGHT, true) },
    set paddingBottom(val: number) { v.setFloat32(base + F_PADDING_BOTTOM, val, true) },
    get paddingBottom() { return v.getFloat32(base + F_PADDING_BOTTOM, true) },
    set paddingLeft(val: number) { v.setFloat32(base + F_PADDING_LEFT, val, true) },
    get paddingLeft() { return v.getFloat32(base + F_PADDING_LEFT, true) },
    set marginTop(val: number) { v.setFloat32(base + F_MARGIN_TOP, val, true) },
    get marginTop() { return v.getFloat32(base + F_MARGIN_TOP, true) },
    set marginRight(val: number) { v.setFloat32(base + F_MARGIN_RIGHT, val, true) },
    get marginRight() { return v.getFloat32(base + F_MARGIN_RIGHT, true) },
    set marginBottom(val: number) { v.setFloat32(base + F_MARGIN_BOTTOM, val, true) },
    get marginBottom() { return v.getFloat32(base + F_MARGIN_BOTTOM, true) },
    set marginLeft(val: number) { v.setFloat32(base + F_MARGIN_LEFT, val, true) },
    get marginLeft() { return v.getFloat32(base + F_MARGIN_LEFT, true) },
    set gap(val: number) { v.setFloat32(base + F_GAP, val, true) },
    get gap() { return v.getFloat32(base + F_GAP, true) },
    set rowGap(val: number) { v.setFloat32(base + F_ROW_GAP, val, true) },
    get rowGap() { return v.getFloat32(base + F_ROW_GAP, true) },
    set columnGap(val: number) { v.setFloat32(base + F_COLUMN_GAP, val, true) },
    get columnGap() { return v.getFloat32(base + F_COLUMN_GAP, true) },

    // Layout enums
    set flexDirection(val: number) { v.setUint8(base + U_FLEX_DIRECTION, val) },
    get flexDirection() { return v.getUint8(base + U_FLEX_DIRECTION) },
    set flexWrap(val: number) { v.setUint8(base + U_FLEX_WRAP, val) },
    get flexWrap() { return v.getUint8(base + U_FLEX_WRAP) },
    set justifyContent(val: number) { v.setUint8(base + U_JUSTIFY_CONTENT, val) },
    get justifyContent() { return v.getUint8(base + U_JUSTIFY_CONTENT) },
    set alignItems(val: number) { v.setUint8(base + U_ALIGN_ITEMS, val) },
    get alignItems() { return v.getUint8(base + U_ALIGN_ITEMS) },
    set alignContent(val: number) { v.setUint8(base + U_ALIGN_CONTENT, val) },
    get alignContent() { return v.getUint8(base + U_ALIGN_CONTENT) },
    set alignSelf(val: number) { v.setUint8(base + U_ALIGN_SELF, val) },
    get alignSelf() { return v.getUint8(base + U_ALIGN_SELF) },
    set position(val: number) { v.setUint8(base + U_POSITION, val) },
    get position() { return v.getUint8(base + U_POSITION) },
    set overflow(val: number) { v.setUint8(base + U_OVERFLOW, val) },
    get overflow() { return v.getUint8(base + U_OVERFLOW) },
    set display(val: number) { v.setUint8(base + U_DISPLAY, val) },
    get display() { return v.getUint8(base + U_DISPLAY) },
    set borderTop(val: number) { v.setUint8(base + U_BORDER_TOP, val) },
    get borderTop() { return v.getUint8(base + U_BORDER_TOP) },
    set borderRight(val: number) { v.setUint8(base + U_BORDER_RIGHT, val) },
    get borderRight() { return v.getUint8(base + U_BORDER_RIGHT) },
    set borderBottom(val: number) { v.setUint8(base + U_BORDER_BOTTOM, val) },
    get borderBottom() { return v.getUint8(base + U_BORDER_BOTTOM) },
    set borderLeft(val: number) { v.setUint8(base + U_BORDER_LEFT, val) },
    get borderLeft() { return v.getUint8(base + U_BORDER_LEFT) },
    set componentType(val: number) { v.setUint8(base + U_COMPONENT_TYPE, val) },
    get componentType() { return v.getUint8(base + U_COMPONENT_TYPE) },
    set visible(val: number) { v.setUint8(base + U_VISIBLE, val) },
    get visible() { return v.getUint8(base + U_VISIBLE) },

    // Visual
    set fgColor(val: number) { v.setUint32(base + C_FG_COLOR, val, true) },
    get fgColor() { return v.getUint32(base + C_FG_COLOR, true) },
    set bgColor(val: number) { v.setUint32(base + C_BG_COLOR, val, true) },
    get bgColor() { return v.getUint32(base + C_BG_COLOR, true) },
    set borderColor(val: number) { v.setUint32(base + C_BORDER_COLOR, val, true) },
    get borderColor() { return v.getUint32(base + C_BORDER_COLOR, true) },
    set opacity(val: number) { v.setUint8(base + U_OPACITY, val) },
    get opacity() { return v.getUint8(base + U_OPACITY) },
    set zIndex(val: number) { v.setInt8(base + I_Z_INDEX, val) },
    get zIndex() { return v.getInt8(base + I_Z_INDEX) },

    // Hierarchy
    set parentIndex(val: number) { v.setInt32(base + I_PARENT_INDEX, val, true) },
    get parentIndex() { return v.getInt32(base + I_PARENT_INDEX, true) },

    // Methods
    markDirty(flags: number) {
      const current = v.getUint8(base + U_DIRTY_FLAGS)
      v.setUint8(base + U_DIRTY_FLAGS, current | flags)
    },

    setText(text: string) {
      setNodeText(buf, nodeIndex, text)
    },
  }
}
