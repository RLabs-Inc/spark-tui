/**
 * TUI Framework - Taffy Layout Arrays
 *
 * TypedArray-backed layout properties for zero-copy FFI integration with Taffy.
 * All arrays share a single ReactiveSet for dirty tracking.
 *
 * Usage in primitives:
 *   taffy.width.setSource(index, props.width)
 *   taffy.flexDirection.setSource(index, enumSource(props.flexDirection, flexDirToNum))
 *
 * Usage in layoutDerived:
 *   const dirty = taffy.syncAndGetDirty()
 *   taffyNative.syncAndCompute(dirty, taffy.width.buffer, taffy.height.buffer, ...)
 *
 * Value conventions (matching taffy-native):
 *   - NaN = auto (for dimensions, basis)
 *   - 0-1 (fractional) = percentage (0.5 = 50%)
 *   - >1 = absolute pixels/cells
 *   - Enums use Uint8Array with numeric codes
 */

import { typedSlotArray, typedSlotArrayGroup, ReactiveSet } from '@rlabs-inc/signals'
import type { TypedSlotArray, TypedSlotArrayGroup } from '@rlabs-inc/signals'

// =============================================================================
// SHARED DIRTY TRACKING
// =============================================================================

/** Shared dirty set - tracks which indices have changed since last sync */
export const dirtySet = new ReactiveSet<number>()

// =============================================================================
// INITIAL CAPACITY
// =============================================================================

const INITIAL_CAPACITY = 1024

// =============================================================================
// LAYOUT ARRAYS GROUP
// =============================================================================

/**
 * All layout properties in a single group with shared dirty tracking.
 * Buffers can be passed directly to taffy-native FFI.
 */
export const layoutGroup = typedSlotArrayGroup({
  // ---------------------------------------------------------------------------
  // DIMENSIONS (Float32Array) - NaN = auto
  // ---------------------------------------------------------------------------
  width: { type: Float32Array, defaultValue: NaN },
  height: { type: Float32Array, defaultValue: NaN },
  minWidth: { type: Float32Array, defaultValue: NaN },
  maxWidth: { type: Float32Array, defaultValue: NaN },
  minHeight: { type: Float32Array, defaultValue: NaN },
  maxHeight: { type: Float32Array, defaultValue: NaN },

  // ---------------------------------------------------------------------------
  // FLEX CONTAINER (Uint8Array) - Enum codes
  // flexDirection: 0=row, 1=column, 2=row-reverse, 3=column-reverse
  // flexWrap: 0=nowrap, 1=wrap, 2=wrap-reverse
  // justifyContent: 0=flex-start, 1=center, 2=flex-end, 3=space-between, 4=space-around, 5=space-evenly
  // alignItems: 0=stretch, 1=flex-start, 2=center, 3=flex-end, 4=baseline
  // ---------------------------------------------------------------------------
  flexDirection: { type: Uint8Array, defaultValue: 1 },  // Default: column (TUI convention)
  flexWrap: { type: Uint8Array, defaultValue: 0 },
  justifyContent: { type: Uint8Array, defaultValue: 0 },
  alignItems: { type: Uint8Array, defaultValue: 0 },

  // ---------------------------------------------------------------------------
  // FLEX ITEM (mixed)
  // alignSelf: 0=auto, 1=stretch, 2=flex-start, 3=center, 4=flex-end, 5=baseline
  // ---------------------------------------------------------------------------
  alignSelf: { type: Uint8Array, defaultValue: 0 },
  grow: { type: Float32Array, defaultValue: 0 },
  shrink: { type: Float32Array, defaultValue: 1 },
  basis: { type: Float32Array, defaultValue: NaN }, // NaN = auto

  // ---------------------------------------------------------------------------
  // SPACING (Float32Array) - All in cells
  // ---------------------------------------------------------------------------
  gap: { type: Float32Array, defaultValue: 0 },
  paddingTop: { type: Float32Array, defaultValue: 0 },
  paddingRight: { type: Float32Array, defaultValue: 0 },
  paddingBottom: { type: Float32Array, defaultValue: 0 },
  paddingLeft: { type: Float32Array, defaultValue: 0 },
  marginTop: { type: Float32Array, defaultValue: 0 },
  marginRight: { type: Float32Array, defaultValue: 0 },
  marginBottom: { type: Float32Array, defaultValue: 0 },
  marginLeft: { type: Float32Array, defaultValue: 0 },

  // ---------------------------------------------------------------------------
  // VISIBILITY (Uint8Array) - 0=hidden, 1=visible
  // ---------------------------------------------------------------------------
  visible: { type: Uint8Array, defaultValue: 1 },

  // ---------------------------------------------------------------------------
  // BORDER WIDTHS (Uint8Array) - for layout spacing (0 or 1)
  // Note: Border STYLE goes to visual arrays, border WIDTH goes here for layout
  // ---------------------------------------------------------------------------
  borderTop: { type: Uint8Array, defaultValue: 0 },
  borderRight: { type: Uint8Array, defaultValue: 0 },
  borderBottom: { type: Uint8Array, defaultValue: 0 },
  borderLeft: { type: Uint8Array, defaultValue: 0 },
}, INITIAL_CAPACITY, dirtySet)

// =============================================================================
// HIERARCHY (Int32Array) - Separate for tree structure
// =============================================================================

/** Parent index for each component. -1 = root (no parent) */
export const parentIndex = typedSlotArray(Int32Array, INITIAL_CAPACITY, dirtySet, -1)

// =============================================================================
// CONVENIENCE EXPORTS - Direct access to arrays
// =============================================================================

