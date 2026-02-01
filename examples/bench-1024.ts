/**
 * 1024-byte SharedBuffer Benchmark (v3.0)
 *
 * Compare performance of 1024-byte stride vs previous 512-byte.
 * Run: bun run examples/bench-1024.ts
 */

import { ptr } from 'bun:ffi'
import { dlopen, FFIType } from 'bun:ffi'

import {
  createSharedBuffer,
  setTerminalSize,
  setNodeCount,
  setParentIndex,
  setComponentType,
  setVisible,
  setWidth,
  setHeight,
  setFlexDirection,
  setFlexGrow,
  setBgColor,
  setFgColor,
  setBorderWidth,
  setBorderColor,
  setPadding,
  setText,
  markDirty,
  packColor,
  getF32,
  getU8,
  COMPONENT_BOX,
  COMPONENT_TEXT,
  DIRTY_LAYOUT,
  DIRTY_VISUAL,
  DIRTY_HIERARCHY,
  NODE_STRIDE,
  HEADER_SIZE,
  N_WIDTH,
  N_HEIGHT,
  N_FLEX_DIRECTION,
  N_VISIBLE,
  FlexDirection,
  type SharedBuffer,
} from '../ts/bridge/shared-buffer'

console.log('╔══════════════════════════════════════════════════════════════════╗')
console.log('║           1024-byte SharedBuffer Benchmark (v3.0)               ║')
console.log('╚══════════════════════════════════════════════════════════════════╝\n')

console.log(`NODE_STRIDE: ${NODE_STRIDE} bytes`)
console.log(`HEADER_SIZE: ${HEADER_SIZE} bytes\n`)

// =============================================================================
// SETUP
// =============================================================================

const buf = createSharedBuffer({ maxNodes: 10_000 })
setTerminalSize(buf, 120, 40)

// Load the Rust library
const libPath = new URL('../rust/target/release/libspark_tui_engine.dylib', import.meta.url).pathname

const engine = dlopen(libPath, {
  spark_init: { args: [FFIType.ptr, FFIType.u32], returns: FFIType.i32 },
  spark_compute_layout: { args: [], returns: FFIType.u32 },
}).symbols

// Initialize engine
const initResult = engine.spark_init(ptr(buf.raw), buf.raw.byteLength)
console.log(`Rust engine initialized (result: ${initResult})\n`)

// =============================================================================
// HELPERS
// =============================================================================

function buildTree(nodeCount: number) {
  setNodeCount(buf, nodeCount)

  // Root
  setComponentType(buf, 0, COMPONENT_BOX)
  setVisible(buf, 0, true)
  setFlexDirection(buf, 0, FlexDirection.Column)
  setWidth(buf, 0, 120)
  setHeight(buf, 0, 40)
  setParentIndex(buf, 0, -1)
  setBgColor(buf, 0, packColor(20, 20, 40, 255))
  markDirty(buf, 0, DIRTY_LAYOUT | DIRTY_VISUAL | DIRTY_HIERARCHY)

  let parent = 0
  const childrenPerGroup = 5

  for (let i = 1; i < nodeCount; i++) {
    const isContainer = i < nodeCount / 2 && (i % childrenPerGroup === 0)

    setComponentType(buf, i, isContainer ? COMPONENT_BOX : COMPONENT_TEXT)
    setVisible(buf, i, true)
    setParentIndex(buf, i, parent)

    if (isContainer) {
      setFlexDirection(buf, i, (i % 2 === 0) ? FlexDirection.Row : FlexDirection.Column)
      setFlexGrow(buf, i, 1)
      setBgColor(buf, i, packColor(30 + (i % 20), 30, 60, 255))
      setBorderWidth(buf, i, 1, 1, 1, 1)
      setBorderColor(buf, i, packColor(80, 80, 120, 255))
      setPadding(buf, i, 1, 1, 1, 1)
      parent = i
    } else {
      setFgColor(buf, i, packColor(200, 200, 220, 255))
      setText(buf, i, `Node ${i}`)
    }

    markDirty(buf, i, DIRTY_LAYOUT | DIRTY_VISUAL | DIRTY_HIERARCHY)

    if (i % (childrenPerGroup * 3) === 0 && parent > 0) {
      parent = Math.max(0, parent - childrenPerGroup)
    }
  }
}

