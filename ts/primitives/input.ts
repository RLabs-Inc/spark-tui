/**
 * TUI Framework - Input Primitive
 *
 * Single-line text input with full reactivity.
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

import { signal } from '@rlabs-inc/signals'
import { ComponentType } from '../types'
import { allocateIndex, releaseIndex, getCurrentParentIndex } from '../engine/registry'
import {
  pushCurrentComponent,
  popCurrentComponent,
  runMountCallbacks,
} from '../engine/lifecycle'
import { cleanupIndex as cleanupKeyboardListeners, onFocused } from '../state/keyboard'
import { onComponent as onMouseComponent } from '../state/mouse'
import { getVariantStyle, t } from '../state/theme'
import { focus as focusComponent, registerFocusCallbacks } from '../state/focus'
import { createCursor } from '../state/drawnCursor'
import { getActiveScope } from './scope'

// Import arrays
import * as core from '../engine/arrays/core'
import * as taffy from '../engine/arrays/taffy'
import * as visual from '../engine/arrays/visual'
import * as textArrays from '../engine/arrays/text'
import * as interaction from '../engine/arrays/interaction'

// Import types
import type { InputProps, Cleanup, BlinkConfig } from './types'
import type { KeyboardEvent } from '../state/keyboard'

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
  const index = allocateIndex(props.id)

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
  // CORE
  // ==========================================================================

  core.componentType[index] = ComponentType.INPUT
  core.parentIndex.setSource(index, getCurrentParentIndex())

  // Visibility
  if (props.visible !== undefined) {
    taffy.visible.setSource(index, props.visible)
    core.visible.setSource(index, props.visible)
  }

  // ==========================================================================
  // TAFFY LAYOUT ARRAYS - Direct binding to TypedArrays
  // ==========================================================================

  // Parent hierarchy (for tree structure)
  taffy.parentIndex.setSource(index, getCurrentParentIndex())

  // Dimensions
  if (props.width !== undefined) {
    taffy.width.setSource(index, taffy.dimensionSource(props.width))
  }
  if (props.height !== undefined) {
    taffy.height.setSource(index, taffy.dimensionSource(props.height))
  }
  if (props.minWidth !== undefined) {
    taffy.minWidth.setSource(index, taffy.dimensionSource(props.minWidth))
  }
  if (props.maxWidth !== undefined) {
    taffy.maxWidth.setSource(index, taffy.dimensionSource(props.maxWidth))
  }
  if (props.minHeight !== undefined) {
    taffy.minHeight.setSource(index, taffy.dimensionSource(props.minHeight))
  }
  if (props.maxHeight !== undefined) {
    taffy.maxHeight.setSource(index, taffy.dimensionSource(props.maxHeight))
  }

  // Spacing - Padding
  if (props.padding !== undefined) {
    taffy.paddingTop.setSource(index, props.paddingTop ?? props.padding)
    taffy.paddingRight.setSource(index, props.paddingRight ?? props.padding)
    taffy.paddingBottom.setSource(index, props.paddingBottom ?? props.padding)
    taffy.paddingLeft.setSource(index, props.paddingLeft ?? props.padding)
  } else {
    if (props.paddingTop !== undefined) taffy.paddingTop.setSource(index, props.paddingTop)
    if (props.paddingRight !== undefined) taffy.paddingRight.setSource(index, props.paddingRight)
    if (props.paddingBottom !== undefined) taffy.paddingBottom.setSource(index, props.paddingBottom)
    if (props.paddingLeft !== undefined) taffy.paddingLeft.setSource(index, props.paddingLeft)
  }

  // Spacing - Margin
  if (props.margin !== undefined) {
    taffy.marginTop.setSource(index, props.marginTop ?? props.margin)
    taffy.marginRight.setSource(index, props.marginRight ?? props.margin)
    taffy.marginBottom.setSource(index, props.marginBottom ?? props.margin)
    taffy.marginLeft.setSource(index, props.marginLeft ?? props.margin)
  } else {
    if (props.marginTop !== undefined) taffy.marginTop.setSource(index, props.marginTop)
    if (props.marginRight !== undefined) taffy.marginRight.setSource(index, props.marginRight)
    if (props.marginBottom !== undefined) taffy.marginBottom.setSource(index, props.marginBottom)
    if (props.marginLeft !== undefined) taffy.marginLeft.setSource(index, props.marginLeft)
  }

  // ==========================================================================
  // TEXT CONTENT - Display via slot array
  // ==========================================================================

  const getDisplayText = () => {
    const val = getValue()
    if (val.length === 0 && props.placeholder) {
      return props.placeholder
    }
    return props.password ? maskChar.repeat(val.length) : val
  }

  textArrays.textContent.setSource(index, getDisplayText)

  if (props.attrs !== undefined) textArrays.textAttrs.setSource(index, props.attrs)

  // ==========================================================================
  // CURSOR - Create cursor with full customization
  // ==========================================================================

  const cursorConfig = props.cursor ?? {}

  let blinkEnabled = true
  let blinkFps = 2
  let altChar: string | undefined

  if (cursorConfig.blink === false) {
    blinkEnabled = false
  } else if (typeof cursorConfig.blink === 'object') {
    const blinkConfig = cursorConfig.blink as BlinkConfig
    blinkEnabled = blinkConfig.enabled !== false
    blinkFps = blinkConfig.fps ?? 2
    altChar = blinkConfig.altChar
  }

  const cursor = createCursor(index, {
    style: cursorConfig.style,
    char: cursorConfig.char,
    blink: blinkEnabled,
    fps: blinkFps,
    altChar,
  })

  // Sync cursor position (clamped to value length)
  interaction.cursorPosition.setSource(index, () => Math.min(cursorPos.value, getValue().length))

  // ==========================================================================
  // BORDER
  // ==========================================================================

  if (props.border !== undefined) visual.borderStyle.setSource(index, props.border)
  if (props.borderTop !== undefined) visual.borderTop.setSource(index, props.borderTop)
  if (props.borderRight !== undefined) visual.borderRight.setSource(index, props.borderRight)
  if (props.borderBottom !== undefined) visual.borderBottom.setSource(index, props.borderBottom)
  if (props.borderLeft !== undefined) visual.borderLeft.setSource(index, props.borderLeft)
  if (props.borderColor !== undefined) visual.borderColor.setSource(index, props.borderColor)

  // ==========================================================================
  // VISUAL - Colors with variant support
  // ==========================================================================

  if (props.variant && props.variant !== 'default') {
    const variant = props.variant
    if (props.fg !== undefined) {
      visual.fgColor.setSource(index, props.fg)
    } else {
      visual.fgColor.setSource(index, () => getVariantStyle(variant).fg)
    }
    if (props.bg !== undefined) {
      visual.bgColor.setSource(index, props.bg)
    } else {
      visual.bgColor.setSource(index, () => getVariantStyle(variant).bg)
    }
    if (props.borderColor === undefined) {
      visual.borderColor.setSource(index, () => getVariantStyle(variant).border)
    }
  } else {
    visual.fgColor.setSource(index, props.fg !== undefined ? props.fg : t.textBright)
    if (props.bg !== undefined) visual.bgColor.setSource(index, props.bg)
  }
  if (props.opacity !== undefined) visual.opacity.setSource(index, props.opacity)

  // ==========================================================================
  // FOCUS - Inputs are always focusable
  // ==========================================================================

  interaction.focusable.setSource(index, 1)
  if (props.tabIndex !== undefined) {
    interaction.tabIndex.setSource(index, props.tabIndex)
  }

  // ==========================================================================
  // KEYBOARD HANDLERS
  // ==========================================================================

  const handleKeyEvent = (event: KeyboardEvent): boolean => {
    const val = getValue()
    const pos = Math.min(cursorPos.value, val.length)
    const maxLen = props.maxLength ?? 0

    switch (event.key) {
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
        if (event.key.length === 1 && !event.modifiers.ctrl && !event.modifiers.alt && !event.modifiers.meta) {
          if (maxLen > 0 && val.length >= maxLen) {
            return true
          }
          const newVal = val.slice(0, pos) + event.key + val.slice(pos)
          setValue(newVal)
          cursorPos.value = pos + 1
          props.onChange?.(newVal)
          return true
        }
        return false
    }
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
    unsubFocusCallbacks()
    unsubMouse()
    cursor.dispose()
    unsubKeyboard()
    cleanupKeyboardListeners(index)
    interaction.cursorPosition.clear(index)
    releaseIndex(index)
  }

  const scope = getActiveScope()
  if (scope) {
    scope.cleanups.push(cleanup)
  }

  return cleanup
}
