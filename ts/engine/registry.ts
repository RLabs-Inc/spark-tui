/**
 * TUI Framework - Component Registry
 *
 * Manages index allocation for the parallel arrays pattern.
 * Each component gets a unique index, which is used across all arrays.
 *
 * Features:
 * - ID ↔ Index bidirectional mapping
 * - Free index pool for O(1) reuse
 * - ReactiveSet for allocatedIndices (deriveds react to add/remove)
 */

import { ReactiveSet } from '@rlabs-inc/signals'
import { runDestroyCallbacks, resetLifecycle } from './lifecycle'
import { isInitialized, getArrays, getViews } from '../bridge'
import {
  setNodeCount,
  I32_PARENT_INDEX,
  COMPONENT_NONE,
  U8_COMPONENT_TYPE,
} from '../bridge/shared-buffer'

// =============================================================================
// Registry State
// =============================================================================

/** Map component ID to array index */
const idToIndex = new Map<string, number>()

/** Map array index to component ID */
const indexToId = new Map<number, string>()

/**
 * Set of currently allocated indices (for iteration).
 *
 * Using ReactiveSet so deriveds that iterate over this set
 * automatically react when components are added or removed.
 */
const allocatedIndices = new ReactiveSet<number>()

/** Pool of freed indices for reuse */
const freeIndices: number[] = []

/** Next index to allocate if pool is empty */
let nextIndex = 0

/** Counter for generating unique IDs */
let idCounter = 0

// =============================================================================
// Parent Context Stack
// =============================================================================

/** Stack of parent indices for nested component creation */
const parentStack: number[] = []

/** Get current parent index (-1 if at root) */
export function getCurrentParentIndex(): number {
  return parentStack.length > 0 ? (parentStack[parentStack.length - 1] ?? -1) : -1
}

/** Push a parent index onto the stack */
export function pushParentContext(index: number): void {
  parentStack.push(index)
}

/** Pop a parent index from the stack */
export function popParentContext(): void {
  parentStack.pop()
}

// =============================================================================
// Index Allocation
// =============================================================================

/**
 * Allocate an index for a new component.
 *
 * @param id - Optional component ID. If not provided, one is generated.
 * @returns The allocated index.
 */
export function allocateIndex(id?: string): number {
  // Generate ID if not provided
  const componentId = id ?? `c${idCounter++}`

  // Check if already allocated
  const existing = idToIndex.get(componentId)
  if (existing !== undefined) {
    return existing
  }

  // Reuse free index or allocate new
  const index = freeIndices.length > 0
    ? freeIndices.pop()!
    : nextIndex++

  // Register mappings
  idToIndex.set(componentId, index)
  indexToId.set(index, componentId)
  allocatedIndices.add(index)

  // Update node count in shared buffer header
  if (isInitialized()) {
    const views = getViews()
    const count = allocatedIndices.size
    setNodeCount(views, count > nextIndex ? count : nextIndex)
  }

  return index
}

/**
 * Release an index back to the pool.
 * Also recursively releases all children!
 *
 * @param index - The index to release.
 */
export function releaseIndex(index: number): void {
  const id = indexToId.get(index)
  if (id === undefined) return

  // FIRST: Find and release all children (recursive!)
  // Read parent index from SharedArrayBuffer (raw i32 view — non-reactive, just a lookup)
  const views = isInitialized() ? getViews() : null
  const children: number[] = []
  for (const childIndex of allocatedIndices) {
    const parentIdx = views
      ? views.i32[I32_PARENT_INDEX][childIndex]
      : -1
    if (parentIdx === index) {
      children.push(childIndex)
    }
  }
  // Release children recursively
  for (const childIndex of children) {
    releaseIndex(childIndex)
  }

  // Run destroy callbacks before cleanup
  runDestroyCallbacks(index)

  // Clean up mappings
  idToIndex.delete(id)
  indexToId.delete(index)
  allocatedIndices.delete(index)

  // Mark node as unused in SharedBuffer (Rust skips NONE component type)
  if (views) {
    views.u8[U8_COMPONENT_TYPE][index] = COMPONENT_NONE
    views.i32[I32_PARENT_INDEX][index] = -1
  }

  // Return to pool for reuse
  freeIndices.push(index)

  // AUTO-CLEANUP: When all components destroyed, reset counters
  if (allocatedIndices.size === 0) {
    freeIndices.length = 0
    nextIndex = 0
    if (views) setNodeCount(views, 0)
  }
}

// =============================================================================
// Lookups
// =============================================================================

/** Get index for a component ID */
export function getIndex(id: string): number | undefined {
  return idToIndex.get(id)
}

/** Get ID for an index */
export function getId(index: number): string | undefined {
  return indexToId.get(index)
}

/** Get all currently allocated indices */
export function getAllocatedIndices(): Set<number> {
  return allocatedIndices
}

/** Check if an index is currently allocated */
export function isAllocated(index: number): boolean {
  return allocatedIndices.has(index)
}

/** Get the current capacity (highest index that would be allocated next) */
export function getCapacity(): number {
  return nextIndex
}

/** Get the count of currently allocated components */
export function getAllocatedCount(): number {
  return allocatedIndices.size
}

// =============================================================================
// Reset (for testing)
// =============================================================================

/** Reset all registry state (for testing) */
export function resetRegistry(): void {
  idToIndex.clear()
  indexToId.clear()
  allocatedIndices.clear()
  freeIndices.length = 0
  nextIndex = 0
  idCounter = 0
  parentStack.length = 0
  resetLifecycle()
  if (isInitialized()) {
    setNodeCount(getViews(), 0)
  }
}
