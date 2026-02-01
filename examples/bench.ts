/**
 * SparkTUI Benchmarks & Stress Tests
 *
 * Comprehensive performance testing for the reactive TUI framework.
 * Uses Bun.nanoseconds() for precise measurements.
 *
 * IMPORTANT: Uses a SINGLE mount() to avoid memory issues.
 * The Rust engine can only be initialized once per process.
 *
 * Run: bun examples/bench.ts
 */

import { signal, derived, effect } from '@rlabs-inc/signals'
import { box, text } from '../ts/primitives'
import { mountSync } from '../ts/engine'
import type { MountHandle } from '../ts/engine'

// =============================================================================
// HELPERS
// =============================================================================

function formatNs(ns: number): string {
  if (ns < 1000) return `${ns.toFixed(0)}ns`
  if (ns < 1_000_000) return `${(ns / 1000).toFixed(2)}μs`
  if (ns < 1_000_000_000) return `${(ns / 1_000_000).toFixed(2)}ms`
  return `${(ns / 1_000_000_000).toFixed(2)}s`
}

function formatOps(opsPerSec: number): string {
  if (opsPerSec >= 1_000_000_000) return `${(opsPerSec / 1_000_000_000).toFixed(2)}B`
  if (opsPerSec >= 1_000_000) return `${(opsPerSec / 1_000_000).toFixed(2)}M`
  if (opsPerSec >= 1_000) return `${(opsPerSec / 1_000).toFixed(2)}K`
  return opsPerSec.toFixed(0)
}

function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms))
}

// =============================================================================
// BENCHMARK: Signal Updates (Pure TS, no engine)
// =============================================================================

function benchSignalUpdates() {
  console.log('\n📊 Signal Updates (Pure TS)')
  console.log('─'.repeat(50))

  const count = signal(0)
  const iterations = 1_000_000

  // Warm up
  for (let i = 0; i < 1000; i++) {
    count.value = i
  }

  const start = Bun.nanoseconds()
  for (let i = 0; i < iterations; i++) {
    count.value = i
  }
  const end = Bun.nanoseconds()

  const totalNs = end - start
  const perOp = totalNs / iterations
  const opsPerSec = 1_000_000_000 / perOp

  console.log(`  Iterations: ${iterations.toLocaleString()}`)
  console.log(`  Per update: ${formatNs(perOp)}`)
  console.log(`  Throughput: ${formatOps(opsPerSec)} ops/sec`)
}

// =============================================================================
// BENCHMARK: Derived Chain (Pure TS)
// =============================================================================

function benchDerivedChain() {
  console.log('\n📊 Derived Chain (Pure TS)')
  console.log('─'.repeat(50))

  const source = signal(0)
  const d1 = derived(() => source.value + 1)
  const d2 = derived(() => d1.value * 2)
  const d3 = derived(() => d2.value - 1)
  const d4 = derived(() => d3.value + 10)

  const iterations = 100_000

  // Warm up
  for (let i = 0; i < 1000; i++) {
    source.value = i
    const _ = d4.value
  }

  const start = Bun.nanoseconds()
  for (let i = 0; i < iterations; i++) {
    source.value = i
    const _ = d4.value // Force propagation
  }
  const end = Bun.nanoseconds()

  const totalNs = end - start
  const perOp = totalNs / iterations
  const opsPerSec = 1_000_000_000 / perOp

  console.log(`  Chain depth: 4 deriveds`)
  console.log(`  Iterations: ${iterations.toLocaleString()}`)
  console.log(`  Per propagation: ${formatNs(perOp)}`)
  console.log(`  Throughput: ${formatOps(opsPerSec)} chains/sec`)
}

// =============================================================================
// MAIN - SINGLE APP INSTANCE FOR ALL BENCHMARKS
// =============================================================================

console.log('═'.repeat(60))
console.log('  SparkTUI Benchmarks & Stress Tests')
console.log('═'.repeat(60))

// Pure TS benchmarks (no engine needed)
benchSignalUpdates()
benchDerivedChain()

// =============================================================================
// ENGINE BENCHMARKS - Using single mount
// =============================================================================

console.log('\n📊 Engine Benchmarks')
console.log('─'.repeat(50))

// Create signals for testing
const counter = signal(0)
const textContent = signal('Initial')
const nodeSignals: ReturnType<typeof signal<number>>[] = []
const NODE_COUNT = 100

for (let i = 0; i < NODE_COUNT; i++) {
  nodeSignals.push(signal(i))
}

// Mount a single app for all tests
const app = mountSync(() => {
  box({
    flexDirection: 'column',
    children: () => {
      // Counter display
      text({ content: counter })

      // Text content display
      text({ content: textContent })

      // Many nodes for stress test
      box({
        flexDirection: 'row',
        flexWrap: 'wrap',
        children: () => {
          for (let i = 0; i < NODE_COUNT; i++) {
            text({ content: nodeSignals[i] })
          }
        },
      })
    },
  })
}, { mode: 'inline' })

// Wait for initial render
await sleep(200)

// =============================================================================
// TEST: Single Signal Updates
// =============================================================================

