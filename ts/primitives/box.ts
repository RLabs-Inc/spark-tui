/**
 * TUI Framework - Box Primitive (v3 Buffer)
 *
 * Container component with flexbox/grid layout, borders, and background.
 * Uses v3 SharedBuffer (1024-byte stride) with full Grid support.
 *
 * REACTIVITY: Props flow through repeat() → SharedSlotBuffer → SharedArrayBuffer.
 * repeat() handles signals, getters, deriveds, and static values natively.
 *
 * Usage:
 * ```ts
 * const width = signal(40)
 * box({
 *   width,         // Reactive! repeat() wires it to SharedArrayBuffer
 *   height: 10,    // Static — repeat() sets once
 *   border: 1,
 *   display: 'grid',
 *   gridTemplateColumns: ['1fr', '2fr'],
 *   children: () => {
 *     text({ content: 'Hello!' })
 *   }
 * })
 * ```
 */

import { repeat } from '@rlabs-inc/signals'
import { ComponentType } from '../types'
import type { RGBA } from '../types'
import {
  allocateIndex,
  releaseIndex,
  getCurrentParentIndex,
  pushParentContext,
  popParentContext,
  registerParent,
} from '../engine/registry'
import {
  pushCurrentComponent,
  popCurrentComponent,
  runMountCallbacks,
} from '../engine/lifecycle'
import { cleanupIndex as cleanupKeyboardListeners, onFocused } from '../state/keyboard'
import { registerFocusCallbacks, focus as focusComponent } from '../state/focus'
import { onComponent as onMouseComponent } from '../state/mouse'
import { getVariantStyle } from '../state/theme'
import { getActiveScope } from './scope'
import { getArrays, getBuffer } from '../bridge'
import {
  packColor,
  setText,
  setGridColumnTracks,
  setGridRowTracks,
  TrackType,
  Display,
  FLAG_FOCUSABLE,
  DIRTY_LAYOUT,
  markDirty,
  type GridTrack,
  type SharedBuffer,
} from '../bridge/shared-buffer'
import type { ReactiveArrays } from '../bridge/reactive-arrays'
import type { BoxProps, Cleanup, GridTrackSize, GridTemplate, GridLine } from './types'

// =============================================================================
// CONVERSION HELPERS
// =============================================================================

/** Dimension → Taffy float: NaN = auto, negative = percentage, positive = pixels */
function toDim(dim: number | string | undefined | null): number {
  if (dim === undefined || dim === null || dim === 0) return NaN
  if (typeof dim === 'string') {
    if (dim.endsWith('%')) return -parseFloat(dim) // '100%' → -100.0
    return parseFloat(dim) || NaN
  }
  return dim
}

/** Unwrap any prop shape to its current value */
function unwrap<T>(prop: T | (() => T) | { readonly value: T }): T {
  if (typeof prop === 'function') return (prop as () => T)()
  if (prop !== null && typeof prop === 'object' && 'value' in prop) return (prop as { value: T }).value
  return prop
}

/** Is this prop reactive (not a static value)? */
function isReactive(prop: unknown): boolean {
  return typeof prop === 'function' || (prop !== null && typeof prop === 'object' && 'value' in (prop as any))
}

/** Pack RGBA or number to u32 */
function toPackedColor(c: RGBA | number | null | undefined): number {
  if (c === null || c === undefined) return 0
  if (typeof c === 'number') return c
  return packColor(c.r, c.g, c.b, c.a ?? 255)
}

// Dimension: wrap prop for repeat()
function dimInput(prop: BoxProps['width']): number | (() => number) {
  if (prop === undefined) return NaN
  if (typeof prop === 'number' || typeof prop === 'string') return toDim(prop)
  return () => toDim(unwrap(prop))
}

// Enum: wrap prop for repeat()
function enumInput(prop: unknown, converter: (v: any) => number): number | (() => number) {
  if (prop === undefined) return converter(undefined)
  if (typeof prop === 'string') return converter(prop)
  if (isReactive(prop)) return () => converter(unwrap(prop))
  return converter(prop as string)
}

// Color: wrap prop for repeat()
function colorInput(prop: BoxProps['fg']): number | (() => number) {
  if (prop === undefined) return 0
  if (!isReactive(prop)) return toPackedColor(prop as RGBA | number | null)
  return () => toPackedColor(unwrap(prop as any))
}

