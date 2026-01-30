/**
 * AoS SharedBuffer Benchmark
 *
 * Compare SoA vs AoS memory layout performance.
 * Run: bun run examples/bench-aos.ts
 */

import { ptr } from 'bun:ffi'
import { dlopen, FFIType } from 'bun:ffi'

import {
  createAoSBuffer,
  setTerminalSize,
  setNodeCount,
  createNodeWriter,
  markDirty,
  packColor,
  COMPONENT_BOX,
  COMPONENT_TEXT,
  DIRTY_LAYOUT,
  DIRTY_VISUAL,
  DIRTY_HIERARCHY,
  STRIDE,
  HEADER_SIZE,
  type AoSBuffer,
} from '../ts/bridge/shared-buffer-aos'

console.log('╔══════════════════════════════════════════════════════════════════╗')
console.log('║           AoS SharedBuffer Benchmark                             ║')
console.log('╚══════════════════════════════════════════════════════════════════╝\n')

// =============================================================================
// SETUP
// =============================================================================

const buf = createAoSBuffer()
setTerminalSize(buf, 120, 40)

// Load the Rust library
const libPath = new URL('../rust/target/release/libspark_tui_engine.dylib', import.meta.url).pathname

const engine = dlopen(libPath, {
  spark_init_aos: { args: [FFIType.ptr, FFIType.u32], returns: FFIType.i32 },
  spark_compute_layout_aos: { args: [], returns: FFIType.u32 },
}).symbols

// Initialize engine
const initResult = engine.spark_init_aos(ptr(buf.buffer), buf.buffer.byteLength)
console.log(`Rust AoS engine initialized (result: ${initResult})\n`)

// =============================================================================
// HELPERS
// =============================================================================

