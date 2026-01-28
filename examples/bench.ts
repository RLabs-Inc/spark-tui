/**
 * SparkTUI Performance Benchmark
 *
 * Measures: buffer creation, node writing, FFI layout computation, output reading.
 */

import { ptr } from 'bun:ffi'
import { loadEngine } from '../ts/bridge/ffi'
import {
  createSharedBuffer,
  setNodeMeta,
  setNodeFloat,
  setNodeParent,
  setTerminalSize,
  setNodeCount,
  getNodeOutput,
  COMPONENT_BOX,
  META_COMPONENT_TYPE,
  META_VISIBLE,
  META_FLEX_DIRECTION,
  FLOAT_WIDTH,
  FLOAT_HEIGHT,
  FLOAT_GROW,
  FLOAT_GAP,
  FLOAT_PADDING_TOP,
  FLOAT_PADDING_RIGHT,
  FLOAT_PADDING_BOTTOM,
  FLOAT_PADDING_LEFT,
  MAX_NODES,
} from '../ts/bridge/shared-buffer'

// =============================================================================
// SETUP
// =============================================================================

const views = createSharedBuffer()
const engine = loadEngine()
engine.init(ptr(views.buffer), views.buffer.byteLength)
setTerminalSize(views, 200, 50)

// =============================================================================
// HELPERS
// =============================================================================

function bench(name: string, fn: () => void, iterations: number = 10000): void {
  // Warmup
  for (let i = 0; i < 100; i++) fn()

  const start = performance.now()
  for (let i = 0; i < iterations; i++) fn()
  const elapsed = performance.now() - start

  const perOp = (elapsed / iterations) * 1000 // microseconds
  console.log(`  ${name}: ${perOp.toFixed(2)}µs/op (${iterations} iterations, ${elapsed.toFixed(1)}ms total)`)
}

function buildTree(nodeCount: number): void {
  // Root
  setNodeMeta(views, 0, META_COMPONENT_TYPE, COMPONENT_BOX)
  setNodeMeta(views, 0, META_VISIBLE, 1)
  setNodeMeta(views, 0, META_FLEX_DIRECTION, 0) // column
  setNodeFloat(views, 0, FLOAT_WIDTH, 200)
  setNodeFloat(views, 0, FLOAT_HEIGHT, 50)
  setNodeFloat(views, 0, FLOAT_GAP, 1)
  setNodeFloat(views, 0, FLOAT_PADDING_TOP, 1)
  setNodeFloat(views, 0, FLOAT_PADDING_RIGHT, 1)
  setNodeFloat(views, 0, FLOAT_PADDING_BOTTOM, 1)
  setNodeFloat(views, 0, FLOAT_PADDING_LEFT, 1)

  // Children
  for (let i = 1; i < nodeCount; i++) {
    setNodeMeta(views, i, META_COMPONENT_TYPE, COMPONENT_BOX)
    setNodeMeta(views, i, META_VISIBLE, 1)
    setNodeMeta(views, i, META_FLEX_DIRECTION, 1) // row
    setNodeFloat(views, i, FLOAT_GROW, 1)
    setNodeFloat(views, i, FLOAT_HEIGHT, 3)
    setNodeParent(views, i, 0)
  }

  setNodeCount(views, nodeCount)
}

// =============================================================================
// BENCHMARKS
// =============================================================================

console.log('=== SparkTUI Performance Benchmark ===\n')

// --- Buffer creation ---
console.log('Buffer creation:')
bench('createSharedBuffer()', () => { createSharedBuffer() }, 1000)

// --- Small tree (10 nodes) ---
console.log('\nSmall tree (10 nodes):')
buildTree(10)
bench('write 10 nodes', () => { buildTree(10) }, 10000)
bench('computeLayout (10)', () => { engine.computeLayout() }, 10000)
bench('read 10 outputs', () => { for (let i = 0; i < 10; i++) getNodeOutput(views, i) }, 10000)

// --- Medium tree (100 nodes) ---
console.log('\nMedium tree (100 nodes):')
buildTree(100)
bench('write 100 nodes', () => { buildTree(100) }, 5000)
bench('computeLayout (100)', () => { engine.computeLayout() }, 5000)
bench('read 100 outputs', () => { for (let i = 0; i < 100; i++) getNodeOutput(views, i) }, 5000)

// --- Large tree (500 nodes) ---
console.log('\nLarge tree (500 nodes):')
buildTree(500)
bench('write 500 nodes', () => { buildTree(500) }, 1000)
bench('computeLayout (500)', () => { engine.computeLayout() }, 1000)
bench('read 500 outputs', () => { for (let i = 0; i < 500; i++) getNodeOutput(views, i) }, 1000)

// --- Large tree (1000 nodes) ---
console.log('\nLarge tree (1000 nodes):')
buildTree(1000)
bench('write 1000 nodes', () => { buildTree(1000) }, 500)
bench('computeLayout (1000)', () => { engine.computeLayout() }, 500)
bench('read 1000 outputs', () => { for (let i = 0; i < 1000; i++) getNodeOutput(views, i) }, 500)

// --- Just the FFI call overhead ---
console.log('\nFFI overhead (1 node tree):')
buildTree(1)
bench('computeLayout (1 node)', () => { engine.computeLayout() }, 50000)

// --- 16ms budget analysis ---
console.log('\n--- 60fps Budget Analysis ---')
buildTree(100)
const runs = 1000
const start = performance.now()
for (let i = 0; i < runs; i++) engine.computeLayout()
const avg = (performance.now() - start) / runs
console.log(`  100-node layout: ${(avg * 1000).toFixed(0)}µs average`)
console.log(`  Fits in 16ms frame: ${avg < 16 ? 'YES' : 'NO'} (${(avg / 16 * 100).toFixed(1)}% of budget)`)
console.log(`  Layouts per frame: ${Math.floor(16 / avg)}`)

engine.close()