// Numeric: wrap prop for repeat()
function numInput(prop: unknown, defaultVal = 0): number | (() => number) | { readonly value: number } {
  if (prop === undefined) return defaultVal
  return prop as any
}

// Boolean → number
function boolInput(prop: unknown, defaultVal = 1): number | (() => number) {
  if (prop === undefined) return defaultVal
  if (typeof prop === 'boolean') return prop ? 1 : 0
  if (typeof prop === 'function') return () => (prop as () => boolean)() ? 1 : 0
  if (isReactive(prop)) return () => unwrap(prop as any) ? 1 : 0
  return prop ? 1 : 0
}

// =============================================================================
// ENUM CONVERSIONS
// =============================================================================

function displayToNum(d: string | undefined): number {
  switch (d) {
    case 'none': return Display.None
    case 'grid': return Display.Grid
    default: return Display.Flex // 'flex' or undefined
  }
}

function flexDirectionToNum(dir: string | undefined): number {
  switch (dir) {
    case 'row': return 0
    case 'column': return 1
    case 'row-reverse': return 2
    case 'column-reverse': return 3
    default: return 1 // column (TUI default)
  }
}

function flexWrapToNum(wrap: string | undefined): number {
  switch (wrap) {
    case 'wrap': return 1
    case 'wrap-reverse': return 2
    default: return 0 // nowrap
  }
}

function justifyToNum(j: string | undefined): number {
  switch (j) {
    case 'flex-start': return 0
    case 'flex-end': return 1
    case 'center': return 2
    case 'space-between': return 3
    case 'space-around': return 4
    case 'space-evenly': return 5
    default: return 0 // flex-start
  }
}

function alignItemsToNum(a: string | undefined): number {
  switch (a) {
    case 'flex-start': return 0
    case 'flex-end': return 1
    case 'center': return 2
    case 'baseline': return 3
    case 'stretch': return 4
    default: return 4 // stretch (default)
  }
}

function alignContentToNum(a: string | undefined): number {
  switch (a) {
    case 'flex-start': return 0
    case 'flex-end': return 1
    case 'center': return 2
    case 'space-between': return 3
    case 'space-around': return 4
    case 'space-evenly': return 5
    case 'stretch': return 6
    default: return 0 // flex-start
  }
}

function alignSelfToNum(a: string | undefined): number {
  switch (a) {
    case 'auto': return 0
    case 'flex-start': return 1
    case 'flex-end': return 2
    case 'center': return 3
    case 'baseline': return 4
    case 'stretch': return 5
    default: return 0 // auto
  }
}

function overflowToNum(o: string | undefined): number {
  switch (o) {
    case 'hidden': return 1
    case 'scroll': return 2
    case 'auto': return 2 // treat auto as scroll
    default: return 0 // visible
  }
}

// Grid enums
function gridAutoFlowToNum(f: string | undefined): number {
  switch (f) {
    case 'column': return 1
    case 'row dense': return 2
    case 'column dense': return 3
    default: return 0 // row
  }
}

function justifyItemsToNum(j: string | undefined): number {
  switch (j) {
    case 'end': return 1
    case 'center': return 2
    case 'stretch': return 3
    default: return 0 // start
  }
}

function justifySelfToNum(j: string | undefined): number {
  switch (j) {
    case 'start': return 1
    case 'end': return 2
    case 'center': return 3
    case 'stretch': return 4
    default: return 0 // auto
  }
}

// =============================================================================
// GRID TRACK PARSING
// =============================================================================

/** Parse a single track size to GridTrack */
function parseTrackSize(size: GridTrackSize): GridTrack {
  if (typeof size === 'number') {
    return { trackType: TrackType.Length, value: size }
  }
  if (size === 'auto') {
    return { trackType: TrackType.Auto, value: 0 }
  }
  if (size === 'min-content') {
    return { trackType: TrackType.MinContent, value: 0 }
  }
  if (size === 'max-content') {
    return { trackType: TrackType.MaxContent, value: 0 }
  }
  if (size.endsWith('%')) {
    return { trackType: TrackType.Percent, value: parseFloat(size) / 100 }
  }
  if (size.endsWith('fr')) {
    return { trackType: TrackType.Fr, value: parseFloat(size) }
  }
  return { trackType: TrackType.Auto, value: 0 }
}

