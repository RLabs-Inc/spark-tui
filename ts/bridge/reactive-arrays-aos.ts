/**
 * SparkTUI - Reactive AoS Arrays (Fast Path)
 *
 * Direct DataView writes, no Proxy overhead.
 * Each field maps to a slot buffer that writes directly to AoS memory.
 */

import type { Notifier, SharedSlotBuffer } from '@rlabs-inc/signals'
import type { AoSBuffer } from './shared-buffer-aos'
import { aosSlotBuffer } from './aos-slot-buffer'
import {
  // Layout floats
  F_WIDTH, F_HEIGHT, F_MIN_WIDTH, F_MIN_HEIGHT, F_MAX_WIDTH, F_MAX_HEIGHT,
  F_FLEX_BASIS, F_FLEX_GROW, F_FLEX_SHRINK,
  F_PADDING_TOP, F_PADDING_RIGHT, F_PADDING_BOTTOM, F_PADDING_LEFT,
  F_MARGIN_TOP, F_MARGIN_RIGHT, F_MARGIN_BOTTOM, F_MARGIN_LEFT,
  F_GAP, F_ROW_GAP, F_COLUMN_GAP,
  F_INSET_TOP, F_INSET_RIGHT, F_INSET_BOTTOM, F_INSET_LEFT,
  // Layout enums
  U_FLEX_DIRECTION, U_FLEX_WRAP, U_JUSTIFY_CONTENT, U_ALIGN_ITEMS,
  U_ALIGN_CONTENT, U_ALIGN_SELF, U_POSITION, U_OVERFLOW, U_DISPLAY,
  U_BORDER_TOP, U_BORDER_RIGHT, U_BORDER_BOTTOM, U_BORDER_LEFT,
  U_COMPONENT_TYPE, U_VISIBLE,
  // Visual
  C_FG_COLOR, C_BG_COLOR, C_BORDER_COLOR, C_FOCUS_RING_COLOR,
  C_CURSOR_FG, C_CURSOR_BG, C_SELECTION_COLOR, U_OPACITY, I_Z_INDEX,
  U_BORDER_STYLE, U_BORDER_STYLE_TOP, U_BORDER_STYLE_RIGHT,
  U_BORDER_STYLE_BOTTOM, U_BORDER_STYLE_LEFT,
  // Interaction
  I_SCROLL_X, I_SCROLL_Y, I_TAB_INDEX, I_CURSOR_POSITION,
  I_SELECTION_START, I_SELECTION_END, U_INTERACTION_FLAGS,
  // Cursor
  U_CURSOR_FLAGS, U_CURSOR_STYLE, U_CURSOR_CHAR,
  // Hierarchy
  I_PARENT_INDEX,
  // Text
  U_TEXT_OFFSET, U_TEXT_LENGTH, U_TEXT_ALIGN, U_TEXT_WRAP, U_TEXT_OVERFLOW,
  // Output
  F_COMPUTED_X, F_COMPUTED_Y, F_COMPUTED_WIDTH, F_COMPUTED_HEIGHT,
  F_SCROLL_WIDTH, F_SCROLL_HEIGHT, F_CONTENT_WIDTH,
  // Flags
  U_DIRTY_FLAGS,
} from './shared-buffer-aos'

// =============================================================================
// REACTIVE ARRAYS INTERFACE
// =============================================================================

export interface ReactiveArraysAoS {
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
  rowGap: SharedSlotBuffer
  columnGap: SharedSlotBuffer
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

  // --- Float32 layout output (Rust writes, TS reads) ---
  computedX: SharedSlotBuffer
  computedY: SharedSlotBuffer
  computedWidth: SharedSlotBuffer
  computedHeight: SharedSlotBuffer
  scrollWidth: SharedSlotBuffer
  scrollHeight: SharedSlotBuffer
  contentWidth: SharedSlotBuffer

  // --- Uint32 colors ---
  fgColor: SharedSlotBuffer
  bgColor: SharedSlotBuffer
  borderColor: SharedSlotBuffer
  focusRingColor: SharedSlotBuffer
  cursorFg: SharedSlotBuffer
  cursorBg: SharedSlotBuffer
  selectionColor: SharedSlotBuffer

  // --- Cursor ---
  cursorPosition: SharedSlotBuffer
  cursorFlags: SharedSlotBuffer
  cursorStyle: SharedSlotBuffer
  cursorChar: SharedSlotBuffer

  // --- Uint32 text index ---
  textOffset: SharedSlotBuffer
  textLength: SharedSlotBuffer

  // --- Int32 hierarchy ---
  parentIndex: SharedSlotBuffer

  // --- Int32 interaction ---
  scrollX: SharedSlotBuffer
  scrollY: SharedSlotBuffer
  tabIndex: SharedSlotBuffer
  selectionStart: SharedSlotBuffer
  selectionEnd: SharedSlotBuffer

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
  display: SharedSlotBuffer
  borderTopWidth: SharedSlotBuffer
  borderRightWidth: SharedSlotBuffer
  borderBottomWidth: SharedSlotBuffer
  borderLeftWidth: SharedSlotBuffer
  borderStyle: SharedSlotBuffer
  borderStyleTop: SharedSlotBuffer
  borderStyleRight: SharedSlotBuffer
  borderStyleBottom: SharedSlotBuffer
  borderStyleLeft: SharedSlotBuffer
  opacity: SharedSlotBuffer
  zIndex: SharedSlotBuffer
  textAlign: SharedSlotBuffer
  textWrap: SharedSlotBuffer
  textOverflow: SharedSlotBuffer
  interactionFlags: SharedSlotBuffer
  dirtyFlags: SharedSlotBuffer
}

