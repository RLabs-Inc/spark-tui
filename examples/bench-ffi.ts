/**
 * FFI Call Overhead Benchmark
 *
 * Measures the raw cost of calling Rust functions from TypeScript via Bun FFI.
 * This helps us decide if FFI-based wake notifications are viable.
 */

import { dlopen, FFIType } from 'bun:ffi'
import { join } from 'path'

const libPath = join(import.meta.dir, '../rust/target/release/libspark_tui_engine.dylib')

console.log('=== FFI Call Overhead Benchmark ===\n')
console.log(`Library: ${libPath}\n`)

// Load the library with various function signatures
const lib = dlopen(libPath, {
  // Pure no-op (absolute minimum FFI cost)
  spark_noop: {
    args: [],
    returns: FFIType.void,
  },
  // No-op with args (measures marshaling)
  spark_noop_args: {
    args: [FFIType.u32, FFIType.u32],
    returns: FFIType.u32,
  },
  // No-op that touches an atomic (realistic minimum work)
  spark_noop_atomic: {
    args: [],
    returns: FFIType.void,
  },
  // Returns u32 (no init required)
  spark_buffer_size: {
    args: [],
    returns: FFIType.u32,
  },
})

// ============================================================================
// Benchmark 1: Pure no-op FFI call
// ============================================================================

function benchmarkNoop() {
  console.log('--- Benchmark 1: Pure no-op FFI call ---\n')

  // Warmup
  for (let i = 0; i < 10000; i++) {
    lib.symbols.spark_noop()
  }

  const iterations = 10_000_000

  const start = Bun.nanoseconds()
  for (let i = 0; i < iterations; i++) {
    lib.symbols.spark_noop()
  }
  const elapsed = Bun.nanoseconds() - start

  const nsPerCall = elapsed / iterations
  const callsPerSecond = (iterations / elapsed) * 1e9

  console.log(`  Iterations: ${iterations.toLocaleString()}`)
  console.log(`  Total time: ${(elapsed / 1e6).toFixed(2)}ms`)
  console.log(`  Per call: ${nsPerCall.toFixed(1)}ns`)
  console.log(`  Throughput: ${(callsPerSecond / 1e6).toFixed(2)}M calls/sec`)
  console.log('')

  return nsPerCall
}

// ============================================================================
// Benchmark 2: No-op with arguments
// ============================================================================

function benchmarkNoopArgs() {
  console.log('--- Benchmark 2: No-op with u32 args + return ---\n')

  // Warmup
  for (let i = 0; i < 10000; i++) {
    lib.symbols.spark_noop_args(1, 2)
  }

  const iterations = 10_000_000
  let sum = 0

  const start = Bun.nanoseconds()
  for (let i = 0; i < iterations; i++) {
    sum += lib.symbols.spark_noop_args(i, i + 1)
  }
  const elapsed = Bun.nanoseconds() - start

  const nsPerCall = elapsed / iterations
  const callsPerSecond = (iterations / elapsed) * 1e9

  console.log(`  Iterations: ${iterations.toLocaleString()}`)
  console.log(`  Total time: ${(elapsed / 1e6).toFixed(2)}ms`)
  console.log(`  Per call: ${nsPerCall.toFixed(1)}ns`)
  console.log(`  Throughput: ${(callsPerSecond / 1e6).toFixed(2)}M calls/sec`)
  console.log(`  (sum=${sum} to prevent optimization)`)
  console.log('')

  return nsPerCall
}

// ============================================================================
// Benchmark 3: No-op with atomic increment (realistic wake)
// ============================================================================