function bench(fn: () => void, iterations: number): {
  avg: number, min: number, max: number, p50: number, p99: number
} {
  // Warmup
  for (let i = 0; i < 3; i++) fn()

  const times: number[] = []
  for (let i = 0; i < iterations; i++) {
    const t0 = performance.now()
    fn()
    const t1 = performance.now()
    times.push((t1 - t0) * 1000) // μs
  }

  times.sort((a, b) => a - b)
  return {
    avg: times.reduce((a, b) => a + b, 0) / times.length,
    min: times[0],
    max: times[times.length - 1],
    p50: times[Math.floor(times.length * 0.5)],
    p99: times[Math.floor(times.length * 0.99)],
  }
}

function fmt(μs: number): string {
  if (μs < 1) return `${(μs * 1000).toFixed(0)}ns`
  if (μs < 1000) return `${μs.toFixed(0)}μs`
  return `${(μs / 1000).toFixed(2)}ms`
}

function printBench(label: string, r: ReturnType<typeof bench>) {
  const fps = 1_000_000 / r.avg
  const x60 = fps / 60
  console.log(`  ${label.padEnd(45)} avg: ${fmt(r.avg).padStart(8)}  p99: ${fmt(r.p99).padStart(8)}  (${x60.toFixed(1)}x 60fps)`)
}

// =============================================================================
// BENCHMARK 1: Tree Build Speed (TS side)
// =============================================================================

console.log('┌─────────────────────────────────────────────────────────────────────┐')
console.log('│ 1. TREE BUILD SPEED — TS writing to 1024-byte buffer               │')
console.log('└─────────────────────────────────────────────────────────────────────┘\n')

for (const count of [100, 500, 1000, 2000, 5000]) {
  const r = bench(() => {
    buildTree(count)
  }, 100)
  printBench(`${count} nodes full tree build`, r)
}

// =============================================================================
// BENCHMARK 2: Individual Field Writes
// =============================================================================

console.log('\n┌─────────────────────────────────────────────────────────────────────┐')
console.log('│ 2. FIELD WRITE SPEED — Individual property updates                 │')
console.log('└─────────────────────────────────────────────────────────────────────┘\n')

buildTree(1000)

{
  const iterations = 100_000
  const nodeCount = 100

  // Using setters
  const t0 = performance.now()
  for (let i = 0; i < iterations; i++) {
    setWidth(buf, i % nodeCount, 100 + (i % 50))
  }
  const t1 = performance.now()
  const writes = iterations
  const rate = writes / ((t1 - t0) / 1000)
  console.log(`  setWidth():                        ${(rate / 1_000_000).toFixed(1)}M writes/sec`)

  // Direct DataView access for comparison
  const t2 = performance.now()
  for (let i = 0; i < iterations; i++) {
    const nodeIndex = i % nodeCount
    buf.view.setFloat32(HEADER_SIZE + nodeIndex * NODE_STRIDE + N_WIDTH, 100 + (i % 50), true)
  }
  const t3 = performance.now()
  const rate2 = writes / ((t3 - t2) / 1000)
  console.log(`  Direct DataView.setFloat32:        ${(rate2 / 1_000_000).toFixed(1)}M writes/sec`)
}

// =============================================================================
// BENCHMARK 3: Memory Access Pattern — Cache Line Test
// =============================================================================

console.log('\n┌─────────────────────────────────────────────────────────────────────┐')
console.log('│ 3. CACHE LINE ACCESS — Read layout props (lines 1-4: 256 bytes)    │')
console.log('└─────────────────────────────────────────────────────────────────────┘\n')

buildTree(1000)

