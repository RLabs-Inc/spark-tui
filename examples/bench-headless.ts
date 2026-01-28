/**
 * SparkTUI — Headless Computation Benchmarks
 *
 * Pure computation speed: layout + framebuffer, no engine thread, no terminal.
 * Tests the raw throughput of the Rust computation pipeline.
 *
 * Run: bun run examples/bench-headless.ts
 */

import { ptr } from 'bun:ffi'
import { loadEngine } from '../ts/bridge/ffi'
import {
  createSharedBuffer,
  setTerminalSize,
  setNodeText,
  markDirty,
  packColor,
  COMPONENT_BOX,
  COMPONENT_TEXT,
  U8_COMPONENT_TYPE,
  U8_VISIBLE,
  U8_FLEX_DIRECTION,
  U8_ALIGN_ITEMS,
  U8_JUSTIFY_CONTENT,
  U8_BORDER_STYLE,
  U8_BORDER_TOP_WIDTH,
  U8_BORDER_RIGHT_WIDTH,
  U8_BORDER_BOTTOM_WIDTH,
  U8_BORDER_LEFT_WIDTH,
  U32_FG_COLOR,
  U32_BG_COLOR,
  U32_BORDER_COLOR,
  I32_PARENT_INDEX,
  F32_WIDTH,
  F32_HEIGHT,
  F32_PADDING_TOP,
  F32_PADDING_LEFT,
  F32_GROW,
  DIRTY_LAYOUT,
  DIRTY_VISUAL,
  DIRTY_TEXT,
  DIRTY_HIERARCHY,
  HEADER_NODE_COUNT,
} from '../ts/bridge/shared-buffer'

console.log('=== SparkTUI Headless Computation Benchmarks ===\n')

// =============================================================================
// SETUP
// =============================================================================

const views = createSharedBuffer()
setTerminalSize(views, 120, 40)

const engine = loadEngine()

// Initialize Rust (buffer only, no engine thread since we call computeLayout directly)
// We use a low-level approach: just pass the buffer pointer

// Helper: build a tree of N nodes
function buildTree(nodeCount: number) {
  // Reset
  views.header[HEADER_NODE_COUNT] = 0

  // Root: full terminal box
  views.u8[U8_COMPONENT_TYPE][0] = COMPONENT_BOX
  views.u8[U8_VISIBLE][0] = 1
  views.u8[U8_FLEX_DIRECTION][0] = 1  // column
  views.f32[F32_WIDTH][0] = 120
  views.f32[F32_HEIGHT][0] = 40
  views.i32[I32_PARENT_INDEX][0] = -1
  views.u32[U32_BG_COLOR][0] = packColor(20, 20, 40, 255)
  markDirty(views, 0, DIRTY_LAYOUT | DIRTY_VISUAL | DIRTY_HIERARCHY)

  // Build children
  let parent = 0
  const childrenPerGroup = 5

  for (let i = 1; i < nodeCount; i++) {
    const isContainer = i < nodeCount / 2 && (i % childrenPerGroup === 0)

    views.u8[U8_COMPONENT_TYPE][i] = isContainer ? COMPONENT_BOX : COMPONENT_TEXT
    views.u8[U8_VISIBLE][i] = 1
    views.i32[I32_PARENT_INDEX][i] = parent

    if (isContainer) {
      views.u8[U8_FLEX_DIRECTION][i] = (i % 2 === 0) ? 0 : 1 // alternate row/column
      views.f32[F32_GROW][i] = 1
      views.u32[U32_BG_COLOR][i] = packColor(30 + (i % 20), 30, 60, 255)
      views.u8[U8_BORDER_STYLE][i] = 1
      views.u8[U8_BORDER_TOP_WIDTH][i] = 1
      views.u8[U8_BORDER_RIGHT_WIDTH][i] = 1
      views.u8[U8_BORDER_BOTTOM_WIDTH][i] = 1
      views.u8[U8_BORDER_LEFT_WIDTH][i] = 1
      views.u32[U32_BORDER_COLOR][i] = packColor(80, 80, 120, 255)
      views.f32[F32_PADDING_TOP][i] = 1
      views.f32[F32_PADDING_LEFT][i] = 1
      parent = i
    } else {
      views.u32[U32_FG_COLOR][i] = packColor(200, 200, 220, 255)
      setNodeText(views, i, `Node ${i}`)
    }

    markDirty(views, i, DIRTY_LAYOUT | DIRTY_VISUAL | DIRTY_TEXT | DIRTY_HIERARCHY)

    // Rotate parent back occasionally
    if (i % (childrenPerGroup * 3) === 0 && parent > 0) {
      parent = Math.max(0, parent - childrenPerGroup)
    }
  }

  views.header[HEADER_NODE_COUNT] = nodeCount
}

// Helper: benchmark a function
function bench(name: string, fn: () => void, iterations: number): { avg: number, min: number, max: number, p50: number, p99: number } {
  // Warmup
  for (let i = 0; i < 5; i++) fn()

  const times: number[] = []
  for (let i = 0; i < iterations; i++) {
    const t0 = performance.now()
    fn()
    const t1 = performance.now()
    times.push((t1 - t0) * 1000) // μs
  }

  times.sort((a, b) => a - b)
  const avg = times.reduce((a, b) => a + b, 0) / times.length
  const min = times[0]
  const max = times[times.length - 1]
  const p50 = times[Math.floor(times.length * 0.5)]
  const p99 = times[Math.floor(times.length * 0.99)]

  return { avg, min, max, p50, p99 }
}

