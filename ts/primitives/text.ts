/**
 * TUI Framework - Text Primitive (v3 Buffer)
 *
 * Display text with styling, alignment, and wrapping.
 * Uses v3 SharedBuffer (1024-byte stride) with Grid support.
 *
 * REACTIVITY: All props flow through repeat() → SharedSlotBuffer → SharedArrayBuffer.
 * Text content uses a SINGLE repeater — the readFn encodes UTF-8 bytes into the
 * text pool and marks the node dirty. Rust handles text measurement via Taffy.
 *
 * Usage:
 * ```ts
 * const msg = signal('Hello')
 * text({ content: msg })
 * msg.value = 'Updated!'  // repeat() fires inline → pool write → Rust wakes
 * ```
 */

import { repeat } from '@rlabs-inc/signals'
import { ComponentType } from '../types'
import type { RGBA } from '../types'
import {
  allocateIndex,
  releaseIndex,
  getCurrentParentIndex,
  registerParent,
} from '../engine/registry'
import {
  pushCurrentComponent,
  popCurrentComponent,
  runMountCallbacks,
} from '../engine/lifecycle'
import { cleanupIndex as cleanupKeyboardListeners } from '../state/keyboard'
import { onComponent as onMouseComponent } from '../state/mouse'
import { getVariantStyle } from '../state/theme'
import { getActiveScope } from './scope'
import { getArrays, getBuffer } from '../bridge'
import {
  packColor,
  setText,
  getU32,
  N_TEXT_OFFSET,
  DIRTY_TEXT,
  markDirty,
  type SharedBuffer,
} from '../bridge/shared-buffer'
import type { TextProps, Cleanup, GridLine } from './types'

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

function unwrap<T>(prop: T | (() => T) | { readonly value: T }): T {
  if (typeof prop === 'function') return (prop as () => T)()
  if (prop !== null && typeof prop === 'object' && 'value' in prop) return (prop as { value: T }).value
  return prop
}

function isReactive(prop: unknown): boolean {
  return typeof prop === 'function' || (prop !== null && typeof prop === 'object' && 'value' in (prop as any))
}

function toPackedColor(c: RGBA | number | null | undefined): number {
  if (c === null || c === undefined) return 0
  if (typeof c === 'number') return c
  return packColor(c.r, c.g, c.b, c.a ?? 255)
}

function dimInput(prop: TextProps['width']): number | (() => number) {
  if (prop === undefined) return NaN
  if (typeof prop === 'number' || typeof prop === 'string') return toDim(prop)
  return () => toDim(unwrap(prop))
}

function enumInput(prop: unknown, converter: (v: any) => number): number | (() => number) {
  if (prop === undefined) return converter(undefined)
  if (typeof prop === 'string') return converter(prop)
  if (isReactive(prop)) return () => converter(unwrap(prop))
  return converter(prop as string)
}

function colorInput(prop: TextProps['fg']): number | (() => number) {
  if (prop === undefined) return 0
  if (!isReactive(prop)) return toPackedColor(prop as RGBA | number | null)
  return () => toPackedColor(unwrap(prop as any))
}

function numInput(prop: unknown, defaultVal = 0): number | (() => number) | { readonly value: number } {
  if (prop === undefined) return defaultVal
  return prop as any
}

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

function textAlignToNum(align: string | undefined): number {
  switch (align) {
    case 'center': return 1
    case 'right': return 2
    default: return 0 // left
  }
}

