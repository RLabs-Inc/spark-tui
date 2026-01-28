/**
 * SparkTUI - Buffer Bridge
 *
 * SharedBuffer singleton and color packing utilities.
 * The actual arrays live in engine/arrays/ — this module only provides:
 *   1. SharedBuffer singleton (initBuffer/getViews)
 *   2. Color packing (packRgba) with ANSI sentinel encoding
 *   3. Packing source helper (packingSource) for reactive RGBA → u32
 *
 * Color encoding contract (must match Rust shared_buffer.rs unpack_to_rgba):
 *   packed == 0         → terminal default (inherit/null)
 *   alpha == 1          → ANSI palette color (r byte = palette index 0-255)
 *   alpha == 255 (else) → normal ARGB color
 */

import type { RGBA } from '../types'
import type { SharedBufferViews } from './shared-buffer'

// =============================================================================
// SHARED BUFFER SINGLETON
// =============================================================================

let _views: SharedBufferViews | null = null

/** Initialize the buffer bridge with SharedBuffer views. Call once at startup. */
export function initBuffer(views: SharedBufferViews): void {
  _views = views
}

/** Get the SharedBuffer views. Throws if not initialized. */
export function getViews(): SharedBufferViews {
  if (!_views) throw new Error('Buffer not initialized. Call initBuffer() first.')
  return _views
}

// =============================================================================
// COLOR PACKING — RGBA → packed u32 ARGB
// =============================================================================

/**
 * Pack an RGBA value to u32 for SharedBuffer.
 *
 * Handles special markers:
 * - null → 0 (terminal default)
 * - r=-1 (TERMINAL_DEFAULT) → 0
 * - r=-2 (ANSI marker) → alpha=1 sentinel with palette index in r byte
 * - normal RGB → standard ARGB packing
 */
export function packRgba(rgba: RGBA | null): number {
  if (rgba === null) return 0

  // Terminal default marker
  if (rgba.r === -1) return 0

  // ANSI palette marker: alpha=1 sentinel, r byte = index
  if (rgba.r === -2) {
    return (1 << 24) | ((rgba.g & 0xFF) << 16)
  }

  // Normal ARGB
  return ((rgba.a & 0xFF) << 24) | ((rgba.r & 0xFF) << 16) | ((rgba.g & 0xFF) << 8) | (rgba.b & 0xFF)
}

// =============================================================================
// HELPER — Create a packing source for primitives
// =============================================================================

/**
 * Create a source function that packs RGBA to u32.
 *
 * Wraps any Reactive<RGBA | null> into a number or getter returning number,
 * suitable for TypedSlotArray.setSource().
 *
 * Usage in primitives:
 * ```ts
 * import * as visual from '../engine/arrays/visual'
 * import { packingSource } from '../bridge/buffer'
 *
 * visual.fgColor.setSource(index, packingSource(props.fg))
 * ```
 *
 * Handles all reactive source types:
 * - null → 0 (terminal default)
 * - Static RGBA → returns packed u32
 * - Signal<RGBA> → returns getter that packs on read
 * - Derived<RGBA> → returns getter that packs on read
 * - () => RGBA → returns getter that packs on read
 */
export function packingSource(
  source: RGBA | null | (() => RGBA | null) | { value: RGBA | null } | { readonly value: RGBA | null }
): number | (() => number) {
  // Null → 0 (terminal default)
  if (source === null) return 0

  // Static RGBA object (not a function, not a signal)
  if (typeof source === 'object' && !('value' in source) && 'r' in source) {
    return packRgba(source as RGBA)
  }

  // Getter function
  if (typeof source === 'function') {
    return () => packRgba((source as () => RGBA | null)())
  }

  // Signal or Derived (has .value)
  if (typeof source === 'object' && 'value' in source) {
    return () => packRgba(source.value as RGBA | null)
  }

  return 0
}
