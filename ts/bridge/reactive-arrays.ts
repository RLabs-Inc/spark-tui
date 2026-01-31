/**
 * SparkTUI - Reactive Arrays
 *
 * Direct DataView writes with reactive notification.
 * Each field maps to a slot buffer that writes directly to shared memory.
 *
 * Layout: 1024 bytes per node, 16 cache lines
 * Grid tracks are handled via helper functions in shared-buffer.ts,
 * not as reactive arrays (they're compound structures).
 */

import type { Notifier, SharedSlotBuffer } from '@rlabs-inc/signals'
import type { SharedBuffer } from './shared-buffer'
import { createSlotBuffer } from './slot-buffer'
import {
  // === Cache Line 1 (0-63): Core Layout Dimensions ===
  N_WIDTH, N_HEIGHT, N_MIN_WIDTH, N_MIN_HEIGHT, N_MAX_WIDTH, N_MAX_HEIGHT,
  N_ASPECT_RATIO, N_COMPONENT_TYPE, N_DISPLAY, N_POSITION, N_OVERFLOW,
  N_VISIBLE, N_BOX_SIZING, N_DIRTY_FLAGS,

  // === Cache Line 2 (64-127): Flexbox Properties ===
  N_FLEX_DIRECTION, N_FLEX_WRAP, N_JUSTIFY_CONTENT, N_ALIGN_ITEMS,
  N_ALIGN_CONTENT, N_ALIGN_SELF, N_FLEX_GROW, N_FLEX_SHRINK, N_FLEX_BASIS,
  N_GAP, N_ROW_GAP, N_COLUMN_GAP,

  // === Cache Line 3 (128-191): Spacing Properties ===
  N_PADDING_TOP, N_PADDING_RIGHT, N_PADDING_BOTTOM, N_PADDING_LEFT,
  N_MARGIN_TOP, N_MARGIN_RIGHT, N_MARGIN_BOTTOM, N_MARGIN_LEFT,
  N_INSET_TOP, N_INSET_RIGHT, N_INSET_BOTTOM, N_INSET_LEFT,
  N_BORDER_WIDTH_TOP, N_BORDER_WIDTH_RIGHT, N_BORDER_WIDTH_BOTTOM, N_BORDER_WIDTH_LEFT,
  N_PARENT_INDEX, N_TAB_INDEX,

  // === Cache Line 4 (192-255): Grid Container Properties ===
  N_GRID_AUTO_FLOW, N_JUSTIFY_ITEMS, N_GRID_COLUMN_COUNT, N_GRID_ROW_COUNT,
  N_GRID_AUTO_COLUMNS_TYPE, N_GRID_AUTO_ROWS_TYPE,
  N_GRID_AUTO_COLUMNS_VALUE, N_GRID_AUTO_ROWS_VALUE,
  N_GRID_COLUMN_START, N_GRID_COLUMN_END, N_GRID_ROW_START, N_GRID_ROW_END,
  N_JUSTIFY_SELF,

  // === Cache Lines 5-10: Grid Tracks ===
  // Handled via setGridColumnTracks()/setGridRowTracks() - not as slot buffers

  // === Cache Line 11 (640-703): Computed Output ===
  N_COMPUTED_X, N_COMPUTED_Y, N_COMPUTED_WIDTH, N_COMPUTED_HEIGHT,
  N_CONTENT_WIDTH, N_CONTENT_HEIGHT, N_MAX_SCROLL_X, N_MAX_SCROLL_Y,
  N_IS_SCROLLABLE,

  // === Cache Line 12 (704-767): Visual Properties ===
  N_OPACITY, N_Z_INDEX, N_BORDER_STYLE,
  N_BORDER_STYLE_TOP, N_BORDER_STYLE_RIGHT, N_BORDER_STYLE_BOTTOM, N_BORDER_STYLE_LEFT,
  N_SCROLLBAR_VISIBILITY,
  N_BORDER_CHAR_H, N_BORDER_CHAR_V,
  N_BORDER_CHAR_TL, N_BORDER_CHAR_TR, N_BORDER_CHAR_BL, N_BORDER_CHAR_BR,
  N_FOCUS_INDICATOR_CHAR, N_FOCUS_INDICATOR_ENABLED,

  // === Cache Line 13 (768-831): Colors ===
  N_FG_COLOR, N_BG_COLOR, N_BORDER_COLOR,
  N_BORDER_TOP_COLOR, N_BORDER_RIGHT_COLOR, N_BORDER_BOTTOM_COLOR, N_BORDER_LEFT_COLOR,
  N_FOCUS_RING_COLOR, N_CURSOR_FG_COLOR, N_CURSOR_BG_COLOR, N_SELECTION_COLOR,

  // === Cache Line 14 (832-895): Text Properties ===
  N_TEXT_OFFSET, N_TEXT_LENGTH, N_TEXT_ALIGN, N_TEXT_WRAP, N_TEXT_OVERFLOW,
  N_TEXT_ATTRS, N_TEXT_DECORATION, N_TEXT_DECORATION_STYLE, N_TEXT_DECORATION_COLOR,
  N_LINE_HEIGHT, N_LETTER_SPACING, N_MAX_LINES,

  // === Cache Line 15 (896-959): Interaction State ===
  N_SCROLL_X, N_SCROLL_Y, N_CURSOR_POSITION, N_SELECTION_START, N_SELECTION_END,
  N_CURSOR_CHAR, N_CURSOR_ALT_CHAR,
  N_INTERACTION_FLAGS, N_CURSOR_FLAGS, N_CURSOR_STYLE, N_CURSOR_BLINK_RATE,
  N_MAX_LENGTH, N_INPUT_TYPE,
} from './shared-buffer'