function textWrapToNum(wrap: string | undefined): number {
  switch (wrap) {
    case 'nowrap': return 0
    case 'truncate': return 2
    default: return 1 // wrap
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
// TEXT POOL WRITER
// =============================================================================

/**
 * Write text to the text pool via setText() helper.
 * Returns the text offset for the repeater (reads from node after write).
 */
function writeTextToPool(buf: SharedBuffer, index: number, text: string): number {
  const result = setText(buf, index, text)

  if (!result.success) {
    const { liveBytes, poolSize, needed } = result
    const liveMB = (liveBytes / 1024 / 1024).toFixed(2)
    const poolMB = (poolSize / 1024 / 1024).toFixed(2)
    throw new Error(
      `Text pool full (${liveMB}MB live / ${poolMB}MB total). ` +
      `Cannot allocate ${needed} bytes for node ${index}. ` +
      `Increase textPoolSize in mount() config.`
    )
  }

  // Read the actual offset from the node (may differ due to slot reuse or compaction)
  return getU32(buf, index, N_TEXT_OFFSET)
}

// =============================================================================
// TEXT COMPONENT
// =============================================================================

export function text(props: TextProps): Cleanup {
  const buf = getBuffer()
  const arrays = getArrays()
  const index = allocateIndex(props.id)
  const disposals: (() => void)[] = []
  const parentIdx = getCurrentParentIndex()

  pushCurrentComponent(index)

  // --------------------------------------------------------------------------
  // CORE
  // --------------------------------------------------------------------------
  arrays.componentType.set(index, ComponentType.TEXT)

  // Set parent index and register in O(1) linked list
  arrays.parentIndex.set(index, parentIdx)
  registerParent(index, parentIdx)

  // Visibility (default: visible)
  disposals.push(repeat(boolInput(props.visible, 1), arrays.visible, index))

  // --------------------------------------------------------------------------
  // TEXT CONTENT — single repeater, no effects
  // --------------------------------------------------------------------------
  if (isReactive(props.content)) {
    disposals.push(repeat(
      () => writeTextToPool(buf, index, String(unwrap(props.content))),
      arrays.textOffset,
      index
    ))
  } else {
    // Static text — write once, no repeater needed
    const result = setText(buf, index, String(props.content))
    if (!result.success) {
      const { liveBytes, poolSize, needed } = result
      const liveMB = (liveBytes / 1024 / 1024).toFixed(2)
      const poolMB = (poolSize / 1024 / 1024).toFixed(2)
      throw new Error(
        `Text pool full (${liveMB}MB live / ${poolMB}MB total). ` +
        `Cannot allocate ${needed} bytes for node ${index}. ` +
        `Increase textPoolSize in mount() config.`
      )
    }
  }

  // --------------------------------------------------------------------------
  // LAYOUT — dimensions, flex item
  // --------------------------------------------------------------------------
  if (props.width !== undefined) disposals.push(repeat(dimInput(props.width), arrays.width, index))
  if (props.height !== undefined) disposals.push(repeat(dimInput(props.height), arrays.height, index))
  if (props.minWidth !== undefined) disposals.push(repeat(dimInput(props.minWidth), arrays.minWidth, index))
  if (props.maxWidth !== undefined) disposals.push(repeat(dimInput(props.maxWidth), arrays.maxWidth, index))
  if (props.minHeight !== undefined) disposals.push(repeat(dimInput(props.minHeight), arrays.minHeight, index))
  if (props.maxHeight !== undefined) disposals.push(repeat(dimInput(props.maxHeight), arrays.maxHeight, index))

  // Flex item
  if (props.grow !== undefined) disposals.push(repeat(numInput(props.grow), arrays.flexGrow, index))
  if (props.shrink !== undefined) disposals.push(repeat(numInput(props.shrink), arrays.flexShrink, index))
  if (props.flexBasis !== undefined) disposals.push(repeat(dimInput(props.flexBasis), arrays.flexBasis, index))
  if (props.alignSelf !== undefined) disposals.push(repeat(enumInput(props.alignSelf, alignSelfToNum), arrays.alignSelf, index))

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

  // Z-index
  if (props.zIndex !== undefined) disposals.push(repeat(numInput(props.zIndex), arrays.zIndex, index))

  // Text styling
  if (props.align !== undefined) disposals.push(repeat(enumInput(props.align, textAlignToNum), arrays.textAlign, index))
  if (props.wrap !== undefined) disposals.push(repeat(enumInput(props.wrap, textWrapToNum), arrays.textWrap, index))

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
  // VISUAL — colors with variant support
  // --------------------------------------------------------------------------
  if (props.variant && props.variant !== 'default') {
    const variant = props.variant
    disposals.push(repeat(
      props.fg !== undefined ? colorInput(props.fg) : () => toPackedColor(getVariantStyle(variant).fg),
      arrays.fgColor, index
    ))
    disposals.push(repeat(
      props.bg !== undefined ? colorInput(props.bg) : () => toPackedColor(getVariantStyle(variant).bg),
      arrays.bgColor, index
    ))
  } else {
    if (props.fg !== undefined) disposals.push(repeat(colorInput(props.fg), arrays.fgColor, index))
    if (props.bg !== undefined) disposals.push(repeat(colorInput(props.bg), arrays.bgColor, index))
  }
  if (props.opacity !== undefined) disposals.push(repeat(numInput(props.opacity), arrays.opacity, index))

  // --------------------------------------------------------------------------
  // MOUSE HANDLERS
  // --------------------------------------------------------------------------
  let unsubMouse: (() => void) | undefined

  if (props.onMouseDown || props.onMouseUp || props.onClick || props.onMouseEnter || props.onMouseLeave || props.onScroll) {
    unsubMouse = onMouseComponent(index, {
      onMouseDown: props.onMouseDown,
      onMouseUp: props.onMouseUp,
      onClick: props.onClick,
      onMouseEnter: props.onMouseEnter,
      onMouseLeave: props.onMouseLeave,
      onScroll: props.onScroll,
    })
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
    unsubMouse?.()
    cleanupKeyboardListeners(index)
    releaseIndex(index)
  }

  const scope = getActiveScope()
  if (scope) scope.cleanups.push(cleanup)

  return cleanup
}
