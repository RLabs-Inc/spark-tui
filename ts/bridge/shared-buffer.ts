/**
 * SparkTUI Shared Buffer - TypeScript Implementation
 *
 * This file implements the shared memory contract defined in SHARED-BUFFER-SPEC.md.
 * Both TypeScript and Rust MUST match this spec exactly.
 *
 * Memory Layout:
 *   - Header (256 bytes): Global state, wake flags, config
 *   - Nodes (512 bytes × MAX_NODES): Per-component data
 *   - Text Pool (configurable): UTF-8 text content
 *   - Event Ring (5,132 bytes): Rust → TS event queue
 *
 * @version 2.0
 * @date 2026-01-30
 */

// =============================================================================
// CONSTANTS
// =============================================================================

/** Header size in bytes */
export const HEADER_SIZE = 256;

/** Bytes per node (8 cache lines × 64 bytes) */
export const NODE_STRIDE = 512;

/** Default maximum nodes */
export const DEFAULT_MAX_NODES = 10_000;

/** Default text pool size (10 MB) */
export const DEFAULT_TEXT_POOL_SIZE = 10 * 1024 * 1024;

/** Event ring header size */
export const EVENT_RING_HEADER_SIZE = 12;

/** Bytes per event slot */
export const EVENT_SLOT_SIZE = 20;

/** Maximum events in ring buffer */
export const MAX_EVENTS = 256;

/** Total event ring size */
export const EVENT_RING_SIZE = EVENT_RING_HEADER_SIZE + MAX_EVENTS * EVENT_SLOT_SIZE;

/** NaN represents "auto" for dimension values */
export const AUTO = NaN;

// =============================================================================
// HEADER OFFSETS (256 bytes total)
// =============================================================================

// --- Bytes 0-63: Core ---
export const H_VERSION = 0;
export const H_NODE_COUNT = 4;
export const H_MAX_NODES = 8;
export const H_TERMINAL_WIDTH = 12;
export const H_TERMINAL_HEIGHT = 16;
export const H_GENERATION = 20;
export const H_TEXT_POOL_SIZE = 24;
export const H_TEXT_POOL_WRITE_PTR = 28;
// 32-63: reserved

// --- Bytes 64-95: Wake & Sync (4-byte aligned for Atomics) ---
export const H_WAKE_RUST = 64;
export const H_WAKE_TS = 68;
// 72-95: reserved

// --- Bytes 96-127: State (Rust writes, TS reads) ---
export const H_FOCUSED_INDEX = 96;
export const H_HOVERED_INDEX = 100;
export const H_PRESSED_INDEX = 104;
export const H_MOUSE_X = 108;
export const H_MOUSE_Y = 110;
// 112-127: reserved

// --- Bytes 128-159: Config (TS writes, Rust reads) ---
export const H_CONFIG_FLAGS = 128;
export const H_RENDER_MODE = 132;
export const H_CURSOR_CONFIG = 136;
export const H_SCROLL_SPEED = 140;
// 144-159: reserved

// --- Bytes 160-191: Events ---
export const H_EVENT_WRITE_IDX = 160;
export const H_EVENT_READ_IDX = 164;
export const H_EXIT_REQUESTED = 168;
// 169-191: reserved

// --- Bytes 192-255: Stats & Debug ---
export const H_RENDER_COUNT = 192;
export const H_LAYOUT_COUNT = 196;
// 200-255: reserved

// =============================================================================
// NODE FIELD OFFSETS (512 bytes per node)
// =============================================================================

// --- Cache Line 1 (0-63): Layout Dimensions ---
export const F_WIDTH = 0;
export const F_HEIGHT = 4;
export const F_MIN_WIDTH = 8;
export const F_MIN_HEIGHT = 12;
export const F_MAX_WIDTH = 16;
export const F_MAX_HEIGHT = 20;
export const F_FLEX_BASIS = 24;
export const F_FLEX_GROW = 28;
export const F_FLEX_SHRINK = 32;
export const F_PADDING_TOP = 36;
export const F_PADDING_RIGHT = 40;
export const F_PADDING_BOTTOM = 44;
export const F_PADDING_LEFT = 48;
export const F_MARGIN_TOP = 52;
export const F_MARGIN_RIGHT = 56;
export const F_MARGIN_BOTTOM = 60;

// --- Cache Line 2 (64-127): Layout Spacing & Enums ---
export const F_MARGIN_LEFT = 64;
export const F_GAP = 68;
export const F_ROW_GAP = 72;
export const F_COLUMN_GAP = 76;
export const F_INSET_TOP = 80;
export const F_INSET_RIGHT = 84;
export const F_INSET_BOTTOM = 88;
export const F_INSET_LEFT = 92;
export const U_FLEX_DIRECTION = 96;
export const U_FLEX_WRAP = 97;
export const U_JUSTIFY_CONTENT = 98;
export const U_ALIGN_ITEMS = 99;
export const U_ALIGN_CONTENT = 100;
export const U_ALIGN_SELF = 101;
export const U_POSITION = 102;
export const U_OVERFLOW = 103;
export const U_DISPLAY = 104;
export const U_BORDER_WIDTH_TOP = 105;
export const U_BORDER_WIDTH_RIGHT = 106;
export const U_BORDER_WIDTH_BOTTOM = 107;
export const U_BORDER_WIDTH_LEFT = 108;
export const U_COMPONENT_TYPE = 109;
export const U_VISIBLE = 110;
// 111: reserved
export const I_PARENT_INDEX = 112;
export const I_TAB_INDEX = 116;
export const I_CHILD_COUNT = 120;
// 124-127: reserved

// --- Cache Line 3 (128-191): Output & Colors ---
export const F_COMPUTED_X = 128;
export const F_COMPUTED_Y = 132;
export const F_COMPUTED_WIDTH = 136;
export const F_COMPUTED_HEIGHT = 140;
export const F_SCROLL_WIDTH = 144;
export const F_SCROLL_HEIGHT = 148;
export const F_MAX_SCROLL_X = 152;
export const F_MAX_SCROLL_Y = 156;
export const C_FG_COLOR = 160;
export const C_BG_COLOR = 164;
export const C_BORDER_COLOR = 168;
export const C_BORDER_TOP_COLOR = 172;
export const C_BORDER_RIGHT_COLOR = 176;
export const C_BORDER_BOTTOM_COLOR = 180;
export const C_BORDER_LEFT_COLOR = 184;
export const C_FOCUS_RING_COLOR = 188;

