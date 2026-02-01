/**
 * SparkTUI Comprehensive Benchmark Suite
 *
 * Tests EVERYTHING using ONLY the real public APIs.
 * No FFI bypassing, no internal access - just what users would write.
 *
 * Measures:
 * 1. Pure TypeScript reactivity (signals, deriveds, effects, batches)
 * 2. SharedArrayBuffer raw operations
 * 3. Full E2E: signal.value = x → terminal screen update
 * 4. Rust pipeline timing (from SharedBuffer header)
 * 5. Sustained throughput
 * 6. Batch/coalescing behavior
 * 7. Wake latency (cold vs hot)
 * 8. Different operation types comparison
 *
 * Run: bun examples/bench-comprehensive.ts
 */

import { signal, derived, effect, batch } from '@rlabs-inc/signals'
import { box, text } from '../ts/primitives'
import { mountSync, type MountHandle } from '../ts/engine'
import {
  getTimingStats,
  getRenderCount,
  getNodeCount,
} from '../ts/bridge/shared-buffer'

// =============================================================================
// ANSI & FORMATTING
// =============================================================================

const c = {
  reset: '\x1b[0m',
  bold: '\x1b[1m',
  dim: '\x1b[2m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  magenta: '\x1b[35m',
  cyan: '\x1b[36m',
  bgMagenta: '\x1b[45m',
}

function fmt(ns: number): string {
  if (ns < 1000) return `${ns.toFixed(0)}ns`
  if (ns < 1_000_000) return `${(ns / 1000).toFixed(2)}μs`
  if (ns < 1_000_000_000) return `${(ns / 1_000_000).toFixed(2)}ms`
  return `${(ns / 1_000_000_000).toFixed(2)}s`
}

function fmtOps(ops: number): string {
  if (ops >= 1_000_000_000) return `${(ops / 1_000_000_000).toFixed(2)}B`
  if (ops >= 1_000_000) return `${(ops / 1_000_000).toFixed(2)}M`
  if (ops >= 1_000) return `${(ops / 1_000).toFixed(2)}K`
  return ops.toFixed(0)
}

function fmtFps(fps: number): string {
  if (fps >= 1000) return `${(fps / 1000).toFixed(1)}K`
  return fps.toFixed(0)
}

function percentile(sorted: number[], p: number): number {
  const idx = Math.floor(sorted.length * p)
  return sorted[Math.min(idx, sorted.length - 1)]
}

function stats(samples: number[]): { avg: number; p50: number; p95: number; p99: number; min: number; max: number } {
  if (samples.length === 0) return { avg: 0, p50: 0, p95: 0, p99: 0, min: 0, max: 0 }
  const sorted = [...samples].sort((a, b) => a - b)
  const avg = samples.reduce((a, b) => a + b, 0) / samples.length
  return {
    avg,
    p50: percentile(sorted, 0.5),
    p95: percentile(sorted, 0.95),
    p99: percentile(sorted, 0.99),
    min: sorted[0],
    max: sorted[sorted.length - 1],
  }
}

function printStats(label: string, samples: number[], unit: 'ns' | 'us' = 'ns') {
  const s = stats(samples)
  const scale = unit === 'us' ? 1000 : 1
  console.log(`  ${c.cyan}${label}${c.reset}`)
  console.log(`    avg: ${c.yellow}${fmt(s.avg * scale)}${c.reset}  p50: ${fmt(s.p50 * scale)}  p95: ${fmt(s.p95 * scale)}  p99: ${fmt(s.p99 * scale)}`)
  console.log(`    min: ${fmt(s.min * scale)}  max: ${fmt(s.max * scale)}`)
}

function header(title: string) {
  console.log()
  console.log(`${c.bgMagenta}${c.bold} ${title} ${c.reset}`)
  console.log()
}

function subheader(title: string) {
  console.log(`  ${c.bold}${c.blue}${title}${c.reset}`)
}

// =============================================================================
// SECTION 1: PURE TYPESCRIPT BENCHMARKS (No Engine)
// =============================================================================

console.log()
console.log(`${c.bold}${c.magenta}${'═'.repeat(60)}${c.reset}`)
console.log(`${c.bold}${c.magenta}  SparkTUI Comprehensive Benchmark Suite${c.reset}`)
console.log(`${c.bold}${c.magenta}${'═'.repeat(60)}${c.reset}`)

header('1. PURE TYPESCRIPT REACTIVITY (no engine)')

// Signal Creation
{
  subheader('Signal Creation')
  const iterations = 100_000
  const samples: number[] = []

  // Warmup
  for (let i = 0; i < 1000; i++) signal(i)

  for (let i = 0; i < iterations; i++) {
    const t0 = Bun.nanoseconds()
    signal(i)
    samples.push(Bun.nanoseconds() - t0)
  }

  printStats(`${iterations.toLocaleString()} signals`, samples)
  console.log(`    throughput: ${c.green}${fmtOps(1_000_000_000 / stats(samples).avg)}/sec${c.reset}`)
}

// Signal Read
{
  subheader('Signal Read')
  const sig = signal(42)
  const iterations = 1_000_000
  const samples: number[] = []

  for (let i = 0; i < 1000; i++) { const _ = sig.value }

  for (let i = 0; i < iterations; i++) {
    const t0 = Bun.nanoseconds()
    const _ = sig.value
    samples.push(Bun.nanoseconds() - t0)
  }

  printStats(`${iterations.toLocaleString()} reads`, samples)
  console.log(`    throughput: ${c.green}${fmtOps(1_000_000_000 / stats(samples).avg)}/sec${c.reset}`)
}

// Signal Write
{
  subheader('Signal Write')
  const sig = signal(0)
  const iterations = 1_000_000
  const samples: number[] = []

  for (let i = 0; i < 1000; i++) sig.value = i

  for (let i = 0; i < iterations; i++) {
    const t0 = Bun.nanoseconds()
    sig.value = i
    samples.push(Bun.nanoseconds() - t0)
  }

  printStats(`${iterations.toLocaleString()} writes`, samples)
  console.log(`    throughput: ${c.green}${fmtOps(1_000_000_000 / stats(samples).avg)}/sec${c.reset}`)
}

// Derived Chain
{
  subheader('Derived Chain (4 deep)')
  const source = signal(0)
  const d1 = derived(() => source.value + 1)
  const d2 = derived(() => d1.value * 2)
  const d3 = derived(() => d2.value - 1)
  const d4 = derived(() => d3.value + 10)

  const iterations = 100_000
  const samples: number[] = []

  for (let i = 0; i < 1000; i++) { source.value = i; const _ = d4.value }

  for (let i = 0; i < iterations; i++) {
    const t0 = Bun.nanoseconds()
    source.value = i
    const _ = d4.value
    samples.push(Bun.nanoseconds() - t0)
  }

  printStats(`${iterations.toLocaleString()} propagations`, samples)
  console.log(`    throughput: ${c.green}${fmtOps(1_000_000_000 / stats(samples).avg)}/sec${c.reset}`)
}

// Effect Firing
{
  subheader('Effect Firing')
  const sig = signal(0)
  let runs = 0
  const stop = effect(() => { const _ = sig.value; runs++ })

  const iterations = 100_000
  const samples: number[] = []

  for (let i = 0; i < 1000; i++) sig.value = i
  runs = 0

  for (let i = 0; i < iterations; i++) {
    const t0 = Bun.nanoseconds()
    sig.value = i
    samples.push(Bun.nanoseconds() - t0)
  }

  stop()
  printStats(`${iterations.toLocaleString()} effect triggers`, samples)
  console.log(`    throughput: ${c.green}${fmtOps(1_000_000_000 / stats(samples).avg)}/sec${c.reset}`)
}

// Batch Updates
{
  subheader('Batch Updates (100 signals)')
  const signals = Array.from({ length: 100 }, (_, i) => signal(i))
  const iterations = 10_000
  const samples: number[] = []

  for (let iter = 0; iter < 100; iter++) {
    batch(() => { for (let i = 0; i < 100; i++) signals[i].value = iter })
  }

  for (let iter = 0; iter < iterations; iter++) {
    const t0 = Bun.nanoseconds()
    batch(() => {
      for (let i = 0; i < 100; i++) signals[i].value = iter
    })
    samples.push(Bun.nanoseconds() - t0)
  }

  printStats(`${iterations.toLocaleString()} batches (100 writes each)`, samples)
  console.log(`    per signal: ${c.dim}${fmt(stats(samples).avg / 100)}${c.reset}`)
}

// =============================================================================
// SECTION 2: SHAREDBUFFER RAW OPERATIONS
// =============================================================================

header('2. SHAREDBUFFER RAW OPERATIONS')

{
  subheader('DataView.setFloat32')
  const sab = new SharedArrayBuffer(1024 * 1024)
  const view = new DataView(sab)
  const iterations = 10_000_000

  const t0 = Bun.nanoseconds()
  for (let i = 0; i < iterations; i++) {
    view.setFloat32((i % 1000) * 4, i, true)
  }
  const elapsed = Bun.nanoseconds() - t0

  const perOp = elapsed / iterations
  console.log(`  ${c.cyan}${iterations.toLocaleString()} writes${c.reset}`)
  console.log(`    per write: ${c.yellow}${fmt(perOp)}${c.reset}`)
  console.log(`    throughput: ${c.green}${fmtOps(1_000_000_000 / perOp)}/sec${c.reset}`)
}

{
  subheader('Atomics.store + Atomics.notify')
  const sab = new SharedArrayBuffer(4)
  const int32 = new Int32Array(sab)
  const iterations = 100_000
  const samples: number[] = []

  for (let i = 0; i < iterations; i++) {
    const t0 = Bun.nanoseconds()
    Atomics.store(int32, 0, i)
    Atomics.notify(int32, 0, 1)
    samples.push(Bun.nanoseconds() - t0)
  }

  printStats(`${iterations.toLocaleString()} notify cycles`, samples)
}

// =============================================================================
// SECTION 3: MOUNT APP (real API)
// =============================================================================

header('3. APP MOUNTING')

// Create signals for reactive testing - these live outside mount
const counter = signal(0)
const boxWidth = signal(50)
const boxColor = signal(0xFF00FF00)
const textContent = signal('Hello SparkTUI')
const listItems = Array.from({ length: 100 }, (_, i) => signal(`Item ${i}`))

console.log(`  ${c.dim}Mounting app with ~200 components...${c.reset}`)

const mountStart = Bun.nanoseconds()
let handle: MountHandle

try {
  handle = mountSync(() => {
    box({
      width: '100%',
      height: '100%',
      flexDirection: 'column',
      children: () => {
        // Header with reactive width and color
        box({
          width: boxWidth,
          height: 3,
          border: 1,
          bg: boxColor,
          justifyContent: 'center',
          alignItems: 'center',
          children: () => {
            text({ content: textContent })
          }
        })

        // Counter display
        box({
          height: 1,
          children: () => {
            text({ content: derived(() => `Counter: ${counter.value}`) })
          }
        })

        // List of 100 items (200 components: 100 boxes + 100 texts)
        box({
          flexDirection: 'column',
          flexGrow: 1,
          children: () => {
            for (let i = 0; i < 100; i++) {
              box({
                height: 1,
                children: () => {
                  text({ content: listItems[i] })
                }
              })
            }
          }
        })
      }
    })
  }, { mode: 'inline' })
} catch (e) {
  console.error(`${c.red}Mount failed:${c.reset}`, e)
  process.exit(1)
}

const mountElapsed = Bun.nanoseconds() - mountStart

console.log(`  ${c.cyan}mount() time${c.reset}: ${c.yellow}${fmt(mountElapsed)}${c.reset}`)
console.log(`  ${c.cyan}Node count${c.reset}: ${c.yellow}${getNodeCount(handle.buffer)}${c.reset}`)
console.log(`  ${c.green}✓${c.reset} Engine initialized and running`)

// Let engine settle
await Bun.sleep(200)

// =============================================================================
// HELPER: Wait for render (spin, no sleep!)
// =============================================================================

function waitForRender(prevCount: number, maxSpins = 10_000_000): boolean {
  for (let i = 0; i < maxSpins; i++) {
    if (getRenderCount(handle.buffer) !== prevCount) return true
  }
  return false
}

// =============================================================================
// SECTION 4: RUST PIPELINE TIMING
// =============================================================================

header('4. RUST PIPELINE TIMING (from SharedBuffer)')

{
  subheader('Per-Frame Breakdown (50 samples)')

  const layoutSamples: number[] = []
  const fbSamples: number[] = []
  const renderSamples: number[] = []
  const totalSamples: number[] = []

  for (let i = 0; i < 50; i++) {
    const prevCount = getRenderCount(handle.buffer)
    counter.value = i

    if (waitForRender(prevCount)) {
      const t = getTimingStats(handle.buffer)
      layoutSamples.push(t.layoutUs)
      fbSamples.push(t.framebufferUs)
      renderSamples.push(t.renderUs)
      totalSamples.push(t.totalFrameUs)
    }
  }

  if (layoutSamples.length > 0) {
    printStats('Layout (Taffy)', layoutSamples, 'us')
    printStats('Framebuffer (2D grid)', fbSamples, 'us')
    printStats('Render (diff → ANSI)', renderSamples, 'us')
    printStats('Total Frame', totalSamples, 'us')

    const avgTotal = stats(totalSamples).avg
    const fps = avgTotal > 0 ? 1_000_000 / avgTotal : 0
    console.log(`  ${c.cyan}Achievable FPS${c.reset}: ${c.green}${fmtFps(fps)}${c.reset}`)
  } else {
    console.log(`  ${c.red}No renders detected! Check engine.${c.reset}`)
  }
}

// =============================================================================
// SECTION 5: TRUE END-TO-END TIMING
// =============================================================================

header('5. TRUE END-TO-END (signal → screen)')

{
  subheader('Single Update E2E (200 samples)')
  const samples: number[] = []

  for (let i = 0; i < 200; i++) {
    const prevCount = getRenderCount(handle.buffer)

    const t0 = Bun.nanoseconds()
    counter.value = 10000 + i
    while (getRenderCount(handle.buffer) === prevCount) { /* spin */ }
    const t1 = Bun.nanoseconds()

    samples.push(t1 - t0)
  }

  printStats('E2E latency', samples)
  const fps = 1_000_000_000 / stats(samples).avg
  console.log(`  ${c.cyan}E2E FPS${c.reset}: ${c.green}${fmtFps(fps)}${c.reset}`)
}

{
  subheader('Layout-Triggering E2E (width change)')
  const samples: number[] = []

  for (let i = 0; i < 100; i++) {
    const prevCount = getRenderCount(handle.buffer)

    const t0 = Bun.nanoseconds()
    boxWidth.value = 30 + (i % 40)
    while (getRenderCount(handle.buffer) === prevCount) {}
    const t1 = Bun.nanoseconds()

    samples.push(t1 - t0)
  }

  printStats('Layout E2E', samples)
  console.log(`  ${c.cyan}Layout FPS${c.reset}: ${c.green}${fmtFps(1_000_000_000 / stats(samples).avg)}${c.reset}`)
}

{
  subheader('Visual-Only E2E (color change, should skip layout)')
  const samples: number[] = []

  for (let i = 0; i < 100; i++) {
    const prevCount = getRenderCount(handle.buffer)

    const t0 = Bun.nanoseconds()
    boxColor.value = 0xFF000000 | ((i * 123456) & 0xFFFFFF)
    while (getRenderCount(handle.buffer) === prevCount) {}
    const t1 = Bun.nanoseconds()

    samples.push(t1 - t0)
  }

  printStats('Visual E2E', samples)
  console.log(`  ${c.cyan}Visual FPS${c.reset}: ${c.green}${fmtFps(1_000_000_000 / stats(samples).avg)}${c.reset}`)
}

// =============================================================================
// SECTION 6: SUSTAINED THROUGHPUT
// =============================================================================

header('6. SUSTAINED THROUGHPUT')

{
  subheader('Max Write Rate (1 second, fire-and-forget)')
  let writes = 0
  const duration = 1_000_000_000  // 1 second in ns

  const t0 = Bun.nanoseconds()
  while (Bun.nanoseconds() - t0 < duration) {
    counter.value = writes++
  }
  const elapsed = Bun.nanoseconds() - t0

  const rate = (writes / elapsed) * 1_000_000_000
  console.log(`  ${c.cyan}Total writes${c.reset}: ${c.yellow}${writes.toLocaleString()}${c.reset}`)
  console.log(`  ${c.cyan}Write rate${c.reset}: ${c.green}${fmtOps(rate)}/sec${c.reset}`)

  await Bun.sleep(100)  // Let Rust catch up
  const renderCount = getRenderCount(handle.buffer)
  console.log(`  ${c.cyan}Renders${c.reset}: ${c.yellow}${renderCount}${c.reset}`)
  console.log(`  ${c.dim}(Many writes coalesce into one render - that's GOOD)${c.reset}`)
}

{
  subheader('Sustained Render Rate (1 second, wait for each)')
  let renders = 0
  const samples: number[] = []

  const deadline = Bun.nanoseconds() + 1_000_000_000
  while (Bun.nanoseconds() < deadline) {
    const prevCount = getRenderCount(handle.buffer)
    const t0 = Bun.nanoseconds()

    counter.value = renders++
    while (getRenderCount(handle.buffer) === prevCount) {}

    samples.push(Bun.nanoseconds() - t0)
  }

  console.log(`  ${c.cyan}Total renders${c.reset}: ${c.yellow}${renders}${c.reset}`)
  console.log(`  ${c.cyan}Sustained FPS${c.reset}: ${c.green}${renders}${c.reset}`)
  console.log(`  ${c.cyan}Per-frame${c.reset}: avg ${c.yellow}${fmt(stats(samples).avg)}${c.reset}  p99 ${fmt(stats(samples).p99)}`)
}

// =============================================================================
// SECTION 7: BATCH & COALESCING
// =============================================================================

header('7. BATCH UPDATES & COALESCING')

{
  subheader('100 Item Batch → Single Render')
  const samples: number[] = []

  for (let iter = 0; iter < 50; iter++) {
    const prevCount = getRenderCount(handle.buffer)

    const t0 = Bun.nanoseconds()
    batch(() => {
      for (let i = 0; i < 100; i++) {
        listItems[i].value = `Item ${i} v${iter}`
      }
    })
    while (getRenderCount(handle.buffer) === prevCount) {}
    const t1 = Bun.nanoseconds()

    samples.push(t1 - t0)
  }

  printStats('Batch E2E (50 × 100 items)', samples)
  console.log(`  ${c.cyan}Per-item${c.reset}: ${c.dim}${fmt(stats(samples).avg / 100)}${c.reset}`)
}

{
  subheader('Coalescing: 1000 Writes → 1 Render')
  const prevCount = getRenderCount(handle.buffer)

  const t0 = Bun.nanoseconds()
  for (let i = 0; i < 1000; i++) {
    counter.value = i
  }
  const writesDone = Bun.nanoseconds()

  while (getRenderCount(handle.buffer) === prevCount) {}
  const renderDone = Bun.nanoseconds()

  console.log(`  ${c.cyan}1000 writes${c.reset}: ${c.yellow}${fmt(writesDone - t0)}${c.reset}`)
  console.log(`  ${c.cyan}Render wait${c.reset}: ${c.yellow}${fmt(renderDone - writesDone)}${c.reset}`)
  console.log(`  ${c.cyan}Total E2E${c.reset}: ${c.yellow}${fmt(renderDone - t0)}${c.reset}`)
}

// =============================================================================
// SECTION 8: WAKE LATENCY
// =============================================================================

header('8. WAKE LATENCY')

{
  subheader('Cold Wake (after 10ms idle)')
  const samples: number[] = []

  for (let i = 0; i < 50; i++) {
    await Bun.sleep(10)  // Ensure WakeWatcher is in sleep mode

    const prevCount = getRenderCount(handle.buffer)
    const t0 = Bun.nanoseconds()
    counter.value = 80000 + i
    while (getRenderCount(handle.buffer) === prevCount) {}
    samples.push(Bun.nanoseconds() - t0)
  }

  printStats('Cold wake (50 samples)', samples)
}

{
  subheader('Hot Wake (back-to-back)')
  const samples: number[] = []

  for (let i = 0; i < 100; i++) {
    const prevCount = getRenderCount(handle.buffer)
    const t0 = Bun.nanoseconds()
    counter.value = 90000 + i
    while (getRenderCount(handle.buffer) === prevCount) {}
    samples.push(Bun.nanoseconds() - t0)
  }

  printStats('Hot wake (100 samples)', samples)
}

// =============================================================================
// SECTION 9: OPERATION TYPE COMPARISON
// =============================================================================

header('9. OPERATION TYPE COMPARISON')

const operations = [
  { name: 'Number signal (counter++)', fn: () => { counter.value++ } },
  { name: 'Width change (layout trigger)', fn: () => { boxWidth.value = 30 + (Math.random() * 40 | 0) } },
  { name: 'Color change (visual only)', fn: () => { boxColor.value = 0xFF000000 | (Math.random() * 0xFFFFFF | 0) } },
  { name: 'Text content change', fn: () => { textContent.value = `Text ${Math.random().toString(36).slice(2, 8)}` } },
]

for (const op of operations) {
  const e2eSamples: number[] = []
  const layoutSamples: number[] = []
  const fbSamples: number[] = []
  const renderSamples: number[] = []

  for (let i = 0; i < 50; i++) {
    const prevCount = getRenderCount(handle.buffer)
    const t0 = Bun.nanoseconds()

    op.fn()

    while (getRenderCount(handle.buffer) === prevCount) {}
    e2eSamples.push(Bun.nanoseconds() - t0)

    const t = getTimingStats(handle.buffer)
    layoutSamples.push(t.layoutUs)
    fbSamples.push(t.framebufferUs)
    renderSamples.push(t.renderUs)
  }

  const e2e = stats(e2eSamples)
  const layout = stats(layoutSamples)
  const fb = stats(fbSamples)
  const render = stats(renderSamples)

  console.log(`  ${c.bold}${op.name}${c.reset}`)
  console.log(`    E2E: ${c.yellow}${fmt(e2e.avg)}${c.reset}  |  Layout: ${layout.avg.toFixed(0)}μs  FB: ${fb.avg.toFixed(0)}μs  Render: ${render.avg.toFixed(0)}μs`)
}

// =============================================================================
// CLEANUP & SUMMARY
// =============================================================================

console.log()
console.log(`${c.bold}${c.magenta}${'═'.repeat(60)}${c.reset}`)
console.log(`${c.bold}${c.magenta}  BENCHMARK COMPLETE${c.reset}`)
console.log(`${c.bold}${c.magenta}${'═'.repeat(60)}${c.reset}`)
console.log()
console.log(`  ${c.green}✓${c.reset} All measurements using real public APIs`)
console.log(`  ${c.green}✓${c.reset} No FFI bypassing, no internal access`)
console.log(`  ${c.green}✓${c.reset} True E2E: signal.value → terminal screen`)
console.log()

handle.unmount()
process.exit(0)