// =============================================================================
// REACTIVE ARRAYS INTERFACE
// =============================================================================

export interface ReactiveArrays {
  // === Cache Line 1: Core Layout Dimensions ===
  width: SharedSlotBuffer              // f32 @ 0
  height: SharedSlotBuffer             // f32 @ 4
  minWidth: SharedSlotBuffer           // f32 @ 8
  minHeight: SharedSlotBuffer          // f32 @ 12
  maxWidth: SharedSlotBuffer           // f32 @ 16
  maxHeight: SharedSlotBuffer          // f32 @ 20
  aspectRatio: SharedSlotBuffer        // f32 @ 24
  componentType: SharedSlotBuffer      // u8 @ 28
  display: SharedSlotBuffer            // u8 @ 29
  position: SharedSlotBuffer           // u8 @ 30
  overflow: SharedSlotBuffer           // u8 @ 31
  visible: SharedSlotBuffer            // u8 @ 32
  boxSizing: SharedSlotBuffer          // u8 @ 33
  dirtyFlags: SharedSlotBuffer         // u8 @ 34

  // === Cache Line 2: Flexbox Properties ===
  flexDirection: SharedSlotBuffer      // u8 @ 64
  flexWrap: SharedSlotBuffer           // u8 @ 65
  justifyContent: SharedSlotBuffer     // u8 @ 66
  alignItems: SharedSlotBuffer         // u8 @ 67
  alignContent: SharedSlotBuffer       // u8 @ 68
  alignSelf: SharedSlotBuffer          // u8 @ 69
  flexGrow: SharedSlotBuffer           // f32 @ 72
  flexShrink: SharedSlotBuffer         // f32 @ 76
  flexBasis: SharedSlotBuffer          // f32 @ 80
  gap: SharedSlotBuffer                // f32 @ 84
  rowGap: SharedSlotBuffer             // f32 @ 88
  columnGap: SharedSlotBuffer          // f32 @ 92