// --- Cache Line 4 (192-255): Visual Properties ---
export const C_CURSOR_FG_COLOR = 192;
export const C_CURSOR_BG_COLOR = 196;
export const C_SELECTION_COLOR = 200;
export const U_OPACITY = 204;
export const I_Z_INDEX = 205;
export const U_BORDER_STYLE = 206;
export const U_BORDER_STYLE_TOP = 207;
export const U_BORDER_STYLE_RIGHT = 208;
export const U_BORDER_STYLE_BOTTOM = 209;
export const U_BORDER_STYLE_LEFT = 210;
export const U_SCROLLABLE_FLAGS = 211;
export const U_BORDER_CHAR_H = 212;
export const U_BORDER_CHAR_V = 214;
export const U_BORDER_CHAR_TL = 216;
export const U_BORDER_CHAR_TR = 218;
export const U_BORDER_CHAR_BL = 220;
export const U_BORDER_CHAR_BR = 222;
export const U_FOCUS_INDICATOR_CHAR = 224;
export const U_FOCUS_INDICATOR_ENABLED = 225;
// 226-255: reserved

// --- Cache Line 5 (256-319): Text Properties ---
export const U_TEXT_OFFSET = 256;
export const U_TEXT_LENGTH = 260;
export const U_TEXT_ALIGN = 264;
export const U_TEXT_WRAP = 265;
export const U_TEXT_OVERFLOW = 266;
export const U_TEXT_ATTRS = 267;
export const U_TEXT_DECORATION = 268;
export const U_TEXT_DECORATION_STYLE = 269;
export const C_TEXT_DECORATION_COLOR = 270;
export const U_LINE_HEIGHT = 274;
export const U_LETTER_SPACING = 275;
export const U_MAX_LINES = 276;
// 277-319: reserved

// --- Cache Line 6 (320-383): Interaction State ---
export const I_SCROLL_X = 320;
export const I_SCROLL_Y = 324;
export const I_CURSOR_POSITION = 328;
export const I_SELECTION_START = 332;
export const I_SELECTION_END = 336;
export const U_CURSOR_CHAR = 340;
export const U_CURSOR_ALT_CHAR = 344;
export const U_DIRTY_FLAGS = 348;
export const U_INTERACTION_FLAGS = 349;
export const U_CURSOR_FLAGS = 350;
export const U_CURSOR_STYLE = 351;
export const U_CURSOR_BLINK_RATE = 352;
export const U_MAX_LENGTH = 353;
export const U_INPUT_TYPE = 354;
// 356-383: reserved

// --- Cache Line 7 (384-447): Animation (Reserved) ---
// Reserved for future animation system

// --- Cache Line 8 (448-511): Effects & Transforms (Reserved) ---
// Reserved for future effects and physics

// =============================================================================
// CONFIG FLAGS (bitfield at H_CONFIG_FLAGS)
// =============================================================================

export const CONFIG_EXIT_ON_CTRL_C = 1 << 0;
export const CONFIG_TAB_NAVIGATION = 1 << 1;
export const CONFIG_ARROW_SCROLL = 1 << 2;
export const CONFIG_PAGE_SCROLL = 1 << 3;
export const CONFIG_HOME_END_SCROLL = 1 << 4;
export const CONFIG_WHEEL_SCROLL = 1 << 5;
export const CONFIG_FOCUS_ON_CLICK = 1 << 6;
export const CONFIG_MOUSE_ENABLED = 1 << 7;
export const CONFIG_KITTY_KEYBOARD = 1 << 8;

/** Default config: bits 0-7 enabled */
export const CONFIG_DEFAULT = 0x00ff;

// =============================================================================
// DIRTY FLAGS (bitfield at U_DIRTY_FLAGS)
// =============================================================================

export const DIRTY_LAYOUT = 1 << 0;
export const DIRTY_VISUAL = 1 << 1;
export const DIRTY_TEXT = 1 << 2;
export const DIRTY_HIERARCHY = 1 << 3;

// =============================================================================
// INTERACTION FLAGS (bitfield at U_INTERACTION_FLAGS)
// =============================================================================

export const FLAG_FOCUSABLE = 1 << 0;
export const FLAG_FOCUSED = 1 << 1;
export const FLAG_HOVERED = 1 << 2;
export const FLAG_PRESSED = 1 << 3;
export const FLAG_DISABLED = 1 << 4;

// =============================================================================
// TEXT ATTRIBUTES (bitfield at U_TEXT_ATTRS)
// =============================================================================

export const ATTR_BOLD = 1 << 0;
export const ATTR_ITALIC = 1 << 1;
export const ATTR_UNDERLINE = 1 << 2;
export const ATTR_STRIKETHROUGH = 1 << 3;
export const ATTR_DIM = 1 << 4;
export const ATTR_BLINK = 1 << 5;
export const ATTR_REVERSE = 1 << 6;
export const ATTR_HIDDEN = 1 << 7;

// =============================================================================
// COMPONENT TYPES
// =============================================================================

export const COMPONENT_NONE = 0;
export const COMPONENT_BOX = 1;
export const COMPONENT_TEXT = 2;
export const COMPONENT_INPUT = 3;

// =============================================================================
// BORDER STYLES
// =============================================================================

export const BORDER_NONE = 0;
export const BORDER_SINGLE = 1;
export const BORDER_DOUBLE = 2;
export const BORDER_ROUNDED = 3;
export const BORDER_THICK = 4;
export const BORDER_DASHED = 5;
export const BORDER_DOTTED = 6;
export const BORDER_ASCII = 7;

// =============================================================================
// EVENT TYPES
// =============================================================================

export const EVENT_NONE = 0;
export const EVENT_KEY = 1;
export const EVENT_MOUSE_DOWN = 2;
export const EVENT_MOUSE_UP = 3;
export const EVENT_CLICK = 4;
export const EVENT_MOUSE_ENTER = 5;
export const EVENT_MOUSE_LEAVE = 6;
export const EVENT_MOUSE_MOVE = 7;
export const EVENT_SCROLL = 8;
export const EVENT_FOCUS = 9;
export const EVENT_BLUR = 10;
export const EVENT_VALUE_CHANGE = 11;
export const EVENT_SUBMIT = 12;
export const EVENT_CANCEL = 13;
export const EVENT_EXIT = 14;
export const EVENT_RESIZE = 15;

// =============================================================================
// ENUMS
// =============================================================================

export const enum FlexDirection {
  Row = 0,
  Column = 1,
  RowReverse = 2,
  ColumnReverse = 3,
}

export const enum FlexWrap {
  NoWrap = 0,
  Wrap = 1,
  WrapReverse = 2,
}

export const enum JustifyContent {
  Start = 0,
  End = 1,
  Center = 2,
  SpaceBetween = 3,
  SpaceAround = 4,
  SpaceEvenly = 5,
}

export const enum AlignItems {
  Start = 0,
  End = 1,
  Center = 2,
  Baseline = 3,
  Stretch = 4,
}

export const enum AlignContent {
  Start = 0,
  End = 1,
  Center = 2,
  SpaceBetween = 3,
  SpaceAround = 4,
  SpaceEvenly = 5,
}

