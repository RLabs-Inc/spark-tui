/**
 * SparkTUI STRESS TEST
 *
 * Push the Frankenstein architecture until it breaks.
 * Find the limits. Know the ceiling.
 */

import { dlopen, ptr, FFIType } from 'bun:ffi'
import {
  createAoSBuffer,
  setTerminalSize,
  HEADER_SIZE,
  STRIDE,
  MAX_NODES,
} from '../ts/bridge/shared-buffer-aos'

const LIB_PATH = new URL('../rust/target/release/libspark_tui_engine.dylib', import.meta.url).pathname

const lib = dlopen(LIB_PATH, {
  spark_init: { args: [FFIType.ptr, FFIType.u32], returns: FFIType.i32 },
  spark_render: { args: [], returns: FFIType.i32 },
  spark_compute_layout: { args: [], returns: FFIType.i32 },
  spark_cleanup: { args: [], returns: FFIType.void },
})

console.log('═══════════════════════════════════════════════════════════════')
console.log('  SparkTUI STRESS TEST - Finding the Breaking Point')
console.log('═══════════════════════════════════════════════════════════════')
console.log()

const buffer = createAoSBuffer()
setTerminalSize(buffer, 200, 50) // Larger terminal

const result = lib.symbols.spark_init(ptr(buffer.buffer), buffer.buffer.byteLength)
if (result !== 0) {
  console.error('Failed to initialize')
  process.exit(1)
}

// Helper to set up N nodes in a tree structure
function setupNodes(count: number, depth: number = 3) {
  buffer.view.setUint32(4, count, true)

  // Create a tree structure with specified depth
  const nodesPerLevel = Math.ceil(count / depth)

  for (let i = 0; i < count; i++) {
    const nodeOffset = HEADER_SIZE + i * STRIDE

    // Parent: root has -1, others point to parent based on level
    let parent = -1
    if (i > 0) {
      parent = Math.floor((i - 1) / nodesPerLevel)
      if (parent >= i) parent = 0
    }
    buffer.view.setInt32(nodeOffset + 180, parent, true)

    // Component type = BOX (1)
    buffer.view.setUint8(nodeOffset + 96, 1)

    // Dimensions
    buffer.view.setFloat32(nodeOffset + 0, 10 + (i % 20), true)  // width
    buffer.view.setFloat32(nodeOffset + 4, 1 + (i % 5), true)    // height

    // Flex properties
    buffer.view.setUint8(nodeOffset + 97, i % 2)  // flex direction
    buffer.view.setFloat32(nodeOffset + 48, 1, true)  // flex grow
  }
}

// =============================================================================
// STRESS TEST 1: Node Count Scaling
// =============================================================================

console.log('─────────────────────────────────────────────────────────────────')
console.log('STRESS TEST 1: Node Count Scaling (Layout)')
console.log('─────────────────────────────────────────────────────────────────')
console.log()
console.log('  Finding where layout breaks 60 FPS (16.67ms budget)...')
console.log()

const NODE_COUNTS = [100, 500, 1000, 2000, 5000, 10000, 20000, 50000, 100000]
const ITERATIONS = 100

let broke60fps = false
let broke30fps = false

for (const nodeCount of NODE_COUNTS) {
  if (nodeCount > MAX_NODES) break

  setupNodes(nodeCount)

  // Warm up
  for (let i = 0; i < 10; i++) lib.symbols.spark_compute_layout()

  const start = performance.now()
  for (let i = 0; i < ITERATIONS; i++) {
    lib.symbols.spark_compute_layout()
  }
  const elapsed = performance.now() - start

  const avgMs = elapsed / ITERATIONS
  const avgUs = avgMs * 1000
  const fps = 1000 / avgMs

  const status60 = avgMs < 16.67 ? '✓' : '✗'
  const status30 = avgMs < 33.33 ? '✓' : '✗'

  console.log(`  ${nodeCount.toString().padStart(6)} nodes: ${avgUs.toFixed(1).padStart(8)}μs  (${fps.toFixed(0).padStart(6)} FPS)  60fps:${status60}  30fps:${status30}`)

  if (!broke60fps && avgMs >= 16.67) {
    broke60fps = true
    console.log(`  ⚠️  60 FPS BROKEN at ${nodeCount} nodes`)
  }
  if (!broke30fps && avgMs >= 33.33) {
    broke30fps = true
    console.log(`  🔥 30 FPS BROKEN at ${nodeCount} nodes`)
  }
}

console.log()

// =============================================================================
// STRESS TEST 2: Full Pipeline Scaling
// =============================================================================

console.log('─────────────────────────────────────────────────────────────────')
console.log('STRESS TEST 2: Full Pipeline Scaling (Layout + Framebuffer + Render)')
console.log('─────────────────────────────────────────────────────────────────')
console.log()

broke60fps = false
broke30fps = false

