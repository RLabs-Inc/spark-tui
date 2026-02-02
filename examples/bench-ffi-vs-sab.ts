/**
 * FFI-per-prop vs SharedBuffer + Single Wake
 *
 * Compares two architectures:
 * A) FFI call for each prop change (spark_set_width(node, value))
 * B) Write props to SharedBuffer + single FFI wake
 */

import { dlopen, FFIType, ptr } from 'bun:ffi'
import { join } from 'path'

const libPath = join(import.meta.dir, '../rust/target/release/libspark_tui_engine.dylib')

console.log('=== FFI-per-prop vs SharedBuffer+Wake ===\n')

const lib = dlopen(libPath, {
  // Simulates FFI call with args (like spark_set_width(node, value))
  spark_noop_args: {
    args: [FFIType.u32, FFIType.u32],
    returns: FFIType.u32,
  },
  // Simulates single wake call (no args)
  spark_noop_atomic: {
    args: [],
    returns: FFIType.void,
  },
})

// SharedArrayBuffer for the "write + wake" approach
const sab = new SharedArrayBuffer(1024 * 1024)
const floatView = new Float32Array(sab)
const int32View = new Int32Array(sab)

// ============================================================================
// Architecture A: FFI call per prop
// ============================================================================

function benchFFIPerProp(propCount: number) {
  console.log(`--- Architecture A: ${propCount} FFI calls (one per prop) ---\n`)

  const iterations = 1_000_000

  // Warmup
  for (let i = 0; i < 10000; i++) {
    for (let p = 0; p < propCount; p++) {
      lib.symbols.spark_noop_args(42, i + p)
    }
  }

  const start = Bun.nanoseconds()
  for (let i = 0; i < iterations; i++) {
    for (let p = 0; p < propCount; p++) {
      lib.symbols.spark_noop_args(42, i + p)  // FFI with node ID + value
    }
  }
  const elapsed = Bun.nanoseconds() - start

  const nsPerUpdate = elapsed / iterations
  const nsPerProp = nsPerUpdate / propCount

  console.log(`  Props per update: ${propCount}`)
  console.log(`  Time per update: ${nsPerUpdate.toFixed(1)}ns`)
  console.log(`  Time per prop: ${nsPerProp.toFixed(1)}ns`)
  console.log(`  Throughput: ${((iterations / elapsed) * 1e9 / 1e6).toFixed(2)}M updates/sec`)
  console.log('')

  return nsPerUpdate
}

// ============================================================================
// Architecture B: SharedBuffer writes + single FFI wake
// ============================================================================

function benchSABPlusWake(propCount: number) {
  console.log(`--- Architecture B: ${propCount} SAB writes + 1 FFI wake ---\n`)

  const iterations = 1_000_000
  const baseOffset = 1000  // Simulate node offset

  // Warmup
  for (let i = 0; i < 10000; i++) {
    for (let p = 0; p < propCount; p++) {
      floatView[baseOffset + p] = i + p
    }
    lib.symbols.spark_noop_atomic()
  }

  const start = Bun.nanoseconds()
  for (let i = 0; i < iterations; i++) {
    for (let p = 0; p < propCount; p++) {
      floatView[baseOffset + p] = i + p  // Direct memory write
    }
    lib.symbols.spark_noop_atomic()  // Single wake
  }
  const elapsed = Bun.nanoseconds() - start

  const nsPerUpdate = elapsed / iterations
  const nsPerProp = nsPerUpdate / propCount

  console.log(`  Props per update: ${propCount}`)
  console.log(`  Time per update: ${nsPerUpdate.toFixed(1)}ns`)
  console.log(`  Time per prop: ${nsPerProp.toFixed(1)}ns`)
  console.log(`  Throughput: ${((iterations / elapsed) * 1e9 / 1e6).toFixed(2)}M updates/sec`)
  console.log('')

  return nsPerUpdate
}

// ============================================================================
// Architecture C: SAB writes + Atomics.notify (current broken approach)
// ============================================================================

