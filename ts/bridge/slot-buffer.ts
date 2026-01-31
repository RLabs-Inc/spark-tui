/**
 * Slot Buffer - Direct DataView writes with reactive tracking
 *
 * Implements SharedSlotBuffer interface from @rlabs-inc/signals.
 * No Proxy, no virtual arrays. Just fast direct memory access.
 */

import type { Notifier, SharedSlotBuffer, Source } from '@rlabs-inc/signals'
import { HEADER_SIZE, NODE_STRIDE, DEFAULT_MAX_NODES } from './shared-buffer'

type DataType = 'f32' | 'u32' | 'i32' | 'u16' | 'i16' | 'u8' | 'i8'

/**
 * Create a slot buffer for a specific field.
 * Direct DataView access - no Proxy overhead.
 * Implements full SharedSlotBuffer interface for compatibility with repeat().
 */
export function createSlotBuffer(
  view: DataView,
  fieldOffset: number,
  dataType: DataType,
  notifier: Notifier,
  defaultValue: number = 0
): SharedSlotBuffer {
  // Create getter/setter based on data type
  let getter: (index: number) => number
  let setter: (index: number, value: number) => void

  switch (dataType) {
    case 'f32':
      getter = (index: number) => view.getFloat32(HEADER_SIZE + index * NODE_STRIDE + fieldOffset, true)
      setter = (index: number, value: number) => view.setFloat32(HEADER_SIZE + index * NODE_STRIDE + fieldOffset, value, true)
      break
    case 'u32':
      getter = (index: number) => view.getUint32(HEADER_SIZE + index * NODE_STRIDE + fieldOffset, true)
      setter = (index: number, value: number) => view.setUint32(HEADER_SIZE + index * NODE_STRIDE + fieldOffset, value, true)
      break
    case 'i32':
      getter = (index: number) => view.getInt32(HEADER_SIZE + index * NODE_STRIDE + fieldOffset, true)
      setter = (index: number, value: number) => view.setInt32(HEADER_SIZE + index * NODE_STRIDE + fieldOffset, value, true)
      break
    case 'u16':
      getter = (index: number) => view.getUint16(HEADER_SIZE + index * NODE_STRIDE + fieldOffset, true)
      setter = (index: number, value: number) => view.setUint16(HEADER_SIZE + index * NODE_STRIDE + fieldOffset, value, true)
      break
    case 'i16':
      getter = (index: number) => view.getInt16(HEADER_SIZE + index * NODE_STRIDE + fieldOffset, true)
      setter = (index: number, value: number) => view.setInt16(HEADER_SIZE + index * NODE_STRIDE + fieldOffset, value, true)
      break
    case 'u8':
      getter = (index: number) => view.getUint8(HEADER_SIZE + index * NODE_STRIDE + fieldOffset)
      setter = (index: number, value: number) => view.setUint8(HEADER_SIZE + index * NODE_STRIDE + fieldOffset, value)
      break
    case 'i8':
      getter = (index: number) => view.getInt8(HEADER_SIZE + index * NODE_STRIDE + fieldOffset)
      setter = (index: number, value: number) => view.setInt8(HEADER_SIZE + index * NODE_STRIDE + fieldOffset, value)
      break
  }

  // Create a dummy typed array for the `raw` property (required by interface)
  // In this layout, we don't have contiguous typed arrays per field, so this is a placeholder
  const dummyRaw = new Float32Array(0)

  return {
    capacity: DEFAULT_MAX_NODES,
    raw: dummyRaw,

    get(index: number): number {
      return getter(index)
    },

    peek(index: number): number {
      // Same as get - we don't track reads in this implementation
      return getter(index)
    },

    set(index: number, value: number): void {
      setter(index, value)
      notifier.notify()
    },

    setBatch(updates: [number, number][]): void {
      for (const [index, value] of updates) {
        setter(index, value)
      }
      notifier.notify()
    },

    getIndexSource(_index: number): Source<number> {
      // Not implemented - would need per-index reactive tracking
      throw new Error('getIndexSource not implemented for SlotBuffer')
    },

    notifyChanged(): void {
      notifier.notify()
    },

    notifyIndicesChanged(_indices: number[]): void {
      // Same as notifyChanged - we don't have per-index granularity
      notifier.notify()
    },

    clear(index: number): void {
      setter(index, defaultValue)
      notifier.notify()
    },

    dispose(): void {
      // Nothing to clean up - DataView is managed externally
    },
  }
}

// Re-export the type for convenience
export type { SharedSlotBuffer }
