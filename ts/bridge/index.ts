/**
 * SparkTUI Bridge — Global initialization.
 *
 * Creates the SharedArrayBuffer, SoA views, reactive arrays, and wake notifier.
 * Call initBridge() once at startup. Primitives import getArrays()/getViews().
 *
 * Usage:
 * ```ts
 * import { initBridge, getArrays, getViews } from './bridge'
 *
 * const { views, arrays } = initBridge()
 * // Now primitives use getArrays() internally
 * ```
 */

import { createSharedBuffer, type SharedBufferViews } from './shared-buffer'
import { createReactiveArrays, type ReactiveArrays } from './reactive-arrays'
import { createWakeNotifier, createNoopNotifier } from './notify'
import type { Notifier } from '@rlabs-inc/signals'

// =============================================================================
// Singleton state
// =============================================================================

let _views: SharedBufferViews | null = null
let _arrays: ReactiveArrays | null = null
let _notifier: Notifier | null = null

// =============================================================================
// Init
// =============================================================================

export interface BridgeOptions {
  /** Use NoopNotifier (for testing without Rust side) */
  noopNotifier?: boolean
}

/**
 * Initialize the shared memory bridge.
 *
 * Creates SharedArrayBuffer + SoA views + reactive SharedSlotBuffers + notifier.
 * Safe to call multiple times — returns existing state after first init.
 */
export function initBridge(opts?: BridgeOptions): {
  views: SharedBufferViews
  arrays: ReactiveArrays
  notifier: Notifier
} {
  if (_views) {
    return { views: _views, arrays: _arrays!, notifier: _notifier! }
  }

  _views = createSharedBuffer()
  _notifier = opts?.noopNotifier
    ? createNoopNotifier()
    : createWakeNotifier(_views)
  _arrays = createReactiveArrays(_views, _notifier)

  return { views: _views, arrays: _arrays, notifier: _notifier }
}

// =============================================================================
// Accessors (for primitives and internal use)
// =============================================================================

/** Get the SharedBufferViews. Throws if not initialized. */
export function getViews(): SharedBufferViews {
  if (!_views) throw new Error('Bridge not initialized — call initBridge() first')
  return _views
}

/** Get the reactive arrays. Throws if not initialized. */
export function getArrays(): ReactiveArrays {
  if (!_arrays) throw new Error('Bridge not initialized — call initBridge() first')
  return _arrays
}

/** Get the notifier. Throws if not initialized. */
export function getNotifier(): Notifier {
  if (!_notifier) throw new Error('Bridge not initialized — call initBridge() first')
  return _notifier
}

/** Check if bridge is initialized. */
export function isInitialized(): boolean {
  return _views !== null
}

// =============================================================================
// Reset (for testing)
// =============================================================================

/** Reset bridge state. For testing only. */
export function resetBridge(): void {
  _views = null
  _arrays = null
  _notifier = null
}
