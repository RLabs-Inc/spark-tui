/**
 * SparkTUI Keyboard State
 *
 * Reactive keyboard state exposed as signals.
 * State updates when events arrive from the dispatcher.
 *
 * PURELY REACTIVE: No polling, no intervals.
 * Signals update when the event dispatcher routes keyboard events.
 */

import { signal, derived } from '@rlabs-inc/signals'
import type { KeyEvent } from '../engine/events'
import {
  EventType,
  registerKeyHandler,
  registerGlobalKeyHandler,
  cleanupHandlers,
  MODIFIER_CTRL,
  MODIFIER_ALT,
  MODIFIER_SHIFT,
  MODIFIER_META,
  KEY_STATE_PRESS,
  KEY_STATE_REPEAT,
  KEY_STATE_RELEASE,
  hasCtrl,
  hasAlt,
  hasShift,
  hasMeta,
} from '../engine/events'

// =============================================================================
// RE-EXPORTS for convenience
// =============================================================================

export { MODIFIER_CTRL, MODIFIER_ALT, MODIFIER_SHIFT, MODIFIER_META }
export { KEY_STATE_PRESS, KEY_STATE_REPEAT, KEY_STATE_RELEASE }
export { hasCtrl, hasAlt, hasShift, hasMeta }
export type { KeyEvent }

// =============================================================================
// REACTIVE STATE
// =============================================================================

/** Internal signal for last key event - updated by event dispatcher */
const lastEventSignal = signal<KeyEvent | null>(null)

/**
 * Last keyboard event received.
 * Reactive signal - updates when any key is pressed.
 */
export const lastEvent = lastEventSignal

/**
 * Last key pressed as a string.
 * Derived from lastEvent - automatically updates.
 */
export const lastKey = derived(() => {
  const ev = lastEventSignal.value
  if (!ev) return null
  // Convert keycode to string - handles printable characters
  // For special keys, return the keycode as-is for now
  try {
    return String.fromCodePoint(ev.keycode)
  } catch {
    return null
  }
})

/**
 * Whether any modifier key is currently held.
 * Useful for detecting ctrl+key, alt+key combinations.
 */
export const modifiers = derived(() => {
  const ev = lastEventSignal.value
  if (!ev) {
    return { ctrl: false, alt: false, shift: false, meta: false }
  }
  return {
    ctrl: (ev.modifiers & MODIFIER_CTRL) !== 0,
    alt: (ev.modifiers & MODIFIER_ALT) !== 0,
    shift: (ev.modifiers & MODIFIER_SHIFT) !== 0,
    meta: (ev.modifiers & MODIFIER_META) !== 0,
  }
})

// =============================================================================
// INTERNAL UPDATE FUNCTION
// =============================================================================

/**
 * Called by event dispatcher when a key event arrives.
 * @internal
 */
export function _updateLastEvent(event: KeyEvent): void {
  lastEventSignal.value = event
}

// =============================================================================
// PUBLIC API - GLOBAL HANDLERS
// =============================================================================

/**
 * Register a global key handler.
 * Called for all key events regardless of focus.
 *
 * Return `true` from handler to stop propagation.
 *
 * @example
 * ```ts
 * import { on } from './state/keyboard'
 *
 * // Handle all key events
 * const unsub = on((event) => {
 *   console.log('Key pressed:', event.keycode)
 *   if (event.keycode === 27) { // Escape
 *     closeModal()
 *     return true // Consumed
 *   }
 * })
 *
 * // Later: unsub()
 * ```
 */
export function on(handler: (event: KeyEvent) => boolean | void): () => void {
  return registerGlobalKeyHandler(handler)
}

/**
 * Register a handler for a specific key.
 *
 * @param key - The key to listen for (e.g., 'Enter', 'a', ' ')
 * @param handler - Called when the key is pressed
 * @returns Unsubscribe function
 *
 * @example
 * ```ts
 * import { onKey } from './state/keyboard'
 *
 * // Handle Enter key globally
 * const unsub = onKey('Enter', () => {
 *   submitForm()
 *   return true
 * })
 * ```
 */
export function onKey(key: string, handler: () => boolean | void): () => void {
  return registerGlobalKeyHandler((event) => {
    // Only handle key press events (not repeat or release)
    if (event.keyState !== KEY_STATE_PRESS) return

    // Check printable character
    try {
      const keyStr = String.fromCodePoint(event.keycode)
      if (keyStr === key) {
        return handler()
      }
    } catch {
      // Non-printable keycode, skip string comparison
    }

    // Also check special key names
    if (getSpecialKeyName(event.keycode) === key) {
      return handler()
    }
  })
}

/**
 * Register a key handler for when a specific component is focused.
 *
 * @param index - Component index
 * @param handler - Called when key is pressed while component is focused
 * @returns Unsubscribe function
 *
 * @example
 * ```ts
 * // Inside a component
 * onFocused(index, (event) => {
 *   if (event.keycode === 13) { // Enter
 *     handleSubmit()
 *     return true
 *   }
 * })
 * ```
 */
export function onFocused(index: number, handler: (event: KeyEvent) => boolean | void): () => void {
  return registerKeyHandler(index, handler)
}

/**
 * Cleanup all keyboard handlers for a component.
 * Called automatically on component unmount.
 *
 * @param index - Component index to cleanup
 */
export function cleanupIndex(index: number): void {
  cleanupHandlers(index)
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/**
 * Get the name of a special key from its keycode.
 * Returns undefined for printable characters.
 */
function getSpecialKeyName(keycode: number): string | undefined {
  switch (keycode) {
    case 13: return 'Enter'
    case 27: return 'Escape'
    case 9: return 'Tab'
    case 8: return 'Backspace'
    case 127: return 'Delete'
    case 32: return 'Space'
    // Arrow keys (using terminal codes)
    case 0x1b5b41: return 'ArrowUp'
    case 0x1b5b42: return 'ArrowDown'
    case 0x1b5b43: return 'ArrowRight'
    case 0x1b5b44: return 'ArrowLeft'
    // Function keys would go here
    default: return undefined
  }
}

/**
 * Check if a key event matches a key combination.
 *
 * @example
 * ```ts
 * if (matchesKey(event, 'Ctrl+S')) {
 *   save()
 * }
 * ```
 */
export function matchesKey(event: KeyEvent, combo: string): boolean {
  const parts = combo.split('+').map(p => p.trim().toLowerCase())
  const key = parts.pop()!
  const mods = new Set(parts)

  // Check modifiers
  if (mods.has('ctrl') !== hasCtrl(event)) return false
  if (mods.has('alt') !== hasAlt(event)) return false
  if (mods.has('shift') !== hasShift(event)) return false
  if (mods.has('meta') !== hasMeta(event)) return false

  // Check key
  try {
    const keyStr = String.fromCodePoint(event.keycode).toLowerCase()
    if (keyStr === key) return true
  } catch {
    // Non-printable
  }

  const specialName = getSpecialKeyName(event.keycode)?.toLowerCase()
  return specialName === key
}

/**
 * Check if the event is a key press (not repeat or release).
 */
export function isPress(event: KeyEvent): boolean {
  return event.keyState === KEY_STATE_PRESS
}

/**
 * Check if the event is a key repeat.
 */
export function isRepeat(event: KeyEvent): boolean {
  return event.keyState === KEY_STATE_REPEAT
}

/**
 * Check if the event is a key release.
 */
export function isRelease(event: KeyEvent): boolean {
  return event.keyState === KEY_STATE_RELEASE
}
