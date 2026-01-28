/**
 * SparkTUI - Reactive Shared Arrays
 *
 * Creates SharedSlotBuffers backed by the SoA SharedArrayBuffer views.
 * Each field (width, height, fgColor, etc.) gets its own SharedSlotBuffer
 * that participates in the reactive graph.
 *
 * These replace the old engine/arrays/ typedSlotArrayGroup for the bridge layer.
 * Developer-facing primitives use repeat() to connect signals to these buffers.
 */

import { sharedSlotBuffer } from '@rlabs-inc/signals'
import type { SharedSlotBuffer, Notifier } from '@rlabs-inc/signals'
import type { SharedBufferViews } from './shared-buffer'
import {
  F32_FIELD_COUNT, U32_FIELD_COUNT, I32_FIELD_COUNT, U8_FIELD_COUNT,
  // Float32 field indices
  F32_WIDTH, F32_HEIGHT, F32_MIN_WIDTH, F32_MAX_WIDTH,
  F32_MIN_HEIGHT, F32_MAX_HEIGHT, F32_GROW, F32_SHRINK, F32_BASIS,
  F32_GAP, F32_PADDING_TOP, F32_PADDING_RIGHT, F32_PADDING_BOTTOM, F32_PADDING_LEFT,
  F32_MARGIN_TOP, F32_MARGIN_RIGHT, F32_MARGIN_BOTTOM, F32_MARGIN_LEFT,
  F32_INSET_TOP, F32_INSET_RIGHT, F32_INSET_BOTTOM, F32_INSET_LEFT,
  F32_ROW_GAP, F32_COLUMN_GAP,
  F32_COMPUTED_X, F32_COMPUTED_Y, F32_COMPUTED_WIDTH, F32_COMPUTED_HEIGHT,
  F32_SCROLLABLE, F32_MAX_SCROLL_X, F32_MAX_SCROLL_Y,
  // Uint32 field indices
  U32_FG_COLOR, U32_BG_COLOR, U32_BORDER_COLOR,
  U32_BORDER_COLOR_TOP, U32_BORDER_COLOR_RIGHT, U32_BORDER_COLOR_BOTTOM, U32_BORDER_COLOR_LEFT,
  U32_FOCUS_RING_COLOR, U32_CURSOR_FG_COLOR, U32_CURSOR_BG_COLOR,
  U32_TEXT_OFFSET, U32_TEXT_LENGTH,
  // Int32 field indices
  I32_PARENT_INDEX, I32_SCROLL_X, I32_SCROLL_Y, I32_TAB_INDEX,
  I32_CURSOR_POS, I32_SELECTION_START, I32_SELECTION_END,
  I32_CURSOR_CHAR, I32_CURSOR_ALT_CHAR, I32_CURSOR_BLINK_FPS,
  I32_HOVERED, I32_PRESSED, I32_CURSOR_VISIBLE,
  // Uint8 field indices
  U8_COMPONENT_TYPE, U8_VISIBLE, U8_FLEX_DIRECTION, U8_FLEX_WRAP,
  U8_JUSTIFY_CONTENT, U8_ALIGN_ITEMS, U8_ALIGN_SELF, U8_ALIGN_CONTENT,
  U8_OVERFLOW, U8_POSITION,
  U8_BORDER_TOP_WIDTH, U8_BORDER_RIGHT_WIDTH, U8_BORDER_BOTTOM_WIDTH, U8_BORDER_LEFT_WIDTH,
  U8_BORDER_STYLE, U8_BORDER_STYLE_TOP, U8_BORDER_STYLE_RIGHT, U8_BORDER_STYLE_BOTTOM, U8_BORDER_STYLE_LEFT,
  U8_SHOW_FOCUS_RING, U8_OPACITY, U8_Z_INDEX,
  U8_TEXT_ATTRS, U8_TEXT_ALIGN, U8_TEXT_WRAP, U8_ELLIPSIS_MODE,
  U8_FOCUSABLE, U8_MOUSE_ENABLED,
} from './shared-buffer'