export const enum AlignSelf {
  Auto = 0,
  Start = 1,
  End = 2,
  Center = 3,
  Baseline = 4,
  Stretch = 5,
}

export const enum Position {
  Relative = 0,
  Absolute = 1,
}

export const enum Overflow {
  Visible = 0,
  Hidden = 1,
  Scroll = 2,
}

export const enum Display {
  None = 0,
  Flex = 1,
}

export const enum TextAlign {
  Left = 0,
  Center = 1,
  Right = 2,
}

export const enum TextWrap {
  NoWrap = 0,
  Wrap = 1,
  Truncate = 2,
}

export const enum TextOverflow {
  Clip = 0,
  Ellipsis = 1,
  Fade = 2,
}

export const enum TextDecoration {
  None = 0,
  Underline = 1,
  Overline = 2,
  LineThrough = 3,
}

export const enum TextDecorationStyle {
  Solid = 0,
  Double = 1,
  Dotted = 2,
  Dashed = 3,
  Wavy = 4,
}

export const enum CursorStyle {
  Block = 0,
  Bar = 1,
  Underline = 2,
}

export const enum InputType {
  Text = 0,
  Password = 1,
  Number = 2,
  Email = 3,
}

export const enum RenderMode {
  Diff = 0,
  Inline = 1,
  Append = 2,
}

// =============================================================================
// BUFFER INTERFACE
// =============================================================================

export interface SharedBuffer {
  /** The underlying SharedArrayBuffer */
  raw: SharedArrayBuffer;
  /** DataView for reading/writing */
  view: DataView;
  /** Int32Array view of header (for Atomics - must be Int32Array for wait/notify) */
  headerI32: Int32Array;
  /** Configured maximum nodes */
  maxNodes: number;
  /** Configured text pool size */
  textPoolSize: number;
  /** Offset where text pool starts */
  textPoolOffset: number;
  /** Offset where event ring starts */
  eventRingOffset: number;
}

export interface SharedBufferConfig {
  /** Maximum number of components. Default: 10,000 */
  maxNodes?: number;
  /** Text pool size in bytes. Default: 10 MB */
  textPoolSize?: number;
}

// =============================================================================
// BUFFER CREATION
// =============================================================================

/**
 * Calculate buffer size for given configuration.
 */
export function calculateBufferSize(config: SharedBufferConfig = {}): number {
  const maxNodes = config.maxNodes ?? DEFAULT_MAX_NODES;
  const textPoolSize = config.textPoolSize ?? DEFAULT_TEXT_POOL_SIZE;
  return HEADER_SIZE + maxNodes * NODE_STRIDE + textPoolSize + EVENT_RING_SIZE;
}

/**
 * Create a new shared buffer with the given configuration.
 */
export function createSharedBuffer(config: SharedBufferConfig = {}): SharedBuffer {
  const maxNodes = config.maxNodes ?? DEFAULT_MAX_NODES;
  const textPoolSize = config.textPoolSize ?? DEFAULT_TEXT_POOL_SIZE;
  const textPoolOffset = HEADER_SIZE + maxNodes * NODE_STRIDE;
  const eventRingOffset = textPoolOffset + textPoolSize;
  const totalSize = eventRingOffset + EVENT_RING_SIZE;

  const raw = new SharedArrayBuffer(totalSize);
  const view = new DataView(raw);
  const headerI32 = new Int32Array(raw, 0, HEADER_SIZE / 4);

  const buffer: SharedBuffer = {
    raw,
    view,
    headerI32,
    maxNodes,
    textPoolSize,
    textPoolOffset,
    eventRingOffset,
  };

  // Initialize header
  view.setUint32(H_VERSION, 2, true);
  view.setUint32(H_NODE_COUNT, 0, true);
  view.setUint32(H_MAX_NODES, maxNodes, true);
  view.setUint32(H_TEXT_POOL_SIZE, textPoolSize, true);
  view.setUint32(H_TEXT_POOL_WRITE_PTR, 0, true);
  view.setUint32(H_GENERATION, 0, true);

  // Initialize wake flags to 0
  view.setUint32(H_WAKE_RUST, 0, true);
  view.setUint32(H_WAKE_TS, 0, true);

  // Initialize state to -1 (none)
  view.setInt32(H_FOCUSED_INDEX, -1, true);
  view.setInt32(H_HOVERED_INDEX, -1, true);
  view.setInt32(H_PRESSED_INDEX, -1, true);
  view.setUint16(H_MOUSE_X, 0, true);
  view.setUint16(H_MOUSE_Y, 0, true);

  // Initialize config with defaults
  view.setUint32(H_CONFIG_FLAGS, CONFIG_DEFAULT, true);
  view.setUint32(H_RENDER_MODE, RenderMode.Diff, true);
  view.setUint32(H_SCROLL_SPEED, 3, true);

  // Initialize event indices
  view.setUint32(H_EVENT_WRITE_IDX, 0, true);
  view.setUint32(H_EVENT_READ_IDX, 0, true);
  view.setUint8(H_EXIT_REQUESTED, 0);

  // Initialize all nodes with defaults
  for (let i = 0; i < maxNodes; i++) {
    initializeNode(buffer, i);
  }

  return buffer;
}

/**
 * Initialize a single node with default values.
 */
