/**
 * SparkTUI Bridge — Global initialization.
 *
 * Creates the SharedArrayBuffer, reactive arrays, and wake notifier.
 * Call initBridge() once at startup. Primitives import getBuffer()/getArrays().
 */

import { createSharedBuffer, type SharedBuffer, DEFAULT_MAX_NODES } from './shared-buffer'
import { createReactiveArrays, type ReactiveArrays } from './reactive-arrays'
import { createNoopNotifier, createWakeNotifier } from './notify'
import type { Notifier } from '@rlabs-inc/signals'

// =============================================================================
// Singleton state
// =============================================================================

let _buffer: SharedBuffer | null = null
let _arrays: ReactiveArrays | null = null
let _notifier: Notifier | null = null

// =============================================================================
// Init
// =============================================================================

export interface BridgeOptions {
  /** Use NoopNotifier (for testing without Rust side) */
  noopNotifier?: boolean
}

/** Check if bridge is initialized. */
export function isInitialized(): boolean {
  return _buffer !== null
}

/**
 * Initialize the shared memory bridge.
 *
 * Creates SharedArrayBuffer + reactive slot buffers + notifier.
 * Safe to call multiple times — returns existing state after first init.
 */
export function initBridge(opts?: BridgeOptions): {
  buffer: SharedBuffer
  arrays: ReactiveArrays
  notifier: Notifier
} {
  if (_buffer) {
    return { buffer: _buffer, arrays: _arrays!, notifier: _notifier! }
  }

  _buffer = createSharedBuffer()
  _notifier = opts?.noopNotifier
    ? createNoopNotifier()
    : createWakeNotifier(_buffer)
  _arrays = createReactiveArrays(_buffer, _notifier)

  return { buffer: _buffer, arrays: _arrays, notifier: _notifier }
}

// =============================================================================
// Accessors
// =============================================================================

/** Get the shared buffer. Throws if not initialized. */
export function getBuffer(): SharedBuffer {
  if (!_buffer) throw new Error('Bridge not initialized — call initBridge() first')
  return _buffer
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

// =============================================================================
// Reset (for testing)
// =============================================================================

/** Reset bridge state. For testing only. */
export function resetBridge(): void {
  _buffer = null
  _arrays = null
  _notifier = null
}

// =============================================================================
// Re-exports for convenience
// =============================================================================

export type { SharedBuffer } from './shared-buffer'
export type { ReactiveArrays } from './reactive-arrays'
export { DEFAULT_MAX_NODES } from './shared-buffer'