// =============================================================================
// REACTIVE ARRAYS — SharedSlotBuffers backed by SharedArrayBuffer
// =============================================================================

export interface ReactiveArrays {
  // --- Float32 layout input ---
  width: SharedSlotBuffer
  height: SharedSlotBuffer
  minWidth: SharedSlotBuffer
  maxWidth: SharedSlotBuffer
  minHeight: SharedSlotBuffer
  maxHeight: SharedSlotBuffer
  grow: SharedSlotBuffer
  shrink: SharedSlotBuffer
  basis: SharedSlotBuffer
  gap: SharedSlotBuffer
  paddingTop: SharedSlotBuffer
  paddingRight: SharedSlotBuffer
  paddingBottom: SharedSlotBuffer
  paddingLeft: SharedSlotBuffer
  marginTop: SharedSlotBuffer
  marginRight: SharedSlotBuffer
  marginBottom: SharedSlotBuffer
  marginLeft: SharedSlotBuffer
  insetTop: SharedSlotBuffer
  insetRight: SharedSlotBuffer
  insetBottom: SharedSlotBuffer
  insetLeft: SharedSlotBuffer
  rowGap: SharedSlotBuffer
  columnGap: SharedSlotBuffer

  // --- Float32 layout output (Rust writes, TS reads) ---
  computedX: SharedSlotBuffer
  computedY: SharedSlotBuffer
  computedWidth: SharedSlotBuffer
  computedHeight: SharedSlotBuffer
  scrollable: SharedSlotBuffer
  maxScrollX: SharedSlotBuffer
  maxScrollY: SharedSlotBuffer

  // --- Uint32 colors ---
  fgColor: SharedSlotBuffer
  bgColor: SharedSlotBuffer
  borderColor: SharedSlotBuffer
  borderColorTop: SharedSlotBuffer
  borderColorRight: SharedSlotBuffer
  borderColorBottom: SharedSlotBuffer
  borderColorLeft: SharedSlotBuffer
  focusRingColor: SharedSlotBuffer
  cursorFgColor: SharedSlotBuffer
  cursorBgColor: SharedSlotBuffer

  // --- Uint32 text index ---
  textOffset: SharedSlotBuffer
  textLength: SharedSlotBuffer

  // --- Int32 hierarchy ---
  parentIndex: SharedSlotBuffer

  // --- Int32 interaction ---
  scrollX: SharedSlotBuffer
  scrollY: SharedSlotBuffer
  tabIndex: SharedSlotBuffer
  cursorPos: SharedSlotBuffer
  selectionStart: SharedSlotBuffer
  selectionEnd: SharedSlotBuffer
  cursorChar: SharedSlotBuffer
  cursorAltChar: SharedSlotBuffer
  cursorBlinkFps: SharedSlotBuffer
  hovered: SharedSlotBuffer
  pressed: SharedSlotBuffer
  cursorVisible: SharedSlotBuffer

  // --- Uint8 metadata ---
  componentType: SharedSlotBuffer
  visible: SharedSlotBuffer
  flexDirection: SharedSlotBuffer
  flexWrap: SharedSlotBuffer
  justifyContent: SharedSlotBuffer
  alignItems: SharedSlotBuffer
  alignSelf: SharedSlotBuffer
  alignContent: SharedSlotBuffer
  overflow: SharedSlotBuffer
  position: SharedSlotBuffer
  borderTopWidth: SharedSlotBuffer
  borderRightWidth: SharedSlotBuffer
  borderBottomWidth: SharedSlotBuffer
  borderLeftWidth: SharedSlotBuffer
  borderStyle: SharedSlotBuffer
  borderStyleTop: SharedSlotBuffer
  borderStyleRight: SharedSlotBuffer
  borderStyleBottom: SharedSlotBuffer
  borderStyleLeft: SharedSlotBuffer
  showFocusRing: SharedSlotBuffer
  opacity: SharedSlotBuffer
  zIndex: SharedSlotBuffer
  textAttrs: SharedSlotBuffer
  textAlign: SharedSlotBuffer
  textWrap: SharedSlotBuffer
  ellipsisMode: SharedSlotBuffer
  focusable: SharedSlotBuffer
  mouseEnabled: SharedSlotBuffer

