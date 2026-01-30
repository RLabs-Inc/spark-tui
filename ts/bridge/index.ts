/**
 * SparkTUI Bridge — Global initialization.
 *
 * Creates the AoS SharedArrayBuffer, reactive arrays, and wake notifier.
 * Call initBridgeAoS() once at startup. Primitives import getAoSArrays()/getAoSBuffer().
 */

import { createAoSBuffer, type AoSBuffer, MAX_NODES } from './shared-buffer-aos'
import { createReactiveArraysAoS, type ReactiveArraysAoS } from './reactive-arrays-aos'
import { createNoopNotifier, createWakeNotifierAoS } from './notify'
import type { Notifier } from '@rlabs-inc/signals'

// =============================================================================
// Singleton state
// =============================================================================

let _aosBuf: AoSBuffer | null = null
let _aosArrays: ReactiveArraysAoS | null = null
let _aosNotifier: Notifier | null = null

// =============================================================================
// Init
// =============================================================================

export interface BridgeOptions {
  /** Use NoopNotifier (for testing without Rust side) */
  noopNotifier?: boolean
}

/** Check if bridge is initialized. */
export function isInitialized(): boolean {
  return _aosBuf !== null
}

/** Alias for isInitialized (for compatibility). */
export const isInitializedAoS = isInitialized

/**
 * Initialize the AoS shared memory bridge.
 *
 * Creates AoS SharedArrayBuffer + reactive SharedSlotBuffers + notifier.
 * Safe to call multiple times — returns existing state after first init.
 */
export function initBridgeAoS(opts?: BridgeOptions): {
  buffer: AoSBuffer
  arrays: ReactiveArraysAoS
  notifier: Notifier
} {
  if (_aosBuf) {
    return { buffer: _aosBuf, arrays: _aosArrays!, notifier: _aosNotifier! }
  }

  _aosBuf = createAoSBuffer()
  _aosNotifier = opts?.noopNotifier
    ? createNoopNotifier()
    : createWakeNotifierAoS(_aosBuf)
  _aosArrays = createReactiveArraysAoS(_aosBuf, _aosNotifier)

  return { buffer: _aosBuf, arrays: _aosArrays, notifier: _aosNotifier }
}

// =============================================================================
// AoS Accessors
// =============================================================================

/** Get the AoS buffer. Throws if not initialized. */
export function getAoSBuffer(): AoSBuffer {
  if (!_aosBuf) throw new Error('AoS Bridge not initialized — call initBridgeAoS() first')
  return _aosBuf
}

/** Get the AoS reactive arrays. Throws if not initialized. */
export function getAoSArrays(): ReactiveArraysAoS {
  if (!_aosArrays) throw new Error('AoS Bridge not initialized — call initBridgeAoS() first')
  return _aosArrays
}

/** Get the AoS notifier. Throws if not initialized. */
export function getAoSNotifier(): Notifier {
  if (!_aosNotifier) throw new Error('AoS Bridge not initialized — call initBridgeAoS() first')
  return _aosNotifier
}

// =============================================================================
// Reset (for testing)
// =============================================================================

/** Reset bridge state. For testing only. */
export function resetBridge(): void {
  _aosBuf = null
  _aosArrays = null
  _aosNotifier = null
}
