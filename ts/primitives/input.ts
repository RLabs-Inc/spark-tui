/**
 * TUI Framework - Input Primitive (AoS)
 *
 * Single-line text input with full reactivity.
 * Uses AoS SharedBuffer for cache-friendly Rust reads.
 *
 * Features:
 * - Two-way value binding via slot arrays
 * - Cursor navigation (arrows, home, end)
 * - Text editing (backspace, delete)
 * - Password mode
 * - Placeholder text
 * - Theme variants
 * - Cursor configuration (style, blink, color)
 *
 * REACTIVITY: Props are passed directly to repeat() which preserves reactive links.
 * repeat() handles signals, getters, deriveds, and static values natively.
 *
 * Usage:
 * ```ts
 * const name = signal('')
 * input({
 *   value: name,
 *   placeholder: 'Enter your name...',
 *   onSubmit: (val) => console.log('Submitted:', val)
 * })
 * ```
 */

import { signal, repeat } from '@rlabs-inc/signals'
import { ComponentType } from '../types'
import type { RGBA } from '../types'
import { allocateIndex, releaseIndex, getCurrentParentIndex } from '../engine/registry'
import {
  pushCurrentComponent,
  popCurrentComponent,
  runMountCallbacks,
} from '../engine/lifecycle'
import { cleanupIndex as cleanupKeyboardListeners, onFocused } from '../state/keyboard'
import type { KeyEvent } from '../state/keyboard'
import { hasCtrl, hasAlt, hasMeta } from '../engine/events'
import { onComponent as onMouseComponent } from '../state/mouse'
import { getVariantStyle, t } from '../state/theme'
import { focus as focusComponent, registerFocusCallbacks } from '../state/focus'
import { getActiveScope } from './scope'
import { pulse } from './animation'
import { getAoSArrays, getAoSBuffer } from '../bridge'
import {
  packColor,
  setNodeText,
  HEADER_SIZE,
  STRIDE,
  H_TEXT_POOL_WRITE_PTR,
  U_TEXT_OFFSET,
  U_TEXT_LENGTH,
  U_CURSOR_FLAGS,
  U_CURSOR_STYLE,
  U_CURSOR_FPS,
  U_CURSOR_CHAR,
  U_MAX_LENGTH,
  C_CURSOR_FG,
  C_CURSOR_BG,
  FLAG_FOCUSABLE,
  type AoSBuffer,
} from '../bridge/shared-buffer-aos'
import type { InputProps, Cleanup, BlinkConfig } from './types'

// =============================================================================
// CONVERSION HELPERS — same pattern as box.ts
// =============================================================================