function benchmarkNoopAtomic() {
  console.log('--- Benchmark 3: No-op + atomic increment (realistic wake) ---\n')

  // Warmup
  for (let i = 0; i < 10000; i++) {
    lib.symbols.spark_noop_atomic()
  }

  const iterations = 10_000_000

  const start = Bun.nanoseconds()
  for (let i = 0; i < iterations; i++) {
    lib.symbols.spark_noop_atomic()
  }
  const elapsed = Bun.nanoseconds() - start

  const nsPerCall = elapsed / iterations
  const callsPerSecond = (iterations / elapsed) * 1e9

  console.log(`  Iterations: ${iterations.toLocaleString()}`)
  console.log(`  Total time: ${(elapsed / 1e6).toFixed(2)}ms`)
  console.log(`  Per call: ${nsPerCall.toFixed(1)}ns`)
  console.log(`  Throughput: ${(callsPerSecond / 1e6).toFixed(2)}M calls/sec`)
  console.log('')

  return nsPerCall
}

// ============================================================================
// Benchmark 4: Real function (spark_buffer_size)
// ============================================================================

function benchmarkBufferSize() {
  console.log('--- Benchmark 4: Real function (spark_buffer_size) ---\n')

  // Warmup
  for (let i = 0; i < 10000; i++) {
    lib.symbols.spark_buffer_size()
  }

  const iterations = 10_000_000
  let sum = 0

  const start = Bun.nanoseconds()
  for (let i = 0; i < iterations; i++) {
    sum += lib.symbols.spark_buffer_size()
  }
  const elapsed = Bun.nanoseconds() - start

  const nsPerCall = elapsed / iterations
  const callsPerSecond = (iterations / elapsed) * 1e9

  console.log(`  Iterations: ${iterations.toLocaleString()}`)
  console.log(`  Total time: ${(elapsed / 1e6).toFixed(2)}ms`)
  console.log(`  Per call: ${nsPerCall.toFixed(1)}ns`)
  console.log(`  Throughput: ${(callsPerSecond / 1e6).toFixed(2)}M calls/sec`)
  console.log(`  (sum=${sum} to prevent optimization)`)
  console.log('')

  return nsPerCall
}

// ============================================================================
// Benchmark 5: Pure JS baseline
// ============================================================================

function benchmarkPureJS() {
  console.log('--- Benchmark 5: Pure JS baseline (function call) ---\n')

  let counter = 0
  const fn = () => { counter++ }

  const iterations = 10_000_000

  const start = Bun.nanoseconds()
  for (let i = 0; i < iterations; i++) {
    fn()
  }
  const elapsed = Bun.nanoseconds() - start

  const nsPerOp = elapsed / iterations

  console.log(`  Iterations: ${iterations.toLocaleString()}`)
  console.log(`  Total time: ${(elapsed / 1e6).toFixed(2)}ms`)
  console.log(`  Per operation: ${nsPerOp.toFixed(1)}ns`)
  console.log(`  (counter=${counter} to prevent optimization)`)
  console.log('')

  return nsPerOp
}

// ============================================================================
// Benchmark 6: SharedArrayBuffer write (current approach)
// ============================================================================

function benchmarkSABWrite() {
  console.log('--- Benchmark 6: SharedArrayBuffer Atomics.store ---\n')

  const sab = new SharedArrayBuffer(64)
  const view = new Int32Array(sab)

  const iterations = 10_000_000

  const start = Bun.nanoseconds()
  for (let i = 0; i < iterations; i++) {
    Atomics.store(view, 0, i)
  }
  const elapsed = Bun.nanoseconds() - start

  const nsPerOp = elapsed / iterations

  console.log(`  Iterations: ${iterations.toLocaleString()}`)
  console.log(`  Total time: ${(elapsed / 1e6).toFixed(2)}ms`)
  console.log(`  Per operation: ${nsPerOp.toFixed(1)}ns`)
  console.log('')

  return nsPerOp
}

// ============================================================================
// Benchmark 7: SharedArrayBuffer write + notify (current wake)
// ============================================================================