function buildTree(nodeCount: number) {
  setNodeCount(buf, nodeCount)

  // Root
  const root = createNodeWriter(buf, 0)
  root.componentType = COMPONENT_BOX
  root.visible = 1
  root.flexDirection = 1 // column
  root.width = 120
  root.height = 40
  root.parentIndex = -1
  root.bgColor = packColor(20, 20, 40, 255)
  root.markDirty(DIRTY_LAYOUT | DIRTY_VISUAL | DIRTY_HIERARCHY)

  let parent = 0
  const childrenPerGroup = 5

  for (let i = 1; i < nodeCount; i++) {
    const isContainer = i < nodeCount / 2 && (i % childrenPerGroup === 0)
    const node = createNodeWriter(buf, i)

    node.componentType = isContainer ? COMPONENT_BOX : COMPONENT_TEXT
    node.visible = 1
    node.parentIndex = parent

    if (isContainer) {
      node.flexDirection = (i % 2 === 0) ? 0 : 1
      node.flexGrow = 1
      node.bgColor = packColor(30 + (i % 20), 30, 60, 255)
      node.borderTop = 1
      node.borderRight = 1
      node.borderBottom = 1
      node.borderLeft = 1
      node.borderColor = packColor(80, 80, 120, 255)
      node.paddingTop = 1
      node.paddingLeft = 1
      parent = i
    } else {
      node.fgColor = packColor(200, 200, 220, 255)
      node.setText(`Node ${i}`)
    }

    node.markDirty(DIRTY_LAYOUT | DIRTY_VISUAL | DIRTY_HIERARCHY)

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
// BENCHMARK 1: AoS Write Speed (TS side)
// =============================================================================

console.log('┌─────────────────────────────────────────────────────────────────────┐')
console.log('│ 1. AoS WRITE SPEED — TS writing to AoS buffer                      │')
console.log('└─────────────────────────────────────────────────────────────────────┘\n')

for (const count of [100, 500, 1000, 2000]) {
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

  // Using NodeWriter
  const t0 = performance.now()
  for (let i = 0; i < iterations; i++) {
    const node = createNodeWriter(buf, i % nodeCount)
    node.width = 100 + (i % 50)
  }
  const t1 = performance.now()
  const writes = iterations
  const rate = writes / ((t1 - t0) / 1000)
  console.log(`  NodeWriter.width = X:              ${(rate / 1_000_000).toFixed(1)}M writes/sec`)

  // Direct DataView access for comparison
  const t2 = performance.now()
  for (let i = 0; i < iterations; i++) {
    const nodeIndex = i % nodeCount
    buf.view.setFloat32(HEADER_SIZE + nodeIndex * STRIDE + 0, 100 + (i % 50), true)
  }
  const t3 = performance.now()
  const rate2 = writes / ((t3 - t2) / 1000)
  console.log(`  Direct DataView.setFloat32:        ${(rate2 / 1_000_000).toFixed(1)}M writes/sec`)
}

// =============================================================================
// BENCHMARK 3: Memory Access Pattern
// =============================================================================

console.log('\n┌─────────────────────────────────────────────────────────────────────┐')
console.log('│ 3. MEMORY ACCESS PATTERN — Read all props for one node             │')
console.log('└─────────────────────────────────────────────────────────────────────┘\n')

buildTree(1000)

{
  // Simulate what Rust does: read ALL properties for a single node
  const iterations = 100_000

  const t0 = performance.now()
  let sum = 0
  for (let iter = 0; iter < iterations; iter++) {
    const nodeIndex = iter % 1000
    const base = HEADER_SIZE + nodeIndex * STRIDE

    // Read all layout floats (24 values) - all contiguous!
    sum += buf.view.getFloat32(base + 0, true)   // width
    sum += buf.view.getFloat32(base + 4, true)   // height
    sum += buf.view.getFloat32(base + 8, true)   // min_width
    sum += buf.view.getFloat32(base + 12, true)  // min_height
    sum += buf.view.getFloat32(base + 16, true)  // max_width
    sum += buf.view.getFloat32(base + 20, true)  // max_height
    sum += buf.view.getFloat32(base + 24, true)  // flex_basis
    sum += buf.view.getFloat32(base + 28, true)  // flex_grow
    sum += buf.view.getFloat32(base + 32, true)  // flex_shrink
    sum += buf.view.getFloat32(base + 36, true)  // padding_top
    sum += buf.view.getFloat32(base + 40, true)  // padding_right
    sum += buf.view.getFloat32(base + 44, true)  // padding_bottom
    sum += buf.view.getFloat32(base + 48, true)  // padding_left
    sum += buf.view.getFloat32(base + 52, true)  // margin_top
    sum += buf.view.getFloat32(base + 56, true)  // margin_right
    sum += buf.view.getFloat32(base + 60, true)  // margin_bottom
    sum += buf.view.getFloat32(base + 64, true)  // margin_left
    sum += buf.view.getFloat32(base + 68, true)  // gap
    sum += buf.view.getFloat32(base + 72, true)  // row_gap
    sum += buf.view.getFloat32(base + 76, true)  // column_gap
    sum += buf.view.getFloat32(base + 80, true)  // inset_top
    sum += buf.view.getFloat32(base + 84, true)  // inset_right
    sum += buf.view.getFloat32(base + 88, true)  // inset_bottom
    sum += buf.view.getFloat32(base + 92, true)  // inset_left

    // Read layout enums (16 values)
    sum += buf.view.getUint8(base + 96)   // flex_direction
    sum += buf.view.getUint8(base + 97)   // flex_wrap
    sum += buf.view.getUint8(base + 98)   // justify_content
    sum += buf.view.getUint8(base + 99)   // align_items
    sum += buf.view.getUint8(base + 100)  // align_content
    sum += buf.view.getUint8(base + 101)  // align_self
    sum += buf.view.getUint8(base + 102)  // position
    sum += buf.view.getUint8(base + 103)  // overflow
    sum += buf.view.getUint8(base + 104)  // display
    sum += buf.view.getUint8(base + 105)  // border_top
    sum += buf.view.getUint8(base + 106)  // border_right
    sum += buf.view.getUint8(base + 107)  // border_bottom
    sum += buf.view.getUint8(base + 108)  // border_left
    sum += buf.view.getUint8(base + 109)  // component_type
    sum += buf.view.getUint8(base + 110)  // visible
  }
  const t1 = performance.now()

  const readsPerNode = 24 + 15 // floats + u8s
  const totalReads = iterations * readsPerNode
  const timeMs = t1 - t0
  const readsPerSec = totalReads / (timeMs / 1000)
  const nodesPerSec = iterations / (timeMs / 1000)

  console.log(`  Read all 39 props/node × ${iterations} iterations:`)
  console.log(`    Time: ${timeMs.toFixed(1)}ms`)
  console.log(`    ${(readsPerSec / 1_000_000).toFixed(0)}M field reads/sec`)
  console.log(`    ${(nodesPerSec / 1_000_000).toFixed(1)}M full-node reads/sec`)
  console.log(`    (sum=${sum.toFixed(0)} to prevent optimization)`)
}

// =============================================================================
// BENCHMARK 4: Compare with SoA-style scattered reads
// =============================================================================

console.log('\n┌─────────────────────────────────────────────────────────────────────┐')
console.log('│ 4. AoS vs SoA — Simulated scattered vs contiguous reads            │')
console.log('└─────────────────────────────────────────────────────────────────────┘\n')

{
  // Create a fake SoA layout for comparison
  const nodeCount = 1000
  const soaBuffer = new ArrayBuffer(nodeCount * 4 * 40) // 40 f32 arrays
  const soaViews: Float32Array[] = []
  for (let i = 0; i < 40; i++) {
    soaViews.push(new Float32Array(soaBuffer, i * nodeCount * 4, nodeCount))
  }

  // Fill with data
  for (let n = 0; n < nodeCount; n++) {
    for (let f = 0; f < 40; f++) {
      soaViews[f][n] = Math.random()
    }
  }

  const iterations = 100_000

  // SoA: scattered reads (40 different arrays)
  const t0 = performance.now()
  let sumSoA = 0
  for (let iter = 0; iter < iterations; iter++) {
    const nodeIndex = iter % nodeCount
    for (let f = 0; f < 40; f++) {
      sumSoA += soaViews[f][nodeIndex]  // Each read from different memory location
    }
  }
  const t1 = performance.now()
  const soaTime = t1 - t0

  // AoS: contiguous reads (one node at a time)
  const t2 = performance.now()
  let sumAoS = 0
  for (let iter = 0; iter < iterations; iter++) {
    const nodeIndex = iter % 1000
    const base = HEADER_SIZE + nodeIndex * STRIDE
    // Read 40 values from contiguous memory
    for (let f = 0; f < 24; f++) {
      sumAoS += buf.view.getFloat32(base + f * 4, true)
    }
    for (let f = 0; f < 16; f++) {
      sumAoS += buf.view.getUint8(base + 96 + f)
    }
  }
  const t3 = performance.now()
  const aosTime = t3 - t2

  console.log(`  Reading 40 props × ${iterations} nodes:`)
  console.log(`    SoA (scattered): ${soaTime.toFixed(1)}ms (sum=${sumSoA.toFixed(0)})`)
  console.log(`    AoS (contiguous): ${aosTime.toFixed(1)}ms (sum=${sumAoS.toFixed(0)})`)
  console.log(`    Speedup: ${(soaTime / aosTime).toFixed(1)}x`)
}

// =============================================================================
// BENCHMARK 5: RUST AoS LAYOUT COMPUTATION (THE BIG TEST!)
// =============================================================================

console.log('\n┌─────────────────────────────────────────────────────────────────────┐')
console.log('│ 5. RUST AoS LAYOUT — The moment of truth!                          │')
console.log('└─────────────────────────────────────────────────────────────────────┘\n')

for (const count of [100, 500, 1000, 2000]) {
  buildTree(count)

  const r = bench(() => {
    engine.spark_compute_layout_aos()
  }, 100)

  printBench(`${count} nodes AoS layout`, r)
}

// =============================================================================
// BENCHMARK 6: COMPARISON WITH OLD SoA LAYOUT
// =============================================================================

console.log('\n┌─────────────────────────────────────────────────────────────────────┐')
console.log('│ 6. COMPARISON — AoS vs SoA (needs old SoA buffer for fair test)    │')
console.log('└─────────────────────────────────────────────────────────────────────┘\n')

// Build tree once
buildTree(1000)

// AoS timing
const aosResult = bench(() => {
  engine.spark_compute_layout_aos()
}, 100)

console.log(`  AoS (1000 nodes): avg ${fmt(aosResult.avg).padStart(8)}  p99 ${fmt(aosResult.p99).padStart(8)}`)
console.log('')
console.log('  Target (pure Rust): ~32μs')
console.log(`  Achieved:           ${fmt(aosResult.avg)}`)
console.log(`  Speedup vs old SoA (~2900μs): ${(2900 / aosResult.avg).toFixed(0)}x`)

// =============================================================================
// SUMMARY
// =============================================================================

console.log('\n╔══════════════════════════════════════════════════════════════════╗')
console.log('║                           SUMMARY                                ║')
console.log('╚══════════════════════════════════════════════════════════════════╝')
console.log(`
  AoS layout stores all properties for each node contiguously.

  Results:
  - AoS 1000 nodes: ${fmt(aosResult.avg)}
  - Pure Rust target: ~32μs
  - Old SoA hybrid: ~2900μs

  If AoS is close to ~32μs, the architecture is VIABLE!
`)

process.exit(0)