// Dimensions
export const width = layoutGroup.arrays.width
export const height = layoutGroup.arrays.height
export const minWidth = layoutGroup.arrays.minWidth
export const maxWidth = layoutGroup.arrays.maxWidth
export const minHeight = layoutGroup.arrays.minHeight
export const maxHeight = layoutGroup.arrays.maxHeight

// Flex container
export const flexDirection = layoutGroup.arrays.flexDirection
export const flexWrap = layoutGroup.arrays.flexWrap
export const justifyContent = layoutGroup.arrays.justifyContent
export const alignItems = layoutGroup.arrays.alignItems

// Flex item
export const alignSelf = layoutGroup.arrays.alignSelf
export const grow = layoutGroup.arrays.grow
export const shrink = layoutGroup.arrays.shrink
export const basis = layoutGroup.arrays.basis

// Spacing
export const gap = layoutGroup.arrays.gap
export const paddingTop = layoutGroup.arrays.paddingTop
export const paddingRight = layoutGroup.arrays.paddingRight
export const paddingBottom = layoutGroup.arrays.paddingBottom
export const paddingLeft = layoutGroup.arrays.paddingLeft
export const marginTop = layoutGroup.arrays.marginTop
export const marginRight = layoutGroup.arrays.marginRight
export const marginBottom = layoutGroup.arrays.marginBottom
export const marginLeft = layoutGroup.arrays.marginLeft

// Visibility
export const visible = layoutGroup.arrays.visible

// Border widths
export const borderTop = layoutGroup.arrays.borderTop
export const borderRight = layoutGroup.arrays.borderRight
export const borderBottom = layoutGroup.arrays.borderBottom
export const borderLeft = layoutGroup.arrays.borderLeft

// =============================================================================
// SYNC API
// =============================================================================

/** Sync all arrays and return dirty indices (clears dirty set) */
export function syncAndGetDirty(): number[] {
  return layoutGroup.syncAndGetDirty()
}

/** Sync all arrays without clearing dirty set */
export function sync(): void {
  layoutGroup.sync()
}

/** Clear dirty tracking */
export function clearDirty(): void {
  dirtySet.clear()
}

/** Mark an index as dirty */
export function markDirty(index: number): void {
  dirtySet.add(index)
}

// =============================================================================
// CAPACITY MANAGEMENT
// =============================================================================

/** Ensure all arrays have capacity for the given index */
export function ensureCapacity(index: number): void {
  layoutGroup.ensureCapacity(index + 1)
  parentIndex.ensureCapacity(index + 1)
}

/** Reset all arrays (clear bindings and dirty state) */
export function reset(): void {
  layoutGroup.reset()
  parentIndex.reset()
}

// =============================================================================
// DIMENSION CONVERSION HELPERS
// =============================================================================

/**
 * Convert TUI Dimension to Taffy float.
 * - 0 or undefined → NaN (auto)
 * - "50%" → 0.5 (percentage as fraction)
 * - number → number (absolute)
 */
export function toDimension(dim: number | string | undefined | null): number {
  if (dim === undefined || dim === null || dim === 0) return NaN
  if (typeof dim === 'string') {
    if (dim.endsWith('%')) {
      return parseFloat(dim) / 100
    }
    return parseFloat(dim) || NaN
  }
  return dim
}

/**
 * Convert Dimension prop to a getter that returns Taffy float.
 * Works with signals, getters, or static values.
 */
export function dimensionSource(
  prop: number | string | (() => number | string) | { value: number | string } | undefined
): number | (() => number) {
  if (prop === undefined) return NaN

  // Static number
  if (typeof prop === 'number') return toDimension(prop)

  // Static string
  if (typeof prop === 'string') return toDimension(prop)

  // Getter function
  if (typeof prop === 'function') {
    return () => toDimension((prop as () => number | string)())
  }

  // Signal-like object with .value
  if (typeof prop === 'object' && prop !== null && 'value' in prop) {
    return () => toDimension((prop as { value: number | string }).value)
  }

  return NaN
}

// =============================================================================
// ENUM CONVERSION CONSTANTS
// =============================================================================

// Flex Direction (matches taffy-native)
export const FLEX_DIRECTION_ROW = 0
export const FLEX_DIRECTION_COLUMN = 1
export const FLEX_DIRECTION_ROW_REVERSE = 2
export const FLEX_DIRECTION_COLUMN_REVERSE = 3

// Flex Wrap
export const FLEX_WRAP_NOWRAP = 0
export const FLEX_WRAP_WRAP = 1
export const FLEX_WRAP_WRAP_REVERSE = 2

// Justify Content
export const JUSTIFY_FLEX_START = 0
export const JUSTIFY_CENTER = 1
export const JUSTIFY_FLEX_END = 2
export const JUSTIFY_SPACE_BETWEEN = 3
export const JUSTIFY_SPACE_AROUND = 4
export const JUSTIFY_SPACE_EVENLY = 5

// Align Items
export const ALIGN_STRETCH = 0
export const ALIGN_FLEX_START = 1
export const ALIGN_CENTER = 2
export const ALIGN_FLEX_END = 3
export const ALIGN_BASELINE = 4

// Align Self (0 = auto/inherit)
export const ALIGN_SELF_AUTO = 0
export const ALIGN_SELF_STRETCH = 1
export const ALIGN_SELF_FLEX_START = 2
export const ALIGN_SELF_CENTER = 3
export const ALIGN_SELF_FLEX_END = 4
export const ALIGN_SELF_BASELINE = 5