function initializeNode(buffer: SharedBuffer, nodeIndex: number): void {
  const base = HEADER_SIZE + nodeIndex * NODE_STRIDE;
  const v = buffer.view;

  // Layout dimensions - NaN = auto
  v.setFloat32(base + F_WIDTH, NaN, true);
  v.setFloat32(base + F_HEIGHT, NaN, true);
  v.setFloat32(base + F_MIN_WIDTH, NaN, true);
  v.setFloat32(base + F_MIN_HEIGHT, NaN, true);
  v.setFloat32(base + F_MAX_WIDTH, NaN, true);
  v.setFloat32(base + F_MAX_HEIGHT, NaN, true);
  v.setFloat32(base + F_FLEX_BASIS, NaN, true);
  v.setFloat32(base + F_FLEX_GROW, 0, true);
  v.setFloat32(base + F_FLEX_SHRINK, 1, true);

  // Padding/margin - 0
  v.setFloat32(base + F_PADDING_TOP, 0, true);
  v.setFloat32(base + F_PADDING_RIGHT, 0, true);
  v.setFloat32(base + F_PADDING_BOTTOM, 0, true);
  v.setFloat32(base + F_PADDING_LEFT, 0, true);
  v.setFloat32(base + F_MARGIN_TOP, 0, true);
  v.setFloat32(base + F_MARGIN_RIGHT, 0, true);
  v.setFloat32(base + F_MARGIN_BOTTOM, 0, true);
  v.setFloat32(base + F_MARGIN_LEFT, 0, true);

  // Gap - 0
  v.setFloat32(base + F_GAP, 0, true);
  v.setFloat32(base + F_ROW_GAP, 0, true);
  v.setFloat32(base + F_COLUMN_GAP, 0, true);

  // Insets - NaN = auto
  v.setFloat32(base + F_INSET_TOP, NaN, true);
  v.setFloat32(base + F_INSET_RIGHT, NaN, true);
  v.setFloat32(base + F_INSET_BOTTOM, NaN, true);
  v.setFloat32(base + F_INSET_LEFT, NaN, true);

  // Layout enums - defaults
  v.setUint8(base + U_FLEX_DIRECTION, FlexDirection.Row);
  v.setUint8(base + U_FLEX_WRAP, FlexWrap.NoWrap);
  v.setUint8(base + U_JUSTIFY_CONTENT, JustifyContent.Start);
  v.setUint8(base + U_ALIGN_ITEMS, AlignItems.Stretch);
  v.setUint8(base + U_ALIGN_CONTENT, AlignContent.Start);
  v.setUint8(base + U_ALIGN_SELF, AlignSelf.Auto);
  v.setUint8(base + U_POSITION, Position.Relative);
  v.setUint8(base + U_OVERFLOW, Overflow.Visible);
  v.setUint8(base + U_DISPLAY, Display.Flex);

  // Border widths - 0
  v.setUint8(base + U_BORDER_WIDTH_TOP, 0);
  v.setUint8(base + U_BORDER_WIDTH_RIGHT, 0);
  v.setUint8(base + U_BORDER_WIDTH_BOTTOM, 0);
  v.setUint8(base + U_BORDER_WIDTH_LEFT, 0);

  // Component type and visibility
  v.setUint8(base + U_COMPONENT_TYPE, COMPONENT_NONE);
  v.setUint8(base + U_VISIBLE, 1);

  // Hierarchy
  v.setInt32(base + I_PARENT_INDEX, -1, true);
  v.setInt32(base + I_TAB_INDEX, 0, true);
  v.setInt32(base + I_CHILD_COUNT, 0, true);

  // Output - Rust writes these, initialize to 0
  v.setFloat32(base + F_COMPUTED_X, 0, true);
  v.setFloat32(base + F_COMPUTED_Y, 0, true);
  v.setFloat32(base + F_COMPUTED_WIDTH, 0, true);
  v.setFloat32(base + F_COMPUTED_HEIGHT, 0, true);
  v.setFloat32(base + F_SCROLL_WIDTH, 0, true);
  v.setFloat32(base + F_SCROLL_HEIGHT, 0, true);
  v.setFloat32(base + F_MAX_SCROLL_X, 0, true);
  v.setFloat32(base + F_MAX_SCROLL_Y, 0, true);

  // Colors - 0 = transparent/inherit
  v.setUint32(base + C_FG_COLOR, 0, true);
  v.setUint32(base + C_BG_COLOR, 0, true);
  v.setUint32(base + C_BORDER_COLOR, 0, true);
  v.setUint32(base + C_BORDER_TOP_COLOR, 0, true);
  v.setUint32(base + C_BORDER_RIGHT_COLOR, 0, true);
  v.setUint32(base + C_BORDER_BOTTOM_COLOR, 0, true);
  v.setUint32(base + C_BORDER_LEFT_COLOR, 0, true);
  v.setUint32(base + C_FOCUS_RING_COLOR, 0, true);
  v.setUint32(base + C_CURSOR_FG_COLOR, 0, true);
  v.setUint32(base + C_CURSOR_BG_COLOR, 0, true);
  v.setUint32(base + C_SELECTION_COLOR, 0, true);

  // Visual properties
  v.setUint8(base + U_OPACITY, 255);
  v.setInt8(base + I_Z_INDEX, 0);
  v.setUint8(base + U_BORDER_STYLE, BORDER_NONE);
  v.setUint8(base + U_BORDER_STYLE_TOP, 0);
  v.setUint8(base + U_BORDER_STYLE_RIGHT, 0);
  v.setUint8(base + U_BORDER_STYLE_BOTTOM, 0);
  v.setUint8(base + U_BORDER_STYLE_LEFT, 0);
  v.setUint8(base + U_SCROLLABLE_FLAGS, 0);

  // Custom border chars - 0 = use style
  v.setUint16(base + U_BORDER_CHAR_H, 0, true);
  v.setUint16(base + U_BORDER_CHAR_V, 0, true);
  v.setUint16(base + U_BORDER_CHAR_TL, 0, true);
  v.setUint16(base + U_BORDER_CHAR_TR, 0, true);
  v.setUint16(base + U_BORDER_CHAR_BL, 0, true);
  v.setUint16(base + U_BORDER_CHAR_BR, 0, true);

  // Focus indicator
  v.setUint8(base + U_FOCUS_INDICATOR_CHAR, 0x2a); // '*'
  v.setUint8(base + U_FOCUS_INDICATOR_ENABLED, 1);

  // Text properties
  v.setUint32(base + U_TEXT_OFFSET, 0, true);
  v.setUint32(base + U_TEXT_LENGTH, 0, true);
  v.setUint8(base + U_TEXT_ALIGN, TextAlign.Left);
  v.setUint8(base + U_TEXT_WRAP, TextWrap.NoWrap);
  v.setUint8(base + U_TEXT_OVERFLOW, TextOverflow.Clip);
  v.setUint8(base + U_TEXT_ATTRS, 0);
  v.setUint8(base + U_TEXT_DECORATION, TextDecoration.None);
  v.setUint8(base + U_TEXT_DECORATION_STYLE, TextDecorationStyle.Solid);
  v.setUint32(base + C_TEXT_DECORATION_COLOR, 0, true);
  v.setUint8(base + U_LINE_HEIGHT, 0);
  v.setUint8(base + U_LETTER_SPACING, 0);
  v.setUint8(base + U_MAX_LINES, 0);

  // Interaction state
  v.setInt32(base + I_SCROLL_X, 0, true);
  v.setInt32(base + I_SCROLL_Y, 0, true);
  v.setInt32(base + I_CURSOR_POSITION, 0, true);
  v.setInt32(base + I_SELECTION_START, -1, true);
  v.setInt32(base + I_SELECTION_END, -1, true);
  v.setUint32(base + U_CURSOR_CHAR, 0, true);
  v.setUint32(base + U_CURSOR_ALT_CHAR, 0, true);
  v.setUint8(base + U_DIRTY_FLAGS, 0);
  v.setUint8(base + U_INTERACTION_FLAGS, 0);
  v.setUint8(base + U_CURSOR_FLAGS, 0);
  v.setUint8(base + U_CURSOR_STYLE, CursorStyle.Block);
  v.setUint8(base + U_CURSOR_BLINK_RATE, 0);
  v.setUint8(base + U_MAX_LENGTH, 0);
  v.setUint16(base + U_INPUT_TYPE, InputType.Text, true);
}

// =============================================================================
// LOW-LEVEL ACCESSORS
// =============================================================================