function benchmarkSABWriteAndNotify() {
  console.log('--- Benchmark 7: Atomics.store + Atomics.notify ---\n')

  const sab = new SharedArrayBuffer(64)
  const view = new Int32Array(sab)

  const iterations = 10_000_000

  const start = Bun.nanoseconds()
  for (let i = 0; i < iterations; i++) {
    Atomics.store(view, 0, 1)
    Atomics.notify(view, 0)
  }
  const elapsed = Bun.nanoseconds() - start

  const nsPerOp = elapsed / iterations

  console.log(`  Iterations: ${iterations.toLocaleString()}`)
  console.log(`  Total time: ${(elapsed / 1e6).toFixed(2)}ms`)
  console.log(`  Per operation: ${nsPerOp.toFixed(1)}ns`)
  console.log('')

  return nsPerOp
}

// ============================================================================
// Run all benchmarks
// ============================================================================

const ffiNoop = benchmarkNoop()
const ffiNoopArgs = benchmarkNoopArgs()
const ffiNoopAtomic = benchmarkNoopAtomic()
const ffiBufferSize = benchmarkBufferSize()
const pureJS = benchmarkPureJS()
const sabWrite = benchmarkSABWrite()
const sabWriteNotify = benchmarkSABWriteAndNotify()

// ============================================================================
// Summary
// ============================================================================

console.log('=== SUMMARY ===\n')
console.log('Operation                        │ Time (ns) │ vs SAB+notify')
console.log('─────────────────────────────────┼───────────┼──────────────')
console.log(`Pure JS function call            │ ${pureJS.toFixed(1).padStart(9)} │ ${(pureJS / sabWriteNotify).toFixed(2)}x`)
console.log(`SAB Atomics.store                │ ${sabWrite.toFixed(1).padStart(9)} │ ${(sabWrite / sabWriteNotify).toFixed(2)}x`)
console.log(`SAB Atomics.store + notify       │ ${sabWriteNotify.toFixed(1).padStart(9)} │ 1.00x (baseline)`)
console.log(`FFI pure no-op                   │ ${ffiNoop.toFixed(1).padStart(9)} │ ${(ffiNoop / sabWriteNotify).toFixed(2)}x`)
console.log(`FFI no-op + args                 │ ${ffiNoopArgs.toFixed(1).padStart(9)} │ ${(ffiNoopArgs / sabWriteNotify).toFixed(2)}x`)
console.log(`FFI no-op + atomic               │ ${ffiNoopAtomic.toFixed(1).padStart(9)} │ ${(ffiNoopAtomic / sabWriteNotify).toFixed(2)}x`)
console.log(`FFI real function                │ ${ffiBufferSize.toFixed(1).padStart(9)} │ ${(ffiBufferSize / sabWriteNotify).toFixed(2)}x`)
console.log('')

const ffiOverhead = ffiNoopAtomic - sabWriteNotify
console.log(`FFI overhead vs current approach: ${ffiOverhead > 0 ? '+' : ''}${ffiOverhead.toFixed(1)}ns`)
console.log('')

console.log('=== CONTEXT ===\n')
console.log(`  Current 1ms sleep polling:     1,000,000ns`)
console.log(`  60fps frame budget:           16,666,667ns`)
console.log(`  FFI wake as % of frame:        ${(ffiNoopAtomic / 16_666_667 * 100).toFixed(6)}%`)
console.log(`  FFI calls per 1ms budget:      ${Math.floor(1_000_000 / ffiNoopAtomic).toLocaleString()}`)
console.log('')

if (ffiNoopAtomic < 100) {
  console.log('✅ FFI overhead is under 100ns - EXCELLENT for wake notifications!')
} else if (ffiNoopAtomic < 500) {
  console.log('✅ FFI overhead is under 500ns - perfectly acceptable')
} else if (ffiNoopAtomic < 1000) {
  console.log('⚠️  FFI overhead is under 1μs - acceptable but notable')
} else {
  console.log('❌ FFI overhead is over 1μs - needs investigation')
}

lib.close()