{
  const iterations = 100_000

  const t0 = performance.now()
  let sum = 0
  for (let iter = 0; iter < iterations; iter++) {
    const nodeIndex = iter % 1000
    const base = HEADER_SIZE + nodeIndex * NODE_STRIDE

    // Read all layout floats from cache lines 1-3 (first 192 bytes)
    for (let offset = 0; offset < 192; offset += 4) {
      sum += buf.view.getFloat32(base + offset, true)
    }
    // Read enums from line 2
    for (let offset = 64; offset < 96; offset++) {
      sum += buf.view.getUint8(base + offset)
    }
  }
  const t1 = performance.now()

  const fieldsPerNode = 48 + 32 // floats + u8s
  const totalReads = iterations * fieldsPerNode
  const timeMs = t1 - t0
  const readsPerSec = totalReads / (timeMs / 1000)
  const nodesPerSec = iterations / (timeMs / 1000)

  console.log(`  Read layout props (80 fields) × ${iterations} nodes:`)
  console.log(`    Time: ${timeMs.toFixed(1)}ms`)
  console.log(`    ${(readsPerSec / 1_000_000).toFixed(0)}M field reads/sec`)
  console.log(`    ${(nodesPerSec / 1_000_000).toFixed(1)}M full-node reads/sec`)
  console.log(`    (sum=${sum.toFixed(0)} to prevent optimization)`)
}

// =============================================================================
// BENCHMARK 4: RUST LAYOUT COMPUTATION
// =============================================================================

console.log('\n┌─────────────────────────────────────────────────────────────────────┐')
console.log('│ 4. RUST LAYOUT — Taffy flexbox computation                         │')
console.log('└─────────────────────────────────────────────────────────────────────┘\n')

for (const count of [100, 500, 1000, 2000, 5000, 10000]) {
  buildTree(count)

  const r = bench(() => {
    engine.spark_compute_layout()
  }, 100)

  printBench(`${count} nodes layout`, r)
}

// =============================================================================
// BENCHMARK 5: END-TO-END (Build + Layout)
// =============================================================================

console.log('\n┌─────────────────────────────────────────────────────────────────────┐')
console.log('│ 5. END-TO-END — Full tree build + layout                           │')
console.log('└─────────────────────────────────────────────────────────────────────┘\n')

for (const count of [100, 500, 1000, 2000]) {
  const r = bench(() => {
    buildTree(count)
    engine.spark_compute_layout()
  }, 100)

  printBench(`${count} nodes E2E`, r)
}

// =============================================================================
// BENCHMARK 6: Memory Footprint
// =============================================================================

console.log('\n┌─────────────────────────────────────────────────────────────────────┐')
console.log('│ 6. MEMORY FOOTPRINT                                                │')
console.log('└─────────────────────────────────────────────────────────────────────┘\n')

const totalSize = buf.raw.byteLength
const nodeSize = NODE_STRIDE
const maxNodes = buf.maxNodes

console.log(`  Total buffer size: ${(totalSize / 1024 / 1024).toFixed(1)} MB`)
console.log(`  Node stride: ${nodeSize} bytes`)
console.log(`  Max nodes: ${maxNodes.toLocaleString()}`)
console.log(`  Node data: ${(maxNodes * nodeSize / 1024 / 1024).toFixed(1)} MB`)
console.log(`  Text pool: ${(buf.textPoolSize / 1024 / 1024).toFixed(1)} MB`)
console.log(`  Per-node overhead: ${nodeSize} bytes (1024 = 16 cache lines)`)

// =============================================================================
// SUMMARY
// =============================================================================

console.log('\n╔══════════════════════════════════════════════════════════════════╗')
console.log('║                           SUMMARY                                ║')
console.log('╚══════════════════════════════════════════════════════════════════╝')

buildTree(1000)
const layoutResult = bench(() => {
  engine.spark_compute_layout()
}, 100)

const e2eResult = bench(() => {
  buildTree(1000)
  engine.spark_compute_layout()
}, 100)

console.log(`
  1024-byte stride (v3.0) with full Grid support:

  Layout (1000 nodes):   ${fmt(layoutResult.avg)} avg, ${fmt(layoutResult.p99)} p99
  E2E (1000 nodes):      ${fmt(e2eResult.avg)} avg, ${fmt(e2eResult.p99)} p99

  Previous benchmarks (512-byte, v2.0):
  - Layout ~32μs target
  - E2E ~4.6μs at 100 nodes

  Memory overhead: +512 bytes/node for Grid tracks
  Cache benefit: All Grid data in lines 5-10, layout data in lines 1-4
`)

process.exit(0)