/** Get base offset for a node */
function nodeBase(nodeIndex: number): number {
  return HEADER_SIZE + nodeIndex * NODE_STRIDE;
}

// --- Float32 ---
export function getF32(buf: SharedBuffer, nodeIndex: number, field: number): number {
  return buf.view.getFloat32(nodeBase(nodeIndex) + field, true);
}

export function setF32(buf: SharedBuffer, nodeIndex: number, field: number, value: number): void {
  buf.view.setFloat32(nodeBase(nodeIndex) + field, value, true);
}

// --- Uint8 ---
export function getU8(buf: SharedBuffer, nodeIndex: number, field: number): number {
  return buf.view.getUint8(nodeBase(nodeIndex) + field);
}

export function setU8(buf: SharedBuffer, nodeIndex: number, field: number, value: number): void {
  buf.view.setUint8(nodeBase(nodeIndex) + field, value);
}

// --- Int8 ---
export function getI8(buf: SharedBuffer, nodeIndex: number, field: number): number {
  return buf.view.getInt8(nodeBase(nodeIndex) + field);
}

export function setI8(buf: SharedBuffer, nodeIndex: number, field: number, value: number): void {
  buf.view.setInt8(nodeBase(nodeIndex) + field, value);
}

// --- Uint16 ---
export function getU16(buf: SharedBuffer, nodeIndex: number, field: number): number {
  return buf.view.getUint16(nodeBase(nodeIndex) + field, true);
}

export function setU16(buf: SharedBuffer, nodeIndex: number, field: number, value: number): void {
  buf.view.setUint16(nodeBase(nodeIndex) + field, value, true);
}

// --- Int32 ---
export function getI32(buf: SharedBuffer, nodeIndex: number, field: number): number {
  return buf.view.getInt32(nodeBase(nodeIndex) + field, true);
}

export function setI32(buf: SharedBuffer, nodeIndex: number, field: number, value: number): void {
  buf.view.setInt32(nodeBase(nodeIndex) + field, value, true);
}

// --- Uint32 ---
export function getU32(buf: SharedBuffer, nodeIndex: number, field: number): number {
  return buf.view.getUint32(nodeBase(nodeIndex) + field, true);
}

export function setU32(buf: SharedBuffer, nodeIndex: number, field: number, value: number): void {
  buf.view.setUint32(nodeBase(nodeIndex) + field, value, true);
}

// =============================================================================
// HEADER ACCESSORS
// =============================================================================

export function getVersion(buf: SharedBuffer): number {
  return buf.view.getUint32(H_VERSION, true);
}

export function getNodeCount(buf: SharedBuffer): number {
  return buf.view.getUint32(H_NODE_COUNT, true);
}

export function setNodeCount(buf: SharedBuffer, count: number): void {
  buf.view.setUint32(H_NODE_COUNT, count, true);
}

export function getTerminalSize(buf: SharedBuffer): { width: number; height: number } {
  return {
    width: buf.view.getUint32(H_TERMINAL_WIDTH, true),
    height: buf.view.getUint32(H_TERMINAL_HEIGHT, true),
  };
}

export function setTerminalSize(buf: SharedBuffer, width: number, height: number): void {
  buf.view.setUint32(H_TERMINAL_WIDTH, width, true);
  buf.view.setUint32(H_TERMINAL_HEIGHT, height, true);
}

export function getGeneration(buf: SharedBuffer): number {
  return buf.view.getUint32(H_GENERATION, true);
}

export function incrementGeneration(buf: SharedBuffer): number {
  const gen = buf.view.getUint32(H_GENERATION, true) + 1;
  buf.view.setUint32(H_GENERATION, gen, true);
  return gen;
}

// --- Config ---
export function getConfigFlags(buf: SharedBuffer): number {
  return buf.view.getUint32(H_CONFIG_FLAGS, true);
}

export function setConfigFlags(buf: SharedBuffer, flags: number): void {
  buf.view.setUint32(H_CONFIG_FLAGS, flags, true);
}

export function hasConfigFlag(buf: SharedBuffer, flag: number): boolean {
  return (getConfigFlags(buf) & flag) !== 0;
}

export function setConfigFlag(buf: SharedBuffer, flag: number, enabled: boolean): void {
  const current = getConfigFlags(buf);
  setConfigFlags(buf, enabled ? current | flag : current & ~flag);
}

export function getRenderMode(buf: SharedBuffer): RenderMode {
  return buf.view.getUint32(H_RENDER_MODE, true);
}

export function setRenderMode(buf: SharedBuffer, mode: RenderMode): void {
  buf.view.setUint32(H_RENDER_MODE, mode, true);
}

export function getScrollSpeed(buf: SharedBuffer): number {
  return buf.view.getUint32(H_SCROLL_SPEED, true);
}

export function setScrollSpeed(buf: SharedBuffer, speed: number): void {
  buf.view.setUint32(H_SCROLL_SPEED, speed, true);
}

// --- State (Rust writes, TS reads) ---
export function getFocusedIndex(buf: SharedBuffer): number {
  return buf.view.getInt32(H_FOCUSED_INDEX, true);
}

export function getHoveredIndex(buf: SharedBuffer): number {
  return buf.view.getInt32(H_HOVERED_INDEX, true);
}

export function getPressedIndex(buf: SharedBuffer): number {
  return buf.view.getInt32(H_PRESSED_INDEX, true);
}

export function getMousePosition(buf: SharedBuffer): { x: number; y: number } {
  return {
    x: buf.view.getUint16(H_MOUSE_X, true),
    y: buf.view.getUint16(H_MOUSE_Y, true),
  };
}

// --- Stats ---
export function getRenderCount(buf: SharedBuffer): number {
  return buf.view.getUint32(H_RENDER_COUNT, true);
}

export function getLayoutCount(buf: SharedBuffer): number {
  return buf.view.getUint32(H_LAYOUT_COUNT, true);
}

export function isExitRequested(buf: SharedBuffer): boolean {
  return buf.view.getUint8(H_EXIT_REQUESTED) !== 0;
}

// =============================================================================
// WAKE MECHANISM
// =============================================================================

/**
 * Wake the Rust side.
 * Sets wake_rust flag and calls Atomics.notify.
 */
export function wakeRust(buf: SharedBuffer): void {
  const idx = H_WAKE_RUST / 4;
  Atomics.store(buf.headerI32, idx, 1);
  Atomics.notify(buf.headerI32, idx, 1);
}

/**
 * Wait for Rust to wake us (blocking).
 * Returns immediately if wake_ts is already set.
 */
export function waitForRust(buf: SharedBuffer, timeout?: number): 'ok' | 'timed-out' | 'not-equal' {
  const idx = H_WAKE_TS / 4;
  const result = Atomics.wait(buf.headerI32, idx, 0, timeout);
  // Reset the flag
  Atomics.store(buf.headerI32, idx, 0);
  return result;
}

