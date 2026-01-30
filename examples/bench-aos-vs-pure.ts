/**
 * AoS Benchmark — Matching Pure Rust Structure
 *
 * This benchmark replicates the exact structure from:
 * ../tui-rust/crates/tui/examples/benchmark.rs
 *
 * So we can compare apples to apples.
 *
 * Run: bun run examples/bench-aos-vs-pure.ts
 */

import { ptr } from 'bun:ffi'
import { dlopen, FFIType } from 'bun:ffi'

import {
  createAoSBuffer,
  setTerminalSize,
  setNodeCount,
  createNodeWriter,
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

// =============================================================================
// SETUP
// =============================================================================

const buf = createAoSBuffer()
setTerminalSize(buf, 200, 50) // Same as pure Rust benchmark

// Load Rust library
const libPath = new URL('../rust/target/release/libspark_tui_engine.dylib', import.meta.url).pathname
const engine = dlopen(libPath, {
  spark_init_aos: { args: [FFIType.ptr, FFIType.u32], returns: FFIType.i32 },
  spark_compute_layout_aos: { args: [], returns: FFIType.u32 },
}).symbols

engine.spark_init_aos(ptr(buf.buffer), buf.buffer.byteLength)

console.log('╔══════════════════════════════════════════════════════════════════╗')
console.log('║    AoS Hybrid Benchmark (Matching Pure Rust Structure)          ║')
console.log('╚══════════════════════════════════════════════════════════════════╝\n')

// =============================================================================
// HELPERS
// =============================================================================

function resetBuffer() {
  // Clear node count and all nodes
  setNodeCount(buf, 0)
  for (let i = 0; i < 4096; i++) {
    const node = createNodeWriter(buf, i)
    node.componentType = 0
    node.visible = 0
    node.parentIndex = -1
  }
  // Reset text pool
  buf.header[8] = 0 // TEXT_POOL_WRITE_PTR
}

// =============================================================================
// BENCHMARK 3: Layout Computation (matching pure Rust)
// =============================================================================
// Pure Rust creates nested layout: rows of columns
// Each row has 10 boxes with borders

console.log('┌──────────────────────────────────────────────────────────────────────┐')
console.log('│ 3. Layout Computation (Taffy) — Matching Pure Rust                  │')
console.log('└──────────────────────────────────────────────────────────────────────┘\n')

for (const count of [100, 500, 1000, 5000]) {
  resetBuffer()

  // Build nested layout matching pure Rust
  // Root with children rows, each row has ~10 boxes
  const rowCount = Math.floor(count / 10)
  let nodeIndex = 0

  // Root
  const root = createNodeWriter(buf, nodeIndex++)
  root.componentType = COMPONENT_BOX
  root.visible = 1
  root.width = -100 // percent (negative = percent)
  root.height = -100
  root.flexDirection = 1 // column
  root.parentIndex = -1
  root.markDirty(DIRTY_LAYOUT | DIRTY_HIERARCHY)

  // Create rows
  for (let r = 0; r < rowCount; r++) {
    const rowNode = createNodeWriter(buf, nodeIndex++)
    rowNode.componentType = COMPONENT_BOX
    rowNode.visible = 1
    rowNode.width = -100 // percent
    rowNode.flexDirection = 0 // row
    rowNode.parentIndex = 0 // child of root
    rowNode.markDirty(DIRTY_LAYOUT | DIRTY_HIERARCHY)

    // Create 10 boxes per row
    for (let c = 0; c < 10; c++) {
      const box = createNodeWriter(buf, nodeIndex++)
      box.componentType = COMPONENT_BOX
      box.visible = 1
      box.width = 10
      box.height = 2
      box.borderTop = 1
      box.borderRight = 1
      box.borderBottom = 1
      box.borderLeft = 1
      box.parentIndex = 1 + r // parent is the row
      box.markDirty(DIRTY_LAYOUT | DIRTY_HIERARCHY)
    }
  }

  setNodeCount(buf, nodeIndex)

  // Warmup
  for (let i = 0; i < 5; i++) {
    engine.spark_compute_layout_aos()
  }

  // Benchmark
  const iterations = 100
  const times: number[] = []

  for (let i = 0; i < iterations; i++) {
    const t0 = performance.now()
    engine.spark_compute_layout_aos()
    const t1 = performance.now()
    times.push((t1 - t0) * 1000) // μs
  }

  times.sort((a, b) => a - b)
  const avg = times.reduce((a, b) => a + b, 0) / times.length
  const p50 = times[Math.floor(times.length * 0.5)]

  console.log(`  ${String(nodeIndex).padStart(5)} nodes: ${avg.toFixed(2).padStart(8)}μs/layout  (${(avg / 1000).toFixed(2)} ms)`)
}

// =============================================================================
// BENCHMARK 6: Laggy Threshold — Full Pipeline
// =============================================================================

console.log('\n┌──────────────────────────────────────────────────────────────────────┐')
console.log('│ 6. Laggy Threshold — Where Does It Feel Slow?                       │')
console.log('└──────────────────────────────────────────────────────────────────────┘')
console.log('  Target: <16.6ms for 60 FPS, <33ms for 30 FPS\n')
console.log('  Full Pipeline (layout only for now):')

for (const count of [1000, 2500, 5000, 10000, 20000, 50000]) {
  resetBuffer()

  // Build nested grid: sqrt(count) × sqrt(count)
  const rows = Math.floor(Math.sqrt(count))
  const cols = rows
  const actualCount = rows * cols
  let nodeIndex = 0

  // Root
  const root = createNodeWriter(buf, nodeIndex++)
  root.componentType = COMPONENT_BOX
  root.visible = 1
  root.width = -100
  root.height = -100
  root.flexDirection = 1
  root.parentIndex = -1
  root.markDirty(DIRTY_LAYOUT | DIRTY_HIERARCHY)

  // Rows and cells
  for (let r = 0; r < rows; r++) {
    const rowNode = createNodeWriter(buf, nodeIndex++)
    rowNode.componentType = COMPONENT_BOX
    rowNode.visible = 1
    rowNode.flexDirection = 0
    rowNode.parentIndex = 0
    rowNode.markDirty(DIRTY_LAYOUT | DIRTY_HIERARCHY)

    for (let c = 0; c < cols; c++) {
      const cell = createNodeWriter(buf, nodeIndex++)
      cell.componentType = COMPONENT_BOX
      cell.visible = 1
      cell.width = 2
      cell.height = 1
      cell.parentIndex = 1 + r
      cell.markDirty(DIRTY_LAYOUT | DIRTY_HIERARCHY)
    }
  }

  setNodeCount(buf, nodeIndex)

  // Measure
  const iterations = 10
  const times: number[] = []

  for (let i = 0; i < iterations; i++) {
    const t0 = performance.now()
    engine.spark_compute_layout_aos()
    const t1 = performance.now()
    times.push(t1 - t0)
  }

  const avgMs = times.reduce((a, b) => a + b, 0) / times.length
  const fps = 1000 / avgMs

  let status: string
  if (avgMs < 16.6) {
    status = 'smooth (60fps+)'
  } else if (avgMs < 33) {
    status = 'acceptable (30fps+)'
  } else if (avgMs < 100) {
    status = 'LAGGY'
  } else {
    status = 'UNUSABLE'
  }

  console.log(`    ${String(nodeIndex).padStart(6)} nodes: ${avgMs.toFixed(2).padStart(8)}ms/frame (${fps.toFixed(0).padStart(5)} FPS) - ${status}`)

  if (avgMs > 100) {
    console.log('    ^ Found the laggy threshold!')
    break
  }
}

// =============================================================================
// COMPARISON TABLE
// =============================================================================

console.log('\n╔══════════════════════════════════════════════════════════════════╗')
console.log('║                    COMPARISON WITH PURE RUST                     ║')
console.log('╚══════════════════════════════════════════════════════════════════╝')
console.log(`
  Pure Rust (from tui-rust benchmark):
    111 nodes:   3.98μs/layout
    551 nodes:  15.41μs/layout
   1101 nodes:  32.15μs/layout
   5501 nodes: 196.78μs/layout

  AoS Hybrid (this benchmark):
    Run above to compare!

  The gap shows what overhead remains from:
  - SharedBuffer read cost
  - Style struct construction
  - Hierarchy rebuilding

  Goal: Get within 2-3x of pure Rust.
`)

process.exit(0)
