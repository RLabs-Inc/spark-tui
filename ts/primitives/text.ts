/**
 * TUI Framework - Text Primitive
 *
 * Display text with styling, alignment, and wrapping.
 *
 * REACTIVITY: All props flow through repeat() → SharedSlotBuffer → SharedArrayBuffer.
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
import { getArrays, getViews } from '../bridge'
import {
  packColor,
  setNodeText,
  U32_TEXT_LENGTH,
  HEADER_TEXT_POOL_WRITE_PTR,
  HEADER_TEXT_POOL_CAPACITY,
} from '../bridge/shared-buffer'
import type { TextProps, Cleanup } from './types'

// =============================================================================
// CONVERSION HELPERS — same pattern as box.ts
// =============================================================================

function toDim(dim: number | string | undefined | null): number {
  if (dim === undefined || dim === null || dim === 0) return NaN
  if (typeof dim === 'string') {
    if (dim.endsWith('%')) return parseFloat(dim) / 100
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
 * Write text bytes to the pool. Returns the byte offset.
 * Sets textLength via raw u32 write (Rust reads it alongside offset).
 * Does NOT go through SharedSlotBuffer for length — the offset write
 * through repeat() triggers the reactive notification.
 */
function writeTextToPool(
  views: ReturnType<typeof getViews>,
  index: number,
  text: string
): number {
  const encoded = textEncoder.encode(text)
  let writePtr = views.header[HEADER_TEXT_POOL_WRITE_PTR]
  const capacity = views.header[HEADER_TEXT_POOL_CAPACITY]

  // Check capacity, compact if needed
  if (writePtr + encoded.length > capacity) {
    // Inline compaction — repack live text to front
    const nodeCount = views.header[1] // HEADER_NODE_COUNT
    let newPtr = 0
    for (let i = 0; i < nodeCount; i++) {
      const off = views.u32[10][i] // U32_TEXT_OFFSET
      const len = views.u32[11][i] // U32_TEXT_LENGTH
      if (len === 0) continue
      if (off !== newPtr) views.textPool.copyWithin(newPtr, off, off + len)
      views.u32[10][i] = newPtr
      newPtr += len
    }
    views.header[HEADER_TEXT_POOL_WRITE_PTR] = newPtr
    writePtr = newPtr

    // If still not enough, truncate
    if (writePtr + encoded.length > capacity) {
      const available = capacity - writePtr
      if (available <= 0) {
        views.u32[U32_TEXT_LENGTH][index] = 0
        return writePtr
      }
      views.textPool.set(encoded.subarray(0, available), writePtr)
      views.u32[U32_TEXT_LENGTH][index] = available
      views.header[HEADER_TEXT_POOL_WRITE_PTR] = writePtr + available
      return writePtr
    }
  }

  views.textPool.set(encoded, writePtr)
  views.u32[U32_TEXT_LENGTH][index] = encoded.length
  views.header[HEADER_TEXT_POOL_WRITE_PTR] = writePtr + encoded.length
  return writePtr
}

// =============================================================================
// TEXT COMPONENT
// =============================================================================

export function text(props: TextProps): Cleanup {
  const arrays = getArrays()
  const views = getViews()
  const index = allocateIndex(props.id)
  const disposals: (() => void)[] = []

  pushCurrentComponent(index)

  // --------------------------------------------------------------------------
  // CORE
  // --------------------------------------------------------------------------
  arrays.componentType.set(index, ComponentType.TEXT)
  disposals.push(repeat(getCurrentParentIndex(), arrays.parentIndex, index))

  if (props.visible !== undefined) {
    disposals.push(repeat(numInput(props.visible, 1), arrays.visible, index))
  }

  // --------------------------------------------------------------------------
  // TEXT CONTENT — single repeater, no effects
  // --------------------------------------------------------------------------
  // The readFn encodes UTF-8 → writes bytes to pool → sets length raw → returns offset.
  // repeat() writes offset to textOffset SharedSlotBuffer → reactive notification → Rust wakes.
  // Rust reads offset + length together. Text measurement handled by Rust (Taffy content_size).
  if (isReactive(props.content)) {
    disposals.push(repeat(
      () => writeTextToPool(views, index, String(unwrap(props.content))),
      arrays.textOffset,
      index
    ))
  } else {
    // Static text — write once, no repeater needed
    setNodeText(views, index, String(props.content))
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
  if (props.attrs !== undefined) disposals.push(repeat(numInput(props.attrs), arrays.textAttrs, index))
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