/**
 * Wait for Rust to wake us (non-blocking, async).
 * Returns a promise that resolves when Rust wakes us.
 */
export function waitForRustAsync(buf: SharedBuffer): Promise<void> {
  const idx = H_WAKE_TS / 4;
  const result = Atomics.waitAsync(buf.headerI32, idx, 0);

  if (result.async) {
    return result.value.then(() => {
      Atomics.store(buf.headerI32, idx, 0);
    });
  } else {
    // Already not equal, reset and return immediately
    Atomics.store(buf.headerI32, idx, 0);
    return Promise.resolve();
  }
}

/**
 * Check if Rust has woken us without blocking.
 */
export function checkWakeFromRust(buf: SharedBuffer): boolean {
  const idx = H_WAKE_TS / 4;
  const value = Atomics.load(buf.headerI32, idx);
  if (value !== 0) {
    Atomics.store(buf.headerI32, idx, 0);
    return true;
  }
  return false;
}

// =============================================================================
// DIRTY FLAGS
// =============================================================================

export function markDirty(buf: SharedBuffer, nodeIndex: number, flags: number): void {
  const current = getU8(buf, nodeIndex, U_DIRTY_FLAGS);
  setU8(buf, nodeIndex, U_DIRTY_FLAGS, current | flags);
}

export function clearDirty(buf: SharedBuffer, nodeIndex: number, flags: number): void {
  const current = getU8(buf, nodeIndex, U_DIRTY_FLAGS);
  setU8(buf, nodeIndex, U_DIRTY_FLAGS, current & ~flags);
}

export function clearAllDirty(buf: SharedBuffer, nodeIndex: number): void {
  setU8(buf, nodeIndex, U_DIRTY_FLAGS, 0);
}

export function isDirty(buf: SharedBuffer, nodeIndex: number, flags: number): boolean {
  return (getU8(buf, nodeIndex, U_DIRTY_FLAGS) & flags) !== 0;
}

export function getDirtyFlags(buf: SharedBuffer, nodeIndex: number): number {
  return getU8(buf, nodeIndex, U_DIRTY_FLAGS);
}

// =============================================================================
// INTERACTION FLAGS
// =============================================================================

export function getInteractionFlags(buf: SharedBuffer, nodeIndex: number): number {
  return getU8(buf, nodeIndex, U_INTERACTION_FLAGS);
}

export function setInteractionFlags(buf: SharedBuffer, nodeIndex: number, flags: number): void {
  setU8(buf, nodeIndex, U_INTERACTION_FLAGS, flags);
}

export function isFocusable(buf: SharedBuffer, nodeIndex: number): boolean {
  return (getInteractionFlags(buf, nodeIndex) & FLAG_FOCUSABLE) !== 0;
}

export function setFocusable(buf: SharedBuffer, nodeIndex: number, value: boolean): void {
  const flags = getInteractionFlags(buf, nodeIndex);
  setInteractionFlags(buf, nodeIndex, value ? flags | FLAG_FOCUSABLE : flags & ~FLAG_FOCUSABLE);
}

export function isFocused(buf: SharedBuffer, nodeIndex: number): boolean {
  return (getInteractionFlags(buf, nodeIndex) & FLAG_FOCUSED) !== 0;
}

export function isHovered(buf: SharedBuffer, nodeIndex: number): boolean {
  return (getInteractionFlags(buf, nodeIndex) & FLAG_HOVERED) !== 0;
}

export function isPressed(buf: SharedBuffer, nodeIndex: number): boolean {
  return (getInteractionFlags(buf, nodeIndex) & FLAG_PRESSED) !== 0;
}

export function isDisabled(buf: SharedBuffer, nodeIndex: number): boolean {
  return (getInteractionFlags(buf, nodeIndex) & FLAG_DISABLED) !== 0;
}

export function setDisabled(buf: SharedBuffer, nodeIndex: number, value: boolean): void {
  const flags = getInteractionFlags(buf, nodeIndex);
  setInteractionFlags(buf, nodeIndex, value ? flags | FLAG_DISABLED : flags & ~FLAG_DISABLED);
}

// =============================================================================
// HIERARCHY
// =============================================================================

export function getParentIndex(buf: SharedBuffer, nodeIndex: number): number {
  return getI32(buf, nodeIndex, I_PARENT_INDEX);
}

export function setParentIndex(buf: SharedBuffer, nodeIndex: number, parentIndex: number): void {
  setI32(buf, nodeIndex, I_PARENT_INDEX, parentIndex);
  markDirty(buf, nodeIndex, DIRTY_HIERARCHY);
}

export function getTabIndex(buf: SharedBuffer, nodeIndex: number): number {
  return getI32(buf, nodeIndex, I_TAB_INDEX);
}

export function setTabIndex(buf: SharedBuffer, nodeIndex: number, tabIndex: number): void {
  setI32(buf, nodeIndex, I_TAB_INDEX, tabIndex);
}

export function getChildCount(buf: SharedBuffer, nodeIndex: number): number {
  return getI32(buf, nodeIndex, I_CHILD_COUNT);
}

export function setChildCount(buf: SharedBuffer, nodeIndex: number, count: number): void {
  setI32(buf, nodeIndex, I_CHILD_COUNT, count);
}

// =============================================================================
// TEXT POOL
// =============================================================================

const textEncoder = new TextEncoder();
const textDecoder = new TextDecoder();

/**
 * Get the current text pool write pointer.
 */
export function getTextPoolWritePtr(buf: SharedBuffer): number {
  return buf.view.getUint32(H_TEXT_POOL_WRITE_PTR, true);
}

/**
 * Get remaining space in text pool.
 */
export function getTextPoolRemaining(buf: SharedBuffer): number {
  return buf.textPoolSize - getTextPoolWritePtr(buf);
}

/**
 * Write text to a node, allocating from the text pool.
 * Returns false if text pool is full.
 */
export function setText(buf: SharedBuffer, nodeIndex: number, text: string): boolean {
  const encoded = textEncoder.encode(text);
  const writePtr = getTextPoolWritePtr(buf);

  if (writePtr + encoded.length > buf.textPoolSize) {
    return false; // Pool full
  }

  // Write text to pool
  const poolView = new Uint8Array(buf.raw, buf.textPoolOffset + writePtr, encoded.length);
  poolView.set(encoded);

  // Update node's offset and length
  setU32(buf, nodeIndex, U_TEXT_OFFSET, writePtr);
  setU32(buf, nodeIndex, U_TEXT_LENGTH, encoded.length);

  // Update pool write pointer
  buf.view.setUint32(H_TEXT_POOL_WRITE_PTR, writePtr + encoded.length, true);

  // Mark dirty
  markDirty(buf, nodeIndex, DIRTY_TEXT);

  return true;
}

/**
 * Get text content for a node.
 */