  // --- Raw indexed access ---
  f32: SharedSlotBuffer[]
  u32: SharedSlotBuffer[]
  i32: SharedSlotBuffer[]
  u8: SharedSlotBuffer[]
}

/**
 * Create reactive SharedSlotBuffers backed by the SharedArrayBuffer views.
 *
 * Each buffer participates in the reactive graph:
 * - get(index) tracks dependency
 * - set(index, value) writes to shared memory + marks reactions dirty + notifies
 *
 * @param views - SoA SharedBufferViews from createSharedBuffer()
 * @param notifier - Notifier for cross-side wake (from createWakeNotifier)
 */
export function createReactiveArrays(views: SharedBufferViews, notifier: Notifier): ReactiveArrays {
  // Create SharedSlotBuffers for each field
  const f32: SharedSlotBuffer[] = new Array(F32_FIELD_COUNT)
  for (let i = 0; i < F32_FIELD_COUNT; i++) {
    f32[i] = sharedSlotBuffer({ buffer: views.f32[i], notifier })
  }

  const u32: SharedSlotBuffer[] = new Array(U32_FIELD_COUNT)
  for (let i = 0; i < U32_FIELD_COUNT; i++) {
    u32[i] = sharedSlotBuffer({ buffer: views.u32[i], notifier })
  }

  const i32: SharedSlotBuffer[] = new Array(I32_FIELD_COUNT)
  for (let i = 0; i < I32_FIELD_COUNT; i++) {
    i32[i] = sharedSlotBuffer({ buffer: views.i32[i], notifier })
  }

  const u8: SharedSlotBuffer[] = new Array(U8_FIELD_COUNT)
  for (let i = 0; i < U8_FIELD_COUNT; i++) {
    u8[i] = sharedSlotBuffer({ buffer: views.u8[i], notifier })
  }

  return {
    // Float32 layout input
    width: f32[F32_WIDTH],
    height: f32[F32_HEIGHT],
    minWidth: f32[F32_MIN_WIDTH],
    maxWidth: f32[F32_MAX_WIDTH],
    minHeight: f32[F32_MIN_HEIGHT],
    maxHeight: f32[F32_MAX_HEIGHT],
    grow: f32[F32_GROW],
    shrink: f32[F32_SHRINK],
    basis: f32[F32_BASIS],
    gap: f32[F32_GAP],
    paddingTop: f32[F32_PADDING_TOP],
    paddingRight: f32[F32_PADDING_RIGHT],
    paddingBottom: f32[F32_PADDING_BOTTOM],
    paddingLeft: f32[F32_PADDING_LEFT],
    marginTop: f32[F32_MARGIN_TOP],
    marginRight: f32[F32_MARGIN_RIGHT],
    marginBottom: f32[F32_MARGIN_BOTTOM],
    marginLeft: f32[F32_MARGIN_LEFT],
    insetTop: f32[F32_INSET_TOP],
    insetRight: f32[F32_INSET_RIGHT],
    insetBottom: f32[F32_INSET_BOTTOM],
    insetLeft: f32[F32_INSET_LEFT],
    rowGap: f32[F32_ROW_GAP],
    columnGap: f32[F32_COLUMN_GAP],

    // Float32 layout output
    computedX: f32[F32_COMPUTED_X],
    computedY: f32[F32_COMPUTED_Y],
    computedWidth: f32[F32_COMPUTED_WIDTH],
    computedHeight: f32[F32_COMPUTED_HEIGHT],
    scrollable: f32[F32_SCROLLABLE],
    maxScrollX: f32[F32_MAX_SCROLL_X],
    maxScrollY: f32[F32_MAX_SCROLL_Y],

    // Uint32 colors
    fgColor: u32[U32_FG_COLOR],
    bgColor: u32[U32_BG_COLOR],
    borderColor: u32[U32_BORDER_COLOR],
    borderColorTop: u32[U32_BORDER_COLOR_TOP],
    borderColorRight: u32[U32_BORDER_COLOR_RIGHT],
    borderColorBottom: u32[U32_BORDER_COLOR_BOTTOM],
    borderColorLeft: u32[U32_BORDER_COLOR_LEFT],
    focusRingColor: u32[U32_FOCUS_RING_COLOR],
    cursorFgColor: u32[U32_CURSOR_FG_COLOR],
    cursorBgColor: u32[U32_CURSOR_BG_COLOR],

    // Uint32 text index
    textOffset: u32[U32_TEXT_OFFSET],
    textLength: u32[U32_TEXT_LENGTH],

    // Int32 hierarchy
    parentIndex: i32[I32_PARENT_INDEX],

    // Int32 interaction
    scrollX: i32[I32_SCROLL_X],
    scrollY: i32[I32_SCROLL_Y],
    tabIndex: i32[I32_TAB_INDEX],
    cursorPos: i32[I32_CURSOR_POS],
    selectionStart: i32[I32_SELECTION_START],
    selectionEnd: i32[I32_SELECTION_END],
    cursorChar: i32[I32_CURSOR_CHAR],
    cursorAltChar: i32[I32_CURSOR_ALT_CHAR],
    cursorBlinkFps: i32[I32_CURSOR_BLINK_FPS],
    hovered: i32[I32_HOVERED],
    pressed: i32[I32_PRESSED],
    cursorVisible: i32[I32_CURSOR_VISIBLE],

    // Uint8 metadata
    componentType: u8[U8_COMPONENT_TYPE],
    visible: u8[U8_VISIBLE],
    flexDirection: u8[U8_FLEX_DIRECTION],
    flexWrap: u8[U8_FLEX_WRAP],
    justifyContent: u8[U8_JUSTIFY_CONTENT],
    alignItems: u8[U8_ALIGN_ITEMS],
    alignSelf: u8[U8_ALIGN_SELF],
    alignContent: u8[U8_ALIGN_CONTENT],
    overflow: u8[U8_OVERFLOW],
    position: u8[U8_POSITION],
    borderTopWidth: u8[U8_BORDER_TOP_WIDTH],
    borderRightWidth: u8[U8_BORDER_RIGHT_WIDTH],
    borderBottomWidth: u8[U8_BORDER_BOTTOM_WIDTH],
    borderLeftWidth: u8[U8_BORDER_LEFT_WIDTH],
    borderStyle: u8[U8_BORDER_STYLE],
    borderStyleTop: u8[U8_BORDER_STYLE_TOP],
    borderStyleRight: u8[U8_BORDER_STYLE_RIGHT],
    borderStyleBottom: u8[U8_BORDER_STYLE_BOTTOM],
    borderStyleLeft: u8[U8_BORDER_STYLE_LEFT],
    showFocusRing: u8[U8_SHOW_FOCUS_RING],
    opacity: u8[U8_OPACITY],
    zIndex: u8[U8_Z_INDEX],
    textAttrs: u8[U8_TEXT_ATTRS],
    textAlign: u8[U8_TEXT_ALIGN],
    textWrap: u8[U8_TEXT_WRAP],
    ellipsisMode: u8[U8_ELLIPSIS_MODE],
    focusable: u8[U8_FOCUSABLE],
    mouseEnabled: u8[U8_MOUSE_ENABLED],

    // Raw indexed access
    f32, u32, i32, u8,
  }
}
