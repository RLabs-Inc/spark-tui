/**
 * SparkTUI Focus State
 *
 * Reactive focus state exposed as signals.
 * State updates when events arrive from the dispatcher.
 *
 * PURELY REACTIVE: No polling, no intervals.
 * Signals update when the event dispatcher routes focus events.
 */

import { signal, derived } from '@rlabs-inc/signals'
import { isInitialized, getBuffer } from '../bridge'
import type { FocusEvent } from '../engine/events'
import { EventType, registerFocusHandler } from '../engine/events'

// Re-export FocusEvent type
export type { FocusEvent }

// =============================================================================
// REACTIVE STATE
// =============================================================================

/** Internal signal for currently focused component index */
const focusedIndexSignal = signal<number>(-1)

/**
 * Currently focused component index.
 * -1 means nothing is focused.
 * Reactive signal - updates when focus changes.
 */
export const focusedIndex = focusedIndexSignal

/**
 * Whether any component is currently focused.
 * Derived from focusedIndex.
 */
export const hasFocus = derived(() => focusedIndexSignal.value >= 0)

// =============================================================================
// ID <-> INDEX MAPPING
// =============================================================================

/** Map from component ID to index for programmatic focus */
const idToIndex = new Map<string, number>()

/** Map from index to ID for reverse lookup */
const indexToId = new Map<number, string>()

/**
 * Register an ID to index mapping.
 * Called internally when components with IDs are created.
 * @internal
 */
export function _registerIdMapping(id: string, index: number): void {
  idToIndex.set(id, index)
  indexToId.set(index, id)
}

/**
 * Unregister an ID mapping.
 * Called internally when components are destroyed.
 * @internal
 */
export function _unregisterIdMapping(id: string): void {
  const index = idToIndex.get(id)
  if (index !== undefined) {
    indexToId.delete(index)
  }
  idToIndex.delete(id)
}

/**
 * Unregister by index.
 * Called internally when components are destroyed.
 * @internal
 */
export function _unregisterIndexMapping(index: number): void {
  const id = indexToId.get(index)
  if (id !== undefined) {
    idToIndex.delete(id)
  }
  indexToId.delete(index)
}

// =============================================================================
// INTERNAL UPDATE FUNCTION
// =============================================================================

/**
 * Called by event dispatcher when a focus event arrives.
 * Updates the focused index signal.
 * @internal
 */
export function _updateFocus(event: FocusEvent): void {
  if (event.type === EventType.Focus) {
    const prevIndex = focusedIndexSignal.value
    focusedIndexSignal.value = event.componentIndex

    // Dispatch callbacks
    if (prevIndex >= 0 && prevIndex !== event.componentIndex) {
      _dispatchFocusCallback(prevIndex, 'blur')
    }
    _dispatchFocusCallback(event.componentIndex, 'focus')
  } else if (event.type === EventType.Blur) {
    const prevIndex = focusedIndexSignal.value
    focusedIndexSignal.value = -1

    if (prevIndex >= 0) {
      _dispatchFocusCallback(prevIndex, 'blur')
    }
  }
}

// =============================================================================
// PUBLIC API - FOCUS STATE
// =============================================================================

/**
 * Get the ID of the currently focused component.
 * Returns a derived signal that updates when focus changes.
 *
 * @example
 * ```ts
 * import { useFocusedId } from './state/focus'
 *
 * const focusedId = useFocusedId()
 *
 * effect(() => {
 *   console.log('Focused:', focusedId.value)
 * })
 * ```
 */
export function useFocusedId(): { readonly value: string | null } {
  return derived(() => {
    const index = focusedIndexSignal.value
    if (index < 0) return null
    return indexToId.get(index) ?? null
  })
}

/**
 * Check if a specific component is currently focused.
 * Returns a derived signal.
 *
 * @param indexOrId - Component index or ID
 */
export function isFocused(indexOrId: number | string): { readonly value: boolean } {
  const index = typeof indexOrId === 'number' ? indexOrId : idToIndex.get(indexOrId)

  return derived(() => {
    if (index === undefined) return false
    return focusedIndexSignal.value === index
  })
}

// =============================================================================
// PUBLIC API - FOCUS CONTROL
// =============================================================================

/**
 * Focus a component by index or ID.
 *
 * @param indexOrId - Component index or ID string
 *
 * @example
 * ```ts
 * import { focus } from './state/focus'
 *
 * // Focus by ID
 * focus('username-input')
 *
 * // Focus by index (internal use)
 * focus(5)
 * ```
 */