// =============================================================================
// CREATE REACTIVE AOS ARRAYS
// =============================================================================

export function createReactiveArraysAoS(
  buf: AoSBuffer,
  notifier: Notifier
): ReactiveArraysAoS {
  const v = buf.view

  const f32 = (offset: number) => aosSlotBuffer(v, offset, 'f32', notifier)
  const u32 = (offset: number) => aosSlotBuffer(v, offset, 'u32', notifier)
  const i32 = (offset: number) => aosSlotBuffer(v, offset, 'i32', notifier)
  const u8 = (offset: number) => aosSlotBuffer(v, offset, 'u8', notifier)
  const i8 = (offset: number) => aosSlotBuffer(v, offset, 'i8', notifier)

  return {
    // Float32 layout input
    width: f32(F_WIDTH),
    height: f32(F_HEIGHT),
    minWidth: f32(F_MIN_WIDTH),
    maxWidth: f32(F_MAX_WIDTH),
    minHeight: f32(F_MIN_HEIGHT),
    maxHeight: f32(F_MAX_HEIGHT),
    grow: f32(F_FLEX_GROW),
    shrink: f32(F_FLEX_SHRINK),
    basis: f32(F_FLEX_BASIS),
    gap: f32(F_GAP),
    rowGap: f32(F_ROW_GAP),
    columnGap: f32(F_COLUMN_GAP),
    paddingTop: f32(F_PADDING_TOP),
    paddingRight: f32(F_PADDING_RIGHT),
    paddingBottom: f32(F_PADDING_BOTTOM),
    paddingLeft: f32(F_PADDING_LEFT),
    marginTop: f32(F_MARGIN_TOP),
    marginRight: f32(F_MARGIN_RIGHT),
    marginBottom: f32(F_MARGIN_BOTTOM),
    marginLeft: f32(F_MARGIN_LEFT),
    insetTop: f32(F_INSET_TOP),
    insetRight: f32(F_INSET_RIGHT),
    insetBottom: f32(F_INSET_BOTTOM),
    insetLeft: f32(F_INSET_LEFT),

    // Float32 layout output
    computedX: f32(F_COMPUTED_X),
    computedY: f32(F_COMPUTED_Y),
    computedWidth: f32(F_COMPUTED_WIDTH),
    computedHeight: f32(F_COMPUTED_HEIGHT),
    scrollWidth: f32(F_SCROLL_WIDTH),
    scrollHeight: f32(F_SCROLL_HEIGHT),
    contentWidth: f32(F_CONTENT_WIDTH),

    // Uint32 colors
    fgColor: u32(C_FG_COLOR),
    bgColor: u32(C_BG_COLOR),
    borderColor: u32(C_BORDER_COLOR),
    focusRingColor: u32(C_FOCUS_RING_COLOR),
    cursorFg: u32(C_CURSOR_FG),
    cursorBg: u32(C_CURSOR_BG),
    selectionColor: u32(C_SELECTION_COLOR),

    // Cursor
    cursorPosition: i32(I_CURSOR_POSITION),
    cursorFlags: u8(U_CURSOR_FLAGS),
    cursorStyle: u8(U_CURSOR_STYLE),
    cursorChar: u32(U_CURSOR_CHAR),

    // Uint32 text index
    textOffset: u32(U_TEXT_OFFSET),
    textLength: u32(U_TEXT_LENGTH),

    // Int32 hierarchy
    parentIndex: i32(I_PARENT_INDEX),

    // Int32 interaction
    scrollX: i32(I_SCROLL_X),
    scrollY: i32(I_SCROLL_Y),
    tabIndex: i32(I_TAB_INDEX),
    selectionStart: i32(I_SELECTION_START),
    selectionEnd: i32(I_SELECTION_END),

    // Uint8 metadata
    componentType: u8(U_COMPONENT_TYPE),
    visible: u8(U_VISIBLE),
    flexDirection: u8(U_FLEX_DIRECTION),
    flexWrap: u8(U_FLEX_WRAP),
    justifyContent: u8(U_JUSTIFY_CONTENT),
    alignItems: u8(U_ALIGN_ITEMS),
    alignSelf: u8(U_ALIGN_SELF),
    alignContent: u8(U_ALIGN_CONTENT),
    overflow: u8(U_OVERFLOW),
    position: u8(U_POSITION),
    display: u8(U_DISPLAY),
    borderTopWidth: u8(U_BORDER_TOP),
    borderRightWidth: u8(U_BORDER_RIGHT),
    borderBottomWidth: u8(U_BORDER_BOTTOM),
    borderLeftWidth: u8(U_BORDER_LEFT),
    borderStyle: u8(U_BORDER_STYLE),
    borderStyleTop: u8(U_BORDER_STYLE_TOP),
    borderStyleRight: u8(U_BORDER_STYLE_RIGHT),
    borderStyleBottom: u8(U_BORDER_STYLE_BOTTOM),
    borderStyleLeft: u8(U_BORDER_STYLE_LEFT),
    opacity: u8(U_OPACITY),
    zIndex: i8(I_Z_INDEX),
    textAlign: u8(U_TEXT_ALIGN),
    textWrap: u8(U_TEXT_WRAP),
    textOverflow: u8(U_TEXT_OVERFLOW),
    interactionFlags: u8(U_INTERACTION_FLAGS),
    dirtyFlags: u8(U_DIRTY_FLAGS),
  }
}
