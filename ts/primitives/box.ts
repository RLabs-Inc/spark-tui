/**
 * TUI Framework - Box Primitive (AoS)
 *
 * Container component with flexbox layout, borders, and background.
 * Uses AoS SharedBuffer for cache-friendly Rust reads.
 *
 * REACTIVITY: Props are passed directly to repeat() which preserves reactive links.
 * repeat() handles signals, getters, deriveds, and static values natively.
 *
 * Usage:
 * ```ts
 * const width = signal(40)
 * box({
 *   width,         // Reactive! repeat() wires it to SharedArrayBuffer
 *   height: 10,    // Static — repeat() sets once
 *   border: 1,
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
import { getAoSArrays } from '../bridge'
import { packColor } from '../bridge/shared-buffer-aos'
import type { BoxProps, Cleanup } from './types'

// =============================================================================
// CONVERSION HELPERS — inline, minimal
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

// Dimension: wrap prop for repeat() — returns number (static) or () => number (reactive)
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

// Numeric: wrap prop for repeat() — pass-through, repeat() handles natively
function numInput(prop: unknown, defaultVal = 0): number | (() => number) | { readonly value: number } {
  if (prop === undefined) return defaultVal
  return prop as any // repeat() handles number | signal | getter natively
}

// Boolean → number: converts boolean props (like visible, focusable) to 0/1
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

function alignToNum(a: string | undefined): number {
  switch (a) {
    case 'flex-start': return 0
    case 'flex-end': return 1
    case 'center': return 2
    case 'stretch': return 3
    case 'baseline': return 4
    default: return 3 // stretch (default)
  }
}

function alignContentToNum(a: string | undefined): number {
  switch (a) {
    case 'flex-start': return 0
    case 'flex-end': return 1
    case 'center': return 2
    case 'stretch': return 3
    case 'space-between': return 4
    case 'space-around': return 5
    default: return 0 // flex-start
  }
}

function alignSelfToNum(a: string | undefined): number {
  switch (a) {
    case 'auto': return 0
    case 'flex-start': return 1
    case 'flex-end': return 2
    case 'center': return 3
    case 'stretch': return 4
    case 'baseline': return 5
    default: return 0 // auto
  }
}

// =============================================================================
// BOX COMPONENT
// =============================================================================

export function box(props: BoxProps = {}): Cleanup {
  const arrays = getAoSArrays()
  const index = allocateIndex(props.id)
  const disposals: (() => void)[] = []

  pushCurrentComponent(index)

  // --------------------------------------------------------------------------
  // CORE — static writes
  // --------------------------------------------------------------------------
  arrays.componentType.set(index, ComponentType.BOX)
  disposals.push(repeat(getCurrentParentIndex(), arrays.parentIndex, index))

  // Visibility (default: visible)
  disposals.push(repeat(boolInput(props.visible, 1), arrays.visible, index))

  // --------------------------------------------------------------------------
  // LAYOUT — dimensions, flex, spacing
  // --------------------------------------------------------------------------

  // Dimensions
  if (props.width !== undefined) disposals.push(repeat(dimInput(props.width), arrays.width, index))
  if (props.height !== undefined) disposals.push(repeat(dimInput(props.height), arrays.height, index))
  if (props.minWidth !== undefined) disposals.push(repeat(dimInput(props.minWidth), arrays.minWidth, index))
  if (props.maxWidth !== undefined) disposals.push(repeat(dimInput(props.maxWidth), arrays.maxWidth, index))
  if (props.minHeight !== undefined) disposals.push(repeat(dimInput(props.minHeight), arrays.minHeight, index))
  if (props.maxHeight !== undefined) disposals.push(repeat(dimInput(props.maxHeight), arrays.maxHeight, index))

  // Flex container
  if (props.flexDirection !== undefined) disposals.push(repeat(enumInput(props.flexDirection, flexDirectionToNum), arrays.flexDirection, index))
  if (props.flexWrap !== undefined) disposals.push(repeat(enumInput(props.flexWrap, flexWrapToNum), arrays.flexWrap, index))
  if (props.justifyContent !== undefined) disposals.push(repeat(enumInput(props.justifyContent, justifyToNum), arrays.justifyContent, index))
  if (props.alignItems !== undefined) disposals.push(repeat(enumInput(props.alignItems, alignToNum), arrays.alignItems, index))
  // if (props.alignContent !== undefined)   disposals.push(repeat(enumInput(props.alignContent, alignContentToNum), arrays.alignContent, index))

  // Flex item
  if (props.grow !== undefined) disposals.push(repeat(numInput(props.grow), arrays.grow, index))
  if (props.shrink !== undefined) disposals.push(repeat(numInput(props.shrink), arrays.shrink, index))
  if (props.flexBasis !== undefined) disposals.push(repeat(dimInput(props.flexBasis), arrays.basis, index))
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

  // Border widths (layout spacing: 0 or 1)
  if (props.border !== undefined) {
    const bw = isReactive(props.border) ? (() => unwrap(props.border!) > 0 ? 1 : 0) : (unwrap(props.border) > 0 ? 1 : 0)
    disposals.push(repeat(bw, arrays.borderTopWidth, index))
    disposals.push(repeat(bw, arrays.borderRightWidth, index))
    disposals.push(repeat(bw, arrays.borderBottomWidth, index))
    disposals.push(repeat(bw, arrays.borderLeftWidth, index))
  }
  if (props.borderTop !== undefined) {
    const bw = isReactive(props.borderTop) ? (() => unwrap(props.borderTop!) > 0 ? 1 : 0) : (unwrap(props.borderTop) > 0 ? 1 : 0)
    disposals.push(repeat(bw, arrays.borderTopWidth, index))
  }
  if (props.borderRight !== undefined) {
    const bw = isReactive(props.borderRight) ? (() => unwrap(props.borderRight!) > 0 ? 1 : 0) : (unwrap(props.borderRight) > 0 ? 1 : 0)
    disposals.push(repeat(bw, arrays.borderRightWidth, index))
  }
  if (props.borderBottom !== undefined) {
    const bw = isReactive(props.borderBottom) ? (() => unwrap(props.borderBottom!) > 0 ? 1 : 0) : (unwrap(props.borderBottom) > 0 ? 1 : 0)
    disposals.push(repeat(bw, arrays.borderBottomWidth, index))
  }
  if (props.borderLeft !== undefined) {
    const bw = isReactive(props.borderLeft) ? (() => unwrap(props.borderLeft!) > 0 ? 1 : 0) : (unwrap(props.borderLeft) > 0 ? 1 : 0)
    disposals.push(repeat(bw, arrays.borderLeftWidth, index))
  }

  // --------------------------------------------------------------------------
  // INTERACTION — focusable, tab index
  // --------------------------------------------------------------------------
  const shouldBeFocusable = props.focusable || (props.overflow === 'scroll' && props.focusable !== false)
  if (shouldBeFocusable) {
    arrays.interactionFlags.set(index, 1) // FLAG_FOCUSABLE
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

  // Border style for rendering
  if (props.border !== undefined) disposals.push(repeat(numInput(props.border), arrays.borderStyle, index))
  if (props.borderTop !== undefined) disposals.push(repeat(numInput(props.borderTop), arrays.borderStyleTop, index))
  if (props.borderRight !== undefined) disposals.push(repeat(numInput(props.borderRight), arrays.borderStyleRight, index))
  if (props.borderBottom !== undefined) disposals.push(repeat(numInput(props.borderBottom), arrays.borderStyleBottom, index))
  if (props.borderLeft !== undefined) disposals.push(repeat(numInput(props.borderLeft), arrays.borderStyleLeft, index))

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
    // Dispose all repeaters
    for (const dispose of disposals) dispose()
    disposals.length = 0
    // Unsub event handlers
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