/** Dimension → Taffy float: NaN = auto, 0-1 = percentage, >1 = absolute */
function toDim(dim: number | string | undefined | null): number {
  if (dim === undefined || dim === null || dim === 0) return NaN
  if (typeof dim === 'string') {
    if (dim.endsWith('%')) return parseFloat(dim) / 100
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
function dimInput(prop: InputProps['width']): number | (() => number) {
  if (prop === undefined) return NaN
  if (typeof prop === 'number' || typeof prop === 'string') return toDim(prop)
  return () => toDim(unwrap(prop))
}

// Color: wrap prop for repeat()
function colorInput(prop: InputProps['fg']): number | (() => number) {
  if (prop === undefined) return 0
  if (!isReactive(prop)) return toPackedColor(prop as RGBA | number | null)
  return () => toPackedColor(unwrap(prop as any))
}

// Numeric: wrap prop for repeat() — pass-through, repeat() handles natively
function numInput(prop: unknown, defaultVal = 0): number | (() => number) | { readonly value: number } {
  if (prop === undefined) return defaultVal
  return prop as any // repeat() handles number | signal | getter natively
}

// Boolean → number: converts boolean props (like visible) to 0/1
function boolInput(prop: unknown, defaultVal = 1): number | (() => number) {
  if (prop === undefined) return defaultVal
  if (typeof prop === 'boolean') return prop ? 1 : 0
  if (typeof prop === 'function') return () => (prop as () => boolean)() ? 1 : 0
  if (isReactive(prop)) return () => unwrap(prop as any) ? 1 : 0
  return prop ? 1 : 0
}

// =============================================================================
// DIRECT BUFFER WRITES (for cursor fields not in reactive arrays)
// =============================================================================

function setU8Direct(buf: AoSBuffer, nodeIndex: number, field: number, value: number): void {
  buf.view.setUint8(HEADER_SIZE + nodeIndex * STRIDE + field, value)
}

function setU32Direct(buf: AoSBuffer, nodeIndex: number, field: number, value: number): void {
  buf.view.setUint32(HEADER_SIZE + nodeIndex * STRIDE + field, value, true)
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
  if (writePtr + encoded.length > buf.textPoolSize) {
    const sizeMB = (buf.textPoolSize / 1024 / 1024).toFixed(0)
    throw new Error(
      `Text pool overflow: pool is ${buf.textPoolSize} bytes (${sizeMB}MB), writePtr is at ${writePtr}, ` +
      `trying to write ${encoded.length} bytes. This usually means text content is being ` +
      `appended without reusing slots. Consider: 1) Reusing text slots for dynamic content, ` +
      `2) Implementing text compaction, or 3) Increasing textPoolSize in createAoSBuffer({ textPoolSize: ... }).`
    )
  }

  // Write to pool
  buf.textPool.set(encoded, writePtr)

  // Set offset and length in AoS node
  const base = HEADER_SIZE + index * STRIDE
  buf.view.setUint32(base + U_TEXT_OFFSET, writePtr, true)
  buf.view.setUint32(base + U_TEXT_LENGTH, encoded.length, true)

  // Advance write pointer
  buf.header[H_TEXT_POOL_WRITE_PTR / 4] = writePtr + encoded.length

  return writePtr
}

// =============================================================================
// KEYCODE HELPERS
// =============================================================================

/** Convert keycode to character string if printable */
function keycodeToChar(keycode: number): string | null {
  // Single byte printable characters
  if (keycode >= 32 && keycode <= 126) {
    return String.fromCharCode(keycode)
  }
  return null
}

/** Get special key name from keycode */
function getSpecialKeyName(keycode: number): string | null {
  switch (keycode) {
    case 13: return 'Enter'
    case 27: return 'Escape'
    case 8: return 'Backspace'
    case 127: return 'Delete'
    // Arrow keys (terminal escape sequences as packed u32)
    case 0x1b5b44: return 'ArrowLeft'
    case 0x1b5b43: return 'ArrowRight'
    case 0x1b5b41: return 'ArrowUp'
    case 0x1b5b42: return 'ArrowDown'
    case 0x1b5b48: return 'Home'
    case 0x1b5b46: return 'End'
    // Alternative Home/End codes
    case 0x1b4f48: return 'Home'
    case 0x1b4f46: return 'End'
    default: return null
  }
}

// =============================================================================
// CURSOR STYLE CONVERSION
// =============================================================================

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

function alignToNum(align: string | undefined): number {
  switch (align) {
    case 'center': return 1
    case 'right': return 2
    default: return 0 // left
  }
}

function cursorStyleToNum(style: string | undefined): number {
  switch (style) {
    case 'bar': return 1
    case 'underline': return 2
    default: return 0 // block
  }
}

// =============================================================================
// INPUT COMPONENT
// =============================================================================

/**
 * Create a single-line text input component.
 *
 * Pass a WritableSignal or Binding for two-way value binding.
 * The component handles keyboard input when focused.
 *
 * Supports theme variants for consistent styling:
 * - Core: default, primary, secondary, tertiary, accent
 * - Status: success, warning, error, info
 * - Surface: muted, surface, elevated, ghost, outline
 */
export function input(props: InputProps): Cleanup {
  const arrays = getAoSArrays()
  const buffer = getAoSBuffer()
  const index = allocateIndex(props.id)
  const disposals: (() => void)[] = []

  // Track current component for lifecycle hooks
  pushCurrentComponent(index)

  // ==========================================================================
  // INTERNAL STATE
  // ==========================================================================

  // Cursor position within the text
  const cursorPos = signal(0)

  // Get/set value (handles both WritableSignal and Binding)
  const getValue = () => props.value.value
  const setValue = (v: string) => { props.value.value = v }

  // Password mask character
  const maskChar = props.maskChar ?? '•'

  // ==========================================================================
  // CORE — static writes
  // ==========================================================================

  arrays.componentType.set(index, ComponentType.INPUT)
  disposals.push(repeat(getCurrentParentIndex(), arrays.parentIndex, index))

  // Visibility (default: visible)
  disposals.push(repeat(boolInput(props.visible, 1), arrays.visible, index))

  // ==========================================================================
  // LAYOUT — dimensions
  // ==========================================================================

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
  if (props.alignSelf !== undefined) disposals.push(repeat(enumInput(props.alignSelf, alignSelfToNum), arrays.alignSelf, index))

  // Text alignment
  if (props.align !== undefined) disposals.push(repeat(enumInput(props.align, alignToNum), arrays.textAlign, index))

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

  // Margin
  if (props.margin !== undefined) {
    disposals.push(repeat(numInput(props.marginTop ?? props.margin), arrays.marginTop, index))
    disposals.push(repeat(numInput(props.marginRight ?? props.margin), arrays.marginRight, index))
    disposals.push(repeat(numInput(props.marginBottom ?? props.margin), arrays.marginBottom, index))
    disposals.push(repeat(numInput(props.marginLeft ?? props.margin), arrays.marginLeft, index))
  } else {
    if (props.marginTop !== undefined)    disposals.push(repeat(numInput(props.marginTop), arrays.marginTop, index))
    if (props.marginRight !== undefined)  disposals.push(repeat(numInput(props.marginRight), arrays.marginRight, index))
    if (props.marginBottom !== undefined) disposals.push(repeat(numInput(props.marginBottom), arrays.marginBottom, index))
    if (props.marginLeft !== undefined)   disposals.push(repeat(numInput(props.marginLeft), arrays.marginLeft, index))
  }

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

  // ==========================================================================
  // TEXT CONTENT - Display via text pool
  // ==========================================================================

  const getDisplayText = () => {
    const val = getValue()
    if (val.length === 0 && props.placeholder) {
      return props.placeholder
    }
    return props.password ? maskChar.repeat(val.length) : val
  }

  // Text content is reactive since getValue() reads from a signal
  disposals.push(repeat(
    () => writeTextToPoolAoS(buffer, index, getDisplayText()),
    arrays.textOffset,
    index
  ))

  // ==========================================================================
  // CURSOR CONFIGURATION
  // ==========================================================================

  const cursorConfig = props.cursor ?? {}

  // Parse blink configuration
  let blinkEnabled = true
  let blinkFps = 2

  if (cursorConfig.blink === false) {
    blinkEnabled = false
  } else if (typeof cursorConfig.blink === 'object') {
    const blinkConfig = cursorConfig.blink as BlinkConfig
    blinkEnabled = blinkConfig.enabled !== false
    blinkFps = blinkConfig.fps ?? 2
  }

  // Cursor visibility (blink effect via pulse signal)
  if (blinkEnabled) {
    const blinkSignal = pulse({ fps: blinkFps })
    disposals.push(repeat(() => blinkSignal.value ? 1 : 0, arrays.cursorPosition, index))
    // Also track blink state directly for cursor flags
    disposals.push(() => {
      // Cleanup happens when scope is destroyed - pulse handles its own cleanup
    })
    // Set cursor flags to indicate cursor should blink
    setU8Direct(buffer, index, U_CURSOR_FLAGS, 1) // visible
    setU8Direct(buffer, index, U_CURSOR_FPS, blinkFps)
  } else {
    // Static cursor - always visible
    setU8Direct(buffer, index, U_CURSOR_FLAGS, 1)
    setU8Direct(buffer, index, U_CURSOR_FPS, 0) // 0 = no blink
  }

  // Cursor style
  setU8Direct(buffer, index, U_CURSOR_STYLE, cursorStyleToNum(cursorConfig.style))

  // Custom cursor character
  if (cursorConfig.char) {
    const charCode = cursorConfig.char.codePointAt(0) ?? 0
    setU32Direct(buffer, index, U_CURSOR_CHAR, charCode)
  }

  // Cursor colors
  if (cursorConfig.fg !== undefined) {
    if (isReactive(cursorConfig.fg)) {
      disposals.push(repeat(() => toPackedColor(unwrap(cursorConfig.fg!)), arrays.cursorFg, index))
    } else {
      setU32Direct(buffer, index, C_CURSOR_FG, toPackedColor(cursorConfig.fg as RGBA))
    }
  }
  if (cursorConfig.bg !== undefined) {
    if (isReactive(cursorConfig.bg)) {
      // We'd need a cursorBgColor array - for now use direct write
      setU32Direct(buffer, index, C_CURSOR_BG, toPackedColor(unwrap(cursorConfig.bg)))
    } else {
      setU32Direct(buffer, index, C_CURSOR_BG, toPackedColor(cursorConfig.bg as RGBA))
    }
  }

  // Sync cursor position (clamped to value length)
  disposals.push(repeat(
    () => Math.min(cursorPos.value, getValue().length),
    arrays.cursorPosition,
    index
  ))

  // Max length
  if (props.maxLength !== undefined) {
    setU8Direct(buffer, index, U_MAX_LENGTH, props.maxLength)
  }

  // ==========================================================================
  // VISUAL — colors with variant support
  // ==========================================================================

  if (props.variant && props.variant !== 'default') {
    const variant = props.variant
    // Variant-based colors with user overrides
    disposals.push(repeat(
      props.fg !== undefined ? colorInput(props.fg) : () => toPackedColor(getVariantStyle(variant).fg),
      arrays.fgColor, index
    ))
    disposals.push(repeat(
      props.bg !== undefined ? colorInput(props.bg) : () => toPackedColor(getVariantStyle(variant).bg),
      arrays.bgColor, index
    ))
    if (props.borderColor !== undefined) {
      disposals.push(repeat(colorInput(props.borderColor), arrays.borderColor, index))
    } else {
      disposals.push(repeat(() => toPackedColor(getVariantStyle(variant).border), arrays.borderColor, index))
    }
  } else {
    // Default styling - use colorInput for theme colors to handle derived signals properly
    disposals.push(repeat(
      colorInput(props.fg ?? t.textBright as any),
      arrays.fgColor, index
    ))
    if (props.bg !== undefined) disposals.push(repeat(colorInput(props.bg), arrays.bgColor, index))
    if (props.borderColor !== undefined) disposals.push(repeat(colorInput(props.borderColor), arrays.borderColor, index))
  }
  if (props.opacity !== undefined) disposals.push(repeat(numInput(props.opacity), arrays.opacity, index))

  // ==========================================================================
  // INTERACTION — inputs are always focusable
  // ==========================================================================

  arrays.interactionFlags.set(index, FLAG_FOCUSABLE)
  if (props.tabIndex !== undefined) {
    disposals.push(repeat(numInput(props.tabIndex, -1), arrays.tabIndex, index))
  }

  // ==========================================================================
  // KEYBOARD HANDLERS
  // ==========================================================================

  const handleKeyEvent = (event: KeyEvent): boolean => {
    const val = getValue()
    const pos = Math.min(cursorPos.value, val.length)
    const maxLen = props.maxLength ?? 0

    // Get key name or character
    const specialKey = getSpecialKeyName(event.keycode)
    const charKey = keycodeToChar(event.keycode)

    if (specialKey) {
      switch (specialKey) {
        case 'ArrowLeft':
          if (pos > 0) cursorPos.value = pos - 1
          return true

        case 'ArrowRight':
          if (pos < val.length) cursorPos.value = pos + 1
          return true

        case 'Home':
          cursorPos.value = 0
          return true

        case 'End':
          cursorPos.value = val.length
          return true

        case 'Backspace':
          if (pos > 0) {
            const newVal = val.slice(0, pos - 1) + val.slice(pos)
            setValue(newVal)
            cursorPos.value = pos - 1
            props.onChange?.(newVal)
          }
          return true

        case 'Delete':
          if (pos < val.length) {
            const newVal = val.slice(0, pos) + val.slice(pos + 1)
            setValue(newVal)
            props.onChange?.(newVal)
          }
          return true

        case 'Enter':
          props.onSubmit?.(val)
          return true

        case 'Escape':
          props.onCancel?.()
          return true

        default:
          return false
      }
    }

    // Handle printable characters
    if (charKey && !hasCtrl(event) && !hasAlt(event) && !hasMeta(event)) {
      if (maxLen > 0 && val.length >= maxLen) {
        return true
      }
      const newVal = val.slice(0, pos) + charKey + val.slice(pos)
      setValue(newVal)
      cursorPos.value = pos + 1
      props.onChange?.(newVal)
      return true
    }

    return false
  }

  const unsubKeyboard = onFocused(index, handleKeyEvent)

  const unsubFocusCallbacks = registerFocusCallbacks(index, {
    onFocus: props.onFocus,
    onBlur: props.onBlur,
  })

  // ==========================================================================
  // MOUSE HANDLERS
  // ==========================================================================

  const unsubMouse = onMouseComponent(index, {
    onMouseDown: props.onMouseDown,
    onMouseUp: props.onMouseUp,
    onClick: (event) => {
      focusComponent(index)
      return props.onClick?.(event)
    },
    onMouseEnter: props.onMouseEnter,
    onMouseLeave: props.onMouseLeave,
    onScroll: props.onScroll,
  })

  // ==========================================================================
  // AUTO FOCUS
  // ==========================================================================

  if (props.autoFocus) {
    queueMicrotask(() => focusComponent(index))
  }

  // ==========================================================================
  // LIFECYCLE COMPLETE
  // ==========================================================================

  popCurrentComponent()
  runMountCallbacks(index)

  // ==========================================================================
  // CLEANUP
  // ==========================================================================

  const cleanup = () => {
    // Dispose all repeaters
    for (const dispose of disposals) dispose()
    disposals.length = 0

    // Unsub event handlers
    unsubFocusCallbacks()
    unsubMouse()
    unsubKeyboard()
    cleanupKeyboardListeners(index)
    releaseIndex(index)
  }

  const scope = getActiveScope()
  if (scope) {
    scope.cleanups.push(cleanup)
  }

  return cleanup
}