export function getText(buf: SharedBuffer, nodeIndex: number): string {
  const offset = getU32(buf, nodeIndex, U_TEXT_OFFSET);
  const length = getU32(buf, nodeIndex, U_TEXT_LENGTH);

  if (length === 0) {
    return '';
  }

  const poolView = new Uint8Array(buf.raw, buf.textPoolOffset + offset, length);
  return textDecoder.decode(poolView);
}

/**
 * Reset text pool write pointer.
 * WARNING: Invalidates all existing text references!
 */
export function resetTextPool(buf: SharedBuffer): void {
  buf.view.setUint32(H_TEXT_POOL_WRITE_PTR, 0, true);
}

// =============================================================================
// EVENT RING
// =============================================================================

/**
 * Get number of pending events.
 */
export function getEventCount(buf: SharedBuffer): number {
  const write = buf.view.getUint32(H_EVENT_WRITE_IDX, true);
  const read = buf.view.getUint32(H_EVENT_READ_IDX, true);
  return write - read;
}

/**
 * Check if there are pending events.
 */
export function hasEvents(buf: SharedBuffer): boolean {
  return getEventCount(buf) > 0;
}

/**
 * Get the next event without consuming it.
 * Returns null if no events pending.
 */
export function peekEvent(buf: SharedBuffer): Event | null {
  if (!hasEvents(buf)) {
    return null;
  }

  const read = buf.view.getUint32(H_EVENT_READ_IDX, true);
  const slot = read % MAX_EVENTS;
  const offset = buf.eventRingOffset + EVENT_RING_HEADER_SIZE + slot * EVENT_SLOT_SIZE;

  return readEventAt(buf, offset);
}

/**
 * Consume and return the next event.
 * Returns null if no events pending.
 */
export function consumeEvent(buf: SharedBuffer): Event | null {
  if (!hasEvents(buf)) {
    return null;
  }

  const read = buf.view.getUint32(H_EVENT_READ_IDX, true);
  const slot = read % MAX_EVENTS;
  const offset = buf.eventRingOffset + EVENT_RING_HEADER_SIZE + slot * EVENT_SLOT_SIZE;

  const event = readEventAt(buf, offset);

  // Increment read index
  buf.view.setUint32(H_EVENT_READ_IDX, read + 1, true);

  return event;
}

/**
 * Consume all pending events.
 */
export function consumeAllEvents(buf: SharedBuffer): Event[] {
  const events: Event[] = [];
  let event: Event | null;
  while ((event = consumeEvent(buf)) !== null) {
    events.push(event);
  }
  return events;
}

export interface Event {
  type: number;
  componentIndex: number;
  data: Uint8Array;
}

function readEventAt(buf: SharedBuffer, offset: number): Event {
  const v = buf.view;
  return {
    type: v.getUint8(offset),
    componentIndex: v.getUint16(offset + 2, true),
    data: new Uint8Array(buf.raw, offset + 4, 16),
  };
}

// =============================================================================
// COLOR HELPERS
// =============================================================================

/**
 * Pack RGBA components into a single u32 (ARGB format).
 */
export function packColor(r: number, g: number, b: number, a: number = 255): number {
  return ((a & 0xff) << 24) | ((r & 0xff) << 16) | ((g & 0xff) << 8) | (b & 0xff);
}

/**
 * Unpack a u32 color into RGBA components.
 */
export function unpackColor(packed: number): { r: number; g: number; b: number; a: number } {
  return {
    a: (packed >>> 24) & 0xff,
    r: (packed >>> 16) & 0xff,
    g: (packed >>> 8) & 0xff,
    b: packed & 0xff,
  };
}

/**
 * Check if a color is transparent (alpha = 0).
 */
export function isTransparent(color: number): boolean {
  return (color >>> 24) === 0;
}

// =============================================================================
// CONVENIENCE SETTERS WITH DIRTY FLAGS
// =============================================================================

// --- Layout properties (mark DIRTY_LAYOUT) ---

export function setWidth(buf: SharedBuffer, nodeIndex: number, value: number): void {
  setF32(buf, nodeIndex, F_WIDTH, value);
  markDirty(buf, nodeIndex, DIRTY_LAYOUT);
}

export function setHeight(buf: SharedBuffer, nodeIndex: number, value: number): void {
  setF32(buf, nodeIndex, F_HEIGHT, value);
  markDirty(buf, nodeIndex, DIRTY_LAYOUT);
}

export function setMinWidth(buf: SharedBuffer, nodeIndex: number, value: number): void {
  setF32(buf, nodeIndex, F_MIN_WIDTH, value);
  markDirty(buf, nodeIndex, DIRTY_LAYOUT);
}

export function setMinHeight(buf: SharedBuffer, nodeIndex: number, value: number): void {
  setF32(buf, nodeIndex, F_MIN_HEIGHT, value);
  markDirty(buf, nodeIndex, DIRTY_LAYOUT);
}

export function setMaxWidth(buf: SharedBuffer, nodeIndex: number, value: number): void {
  setF32(buf, nodeIndex, F_MAX_WIDTH, value);
  markDirty(buf, nodeIndex, DIRTY_LAYOUT);
}

export function setMaxHeight(buf: SharedBuffer, nodeIndex: number, value: number): void {
  setF32(buf, nodeIndex, F_MAX_HEIGHT, value);
  markDirty(buf, nodeIndex, DIRTY_LAYOUT);
}

export function setFlexGrow(buf: SharedBuffer, nodeIndex: number, value: number): void {
  setF32(buf, nodeIndex, F_FLEX_GROW, value);
  markDirty(buf, nodeIndex, DIRTY_LAYOUT);
}

export function setFlexShrink(buf: SharedBuffer, nodeIndex: number, value: number): void {
  setF32(buf, nodeIndex, F_FLEX_SHRINK, value);
  markDirty(buf, nodeIndex, DIRTY_LAYOUT);
}

export function setFlexBasis(buf: SharedBuffer, nodeIndex: number, value: number): void {
  setF32(buf, nodeIndex, F_FLEX_BASIS, value);
  markDirty(buf, nodeIndex, DIRTY_LAYOUT);
}

export function setPadding(buf: SharedBuffer, nodeIndex: number, top: number, right: number, bottom: number, left: number): void {
  setF32(buf, nodeIndex, F_PADDING_TOP, top);
  setF32(buf, nodeIndex, F_PADDING_RIGHT, right);
  setF32(buf, nodeIndex, F_PADDING_BOTTOM, bottom);
  setF32(buf, nodeIndex, F_PADDING_LEFT, left);
  markDirty(buf, nodeIndex, DIRTY_LAYOUT);
}