function printResult(name: string, r: { avg: number, min: number, max: number, p50: number, p99: number }, unit = 'μs') {
  const fps = 1_000_000 / r.avg
  console.log(`  ${name}`)
  console.log(`    avg: ${r.avg.toFixed(0)}${unit}  p50: ${r.p50.toFixed(0)}${unit}  p99: ${r.p99.toFixed(0)}${unit}  min: ${r.min.toFixed(0)}${unit}  max: ${r.max.toFixed(0)}${unit}`)
  console.log(`    throughput: ${fps.toFixed(0)} ops/sec (${(fps / 60).toFixed(1)}x 60fps)`)
}

// =============================================================================
// BENCHMARK 1: Layout computation at various node counts
// =============================================================================

console.log('--- Benchmark 1: Layout Computation Speed ---\n')

// We need to init the engine buffer first (for get_buffer() to work in spark_compute_layout)
const initResult = engine.init(ptr(views.buffer), views.buffer.byteLength)
if (initResult !== 0) {
  // init starts the engine thread which needs a TTY — that's OK,
  // we'll just use computeLayout() directly which works via get_buffer()
  console.log('  Note: Engine thread failed (no TTY), but computeLayout() still works\n')
}

const nodeCounts = [5, 10, 50, 100, 500, 1000, 2000]

for (const count of nodeCounts) {
  buildTree(count)
  const r = bench(`${count} nodes`, () => engine.computeLayout(), 100)
  printResult(`${String(count).padStart(4)} nodes`, r)
  console.log('')
}

// =============================================================================
// BENCHMARK 2: Layout + Framebuffer (spark_render)
// =============================================================================

console.log('--- Benchmark 2: Layout + Framebuffer Speed ---\n')

for (const count of [5, 50, 500, 2000]) {
  buildTree(count)
  const r = bench(`${count} nodes (layout+fb)`, () => engine.render(), 50)
  printResult(`${String(count).padStart(4)} nodes (layout+fb)`, r)
  console.log('')
}

// =============================================================================
// BENCHMARK 3: Visual-only update (no layout recomputation)
// =============================================================================

console.log('--- Benchmark 3: Visual-Only Update (layout skip) ---\n')

buildTree(500)
engine.computeLayout() // compute once

// Now change only colors (no DIRTY_LAYOUT)
const r3 = bench('500 nodes visual-only', () => {
  for (let i = 0; i < 500; i++) {
    views.u32[U32_FG_COLOR][i] = packColor(
      Math.floor(Math.random() * 255),
      Math.floor(Math.random() * 255),
      Math.floor(Math.random() * 255),
      255
    )
    views.dirty[i] |= DIRTY_VISUAL  // visual only, not layout
  }
  engine.computeLayout()
}, 100)
printResult(' 500 nodes visual-only', r3)

// =============================================================================
// BENCHMARK 4: SharedArrayBuffer write throughput (TS side only)
// =============================================================================

console.log('\n--- Benchmark 4: SharedArrayBuffer Write Speed (TS only) ---\n')

{
  const iterations = 10000
  const nodeCount = 1000

  // Float32 writes
  const t0 = performance.now()
  for (let i = 0; i < iterations; i++) {
    for (let n = 0; n < nodeCount; n++) {
      views.f32[F32_WIDTH][n] = 100 + (i % 50)
    }
  }
  const t1 = performance.now()
  const totalWrites = iterations * nodeCount
  const writeTimeMs = t1 - t0
  const writesPerSec = totalWrites / (writeTimeMs / 1000)
  console.log(`  ${totalWrites.toLocaleString()} Float32 writes in ${writeTimeMs.toFixed(1)}ms`)
  console.log(`  ${(writesPerSec / 1_000_000).toFixed(1)}M writes/sec`)

  // Color (u32) writes
  const t2 = performance.now()
  for (let i = 0; i < iterations; i++) {
    for (let n = 0; n < nodeCount; n++) {
      views.u32[U32_FG_COLOR][n] = packColor(i % 255, 100, 200, 255)
    }
  }
  const t3 = performance.now()
  const colorWriteMs = t3 - t2
  const colorWritesSec = totalWrites / (colorWriteMs / 1000)
  console.log(`  ${totalWrites.toLocaleString()} Color u32 writes in ${colorWriteMs.toFixed(1)}ms`)
  console.log(`  ${(colorWritesSec / 1_000_000).toFixed(1)}M writes/sec`)
}

// =============================================================================
// BENCHMARK 5: Text pool write throughput
// =============================================================================

console.log('\n--- Benchmark 5: Text Pool Write Speed ---\n')

{
  buildTree(100)
  const iterations = 1000
  const t0 = performance.now()
  for (let i = 0; i < iterations; i++) {
    // Reset text pool
    views.header[7] = 0 // HEADER_TEXT_POOL_WRITE_PTR
    for (let n = 0; n < 100; n++) {
      setNodeText(views, n, `Counter: ${i * 100 + n} — updating rapidly`)
    }
  }
  const t1 = performance.now()
  const totalTextWrites = iterations * 100
  const textWriteMs = t1 - t0
  console.log(`  ${totalTextWrites.toLocaleString()} text writes in ${textWriteMs.toFixed(1)}ms`)
  console.log(`  ${(totalTextWrites / (textWriteMs / 1000)).toFixed(0)} text writes/sec`)
}

// =============================================================================
// SUMMARY
// =============================================================================

console.log('\n=== Summary ===\n')
console.log('  Layout speed determines max FPS for layout-affecting changes.')
console.log('  Visual-only changes skip layout entirely (smart skip).')
console.log('  SharedArrayBuffer writes are the TS→Rust data path (zero-copy).')
console.log('')
console.log('  Run bench-stress.ts in a real terminal for full pipeline stress tests.')

engine.close()
process.exit(0)
