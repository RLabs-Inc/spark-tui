/**
 * TUI Framework - Text Primitive (AoS)
 *
 * Display text with styling, alignment, and wrapping.
 * Uses AoS SharedBuffer for cache-friendly Rust reads.
 *
 * REACTIVITY: All props flow through repeat() → AoSSlotBuffer → SharedArrayBuffer.
 * Text content uses a SINGLE repeater — the readFn encodes UTF-8 bytes into the
 * text pool and returns the offset. No effects. No TS-side measurement.
 * Rust handles text measurement via Taffy content_size.
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
import { allocateIndex, releaseIndex, getCurrentParentIndex } from '../engine/registry'
import {
  pushCurrentComponent,
  popCurrentComponent,
  runMountCallbacks,
} from '../engine/lifecycle'
import { cleanupIndex as cleanupKeyboardListeners } from '../state/keyboard'
import { onComponent as onMouseComponent } from '../state/mouse'
import { getVariantStyle } from '../state/theme'
import { getActiveScope } from './scope'
import { getAoSArrays, getAoSBuffer } from '../bridge'
import {
  packColor,
  setNodeText,
  H_TEXT_POOL_WRITE_PTR,
  TEXT_POOL_SIZE,
  HEADER_SIZE,
  MAX_NODES,
  STRIDE,
  U_TEXT_OFFSET,
  U_TEXT_LENGTH,
  U_DIRTY_FLAGS,
  DIRTY_TEXT,
  type AoSBuffer,
} from '../bridge/shared-buffer-aos'
import type { TextProps, Cleanup } from './types'

// =============================================================================
// CONVERSION HELPERS — same pattern as box.ts
// =============================================================================

/** Dimension → Taffy float: NaN = auto, negative = percentage, positive = pixels
 *  Rust convention: -100.0 = 100%, -50.0 = 50%, 40.0 = 40px
 */
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

// =============================================================================
// ENUM CONVERSIONS
// =============================================================================

function alignToNum(align: string | undefined): number {
  switch (align) {
    case 'center': return 1
    case 'right': return 2
    default: return 0 // left
  }
}

function wrapToNum(wrap: string | undefined): number {
  switch (wrap) {
    case 'nowrap': return 0
    case 'truncate': return 2
    default: return 1 // wrap
  }
}

// =============================================================================
// TEXT POOL WRITER — encodes UTF-8, writes to pool, returns offset
// =============================================================================

const textEncoder = new TextEncoder()

/**
 * Write text bytes to the AoS text pool. Returns the byte offset.
 * Sets textLength via direct write. Returns offset for repeater.
 */
function writeTextToPoolAoS(
  buf: AoSBuffer,
  index: number,
  text: string
): number {
  const encoded = textEncoder.encode(text)
  let writePtr = buf.header[H_TEXT_POOL_WRITE_PTR / 4]

  // Check capacity
  if (writePtr + encoded.length > TEXT_POOL_SIZE) {
    // Simple reset for now - could implement compaction later
    console.warn('Text pool overflow, resetting')
    writePtr = 0
    buf.header[H_TEXT_POOL_WRITE_PTR / 4] = 0
  }

  // Write to pool
  buf.textPool.set(encoded, writePtr)

  // Set offset and length in AoS node
  const base = HEADER_SIZE + index * STRIDE
  buf.view.setUint32(base + U_TEXT_OFFSET, writePtr, true)
  buf.view.setUint32(base + U_TEXT_LENGTH, encoded.length, true)

  // Mark dirty so layout recalculates text measurement
  const current = buf.view.getUint8(base + U_DIRTY_FLAGS)
  buf.view.setUint8(base + U_DIRTY_FLAGS, current | DIRTY_TEXT)

  // Advance write pointer
  buf.header[H_TEXT_POOL_WRITE_PTR / 4] = writePtr + encoded.length

  return writePtr
}

// =============================================================================
// TEXT COMPONENT
// =============================================================================

export function text(props: TextProps): Cleanup {
  const arrays = getAoSArrays()
  const buffer = getAoSBuffer()
  const index = allocateIndex(props.id)
  const disposals: (() => void)[] = []

  pushCurrentComponent(index)

  // --------------------------------------------------------------------------
  // CORE
  // --------------------------------------------------------------------------
  arrays.componentType.set(index, ComponentType.TEXT)
  disposals.push(repeat(getCurrentParentIndex(), arrays.parentIndex, index))

  // Visibility (default: visible)
  disposals.push(repeat(numInput(props.visible ?? 1, 1), arrays.visible, index))

  // --------------------------------------------------------------------------
  // TEXT CONTENT — single repeater, no effects
  // --------------------------------------------------------------------------
  if (isReactive(props.content)) {
    disposals.push(repeat(
      () => writeTextToPoolAoS(buffer, index, String(unwrap(props.content))),
      arrays.textOffset,
      index
    ))
  } else {
    // Static text — write once, no repeater needed
    setNodeText(buffer, index, String(props.content))
  }

  // --------------------------------------------------------------------------
  // LAYOUT — dimensions, flex item (Rust measures text for auto-sizing)
  // --------------------------------------------------------------------------
  if (props.width !== undefined)     disposals.push(repeat(dimInput(props.width), arrays.width, index))
  if (props.height !== undefined)    disposals.push(repeat(dimInput(props.height), arrays.height, index))
  if (props.minWidth !== undefined)  disposals.push(repeat(dimInput(props.minWidth), arrays.minWidth, index))
  if (props.maxWidth !== undefined)  disposals.push(repeat(dimInput(props.maxWidth), arrays.maxWidth, index))
  if (props.minHeight !== undefined) disposals.push(repeat(dimInput(props.minHeight), arrays.minHeight, index))
  if (props.maxHeight !== undefined) disposals.push(repeat(dimInput(props.maxHeight), arrays.maxHeight, index))

  // Flex item
  if (props.grow !== undefined)      disposals.push(repeat(numInput(props.grow), arrays.grow, index))
  if (props.shrink !== undefined)    disposals.push(repeat(numInput(props.shrink), arrays.shrink, index))
  if (props.flexBasis !== undefined) disposals.push(repeat(dimInput(props.flexBasis), arrays.basis, index))

  // Padding
  if (props.padding !== undefined) {
    disposals.push(repeat(numInput(props.paddingTop ?? props.padding), arrays.paddingTop, index))
    disposals.push(repeat(numInput(props.paddingRight ?? props.padding), arrays.paddingRight, index))
    disposals.push(repeat(numInput(props.paddingBottom ?? props.padding), arrays.paddingBottom, index))
    disposals.push(repeat(numInput(props.paddingLeft ?? props.padding), arrays.paddingLeft, index))
  } else {
    if (props.paddingTop !== undefined)    disposals.push(repeat(numInput(props.paddingTop), arrays.paddingTop, index))
    if (props.paddingRight !== undefined)  disposals.push(repeat(numInput(props.paddingRight), arrays.paddingRight, index))
    if (props.paddingBottom !== undefined) disposals.push(repeat(numInput(props.paddingBottom), arrays.paddingBottom, index))
    if (props.paddingLeft !== undefined)   disposals.push(repeat(numInput(props.paddingLeft), arrays.paddingLeft, index))
  }

  // Text styling
  if (props.align !== undefined) disposals.push(repeat(enumInput(props.align, alignToNum), arrays.textAlign, index))
  if (props.wrap !== undefined)  disposals.push(repeat(enumInput(props.wrap, wrapToNum), arrays.textWrap, index))

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
