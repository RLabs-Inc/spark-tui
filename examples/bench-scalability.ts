/**
 * SparkTUI Scalability Benchmark
 *
 * Tests the reactive pipeline at different scales: 10, 100, 1K, 10K, 100K nodes.
 * Runs each size as a subprocess for clean Rust engine state.
 *
 * Measures each stage of the reactive flow:
 *   Signal Write (TS) → Repeater Fire (TS) → Buffer Write (TS)
 *     → Atomics.notify (TS→Rust) → Layout (Rust/Taffy)
 *       → Framebuffer (Rust) → Render (Rust)
 *
 * Run: bun examples/bench-scalability.ts
 */

import { spawn } from 'child_process'
import { signal, batch } from '@rlabs-inc/signals'
import { box, text } from '../ts/primitives'
import {
  getTimingStats,
  getTextPoolWritePtr,
  getRenderCount,
} from '../ts/bridge/shared-buffer'
import { getBuffer } from '../ts/bridge'
import { mountSync } from '../ts/engine'

// =============================================================================
// FORMATTING
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
  white: '\x1b[37m',
  bgBlue: '\x1b[44m',
  bgMagenta: '\x1b[45m',
  bgCyan: '\x1b[46m',
}

function fmt(ns: number): string {
  if (ns < 1000) return `${ns.toFixed(0)}ns`
  if (ns < 1_000_000) return `${(ns / 1000).toFixed(2)}μs`
  if (ns < 1_000_000_000) return `${(ns / 1_000_000).toFixed(2)}ms`
  return `${(ns / 1_000_000_000).toFixed(2)}s`
}

function fmtUs(us: number): string {
  if (us < 1000) return `${us.toFixed(0)}μs`
  return `${(us / 1000).toFixed(2)}ms`
}

function fmtBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes}B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)}KB`
  return `${(bytes / 1024 / 1024).toFixed(1)}MB`
}

function fmtFps(fps: number): string {
  if (fps >= 1_000_000) return `${(fps / 1_000_000).toFixed(1)}M`
  if (fps >= 1000) return `${(fps / 1000).toFixed(1)}K`
  return fps.toFixed(0)
}

function header(title: string) {
  console.log()
  console.log(`${c.bgMagenta}${c.bold}${c.white} ${title} ${c.reset}`)
  console.log()
}

// =============================================================================
// STATISTICS
// =============================================================================

interface Stats {
  avg: number
  min: number
  max: number
  p50: number
  p95: number
  p99: number
}

function computeStats(samples: number[]): Stats {
  if (samples.length === 0) return { avg: 0, min: 0, max: 0, p50: 0, p95: 0, p99: 0 }
  const sorted = [...samples].sort((a, b) => a - b)
  const sum = samples.reduce((a, b) => a + b, 0)
  return {
    avg: sum / samples.length,
    min: sorted[0],
    max: sorted[sorted.length - 1],
    p50: sorted[Math.floor(sorted.length * 0.5)],
    p95: sorted[Math.floor(sorted.length * 0.95)],
    p99: sorted[Math.floor(sorted.length * 0.99)],
  }
}

// =============================================================================
// BENCHMARK RESULT TYPE
// =============================================================================

interface BenchResult {
  nodeCount: number
  mountTimeMs: number
  e2eNs: Stats         // True end-to-end: signal change → render complete
  tsSignalNs: Stats
  tsBufferNs: Stats
  tsNotifyNs: Stats
  tsTotalNs: Stats
  layoutUs: Stats
  framebufferUs: Stats
  renderUs: Stats
  totalFrameUs: Stats
  theoreticalFps: number
  memoryMB: number
  textPoolBytes: number
}

// =============================================================================
// SUBPROCESS BENCHMARK (runs when BENCH_SIZE env is set)
// =============================================================================

async function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms))
}

function getMemoryMB(): number {
  return process.memoryUsage().rss / 1024 / 1024
}

async function runSingleBenchmark(nodeCount: number, iterations: number, warmup: number): Promise<BenchResult> {
  const memBefore = getMemoryMB()

  // Create tree of N text nodes
  const signals: Array<ReturnType<typeof signal<string>>> = []

  // Calculate buffer size needed (add some overhead for container boxes)
  const maxNodes = Math.max(nodeCount + 100, 10_000)
  // Text pool: ~50 bytes per node average
  const textPoolSize = Math.max(nodeCount * 100, 10 * 1024 * 1024)

  const mountStart = performance.now()
  const handle = mountSync(() => {
    box({ width: '100%', height: '100%' }, () => {
      for (let i = 0; i < nodeCount; i++) {
        const s = signal(`Node ${i}`)
        signals.push(s)
        text({ content: s })
      }
    })
  }, { mode: 'inline', maxNodes, textPoolSize })
  const mountTime = performance.now() - mountStart

  const buf = getBuffer()

  // Wait for initial render
  await sleep(100)

  // Warmup
  for (let i = 0; i < warmup; i++) {
    batch(() => {
      for (const s of signals) {
        s.value = `Warmup ${i}`
      }
    })
    await sleep(10)
  }

  // Collect samples
  const samples = {
    e2eNs: [] as number[],  // True end-to-end time
    tsSignal: [] as number[],
    tsBuffer: [] as number[],
    tsNotify: [] as number[],
    tsTotal: [] as number[],
    layout: [] as number[],
    framebuffer: [] as number[],
    render: [] as number[],
    totalFrame: [] as number[],
  }

  for (let i = 0; i < iterations; i++) {
    // Record state BEFORE update
    const renderCountBefore = getRenderCount(buf)
    const startTime = performance.now()

    // Update all signals
    batch(() => {
      for (let j = 0; j < signals.length; j++) {
        signals[j].value = `Iter ${i} Node ${j}`
      }
    })

    // Spin until render completes (render count increases)
    let spins = 0
    const maxSpins = 100_000
    while (getRenderCount(buf) <= renderCountBefore && spins < maxSpins) {
      spins++
      // Yield to allow Rust thread to run
      if (spins % 1000 === 0) await sleep(0)
    }

    const endTime = performance.now()
    const e2eNs = (endTime - startTime) * 1_000_000 // ms to ns

    samples.e2eNs.push(e2eNs)

    // Also collect the Rust-reported timing
    const stats = getTimingStats(buf)
    samples.tsSignal.push(stats.tsSignalNs)
    samples.tsBuffer.push(stats.tsBufferWriteNs)
    samples.tsNotify.push(stats.tsNotifyNs)
    samples.tsTotal.push(stats.tsTotalNs)
    samples.layout.push(stats.layoutUs)
    samples.framebuffer.push(stats.framebufferUs)
    samples.render.push(stats.renderUs)
    samples.totalFrame.push(stats.totalFrameUs)

    // Small delay between iterations to let system settle
    await sleep(1)
  }

  const memAfter = getMemoryMB()
  const textPoolBytes = getTextPoolWritePtr(buf)

  // Calculate FPS from true E2E time
  const avgE2eNs = samples.e2eNs.reduce((a, b) => a + b, 0) / samples.e2eNs.length
  const avgE2eUs = avgE2eNs / 1000

  handle.unmount()

  return {
    nodeCount,
    mountTimeMs: mountTime,
    e2eNs: computeStats(samples.e2eNs),
    tsSignalNs: computeStats(samples.tsSignal),
    tsBufferNs: computeStats(samples.tsBuffer),
    tsNotifyNs: computeStats(samples.tsNotify),
    tsTotalNs: computeStats(samples.tsTotal),
    layoutUs: computeStats(samples.layout),
    framebufferUs: computeStats(samples.framebuffer),
    renderUs: computeStats(samples.render),
    totalFrameUs: computeStats(samples.totalFrame),
    theoreticalFps: avgE2eUs > 0 ? 1_000_000 / avgE2eUs : 0, // Use E2E time for real FPS
    memoryMB: memAfter - memBefore,
    textPoolBytes,
  }
}

// =============================================================================
// RESULT PRINTING
// =============================================================================

function printResult(r: BenchResult) {
  const divider = '─'.repeat(60)

  console.log(`${c.bold}${c.white}${r.nodeCount.toLocaleString()} Nodes${c.reset}`)
  console.log(divider)

  // Key metrics at top
  console.log(`  ${c.dim}Mount:${c.reset} ${c.yellow}${r.mountTimeMs.toFixed(1)}ms${c.reset}`)
  console.log(`  ${c.bold}${c.green}E2E (signal → render):${c.reset} ${c.bold}${c.yellow}${fmt(r.e2eNs.avg)}${c.reset}  ${c.dim}p99:${c.reset} ${fmt(r.e2eNs.p99)}`)

  const fps = r.theoreticalFps
  const fpsColor = fps >= 60 ? c.green : fps >= 30 ? c.yellow : c.red
  console.log(`  ${c.dim}Theoretical FPS:${c.reset} ${fpsColor}${fmtFps(fps)}${c.reset}`)
  console.log()

  console.log(`  ${c.cyan}Stage Breakdown (avg / p99):${c.reset}`)
  console.log()

  const stages = [
    { name: 'TS Signal', avg: r.tsSignalNs.avg, p99: r.tsSignalNs.p99 },
    { name: 'TS Buffer', avg: r.tsBufferNs.avg, p99: r.tsBufferNs.p99 },
    { name: 'TS Notify', avg: r.tsNotifyNs.avg, p99: r.tsNotifyNs.p99 },
    { name: 'TS Total', avg: r.tsTotalNs.avg, p99: r.tsTotalNs.p99 },
    { name: '---', avg: 0, p99: 0 },
    { name: 'Rust Layout', avg: r.layoutUs.avg * 1000, p99: r.layoutUs.p99 * 1000 },
    { name: 'Rust Framebuf', avg: r.framebufferUs.avg * 1000, p99: r.framebufferUs.p99 * 1000 },
    { name: 'Rust Render', avg: r.renderUs.avg * 1000, p99: r.renderUs.p99 * 1000 },
    { name: 'Rust Total', avg: r.totalFrameUs.avg * 1000, p99: r.totalFrameUs.p99 * 1000 },
  ]

  for (const s of stages) {
    if (s.name === '---') {
      console.log(`    ${c.dim}${'─'.repeat(40)}${c.reset}`)
      continue
    }
    const avgStr = fmt(s.avg).padStart(10)
    const p99Str = fmt(s.p99).padStart(10)
    console.log(`    ${s.name.padEnd(15)} ${c.yellow}${avgStr}${c.reset}  ${c.dim}p99:${c.reset} ${p99Str}`)
  }

  console.log()
  console.log(`  ${c.dim}Memory delta:${c.reset} ${r.memoryMB.toFixed(1)}MB`)
  console.log()
}

function printSummaryTable(results: BenchResult[]) {
  header('SUMMARY TABLE')

  console.log(`  ${c.bold}Nodes      Mount      E2E (avg)    E2E (p99)    FPS${c.reset}`)
  console.log(`  ${c.dim}${'─'.repeat(55)}${c.reset}`)

  for (const r of results) {
    const nodes = r.nodeCount.toLocaleString().padStart(7)
    const mount = `${r.mountTimeMs.toFixed(0)}ms`.padStart(8)
    const e2eAvg = fmt(r.e2eNs.avg).padStart(12)
    const e2eP99 = fmt(r.e2eNs.p99).padStart(12)
    const fps = fmtFps(r.theoreticalFps).padStart(8)

    console.log(`  ${nodes}  ${mount}  ${e2eAvg}  ${e2eP99}  ${fps}`)
  }
  console.log()
}

// =============================================================================
// SUBPROCESS SPAWNER
// =============================================================================

function runBenchmarkSubprocess(nodeCount: number): Promise<BenchResult> {
  return new Promise((resolve, reject) => {
    const child = spawn('bun', ['run', import.meta.path, '--run-single', String(nodeCount)], {
      stdio: ['inherit', 'inherit', 'pipe'], // stdin, stdout inherit; stderr piped
      env: { ...process.env },
    })

    let stderr = ''
    child.stderr!.on('data', (data) => {
      stderr += data.toString()
    })

    child.on('close', (code) => {
      if (code !== 0) {
        reject(new Error(`Subprocess exited with code ${code}: ${stderr}`))
        return
      }

      // Find the BENCH_RESULT line in stderr
      const lines = stderr.split('\n')
      let jsonStr: string | undefined

      for (const line of lines) {
        if (line.startsWith('BENCH_RESULT:')) {
          jsonStr = line.slice('BENCH_RESULT:'.length)
          break
        }
      }

      if (!jsonStr) {
        reject(new Error(`No BENCH_RESULT found in stderr: ${stderr}`))
        return
      }

      try {
        const result = JSON.parse(jsonStr)
        resolve(result)
      } catch (e) {
        reject(new Error(`Failed to parse JSON: ${jsonStr}`))
      }
    })
  })
}

// =============================================================================
// MAIN
// =============================================================================

async function main() {
  // Check if running as subprocess
  if (process.argv.includes('--run-single')) {
    const nodeCount = parseInt(process.argv[process.argv.indexOf('--run-single') + 1])
    const iterations = nodeCount >= 10_000 ? 20 : 100
    const warmup = nodeCount >= 10_000 ? 3 : 10

    const result = await runSingleBenchmark(nodeCount, iterations, warmup)
    // Output to stderr to avoid mixing with terminal ANSI codes
    console.error('BENCH_RESULT:' + JSON.stringify(result))
    process.exit(0)
    return
  }

  // Main orchestrator
  console.log()
  console.log(`${c.bgCyan}${c.bold}${c.white} SparkTUI Scalability Benchmark ${c.reset}`)
  console.log()
  console.log(`${c.dim}Testing reactive pipeline at different scales...${c.reset}`)
  console.log(`${c.dim}Each size runs in a separate process for clean state.${c.reset}`)
  console.log()

  const sizes = [10, 100, 1_000, 10_000, 100_000]
  const results: BenchResult[] = []

  for (const size of sizes) {
    header(`BENCHMARKING ${size.toLocaleString()} NODES`)
    console.log(`${c.dim}Running in subprocess...${c.reset}`)

    try {
      const result = await runBenchmarkSubprocess(size)
      results.push(result)
      printResult(result)
    } catch (err) {
      console.log(`${c.red}ERROR: ${err}${c.reset}`)
      console.log()
      break
    }
  }

  if (results.length > 0) {
    printSummaryTable(results)
  }

  // Memory stability test (runs in this process)
  header('MEMORY STABILITY TEST (30 seconds)')

  try {
    console.log(`${c.dim}Running 1,000 nodes with continuous updates...${c.reset}`)
    console.log()

    const signals: Array<ReturnType<typeof signal<string>>> = []
    const memHandle = mountSync(() => {
      box({ width: '100%', height: '100%' }, () => {
        for (let i = 0; i < 1000; i++) {
          const s = signal(`Node ${i}`)
          signals.push(s)
          text({ content: s })
        }
      })
    }, { mode: 'inline' })

    const buf = getBuffer()
    const memSamples: number[] = []
    const startMem = getMemoryMB()
    const startTime = Date.now()

    let iteration = 0
    while (Date.now() - startTime < 30_000) {
      batch(() => {
        for (let i = 0; i < signals.length; i++) {
          signals[i].value = `Memory test ${iteration} node ${i}`
        }
      })
      iteration++

      if (iteration % 50 === 0) {
        memSamples.push(getMemoryMB())
        const elapsed = ((Date.now() - startTime) / 1000).toFixed(0)
        const mem = memSamples[memSamples.length - 1].toFixed(1)
        const pool = fmtBytes(getTextPoolWritePtr(buf))
        process.stdout.write(`\r  ${c.dim}${elapsed}s: ${iteration} updates, RSS: ${mem}MB, TextPool: ${pool}${c.reset}       `)
      }

      await sleep(10)
    }

    console.log()
    console.log()

    const endMem = getMemoryMB()
    const memGrowth = endMem - startMem

    console.log(`  ${c.cyan}Results:${c.reset}`)
    console.log(`    Total updates: ${iteration.toLocaleString()}`)
    console.log(`    Memory growth: ${memGrowth > 0 ? '+' : ''}${memGrowth.toFixed(1)}MB`)

    if (Math.abs(memGrowth) < 50) {
      console.log(`    ${c.green}✓ Memory stable${c.reset}`)
    } else if (memGrowth > 0) {
      console.log(`    ${c.red}⚠ Memory grew significantly${c.reset}`)
    }

    memHandle.unmount()
  } catch (err) {
    console.log(`${c.red}ERROR: ${err}${c.reset}`)
  }

  console.log()
  console.log(`${c.dim}Benchmark complete.${c.reset}`)
  process.exit(0)
}

main().catch(console.error)