  // === Cache Line 3: Spacing Properties ===
  paddingTop: SharedSlotBuffer         // f32 @ 128
  paddingRight: SharedSlotBuffer       // f32 @ 132
  paddingBottom: SharedSlotBuffer      // f32 @ 136
  paddingLeft: SharedSlotBuffer        // f32 @ 140
  marginTop: SharedSlotBuffer          // f32 @ 144
  marginRight: SharedSlotBuffer        // f32 @ 148
  marginBottom: SharedSlotBuffer       // f32 @ 152
  marginLeft: SharedSlotBuffer         // f32 @ 156
  insetTop: SharedSlotBuffer           // f32 @ 160
  insetRight: SharedSlotBuffer         // f32 @ 164
  insetBottom: SharedSlotBuffer        // f32 @ 168
  insetLeft: SharedSlotBuffer          // f32 @ 172
  borderWidthTop: SharedSlotBuffer     // u8 @ 176
  borderWidthRight: SharedSlotBuffer   // u8 @ 177
  borderWidthBottom: SharedSlotBuffer  // u8 @ 178
  borderWidthLeft: SharedSlotBuffer    // u8 @ 179
  parentIndex: SharedSlotBuffer        // i32 @ 180
  tabIndex: SharedSlotBuffer           // i32 @ 184

  // === Cache Line 4: Grid Container Properties ===
  gridAutoFlow: SharedSlotBuffer       // u8 @ 192
  justifyItems: SharedSlotBuffer       // u8 @ 193
  gridColumnCount: SharedSlotBuffer    // u8 @ 194
  gridRowCount: SharedSlotBuffer       // u8 @ 195
  gridAutoColumnsType: SharedSlotBuffer  // u8 @ 196
  gridAutoRowsType: SharedSlotBuffer   // u8 @ 197
  gridAutoColumnsValue: SharedSlotBuffer // f32 @ 200
  gridAutoRowsValue: SharedSlotBuffer  // f32 @ 204
  gridColumnStart: SharedSlotBuffer    // i16 @ 208
  gridColumnEnd: SharedSlotBuffer      // i16 @ 210
  gridRowStart: SharedSlotBuffer       // i16 @ 212
  gridRowEnd: SharedSlotBuffer         // i16 @ 214
  justifySelf: SharedSlotBuffer        // u8 @ 216

  // === Cache Line 11: Computed Output ===
  computedX: SharedSlotBuffer          // f32 @ 640
  computedY: SharedSlotBuffer          // f32 @ 644
  computedWidth: SharedSlotBuffer      // f32 @ 648
  computedHeight: SharedSlotBuffer     // f32 @ 652
  contentWidth: SharedSlotBuffer       // f32 @ 656
  contentHeight: SharedSlotBuffer      // f32 @ 660
  maxScrollX: SharedSlotBuffer         // f32 @ 664
  maxScrollY: SharedSlotBuffer         // f32 @ 668
  isScrollable: SharedSlotBuffer       // u8 @ 672

  // === Cache Line 12: Visual Properties ===
  opacity: SharedSlotBuffer            // f32 @ 704
  zIndex: SharedSlotBuffer             // i32 @ 708
  borderStyle: SharedSlotBuffer        // u8 @ 712
  borderStyleTop: SharedSlotBuffer     // u8 @ 713
  borderStyleRight: SharedSlotBuffer   // u8 @ 714
  borderStyleBottom: SharedSlotBuffer  // u8 @ 715
  borderStyleLeft: SharedSlotBuffer    // u8 @ 716
  scrollbarVisibility: SharedSlotBuffer // u8 @ 717
  borderCharH: SharedSlotBuffer        // u16 @ 718
  borderCharV: SharedSlotBuffer        // u16 @ 720
  borderCharTL: SharedSlotBuffer       // u16 @ 722
  borderCharTR: SharedSlotBuffer       // u16 @ 724
  borderCharBL: SharedSlotBuffer       // u16 @ 726
  borderCharBR: SharedSlotBuffer       // u16 @ 728
  focusIndicatorChar: SharedSlotBuffer // u8 @ 730
  focusIndicatorEnabled: SharedSlotBuffer // u8 @ 731

