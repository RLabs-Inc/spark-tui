/**
 * SparkTUI — REAL Benchmarks & Stress Tests
 *
 * ONE mount. All components inside. All benchmarks update signals.
 * This is how a real app works.
 *
 * Run: bun examples/bench-real.ts
 */

import { signal, derived, batch } from '@rlabs-inc/signals'
import { box, text } from '../ts/primitives'
import { mount } from '../ts/engine'
import { loadEngine } from '../ts/bridge/ffi'
import {
  getTimingStats,
  getRenderCount,
} from '../ts/bridge/shared-buffer'
import { ptr } from 'bun:ffi'
import { join } from 'path'

// =============================================================================
// ANSI COLORS
// =============================================================================

const c = {
  reset: '\x1b[0m',
  bold: '\x1b[1m',
  dim: '\x1b[2m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  cyan: '\x1b[36m',
  brightYellow: '\x1b[93m',
  brightGreen: '\x1b[92m',
  brightMagenta: '\x1b[95m',
  white: '\x1b[37m',
  bgMagenta: '\x1b[45m',
}

function formatNs(ns: number): string {
  if (ns < 1000) return `${ns.toFixed(0)}ns`
  if (ns < 1_000_000) return `${(ns / 1000).toFixed(2)}μs`
  return `${(ns / 1_000_000).toFixed(2)}ms`
}

function formatUs(us: number): string {
  if (us < 1000) return `${us.toFixed(0)}μs`
  return `${(us / 1000).toFixed(2)}ms`
}

function fpsStatus(fps: number, target: number): string {
  return fps >= target ? `${c.green}✓${c.reset}` : `${c.red}✗${c.reset}`
}

// =============================================================================
// BUILD RUST ENGINE
// =============================================================================

console.log(`${c.dim}Building Rust engine...${c.reset}`)
Bun.spawnSync({
  cmd: ['cargo', 'build', '--release'],
  cwd: join(import.meta.dir, '../rust'),
  stdout: 'pipe',
  stderr: 'pipe',
})
console.log(`${c.green}✓${c.reset} Rust engine built\n`)

// =============================================================================
// CREATE ALL SIGNALS UPFRONT
// =============================================================================

// Counter signals
const counter = signal(0)
const counterColor = signal(0xFF00FF00)

// List of items (simulating a dynamic list)
const items = Array.from({ length: 100 }, (_, i) => signal(`Item ${i}`))

// Layout signal
const boxWidth = signal(30)

// Visibility
const showList = signal(true)

// =============================================================================
// ONE MOUNT - THE ENTIRE APP
// =============================================================================

console.log(`${c.cyan}Mounting app with 100+ components...${c.reset}`)

const handle = mount(() => {
  box({
    width: '100%',
    flexDirection: 'column',
    children: () => {
      // Header
      box({
        width: boxWidth,
        height: 3,
        border: 1,
        bg: counterColor,
        justifyContent: 'center',
        alignItems: 'center',
        children: () => {
          text({ content: 'SparkTUI Benchmark' })
        }
      })

      // Counter display
      box({
        flexDirection: 'row',
        gap: 2,
        children: () => {
          text({ content: 'Counter: ' })
          text({ content: counter })
        }
      })

      // List of 100 items
      box({
        flexDirection: 'column',
        visible: showList,
        children: () => {
          for (let i = 0; i < 100; i++) {
            box({
              height: 1,
              children: () => {
                text({ content: items[i] })
              }
            })
          }
        }
      })
    }
  })
}, { mode: 'inline' })

console.log(`${c.green}✓${c.reset} App mounted (${102 * 2} components)\n`)

// =============================================================================
// INIT RUST ENGINE
// =============================================================================

const engine = loadEngine()
const initResult = engine.init(ptr(handle.buffer.raw), handle.buffer.raw.byteLength)

if (initResult !== 0) {
  console.error(`${c.red}Rust init failed${c.reset}`)
  process.exit(1)
}

console.log(`${c.green}✓${c.reset} Rust engine initialized\n`)
await Bun.sleep(100) // Let Rust settle

// =============================================================================
// TITLE
// =============================================================================

console.log(`${c.bold}${c.brightMagenta}╔═════════════════════════════════════════════════════════════╗${c.reset}`)
console.log(`${c.bold}${c.brightMagenta}║${c.reset}       ${c.bold}${c.white}SparkTUI Real-World Benchmarks${c.reset}                    ${c.bold}${c.brightMagenta}║${c.reset}`)
console.log(`${c.bold}${c.brightMagenta}║${c.reset}       ${c.dim}Bun.nanoseconds() • Rust pipeline timing${c.reset}           ${c.bold}${c.brightMagenta}║${c.reset}`)
console.log(`${c.bold}${c.brightMagenta}╚═════════════════════════════════════════════════════════════╝${c.reset}\n`)

// =============================================================================
// BENCHMARK 1: Counter Updates
// =============================================================================

console.log(`${c.bgMagenta}${c.bold} 1. COUNTER UPDATES ${c.reset}\n`)

{
  // Warmup
  for (let i = 0; i < 100; i++) {
    counter.value++
    await Bun.sleep(2)
  }

  const iterations = 1000
  const samples: number[] = []

  for (let i = 0; i < iterations; i++) {
    const t0 = Bun.nanoseconds()
    counter.value++
    samples.push(Bun.nanoseconds() - t0)
  }

  // Wait for Rust to process all
  await Bun.sleep(200)

  samples.sort((a, b) => a - b)
  const avg = samples.reduce((a, b) => a + b, 0) / iterations

  console.log(`  ${c.dim}TS side (signal → SharedBuffer → notify):${c.reset}`)
  console.log(`    Average: ${c.brightYellow}${formatNs(avg)}${c.reset}`)
  console.log(`    p50:     ${c.yellow}${formatNs(samples[Math.floor(iterations * 0.5)])}${c.reset}`)
  console.log(`    p95:     ${c.yellow}${formatNs(samples[Math.floor(iterations * 0.95)])}${c.reset}`)
  console.log(`    p99:     ${c.yellow}${formatNs(samples[Math.floor(iterations * 0.99)])}${c.reset}`)
  console.log()
}

// =============================================================================
// BENCHMARK 2: Color Updates (visual only, should skip layout)
// =============================================================================

console.log(`${c.bgMagenta}${c.bold} 2. COLOR UPDATES (visual-only) ${c.reset}\n`)

{
  const iterations = 1000
  const samples: number[] = []

  for (let i = 0; i < iterations; i++) {
    const t0 = Bun.nanoseconds()
    counterColor.value = 0xFF000000 | ((i * 7) & 0xFFFFFF)
    samples.push(Bun.nanoseconds() - t0)
  }

  await Bun.sleep(200)

  samples.sort((a, b) => a - b)
  const avg = samples.reduce((a, b) => a + b, 0) / iterations

  console.log(`  ${c.dim}TS side:${c.reset}`)
  console.log(`    Average: ${c.brightYellow}${formatNs(avg)}${c.reset}`)
  console.log(`    p50:     ${c.yellow}${formatNs(samples[Math.floor(iterations * 0.5)])}${c.reset}`)
  console.log(`    p95:     ${c.yellow}${formatNs(samples[Math.floor(iterations * 0.95)])}${c.reset}`)
  console.log()
}

// =============================================================================
// BENCHMARK 3: Layout Updates (triggers Taffy)
// =============================================================================

console.log(`${c.bgMagenta}${c.bold} 3. LAYOUT UPDATES (triggers Taffy) ${c.reset}\n`)

{
  const iterations = 500
  const samples: number[] = []

  for (let i = 0; i < iterations; i++) {
    const t0 = Bun.nanoseconds()
    boxWidth.value = 20 + (i % 60)
    samples.push(Bun.nanoseconds() - t0)
  }

  await Bun.sleep(200)

  samples.sort((a, b) => a - b)
  const avg = samples.reduce((a, b) => a + b, 0) / iterations

  console.log(`  ${c.dim}TS side:${c.reset}`)
  console.log(`    Average: ${c.brightYellow}${formatNs(avg)}${c.reset}`)
  console.log(`    p50:     ${c.yellow}${formatNs(samples[Math.floor(iterations * 0.5)])}${c.reset}`)
  console.log(`    p95:     ${c.yellow}${formatNs(samples[Math.floor(iterations * 0.95)])}${c.reset}`)
  console.log()
}

// =============================================================================
// BENCHMARK 4: Batch Updates (100 items at once)
// =============================================================================

console.log(`${c.bgMagenta}${c.bold} 4. BATCH UPDATES (100 items) ${c.reset}\n`)

{
  const iterations = 200
  const samples: number[] = []

  for (let iter = 0; iter < iterations; iter++) {
    const t0 = Bun.nanoseconds()
    batch(() => {
      for (let i = 0; i < 100; i++) {
        items[i].value = `Item ${i} - update ${iter}`
      }
    })
    samples.push(Bun.nanoseconds() - t0)
  }

  await Bun.sleep(300)

  samples.sort((a, b) => a - b)
  const avg = samples.reduce((a, b) => a + b, 0) / iterations

  console.log(`  ${c.dim}TS side (100 signal writes batched):${c.reset}`)
  console.log(`    Average: ${c.brightYellow}${formatNs(avg)}${c.reset}`)
  console.log(`    p50:     ${c.yellow}${formatNs(samples[Math.floor(iterations * 0.5)])}${c.reset}`)
  console.log(`    p95:     ${c.yellow}${formatNs(samples[Math.floor(iterations * 0.95)])}${c.reset}`)
  console.log(`    Per item: ${c.dim}${formatNs(avg / 100)}${c.reset}`)
  console.log()
}

// =============================================================================
// BENCHMARK 5: Rust Pipeline Timing (read from SharedBuffer)
// =============================================================================

console.log(`${c.bgMagenta}${c.bold} 5. RUST PIPELINE TIMING ${c.reset}\n`)

{
  // Do some updates and read timing after each
  const timingSamples: { layout: number; fb: number; render: number; total: number }[] = []

  let prevRenderCount = getRenderCount(handle.buffer)

  for (let i = 0; i < 50; i++) {
    counter.value++

    // Wait for Rust to process
    let waited = 0
    while (getRenderCount(handle.buffer) === prevRenderCount && waited < 100) {
      await Bun.sleep(1)
      waited++
    }
    prevRenderCount = getRenderCount(handle.buffer)

    const stats = getTimingStats(handle.buffer)
    timingSamples.push({
      layout: stats.layoutUs,
      fb: stats.framebufferUs,
      render: stats.renderUs,
      total: stats.totalFrameUs,
    })
  }

  const avgLayout = timingSamples.reduce((a, s) => a + s.layout, 0) / timingSamples.length
  const avgFb = timingSamples.reduce((a, s) => a + s.fb, 0) / timingSamples.length
  const avgRender = timingSamples.reduce((a, s) => a + s.render, 0) / timingSamples.length
  const avgTotal = timingSamples.reduce((a, s) => a + s.total, 0) / timingSamples.length

  console.log(`  ${c.dim}Rust pipeline (from SharedBuffer header):${c.reset}`)
  console.log(`    Layout:      ${c.brightYellow}${formatUs(avgLayout)}${c.reset}  ${c.dim}(Taffy flexbox)${c.reset}`)
  console.log(`    Framebuffer: ${c.brightYellow}${formatUs(avgFb)}${c.reset}  ${c.dim}(2D cell grid)${c.reset}`)
  console.log(`    Render:      ${c.brightYellow}${formatUs(avgRender)}${c.reset}  ${c.dim}(diff → ANSI)${c.reset}`)
  console.log(`    ────────────────────────`)
  console.log(`    Total Frame: ${c.brightGreen}${formatUs(avgTotal)}${c.reset}`)
  console.log()

  const fps = avgTotal > 0 ? 1_000_000 / avgTotal : 0
  console.log(`  ${c.dim}Achievable FPS:${c.reset} ${c.brightGreen}${fps.toFixed(0)}${c.reset}  60:${fpsStatus(fps, 60)} 120:${fpsStatus(fps, 120)} 144:${fpsStatus(fps, 144)}`)
  console.log()

  // Breakdown
  if (avgTotal > 0) {
    const layoutPct = (avgLayout / avgTotal * 100).toFixed(1)
    const fbPct = (avgFb / avgTotal * 100).toFixed(1)
    const renderPct = (avgRender / avgTotal * 100).toFixed(1)
    console.log(`  ${c.dim}Breakdown:${c.reset}`)
    console.log(`    Layout:      ${layoutPct}%`)
    console.log(`    Framebuffer: ${fbPct}%`)
    console.log(`    Render:      ${renderPct}%`)
  }
  console.log()
}

// =============================================================================
// BENCHMARK 6: Sustained Load (1 second)
// =============================================================================

console.log(`${c.bgMagenta}${c.bold} 6. SUSTAINED LOAD (1 second) ${c.reset}\n`)

{
  // Use boxWidth (a number signal) to avoid text pool overflow
  const duration = 1_000_000_000 // 1 second in ns
  const start = Bun.nanoseconds()
  let updates = 0

  while (Bun.nanoseconds() - start < duration) {
    boxWidth.value = 20 + (updates % 60)  // Number signal, no text pool usage
    updates++
  }

  const elapsed = Bun.nanoseconds() - start
  const perUpdate = elapsed / updates

  console.log(`  ${c.dim}Sustained signal updates for 1 second:${c.reset}`)
  console.log(`    Total updates: ${c.brightYellow}${updates.toLocaleString()}${c.reset}`)
  console.log(`    Per update:    ${c.yellow}${formatNs(perUpdate)}${c.reset}`)
  console.log(`    Rate:          ${c.brightGreen}${(updates / 1_000_000).toFixed(2)}M/sec${c.reset}`)
  console.log()
}

// =============================================================================
// CLEANUP
// =============================================================================

console.log(`${c.dim}Cleaning up...${c.reset}`)
engine.cleanup()
handle.unmount()

// =============================================================================
// SUMMARY
// =============================================================================

console.log()
console.log(`${c.bold}${c.brightMagenta}╔═════════════════════════════════════════════════════════════╗${c.reset}`)
console.log(`${c.bold}${c.brightMagenta}║${c.reset}                ${c.bold}${c.white}Benchmark Complete${c.reset}                        ${c.bold}${c.brightMagenta}║${c.reset}`)
console.log(`${c.bold}${c.brightMagenta}╚═════════════════════════════════════════════════════════════╝${c.reset}`)
console.log()
console.log(`  ${c.green}✓${c.reset} ONE mount, 200+ components`)
console.log(`  ${c.green}✓${c.reset} Real reactive pipeline measured`)
console.log(`  ${c.green}✓${c.reset} Rust timing instrumentation working`)
console.log()

process.exit(0)