export function focus(indexOrId: number | string): void {
  const index = typeof indexOrId === 'number'
    ? indexOrId
    : idToIndex.get(indexOrId)

  if (index === undefined) {
    if (typeof indexOrId === 'string') {
      console.warn(`focus: No component found with ID '${indexOrId}'`)
    }
    return
  }

  // Update local state immediately
  // In the full implementation, this would write to SharedBuffer
  // and let Rust handle focus change + emit Focus/Blur events
  const prevIndex = focusedIndexSignal.value
  if (prevIndex === index) return // Already focused

  focusedIndexSignal.value = index

  // Dispatch callbacks
  if (prevIndex >= 0) {
    _dispatchFocusCallback(prevIndex, 'blur')
  }
  _dispatchFocusCallback(index, 'focus')
}

/**
 * Remove focus from the currently focused component.
 *
 * @example
 * ```ts
 * import { blur } from './state/focus'
 *
 * blur()
 * ```
 */
export function blur(): void {
  const prevIndex = focusedIndexSignal.value
  if (prevIndex < 0) return // Nothing focused

  focusedIndexSignal.value = -1

  _dispatchFocusCallback(prevIndex, 'blur')
}

/**
 * Move focus to the next focusable component.
 * Respects tab order.
 *
 * @example
 * ```ts
 * import { focusNext } from './state/focus'
 *
 * onKey('Tab', () => {
 *   focusNext()
 *   return true
 * })
 * ```
 */
export function focusNext(): void {
  // TODO: Write command to SharedBuffer for Rust to handle
  // Rust has the full tree and can calculate next focusable
  console.warn('focusNext: Not yet implemented - requires Rust integration')
}

/**
 * Move focus to the previous focusable component.
 * Respects tab order.
 *
 * @example
 * ```ts
 * import { focusPrevious } from './state/focus'
 *
 * onKey('Shift+Tab', () => {
 *   focusPrevious()
 *   return true
 * })
 * ```
 */
export function focusPrevious(): void {
  // TODO: Write command to SharedBuffer for Rust to handle
  console.warn('focusPrevious: Not yet implemented - requires Rust integration')
}

/**
 * Focus the first focusable component.
 */
export function focusFirst(): void {
  // TODO: Write command to SharedBuffer for Rust to handle
  console.warn('focusFirst: Not yet implemented - requires Rust integration')
}

/**
 * Focus the last focusable component.
 */
export function focusLast(): void {
  // TODO: Write command to SharedBuffer for Rust to handle
  console.warn('focusLast: Not yet implemented - requires Rust integration')
}

// =============================================================================
// FOCUS CALLBACKS
// =============================================================================

interface FocusCallbacks {
  onFocus?: () => void
  onBlur?: () => void
}

/** Registered focus callbacks per component */
const focusCallbacks = new Map<number, FocusCallbacks>()

/**
 * Register focus/blur callbacks for a component.
 * Returns unsubscribe function.
 *
 * @param index - Component index
 * @param callbacks - Object with onFocus and/or onBlur callbacks
 * @returns Unsubscribe function
 *
 * @example
 * ```ts
 * import { registerFocusCallbacks } from './state/focus'
 *
 * // Inside a component
 * const unsub = registerFocusCallbacks(index, {
 *   onFocus: () => {
 *     console.log('Component focused')
 *   },
 *   onBlur: () => {
 *     console.log('Component blurred')
 *   },
 * })
 * ```
 */
export function registerFocusCallbacks(index: number, callbacks: FocusCallbacks): () => void {
  focusCallbacks.set(index, callbacks)

  return () => {
    focusCallbacks.delete(index)
  }
}

/**
 * Dispatch focus callback to a component.
 * @internal
 */
export function _dispatchFocusCallback(index: number, type: 'focus' | 'blur'): void {
  const callbacks = focusCallbacks.get(index)
  if (callbacks) {
    if (type === 'focus') {
      callbacks.onFocus?.()
    } else {
      callbacks.onBlur?.()
    }
  }
}

/**
 * Cleanup focus callbacks for a component.
 * Called when component unmounts.
 */
export function cleanupFocusCallbacks(index: number): void {
  focusCallbacks.delete(index)
  _unregisterIndexMapping(index)
}