  // === Cache Line 13: Colors ===
  fgColor: SharedSlotBuffer            // u32 @ 768
  bgColor: SharedSlotBuffer            // u32 @ 772
  borderColor: SharedSlotBuffer        // u32 @ 776
  borderTopColor: SharedSlotBuffer     // u32 @ 780
  borderRightColor: SharedSlotBuffer   // u32 @ 784
  borderBottomColor: SharedSlotBuffer  // u32 @ 788
  borderLeftColor: SharedSlotBuffer    // u32 @ 792
  focusRingColor: SharedSlotBuffer     // u32 @ 796
  cursorFgColor: SharedSlotBuffer      // u32 @ 800
  cursorBgColor: SharedSlotBuffer      // u32 @ 804
  selectionColor: SharedSlotBuffer     // u32 @ 808

  // === Cache Line 14: Text Properties ===
  textOffset: SharedSlotBuffer         // u32 @ 832
  textLength: SharedSlotBuffer         // u32 @ 836
  textAlign: SharedSlotBuffer          // u8 @ 840
  textWrap: SharedSlotBuffer           // u8 @ 841
  textOverflow: SharedSlotBuffer       // u8 @ 842
  textAttrs: SharedSlotBuffer          // u8 @ 843
  textDecoration: SharedSlotBuffer     // u8 @ 844
  textDecorationStyle: SharedSlotBuffer // u8 @ 845
  textDecorationColor: SharedSlotBuffer // u32 @ 848
  lineHeight: SharedSlotBuffer         // u8 @ 852
  letterSpacing: SharedSlotBuffer      // u8 @ 853
  maxLines: SharedSlotBuffer           // u8 @ 854

  // === Cache Line 15: Interaction State ===
  scrollX: SharedSlotBuffer            // i32 @ 896
  scrollY: SharedSlotBuffer            // i32 @ 900
  cursorPosition: SharedSlotBuffer     // i32 @ 904
  selectionStart: SharedSlotBuffer     // i32 @ 908
  selectionEnd: SharedSlotBuffer       // i32 @ 912
  cursorChar: SharedSlotBuffer         // u32 @ 916
  cursorAltChar: SharedSlotBuffer      // u32 @ 920
  interactionFlags: SharedSlotBuffer   // u8 @ 924
  cursorFlags: SharedSlotBuffer        // u8 @ 925
  cursorStyle: SharedSlotBuffer        // u8 @ 926
  cursorBlinkRate: SharedSlotBuffer    // u8 @ 927
  maxLength: SharedSlotBuffer          // u8 @ 928
  inputType: SharedSlotBuffer          // u8 @ 929
}

// =============================================================================
// CREATE REACTIVE ARRAYS
// =============================================================================