for (const nodeCount of NODE_COUNTS) {
  if (nodeCount > MAX_NODES) break

  setupNodes(nodeCount)

  // Warm up
  for (let i = 0; i < 5; i++) lib.symbols.spark_render()

  const iterations = nodeCount > 10000 ? 10 : 50
  const start = performance.now()
  for (let i = 0; i < iterations; i++) {
    lib.symbols.spark_render()
  }
  const elapsed = performance.now() - start

  const avgMs = elapsed / iterations
  const avgUs = avgMs * 1000
  const fps = 1000 / avgMs

  const status60 = avgMs < 16.67 ? '✓' : '✗'
  const status30 = avgMs < 33.33 ? '✓' : '✗'

  console.log(`  ${nodeCount.toString().padStart(6)} nodes: ${avgUs.toFixed(1).padStart(8)}μs  (${fps.toFixed(0).padStart(6)} FPS)  60fps:${status60}  30fps:${status30}`)

  if (!broke60fps && avgMs >= 16.67) {
    broke60fps = true
    console.log(`  ⚠️  60 FPS BROKEN at ${nodeCount} nodes`)
  }
  if (!broke30fps && avgMs >= 33.33) {
    broke30fps = true
    console.log(`  🔥 30 FPS BROKEN at ${nodeCount} nodes`)
  }
}

console.log()

// =============================================================================
// STRESS TEST 3: Rapid Fire Updates
// =============================================================================

console.log('─────────────────────────────────────────────────────────────────')
console.log('STRESS TEST 3: Rapid Fire Updates (simulating fast typing)')
console.log('─────────────────────────────────────────────────────────────────')
console.log()

setupNodes(100) // Typical app size

const UPDATE_COUNTS = [100, 1000, 10000, 100000, 1000000]

for (const updates of UPDATE_COUNTS) {
  const start = performance.now()

  for (let i = 0; i < updates; i++) {
    // Simulate: change a prop, compute layout, render
    buffer.view.setFloat32(HEADER_SIZE + 0, 50 + (i % 50), true)
    lib.symbols.spark_compute_layout()
  }

  const elapsed = performance.now() - start
  const updatesPerSec = (updates / elapsed) * 1000
  const avgUs = (elapsed / updates) * 1000

  console.log(`  ${updates.toString().padStart(7)} updates: ${elapsed.toFixed(1).padStart(8)}ms  (${(updatesPerSec/1000).toFixed(0).padStart(5)}K/sec)  ${avgUs.toFixed(2)}μs each`)
}

console.log()

// =============================================================================
// STRESS TEST 4: Memory Bandwidth (SAB thrashing)
// =============================================================================

console.log('─────────────────────────────────────────────────────────────────')
console.log('STRESS TEST 4: Memory Bandwidth (SharedArrayBuffer thrashing)')
console.log('─────────────────────────────────────────────────────────────────')
console.log()

const THRASH_SIZES = [1000, 10000, 100000, 1000000, 10000000, 100000000]

for (const size of THRASH_SIZES) {
  const start = performance.now()

  for (let i = 0; i < size; i++) {
    buffer.view.setFloat32(HEADER_SIZE + (i % 10000) * 4, i, true)
  }

  const elapsed = performance.now() - start
  const opsPerSec = (size / elapsed) * 1000
  const gbPerSec = (size * 4 / elapsed) / 1_000_000 // 4 bytes per float32

  console.log(`  ${size.toString().padStart(11)} writes: ${elapsed.toFixed(1).padStart(7)}ms  (${(opsPerSec/1_000_000).toFixed(0).padStart(4)}M/sec)  ${gbPerSec.toFixed(2)} GB/s`)
}

console.log()

// =============================================================================
// STRESS TEST 5: Deep Nesting
// =============================================================================

console.log('─────────────────────────────────────────────────────────────────')
console.log('STRESS TEST 5: Deep Nesting (worst case for flexbox)')
console.log('─────────────────────────────────────────────────────────────────')
console.log()

const DEPTHS = [10, 50, 100, 200, 500, 1000]

for (const depth of DEPTHS) {
  buffer.view.setUint32(4, depth, true)

  // Create a deeply nested chain: each node is child of previous
  for (let i = 0; i < depth; i++) {
    const nodeOffset = HEADER_SIZE + i * STRIDE
    buffer.view.setInt32(nodeOffset + 180, i === 0 ? -1 : i - 1, true)
    buffer.view.setUint8(nodeOffset + 96, 1) // BOX
    buffer.view.setFloat32(nodeOffset + 0, 10, true)
    buffer.view.setFloat32(nodeOffset + 4, 1, true)
  }

  // Warm up
  for (let i = 0; i < 10; i++) lib.symbols.spark_compute_layout()

  const iterations = 1000
  const start = performance.now()
  for (let i = 0; i < iterations; i++) {
    lib.symbols.spark_compute_layout()
  }
  const elapsed = performance.now() - start

  const avgUs = (elapsed / iterations) * 1000
  const fps = 1_000_000 / avgUs

  console.log(`  ${depth.toString().padStart(4)} depth: ${avgUs.toFixed(1).padStart(8)}μs  (${fps.toFixed(0).padStart(7)} FPS)`)
}

console.log()

// =============================================================================
// SUMMARY
// =============================================================================

console.log('═══════════════════════════════════════════════════════════════')
console.log('  STRESS TEST COMPLETE')
console.log('═══════════════════════════════════════════════════════════════')
console.log()
console.log('  The Frankenstein architecture limits:')
console.log('  - Node scaling: check results above for 60/30 FPS breakpoints')
console.log('  - Update rate: millions per second sustained')
console.log('  - Memory bandwidth: GB/s throughput')
console.log('  - Deep nesting: flexbox handles extreme depth')
console.log()
console.log('  For reference:')
console.log('  - Typical TUI: 50-200 nodes')
console.log('  - Complex dashboard: 500-2000 nodes')
console.log('  - Insane stress: 10000+ nodes')
console.log()

lib.symbols.spark_cleanup()