export function setMargin(buf: SharedBuffer, nodeIndex: number, top: number, right: number, bottom: number, left: number): void {
  setF32(buf, nodeIndex, F_MARGIN_TOP, top);
  setF32(buf, nodeIndex, F_MARGIN_RIGHT, right);
  setF32(buf, nodeIndex, F_MARGIN_BOTTOM, bottom);
  setF32(buf, nodeIndex, F_MARGIN_LEFT, left);
  markDirty(buf, nodeIndex, DIRTY_LAYOUT);
}

export function setGap(buf: SharedBuffer, nodeIndex: number, value: number): void {
  setF32(buf, nodeIndex, F_GAP, value);
  markDirty(buf, nodeIndex, DIRTY_LAYOUT);
}

export function setFlexDirection(buf: SharedBuffer, nodeIndex: number, value: FlexDirection): void {
  setU8(buf, nodeIndex, U_FLEX_DIRECTION, value);
  markDirty(buf, nodeIndex, DIRTY_LAYOUT);
}

export function setFlexWrap(buf: SharedBuffer, nodeIndex: number, value: FlexWrap): void {
  setU8(buf, nodeIndex, U_FLEX_WRAP, value);
  markDirty(buf, nodeIndex, DIRTY_LAYOUT);
}

export function setJustifyContent(buf: SharedBuffer, nodeIndex: number, value: JustifyContent): void {
  setU8(buf, nodeIndex, U_JUSTIFY_CONTENT, value);
  markDirty(buf, nodeIndex, DIRTY_LAYOUT);
}

export function setAlignItems(buf: SharedBuffer, nodeIndex: number, value: AlignItems): void {
  setU8(buf, nodeIndex, U_ALIGN_ITEMS, value);
  markDirty(buf, nodeIndex, DIRTY_LAYOUT);
}

export function setPosition(buf: SharedBuffer, nodeIndex: number, value: Position): void {
  setU8(buf, nodeIndex, U_POSITION, value);
  markDirty(buf, nodeIndex, DIRTY_LAYOUT);
}

export function setOverflow(buf: SharedBuffer, nodeIndex: number, value: Overflow): void {
  setU8(buf, nodeIndex, U_OVERFLOW, value);
  markDirty(buf, nodeIndex, DIRTY_LAYOUT);
}

export function setDisplay(buf: SharedBuffer, nodeIndex: number, value: Display): void {
  setU8(buf, nodeIndex, U_DISPLAY, value);
  markDirty(buf, nodeIndex, DIRTY_LAYOUT);
}

export function setBorderWidth(buf: SharedBuffer, nodeIndex: number, top: number, right: number, bottom: number, left: number): void {
  setU8(buf, nodeIndex, U_BORDER_WIDTH_TOP, top);
  setU8(buf, nodeIndex, U_BORDER_WIDTH_RIGHT, right);
  setU8(buf, nodeIndex, U_BORDER_WIDTH_BOTTOM, bottom);
  setU8(buf, nodeIndex, U_BORDER_WIDTH_LEFT, left);
  markDirty(buf, nodeIndex, DIRTY_LAYOUT);
}

// --- Visual properties (mark DIRTY_VISUAL) ---

export function setFgColor(buf: SharedBuffer, nodeIndex: number, color: number): void {
  setU32(buf, nodeIndex, C_FG_COLOR, color);
  markDirty(buf, nodeIndex, DIRTY_VISUAL);
}

export function setBgColor(buf: SharedBuffer, nodeIndex: number, color: number): void {
  setU32(buf, nodeIndex, C_BG_COLOR, color);
  markDirty(buf, nodeIndex, DIRTY_VISUAL);
}

export function setBorderColor(buf: SharedBuffer, nodeIndex: number, color: number): void {
  setU32(buf, nodeIndex, C_BORDER_COLOR, color);
  markDirty(buf, nodeIndex, DIRTY_VISUAL);
}

export function setBorderStyle(buf: SharedBuffer, nodeIndex: number, style: number): void {
  setU8(buf, nodeIndex, U_BORDER_STYLE, style);
  markDirty(buf, nodeIndex, DIRTY_VISUAL);
}

export function setOpacity(buf: SharedBuffer, nodeIndex: number, value: number): void {
  setU8(buf, nodeIndex, U_OPACITY, Math.round(value * 255));
  markDirty(buf, nodeIndex, DIRTY_VISUAL);
}

export function setZIndex(buf: SharedBuffer, nodeIndex: number, value: number): void {
  setI8(buf, nodeIndex, I_Z_INDEX, value);
  markDirty(buf, nodeIndex, DIRTY_VISUAL);
}

export function setVisible(buf: SharedBuffer, nodeIndex: number, value: boolean): void {
  setU8(buf, nodeIndex, U_VISIBLE, value ? 1 : 0);
  markDirty(buf, nodeIndex, DIRTY_VISUAL);
}

// --- Component type ---

export function setComponentType(buf: SharedBuffer, nodeIndex: number, type: number): void {
  setU8(buf, nodeIndex, U_COMPONENT_TYPE, type);
}

export function getComponentType(buf: SharedBuffer, nodeIndex: number): number {
  return getU8(buf, nodeIndex, U_COMPONENT_TYPE);
}

// --- Scroll ---

export function setScroll(buf: SharedBuffer, nodeIndex: number, x: number, y: number): void {
  setI32(buf, nodeIndex, I_SCROLL_X, x);
  setI32(buf, nodeIndex, I_SCROLL_Y, y);
}

export function getScrollX(buf: SharedBuffer, nodeIndex: number): number {
  return getI32(buf, nodeIndex, I_SCROLL_X);
}

export function getScrollY(buf: SharedBuffer, nodeIndex: number): number {
  return getI32(buf, nodeIndex, I_SCROLL_Y);
}

// --- Output (Rust writes, TS reads) ---

export function getComputedX(buf: SharedBuffer, nodeIndex: number): number {
  return getF32(buf, nodeIndex, F_COMPUTED_X);
}

export function getComputedY(buf: SharedBuffer, nodeIndex: number): number {
  return getF32(buf, nodeIndex, F_COMPUTED_Y);
}

export function getComputedWidth(buf: SharedBuffer, nodeIndex: number): number {
  return getF32(buf, nodeIndex, F_COMPUTED_WIDTH);
}

export function getComputedHeight(buf: SharedBuffer, nodeIndex: number): number {
  return getF32(buf, nodeIndex, F_COMPUTED_HEIGHT);
}

export function getMaxScrollX(buf: SharedBuffer, nodeIndex: number): number {
  return getF32(buf, nodeIndex, F_MAX_SCROLL_X);
}

export function getMaxScrollY(buf: SharedBuffer, nodeIndex: number): number {
  return getF32(buf, nodeIndex, F_MAX_SCROLL_Y);
}

// =============================================================================
// EXPORTS FOR BACKWARDS COMPATIBILITY
// =============================================================================

// Re-export common patterns
export type { SharedBuffer as AoSBuffer };
export { createSharedBuffer as createAoSBuffer };