/** Parse grid template to GridTrack array */
function parseGridTemplate(template: GridTemplate | undefined): GridTrack[] {
  if (!template) return []
  return template.map(parseTrackSize)
}

/** Parse grid line position to i16 value */
function parseGridLine(line: GridLine | undefined): number {
  if (line === undefined || line === 'auto') return 0
  if (typeof line === 'number') return line
  if (typeof line === 'string' && line.startsWith('span ')) {
    return -parseInt(line.slice(5), 10) // negative = span
  }
  return 0
}

// =============================================================================
// BOX COMPONENT
// =============================================================================

export function box(props: BoxProps = {}): Cleanup {
  const buf = getBuffer()
  const arrays = getArrays()
  const index = allocateIndex(props.id)
  const disposals: (() => void)[] = []
  const parentIdx = getCurrentParentIndex()

  pushCurrentComponent(index)

  // --------------------------------------------------------------------------
  // CORE — component type and hierarchy
  // --------------------------------------------------------------------------
  arrays.componentType.set(index, ComponentType.BOX)

  // Set parent index and register in O(1) linked list
  arrays.parentIndex.set(index, parentIdx)
  registerParent(index, parentIdx)

  // Visibility (default: visible)
  disposals.push(repeat(boolInput(props.visible, 1), arrays.visible, index))

  // --------------------------------------------------------------------------
  // DISPLAY MODE
  // --------------------------------------------------------------------------
  if (props.display !== undefined) {
    disposals.push(repeat(enumInput(props.display, displayToNum), arrays.display, index))
  }

  // --------------------------------------------------------------------------
  // LAYOUT — dimensions
  // --------------------------------------------------------------------------
  if (props.width !== undefined) disposals.push(repeat(dimInput(props.width), arrays.width, index))
  if (props.height !== undefined) disposals.push(repeat(dimInput(props.height), arrays.height, index))
  if (props.minWidth !== undefined) disposals.push(repeat(dimInput(props.minWidth), arrays.minWidth, index))
  if (props.maxWidth !== undefined) disposals.push(repeat(dimInput(props.maxWidth), arrays.maxWidth, index))
  if (props.minHeight !== undefined) disposals.push(repeat(dimInput(props.minHeight), arrays.minHeight, index))
  if (props.maxHeight !== undefined) disposals.push(repeat(dimInput(props.maxHeight), arrays.maxHeight, index))

  // Overflow
  if (props.overflow !== undefined) disposals.push(repeat(enumInput(props.overflow, overflowToNum), arrays.overflow, index))

  // --------------------------------------------------------------------------
  // FLEXBOX CONTAINER
  // --------------------------------------------------------------------------
  if (props.flexDirection !== undefined) disposals.push(repeat(enumInput(props.flexDirection, flexDirectionToNum), arrays.flexDirection, index))
  if (props.flexWrap !== undefined) disposals.push(repeat(enumInput(props.flexWrap, flexWrapToNum), arrays.flexWrap, index))
  if (props.justifyContent !== undefined) disposals.push(repeat(enumInput(props.justifyContent, justifyToNum), arrays.justifyContent, index))
  if (props.alignItems !== undefined) disposals.push(repeat(enumInput(props.alignItems, alignItemsToNum), arrays.alignItems, index))
  if (props.alignContent !== undefined) disposals.push(repeat(enumInput(props.alignContent, alignContentToNum), arrays.alignContent, index))

  // --------------------------------------------------------------------------
  // FLEXBOX ITEM
  // --------------------------------------------------------------------------
  if (props.grow !== undefined) disposals.push(repeat(numInput(props.grow), arrays.flexGrow, index))
  if (props.shrink !== undefined) disposals.push(repeat(numInput(props.shrink), arrays.flexShrink, index))
  if (props.flexBasis !== undefined) disposals.push(repeat(dimInput(props.flexBasis), arrays.flexBasis, index))
  if (props.alignSelf !== undefined) disposals.push(repeat(enumInput(props.alignSelf, alignSelfToNum), arrays.alignSelf, index))

  // --------------------------------------------------------------------------
  // SPACING
  // --------------------------------------------------------------------------

  // Padding
  if (props.padding !== undefined) {
    disposals.push(repeat(numInput(props.paddingTop ?? props.padding), arrays.paddingTop, index))
    disposals.push(repeat(numInput(props.paddingRight ?? props.padding), arrays.paddingRight, index))
    disposals.push(repeat(numInput(props.paddingBottom ?? props.padding), arrays.paddingBottom, index))
    disposals.push(repeat(numInput(props.paddingLeft ?? props.padding), arrays.paddingLeft, index))
  } else {
    if (props.paddingTop !== undefined) disposals.push(repeat(numInput(props.paddingTop), arrays.paddingTop, index))
    if (props.paddingRight !== undefined) disposals.push(repeat(numInput(props.paddingRight), arrays.paddingRight, index))
    if (props.paddingBottom !== undefined) disposals.push(repeat(numInput(props.paddingBottom), arrays.paddingBottom, index))
    if (props.paddingLeft !== undefined) disposals.push(repeat(numInput(props.paddingLeft), arrays.paddingLeft, index))
  }

  // Margin
  if (props.margin !== undefined) {
    disposals.push(repeat(numInput(props.marginTop ?? props.margin), arrays.marginTop, index))
    disposals.push(repeat(numInput(props.marginRight ?? props.margin), arrays.marginRight, index))
    disposals.push(repeat(numInput(props.marginBottom ?? props.margin), arrays.marginBottom, index))
    disposals.push(repeat(numInput(props.marginLeft ?? props.margin), arrays.marginLeft, index))
  } else {
    if (props.marginTop !== undefined) disposals.push(repeat(numInput(props.marginTop), arrays.marginTop, index))
    if (props.marginRight !== undefined) disposals.push(repeat(numInput(props.marginRight), arrays.marginRight, index))
    if (props.marginBottom !== undefined) disposals.push(repeat(numInput(props.marginBottom), arrays.marginBottom, index))
    if (props.marginLeft !== undefined) disposals.push(repeat(numInput(props.marginLeft), arrays.marginLeft, index))
  }

  // Gap
  if (props.gap !== undefined) disposals.push(repeat(numInput(props.gap), arrays.gap, index))
  if (props.rowGap !== undefined) disposals.push(repeat(numInput(props.rowGap), arrays.rowGap, index))
  if (props.columnGap !== undefined) disposals.push(repeat(numInput(props.columnGap), arrays.columnGap, index))

  // --------------------------------------------------------------------------
  // BORDER WIDTHS (layout spacing: 0 or 1)
  // --------------------------------------------------------------------------
  if (props.border !== undefined) {
    const bw = isReactive(props.border) ? (() => unwrap(props.border!) > 0 ? 1 : 0) : (unwrap(props.border) > 0 ? 1 : 0)
    disposals.push(repeat(bw, arrays.borderWidthTop, index))
    disposals.push(repeat(bw, arrays.borderWidthRight, index))
    disposals.push(repeat(bw, arrays.borderWidthBottom, index))
    disposals.push(repeat(bw, arrays.borderWidthLeft, index))
  }
  if (props.borderTop !== undefined) {
    const bw = isReactive(props.borderTop) ? (() => unwrap(props.borderTop!) > 0 ? 1 : 0) : (unwrap(props.borderTop) > 0 ? 1 : 0)
    disposals.push(repeat(bw, arrays.borderWidthTop, index))
  }
  if (props.borderRight !== undefined) {
    const bw = isReactive(props.borderRight) ? (() => unwrap(props.borderRight!) > 0 ? 1 : 0) : (unwrap(props.borderRight) > 0 ? 1 : 0)
    disposals.push(repeat(bw, arrays.borderWidthRight, index))
  }
  if (props.borderBottom !== undefined) {
    const bw = isReactive(props.borderBottom) ? (() => unwrap(props.borderBottom!) > 0 ? 1 : 0) : (unwrap(props.borderBottom) > 0 ? 1 : 0)
    disposals.push(repeat(bw, arrays.borderWidthBottom, index))
  }
  if (props.borderLeft !== undefined) {
    const bw = isReactive(props.borderLeft) ? (() => unwrap(props.borderLeft!) > 0 ? 1 : 0) : (unwrap(props.borderLeft) > 0 ? 1 : 0)
    disposals.push(repeat(bw, arrays.borderWidthLeft, index))
  }

  // --------------------------------------------------------------------------
  // GRID CONTAINER PROPERTIES
  // --------------------------------------------------------------------------
  if (props.gridAutoFlow !== undefined) {
    disposals.push(repeat(enumInput(props.gridAutoFlow, gridAutoFlowToNum), arrays.gridAutoFlow, index))
  }
  if (props.justifyItems !== undefined) {
    disposals.push(repeat(enumInput(props.justifyItems, justifyItemsToNum), arrays.justifyItems, index))
  }

  // Grid template columns
  if (props.gridTemplateColumns !== undefined) {
    if (isReactive(props.gridTemplateColumns)) {
      // Reactive: set up effect to update tracks when template changes
      disposals.push(repeat(
        () => {
          const template = unwrap(props.gridTemplateColumns!)
          setGridColumnTracks(buf, index, parseGridTemplate(template))
          return 1 // dummy value for repeat
        },
        arrays.gridColumnCount, // just to trigger the effect
        index
      ))
    } else {
      // Static: set once
      setGridColumnTracks(buf, index, parseGridTemplate(props.gridTemplateColumns as GridTemplate))
    }
  }

  // Grid template rows
  if (props.gridTemplateRows !== undefined) {
    if (isReactive(props.gridTemplateRows)) {
      disposals.push(repeat(
        () => {
          const template = unwrap(props.gridTemplateRows!)
          setGridRowTracks(buf, index, parseGridTemplate(template))
          return 1
        },
        arrays.gridRowCount,
        index
      ))
    } else {
      setGridRowTracks(buf, index, parseGridTemplate(props.gridTemplateRows as GridTemplate))
    }
  }

  // Grid auto columns/rows
  if (props.gridAutoColumns !== undefined) {
    const track = parseTrackSize(unwrap(props.gridAutoColumns))
    arrays.gridAutoColumnsType.set(index, track.trackType)
    arrays.gridAutoColumnsValue.set(index, track.value)
  }
  if (props.gridAutoRows !== undefined) {
    const track = parseTrackSize(unwrap(props.gridAutoRows))
    arrays.gridAutoRowsType.set(index, track.trackType)
    arrays.gridAutoRowsValue.set(index, track.value)
  }

  // --------------------------------------------------------------------------
  // GRID ITEM PROPERTIES
  // --------------------------------------------------------------------------
  if (props.gridColumnStart !== undefined) {
    if (isReactive(props.gridColumnStart)) {
      disposals.push(repeat(() => parseGridLine(unwrap(props.gridColumnStart)), arrays.gridColumnStart, index))
    } else {
      arrays.gridColumnStart.set(index, parseGridLine(props.gridColumnStart as GridLine))
    }
  }
  if (props.gridColumnEnd !== undefined) {
    if (isReactive(props.gridColumnEnd)) {
      disposals.push(repeat(() => parseGridLine(unwrap(props.gridColumnEnd)), arrays.gridColumnEnd, index))
    } else {
      arrays.gridColumnEnd.set(index, parseGridLine(props.gridColumnEnd as GridLine))
    }
  }
  if (props.gridRowStart !== undefined) {
    if (isReactive(props.gridRowStart)) {
      disposals.push(repeat(() => parseGridLine(unwrap(props.gridRowStart)), arrays.gridRowStart, index))
    } else {
      arrays.gridRowStart.set(index, parseGridLine(props.gridRowStart as GridLine))
    }
  }
  if (props.gridRowEnd !== undefined) {
    if (isReactive(props.gridRowEnd)) {
      disposals.push(repeat(() => parseGridLine(unwrap(props.gridRowEnd)), arrays.gridRowEnd, index))
    } else {
      arrays.gridRowEnd.set(index, parseGridLine(props.gridRowEnd as GridLine))
    }
  }
  if (props.justifySelf !== undefined) {
    disposals.push(repeat(enumInput(props.justifySelf, justifySelfToNum), arrays.justifySelf, index))
  }

  // --------------------------------------------------------------------------
  // VISUAL — colors and border styles
  // --------------------------------------------------------------------------
  if (props.variant && props.variant !== 'default') {
    const variant = props.variant
    if (props.fg !== undefined) {
      disposals.push(repeat(colorInput(props.fg), arrays.fgColor, index))
    } else {
      disposals.push(repeat(() => toPackedColor(getVariantStyle(variant).fg), arrays.fgColor, index))
    }
    if (props.bg !== undefined) {
      disposals.push(repeat(colorInput(props.bg), arrays.bgColor, index))
    } else {
      disposals.push(repeat(() => toPackedColor(getVariantStyle(variant).bg), arrays.bgColor, index))
    }
    if (props.borderColor !== undefined) {
      disposals.push(repeat(colorInput(props.borderColor), arrays.borderColor, index))
    } else {
      disposals.push(repeat(() => toPackedColor(getVariantStyle(variant).border), arrays.borderColor, index))
    }
  } else {
    if (props.fg !== undefined) disposals.push(repeat(colorInput(props.fg), arrays.fgColor, index))
    if (props.bg !== undefined) disposals.push(repeat(colorInput(props.bg), arrays.bgColor, index))
    if (props.borderColor !== undefined) disposals.push(repeat(colorInput(props.borderColor), arrays.borderColor, index))
  }
  if (props.opacity !== undefined) disposals.push(repeat(numInput(props.opacity), arrays.opacity, index))
  if (props.zIndex !== undefined) disposals.push(repeat(numInput(props.zIndex), arrays.zIndex, index))

  // Border style for rendering
  if (props.border !== undefined) disposals.push(repeat(numInput(props.border), arrays.borderStyle, index))
  if (props.borderTop !== undefined) disposals.push(repeat(numInput(props.borderTop), arrays.borderStyleTop, index))
  if (props.borderRight !== undefined) disposals.push(repeat(numInput(props.borderRight), arrays.borderStyleRight, index))
  if (props.borderBottom !== undefined) disposals.push(repeat(numInput(props.borderBottom), arrays.borderStyleBottom, index))
  if (props.borderLeft !== undefined) disposals.push(repeat(numInput(props.borderLeft), arrays.borderStyleLeft, index))

  // --------------------------------------------------------------------------
  // INTERACTION — focusable, tab index
  // --------------------------------------------------------------------------
  const shouldBeFocusable = props.focusable || (props.overflow === 'scroll' && props.focusable !== false)
  if (shouldBeFocusable) {
    arrays.interactionFlags.set(index, FLAG_FOCUSABLE)
    if (props.tabIndex !== undefined) disposals.push(repeat(numInput(props.tabIndex, -1), arrays.tabIndex, index))
  }

  // --------------------------------------------------------------------------
  // FOCUS CALLBACKS & KEYBOARD
  // --------------------------------------------------------------------------
  let unsubKeyboard: (() => void) | undefined
  let unsubFocusCallbacks: (() => void) | undefined

  // Key handlers: register for ALL components (not just focusable) to support
  // event bubbling — root boxes can handle global shortcuts like +/-/q
  if (props.onKey) unsubKeyboard = onFocused(index, props.onKey)

  if (shouldBeFocusable) {
    if (props.onFocus || props.onBlur) {
      unsubFocusCallbacks = registerFocusCallbacks(index, {
        onFocus: props.onFocus,
        onBlur: props.onBlur,
      })
    }
  }

  // --------------------------------------------------------------------------
  // MOUSE HANDLERS
  // --------------------------------------------------------------------------
  let unsubMouse: (() => void) | undefined
  const hasMouseHandlers = props.onMouseDown || props.onMouseUp || props.onClick || props.onMouseEnter || props.onMouseLeave || props.onScroll

  if (shouldBeFocusable || hasMouseHandlers) {
    unsubMouse = onMouseComponent(index, {
      onMouseDown: props.onMouseDown,
      onMouseUp: props.onMouseUp,
      onClick: (event) => {
        if (shouldBeFocusable) focusComponent(index)
        return props.onClick?.(event)
      },
      onMouseEnter: props.onMouseEnter,
      onMouseLeave: props.onMouseLeave,
      onScroll: props.onScroll,
    })
  }

  // --------------------------------------------------------------------------
  // CHILDREN
  // --------------------------------------------------------------------------
  if (props.children) {
    pushParentContext(index)
    try {
      props.children()
    } finally {
      popParentContext()
    }
  }

  // Component setup complete
  popCurrentComponent()
  runMountCallbacks(index)

  // --------------------------------------------------------------------------
  // CLEANUP
  // --------------------------------------------------------------------------
  const cleanup = () => {
    for (const dispose of disposals) dispose()
    disposals.length = 0
    unsubFocusCallbacks?.()
    unsubMouse?.()
    unsubKeyboard?.()
    cleanupKeyboardListeners(index)
    releaseIndex(index)
  }

  const scope = getActiveScope()
  if (scope) scope.cleanups.push(cleanup)

  return cleanup
}