export function createReactiveArrays(
  buf: SharedBuffer,
  notifier: Notifier
): ReactiveArrays {
  const v = buf.view

  // Type-specific slot buffer creators
  const f32 = (offset: number) => createSlotBuffer(v, offset, 'f32', notifier)
  const u32 = (offset: number) => createSlotBuffer(v, offset, 'u32', notifier)
  const i32 = (offset: number) => createSlotBuffer(v, offset, 'i32', notifier)
  const u16 = (offset: number) => createSlotBuffer(v, offset, 'u16', notifier)
  const i16 = (offset: number) => createSlotBuffer(v, offset, 'i16', notifier)
  const u8 = (offset: number) => createSlotBuffer(v, offset, 'u8', notifier)

  return {
    // === Cache Line 1: Core Layout Dimensions ===
    width: f32(N_WIDTH),
    height: f32(N_HEIGHT),
    minWidth: f32(N_MIN_WIDTH),
    minHeight: f32(N_MIN_HEIGHT),
    maxWidth: f32(N_MAX_WIDTH),
    maxHeight: f32(N_MAX_HEIGHT),
    aspectRatio: f32(N_ASPECT_RATIO),
    componentType: u8(N_COMPONENT_TYPE),
    display: u8(N_DISPLAY),
    position: u8(N_POSITION),
    overflow: u8(N_OVERFLOW),
    visible: u8(N_VISIBLE),
    boxSizing: u8(N_BOX_SIZING),
    dirtyFlags: u8(N_DIRTY_FLAGS),

    // === Cache Line 2: Flexbox Properties ===
    flexDirection: u8(N_FLEX_DIRECTION),
    flexWrap: u8(N_FLEX_WRAP),
    justifyContent: u8(N_JUSTIFY_CONTENT),
    alignItems: u8(N_ALIGN_ITEMS),
    alignContent: u8(N_ALIGN_CONTENT),
    alignSelf: u8(N_ALIGN_SELF),
    flexGrow: f32(N_FLEX_GROW),
    flexShrink: f32(N_FLEX_SHRINK),
    flexBasis: f32(N_FLEX_BASIS),
    gap: f32(N_GAP),
    rowGap: f32(N_ROW_GAP),
    columnGap: f32(N_COLUMN_GAP),

    // === Cache Line 3: Spacing Properties ===
    paddingTop: f32(N_PADDING_TOP),
    paddingRight: f32(N_PADDING_RIGHT),
    paddingBottom: f32(N_PADDING_BOTTOM),
    paddingLeft: f32(N_PADDING_LEFT),
    marginTop: f32(N_MARGIN_TOP),
    marginRight: f32(N_MARGIN_RIGHT),
    marginBottom: f32(N_MARGIN_BOTTOM),
    marginLeft: f32(N_MARGIN_LEFT),
    insetTop: f32(N_INSET_TOP),
    insetRight: f32(N_INSET_RIGHT),
    insetBottom: f32(N_INSET_BOTTOM),
    insetLeft: f32(N_INSET_LEFT),
    borderWidthTop: u8(N_BORDER_WIDTH_TOP),
    borderWidthRight: u8(N_BORDER_WIDTH_RIGHT),
    borderWidthBottom: u8(N_BORDER_WIDTH_BOTTOM),
    borderWidthLeft: u8(N_BORDER_WIDTH_LEFT),
    parentIndex: i32(N_PARENT_INDEX),
    tabIndex: i32(N_TAB_INDEX),

    // === Cache Line 4: Grid Container Properties ===
    gridAutoFlow: u8(N_GRID_AUTO_FLOW),
    justifyItems: u8(N_JUSTIFY_ITEMS),
    gridColumnCount: u8(N_GRID_COLUMN_COUNT),
    gridRowCount: u8(N_GRID_ROW_COUNT),
    gridAutoColumnsType: u8(N_GRID_AUTO_COLUMNS_TYPE),
    gridAutoRowsType: u8(N_GRID_AUTO_ROWS_TYPE),
    gridAutoColumnsValue: f32(N_GRID_AUTO_COLUMNS_VALUE),
    gridAutoRowsValue: f32(N_GRID_AUTO_ROWS_VALUE),
    gridColumnStart: i16(N_GRID_COLUMN_START),
    gridColumnEnd: i16(N_GRID_COLUMN_END),
    gridRowStart: i16(N_GRID_ROW_START),
    gridRowEnd: i16(N_GRID_ROW_END),
    justifySelf: u8(N_JUSTIFY_SELF),

    // === Cache Line 11: Computed Output ===
    computedX: f32(N_COMPUTED_X),
    computedY: f32(N_COMPUTED_Y),
    computedWidth: f32(N_COMPUTED_WIDTH),
    computedHeight: f32(N_COMPUTED_HEIGHT),
    contentWidth: f32(N_CONTENT_WIDTH),
    contentHeight: f32(N_CONTENT_HEIGHT),
    maxScrollX: f32(N_MAX_SCROLL_X),
    maxScrollY: f32(N_MAX_SCROLL_Y),
    isScrollable: u8(N_IS_SCROLLABLE),

    // === Cache Line 12: Visual Properties ===
    opacity: f32(N_OPACITY),
    zIndex: i32(N_Z_INDEX),
    borderStyle: u8(N_BORDER_STYLE),
    borderStyleTop: u8(N_BORDER_STYLE_TOP),
    borderStyleRight: u8(N_BORDER_STYLE_RIGHT),
    borderStyleBottom: u8(N_BORDER_STYLE_BOTTOM),
    borderStyleLeft: u8(N_BORDER_STYLE_LEFT),
    scrollbarVisibility: u8(N_SCROLLBAR_VISIBILITY),
    borderCharH: u16(N_BORDER_CHAR_H),
    borderCharV: u16(N_BORDER_CHAR_V),
    borderCharTL: u16(N_BORDER_CHAR_TL),
    borderCharTR: u16(N_BORDER_CHAR_TR),
    borderCharBL: u16(N_BORDER_CHAR_BL),
    borderCharBR: u16(N_BORDER_CHAR_BR),
    focusIndicatorChar: u8(N_FOCUS_INDICATOR_CHAR),
    focusIndicatorEnabled: u8(N_FOCUS_INDICATOR_ENABLED),

    // === Cache Line 13: Colors ===
    fgColor: u32(N_FG_COLOR),
    bgColor: u32(N_BG_COLOR),
    borderColor: u32(N_BORDER_COLOR),
    borderTopColor: u32(N_BORDER_TOP_COLOR),
    borderRightColor: u32(N_BORDER_RIGHT_COLOR),
    borderBottomColor: u32(N_BORDER_BOTTOM_COLOR),
    borderLeftColor: u32(N_BORDER_LEFT_COLOR),
    focusRingColor: u32(N_FOCUS_RING_COLOR),
    cursorFgColor: u32(N_CURSOR_FG_COLOR),
    cursorBgColor: u32(N_CURSOR_BG_COLOR),
    selectionColor: u32(N_SELECTION_COLOR),

    // === Cache Line 14: Text Properties ===
    textOffset: u32(N_TEXT_OFFSET),
    textLength: u32(N_TEXT_LENGTH),
    textAlign: u8(N_TEXT_ALIGN),
    textWrap: u8(N_TEXT_WRAP),
    textOverflow: u8(N_TEXT_OVERFLOW),
    textAttrs: u8(N_TEXT_ATTRS),
    textDecoration: u8(N_TEXT_DECORATION),
    textDecorationStyle: u8(N_TEXT_DECORATION_STYLE),
    textDecorationColor: u32(N_TEXT_DECORATION_COLOR),
    lineHeight: u8(N_LINE_HEIGHT),
    letterSpacing: u8(N_LETTER_SPACING),
    maxLines: u8(N_MAX_LINES),

    // === Cache Line 15: Interaction State ===
    scrollX: i32(N_SCROLL_X),
    scrollY: i32(N_SCROLL_Y),
    cursorPosition: i32(N_CURSOR_POSITION),
    selectionStart: i32(N_SELECTION_START),
    selectionEnd: i32(N_SELECTION_END),
    cursorChar: u32(N_CURSOR_CHAR),
    cursorAltChar: u32(N_CURSOR_ALT_CHAR),
    interactionFlags: u8(N_INTERACTION_FLAGS),
    cursorFlags: u8(N_CURSOR_FLAGS),
    cursorStyle: u8(N_CURSOR_STYLE),
    cursorBlinkRate: u8(N_CURSOR_BLINK_RATE),
    maxLength: u8(N_MAX_LENGTH),
    inputType: u8(N_INPUT_TYPE),
  }
}