{
  console.log('\n📊 Single Signal Updates (with engine)')
  console.log('─'.repeat(50))

  const iterations = 1000

  const start = Bun.nanoseconds()
  for (let i = 0; i < iterations; i++) {
    counter.value = i
    await sleep(0) // Yield to allow render
  }
  const end = Bun.nanoseconds()

  const totalNs = end - start
  const perUpdate = totalNs / iterations
  const fps = 1_000_000_000 / perUpdate

  console.log(`  Iterations: ${iterations}`)
  console.log(`  Per update: ${formatNs(perUpdate)}`)
  console.log(`  Effective FPS: ${fps.toFixed(0)}`)
}

// =============================================================================
// TEST: Burst Updates (wake coalescing)
// =============================================================================

{
  console.log('\n📊 Burst Updates (wake coalescing)')
  console.log('─'.repeat(50))

  const burstSize = 100_000

  const start = Bun.nanoseconds()
  for (let i = 0; i < burstSize; i++) {
    counter.value = i
  }
  await sleep(50) // Wait for render to catch up
  const end = Bun.nanoseconds()

  const totalNs = end - start
  const perUpdate = totalNs / burstSize
  const updatesPerSec = 1_000_000_000 / perUpdate

  console.log(`  Burst size: ${burstSize.toLocaleString()} updates`)
  console.log(`  Total time: ${formatNs(totalNs)}`)
  console.log(`  Per update: ${formatNs(perUpdate)}`)
  console.log(`  Throughput: ${formatOps(updatesPerSec)} updates/sec`)
  console.log(`  (Updates coalesce into single frame)`)
}

// =============================================================================
// TEST: Many Nodes Update
// =============================================================================

{
  console.log('\n📊 Many Nodes Update')
  console.log('─'.repeat(50))

  const updateRounds = 100

  const start = Bun.nanoseconds()
  for (let round = 0; round < updateRounds; round++) {
    for (let i = 0; i < NODE_COUNT; i++) {
      nodeSignals[i].value = round * NODE_COUNT + i
    }
    await sleep(0) // Yield to render
  }
  const end = Bun.nanoseconds()

  const totalUpdates = NODE_COUNT * updateRounds
  const totalNs = end - start
  const perUpdate = totalNs / totalUpdates
  const framesRendered = updateRounds

  console.log(`  Nodes: ${NODE_COUNT}`)
  console.log(`  Update rounds: ${updateRounds}`)
  console.log(`  Total updates: ${totalUpdates.toLocaleString()}`)
  console.log(`  Total time: ${formatNs(totalNs)}`)
  console.log(`  Per frame: ${formatNs(totalNs / framesRendered)}`)
}

// =============================================================================
// TEST: Text Content Changes
// =============================================================================

{
  console.log('\n📊 Text Content Changes')
  console.log('─'.repeat(50))

  const iterations = 500
  const strings = [
    'Short',
    'Medium length string here',
    'This is a longer string that requires more processing',
    'A',
    'Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod',
  ]

  const start = Bun.nanoseconds()
  for (let i = 0; i < iterations; i++) {
    textContent.value = strings[i % strings.length] + ` (${i})`
    await sleep(0)
  }
  const end = Bun.nanoseconds()

  const totalNs = end - start
  const perChange = totalNs / iterations

  console.log(`  Text changes: ${iterations}`)
  console.log(`  Per change: ${formatNs(perChange)}`)
  console.log(`  (Includes text measurement + layout)`)
}

// =============================================================================
// TEST: Sustained Update Rate
// =============================================================================

{
  console.log('\n📊 Sustained Update Rate (1 second)')
  console.log('─'.repeat(50))

  let updates = 0
  const duration = 1000 // 1 second

  const start = Date.now()
  while (Date.now() - start < duration) {
    counter.value = updates++
    await sleep(0)
  }
  const elapsed = Date.now() - start

  const fps = (updates / elapsed) * 1000

  console.log(`  Duration: ${elapsed}ms`)
  console.log(`  Updates: ${updates.toLocaleString()}`)
  console.log(`  Sustained FPS: ${fps.toFixed(0)}`)
}

// =============================================================================
// TEST: Read Rust Timing Stats
// =============================================================================

{
  console.log('\n📊 Pipeline Timing (from Rust)')
  console.log('─'.repeat(50))

  // Trigger a render
  counter.value = 999999
  await sleep(100)

  // Read timing stats from buffer
  const view = new DataView(app.buffer.raw)
  const layoutUs = view.getUint32(200, true)
  const fbUs = view.getUint32(204, true)
  const renderUs = view.getUint32(208, true)
  const totalUs = view.getUint32(212, true)

  console.log(`  Layout:      ${layoutUs}μs`)
  console.log(`  Framebuffer: ${fbUs}μs`)
  console.log(`  Render:      ${renderUs}μs`)
  console.log(`  Total:       ${totalUs}μs`)
}

// =============================================================================
// CLEANUP
// =============================================================================

console.log('\n' + '═'.repeat(60))
console.log('  Benchmarks Complete')
console.log('═'.repeat(60) + '\n')

app.unmount()
process.exit(0)