function benchSABPlusAtomics(propCount: number) {
  console.log(`--- Architecture C: ${propCount} SAB writes + Atomics.notify (current) ---\n`)

  const iterations = 1_000_000
  const baseOffset = 1000

  // Warmup
  for (let i = 0; i < 10000; i++) {
    for (let p = 0; p < propCount; p++) {
      floatView[baseOffset + p] = i + p
    }
    Atomics.store(int32View, 0, 1)
    Atomics.notify(int32View, 0)
  }

  const start = Bun.nanoseconds()
  for (let i = 0; i < iterations; i++) {
    for (let p = 0; p < propCount; p++) {
      floatView[baseOffset + p] = i + p
    }
    Atomics.store(int32View, 0, 1)
    Atomics.notify(int32View, 0)
  }
  const elapsed = Bun.nanoseconds() - start

  const nsPerUpdate = elapsed / iterations

  console.log(`  Props per update: ${propCount}`)
  console.log(`  Time per update: ${nsPerUpdate.toFixed(1)}ns`)
  console.log(`  (But doesn't actually wake Rust!)`)
  console.log('')

  return nsPerUpdate
}

// ============================================================================
// Run benchmarks with different prop counts
// ============================================================================

const propCounts = [1, 4, 8, 16, 32]
const results: { props: number; ffiPer: number; sabWake: number; atomics: number }[] = []

for (const count of propCounts) {
  console.log(`\n${'='.repeat(60)}`)
  console.log(`PROP COUNT: ${count}`)
  console.log(`${'='.repeat(60)}\n`)

  const ffiPer = benchFFIPerProp(count)
  const sabWake = benchSABPlusWake(count)
  const atomics = benchSABPlusAtomics(count)

  results.push({ props: count, ffiPer, sabWake, atomics })
}

// ============================================================================
// Summary Table
// ============================================================================

console.log('\n' + '='.repeat(70))
console.log('SUMMARY: Time per update (ns)')
console.log('='.repeat(70))
console.log('')
console.log('Props │ FFI/prop │ SAB+Wake │ Atomics  │ Winner        │ Speedup')
console.log('──────┼──────────┼──────────┼──────────┼───────────────┼────────')

for (const r of results) {
  const winner = r.sabWake < r.ffiPer ? 'SAB+Wake' : 'FFI/prop'
  const speedup = r.sabWake < r.ffiPer
    ? (r.ffiPer / r.sabWake).toFixed(1) + 'x faster'
    : (r.sabWake / r.ffiPer).toFixed(1) + 'x faster'

  console.log(
    `${r.props.toString().padStart(5)} │ ` +
    `${r.ffiPer.toFixed(1).padStart(8)} │ ` +
    `${r.sabWake.toFixed(1).padStart(8)} │ ` +
    `${r.atomics.toFixed(1).padStart(8)} │ ` +
    `${winner.padEnd(13)} │ ` +
    `${speedup}`
  )
}

console.log('')
console.log('Note: Atomics.notify DOES NOT actually wake Rust!')
console.log('')

// ============================================================================
// Crossover Analysis
// ============================================================================

console.log('='.repeat(70))
console.log('ANALYSIS')
console.log('='.repeat(70))
console.log('')

// At what point do the approaches cross over?
const ffi1 = results.find(r => r.props === 1)!
const sab1 = results.find(r => r.props === 1)!

const ffiCallOverhead = ffi1.ffiPer  // ~12ns per FFI call with args
const sabWriteOverhead = 1  // ~1ns per memory write
const wakeOverhead = 5  // ~5ns for wake FFI

// FFI per prop: n * ffiCallOverhead
// SAB + wake: n * sabWriteOverhead + wakeOverhead

// Crossover when: n * ffi = n * sab + wake
// n * (ffi - sab) = wake
// n = wake / (ffi - sab)

const crossover = wakeOverhead / (ffiCallOverhead - sabWriteOverhead)

console.log(`FFI call with args: ~${ffiCallOverhead.toFixed(0)}ns`)
console.log(`SAB write: ~${sabWriteOverhead}ns`)
console.log(`FFI wake: ~${wakeOverhead}ns`)
console.log('')
console.log(`Theoretical crossover: ${crossover.toFixed(1)} props`)
console.log('')
console.log('For any realistic component update (2+ props):')
console.log('  → SharedBuffer + single FFI wake WINS')
console.log('')
console.log('RECOMMENDED ARCHITECTURE:')
console.log('  1. TS writes props directly to SharedBuffer')
console.log('  2. TS calls spark_wake() ONCE after all writes')
console.log('  3. Rust wakes, reads SharedBuffer, propagates reactivity')

lib.close()
